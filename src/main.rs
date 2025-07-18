use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get_with},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{
    Extension, ServiceExt,
    extract::{Json, Request},
};
use macros::my_main;
use rust_workflow_2::workers::active_step_worker;
use rust_workflow_2::workers::azure_adapter::dependencies::active_step_worker::AzureServiceBusActiveStepWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::new_event_worker::AzureServiceBusNewEventWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::next_step_worker::AzureServiceBusNextStepWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::workspace_instance_worker::AzureServiceBusWorkspaceInstanceWorkerDependencies;
use rust_workflow_2::workers::next_step_worker;
use rust_workflow_2::workers::workspace_instance_worker::{self};
use rust_workflow_2::workflows::{TxState, workflow_0::Workflow0};
use rust_workflow_2::{
    workers::azure_adapter::dependencies::control_server::AzureServiceBusControlServerDependencies,
    workflows::init_app_state,
};

use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use rust_workflow_2::workflows::{Tx, TxLayer, Workflow, WorkflowControl, workflow_1::Workflow1};

use sqlx::PgPool;
use tokio::{net::TcpListener, try_join};
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

async fn serve(router: ApiRouter, sqlx_tx_layer: TxLayer) -> anyhow::Result<()> {
    let router = ApiRouter::new().merge(router).layer(sqlx_tx_layer);

    let mut api = base_open_api();

    let router = if cfg!(debug_assertions) {
        let router = router
            .api_route(
                "/openapi.json",
                get_with(serve_api, |op| op.summary("OpenAPI Spec").hidden(false)),
            )
            .route("/docs", Scalar::new("/openapi.json").axum_route());
        router.finish_api(&mut api).layer(Extension(api))
    } else {
        router.into()
    };
    let router = NormalizePathLayer::trim_trailing_slash().layer(router);
    let router = ServiceExt::<Request>::into_make_service(router);

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
#[cfg(feature = "control_server")]
async fn control_server_setup()
-> anyhow::Result<((TxState, TxLayer), ConnectionHandle<()>, SessionHandle<()>)> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-6", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let session = Session::begin(&mut connection).await?;

    Ok((
        Tx::setup(PgPool::connect("postgres://workflow:workflow@localhost:5432/workflow").await?),
        connection,
        session,
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // this shouldn't have to be hard coded
    type ControlServerDependencies<W> = AzureServiceBusControlServerDependencies<W>;

    // my_main!(Workflow0, Workflow1);
    my_main!(Workflow0);

    Ok(())
}

pub fn base_open_api() -> OpenApi {
    OpenApi {
        info: Info {
            description: Some("API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[cfg(any(
    feature = "active_step_worker",
    feature = "new_instance_worker",
    feature = "next_step_worker",
    feature = "new_event_worker"
))]
async fn main_handler<W: Workflow>(
    #[cfg(feature = "active_step_worker")] wf: W,
) -> anyhow::Result<()> {
    use rust_workflow_2::workers::new_event_worker;

    try_join!(
        #[cfg(feature = "active_step_worker")]
        active_step_worker::main::<W, AzureServiceBusActiveStepWorkerDependencies<W>>(wf),
        #[cfg(feature = "new_instance_worker")]
        workspace_instance_worker::main::<W, AzureServiceBusWorkspaceInstanceWorkerDependencies<W>>(
        ),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::main::<W, AzureServiceBusNextStepWorkerDependencies<W>>(),
        #[cfg(feature = "new_event_worker")]
        new_event_worker::main::<W, AzureServiceBusNewEventWorkerDependencies<W>>(),
    )?;

    Ok(())
}
