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
use rust_workflow_2::workers::azure_adapter::dependencies::completed_instance_worker::AzureServiceBusCompletedInstanceWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::completed_step_worker::AzureServiceBusCompletedStepWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::failed_instance_worker::AzureServiceBusFailedInstanceWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::failed_step_worker::AzureServiceBusFailedStepWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::new_event_worker::AzureServiceBusNewEventWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::new_instance_worker::AzureServiceBusNewInstanceWorkerDependencies;
use rust_workflow_2::workers::azure_adapter::dependencies::next_step_worker::AzureServiceBusNextStepWorkerDependencies;
use rust_workflow_2::workers::completed_instance_worker;
use rust_workflow_2::workers::completed_step_worker;
use rust_workflow_2::workers::failed_instance_worker;
use rust_workflow_2::workers::failed_step_worker;
use rust_workflow_2::workers::new_instance_worker;
use rust_workflow_2::workers::next_step_worker;
use rust_workflow_2::workflows::{TxState, workflow_0::Workflow0};
use rust_workflow_2::{
    workers::azure_adapter::dependencies::control_server::AzureServiceBusControlServerDependencies,
    workflows::init_app_state,
};

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
async fn control_server_setup() -> anyhow::Result<(TxState, TxLayer)> {
    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");

    Ok(Tx::setup(PgPool::connect(&connection_string).await?))
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
    feature = "new_event_worker",
    feature = "completed_step_worker",
    feature = "failed_step_worker",
    feature = "failed_instance_worker",
    feature = "completed_instance_worker"
))]
async fn main_handler<W: Workflow>(
    #[cfg(feature = "active_step_worker")] wf: W,
) -> anyhow::Result<()> {
    use rust_workflow_2::workers::new_event_worker;

    try_join!(
        #[cfg(feature = "active_step_worker")]
        active_step_worker::main::<W, AzureServiceBusActiveStepWorkerDependencies<W>>(wf),
        #[cfg(feature = "new_instance_worker")]
        new_instance_worker::main::<W, AzureServiceBusNewInstanceWorkerDependencies<W>>(),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::main::<W, AzureServiceBusNextStepWorkerDependencies<W>>(),
        #[cfg(feature = "new_event_worker")]
        new_event_worker::main::<W, AzureServiceBusNewEventWorkerDependencies<W>>(),
        #[cfg(feature = "completed_step_worker")]
        completed_step_worker::main::<W, AzureServiceBusCompletedStepWorkerDependencies<W>>(),
        #[cfg(feature = "failed_step_worker")]
        failed_step_worker::main::<W, AzureServiceBusFailedStepWorkerDependencies<W>>(),
        #[cfg(feature = "failed_instance_worker")]
        failed_instance_worker::main::<W, AzureServiceBusFailedInstanceWorkerDependencies<W>>(),
        #[cfg(feature = "completed_instance_worker")]
        completed_instance_worker::main::<W, AzureServiceBusCompletedInstanceWorkerDependencies<W>>(
        ),
    )?;

    Ok(())
}
