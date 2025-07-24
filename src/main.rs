use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get_with},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{
    Extension, ServiceExt,
    extract::{Json, Request},
};
use rust_workflow_2::workers::active_step_worker;
use rust_workflow_2::workers::completed_instance_worker;
use rust_workflow_2::workers::completed_step_worker;
use rust_workflow_2::workers::failed_instance_worker;
use rust_workflow_2::workers::failed_step_worker;
use rust_workflow_2::workers::new_event_worker;
use rust_workflow_2::workers::new_instance_worker;
use rust_workflow_2::workers::next_step_worker;
use rust_workflow_2::workflows::workflow_1::MyProject;
use rust_workflow_2::{
    // workers::azure_adapter::dependencies::control_server::AzureServiceBusControlServerDependencies,
    workflows::init_app_state,
};

use rust_workflow_2::workflows::{Project, WorkflowControl};
use tokio::{net::TcpListener, try_join};
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

async fn serve(router: ApiRouter) -> anyhow::Result<()> {
    let router = ApiRouter::new().merge(router);

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Example usage of the updated main_handler with Project-based architecture
    // This demonstrates that the azure_adapter now works with Project trait
    use rust_workflow_2::workflows::workflow_1::{MyProjectWorkflow, Workflow1};

    // NOTE: This would require environment variables to be set:
    // - AZURE_SERVICE_BUS_CONNECTION_STRING
    // - COSMOS_CONNECTION_STRING
    // - APP_USER_DATABASE_URL
    // Uncomment to test (would fail at runtime without env vars):
    // main_handler::<MyProject>(
    //     "workflow_1".to_string(),
    //     MyProjectWorkflow::Workflow1(Workflow1)
    // ).await?;

    // this shouldn't have to be hard coded
    // type ControlServerDependencies<W> = AzureServiceBusControlServerDependencies<W>;

    // my_main!(Workflow0, Workflow1);
    // my_main!(Workflow1);

    main_handler(MyProjectWorkflow::Workflow1(Workflow1)).await?;
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

// TODO: testing with MyProject instead of generic Project
// async fn main_handler<P: Project>(project_workflow: P::Workflow) -> anyhow::Result<()>
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
async fn main_handler(project_workflow: <MyProject as Project>::Workflow) -> anyhow::Result<()> {
    use rust_workflow_2::workers::adapters::dependencies::ActiveStepWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::CompletedInstanceWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::CompletedStepWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::FailedInstanceWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::FailedStepWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::NewEventWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::NewInstanceWorkerDependencyProvider;
    use rust_workflow_2::workers::adapters::dependencies::NextStepWorkerDependencyProvider;
    use rust_workflow_2::workers::azure_adapter::dependencies::{
        AzureAdapterConfig, AzureDependencyManager,
    };

    let mut dependency_manager: AzureDependencyManager =
        AzureDependencyManager::new(AzureAdapterConfig {
            service_bus_connection_string: std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set"),
            cosmos_connection_string: std::env::var("COSMOS_CONNECTION_STRING")
                .expect("COSMOS_CONNECTION_STRING must be set"),
            new_instance_queue_suffix: "instance".into(),
            next_step_queue_suffix: "next-steps".into(),
            completed_instance_queue_suffix: "completed-instances".into(),
            completed_step_queue_suffix: "completed-steps".into(),
            active_step_queue_suffix: "active-steps".into(),
            failed_instance_queue_suffix: "failed-instances".into(),
            failed_step_queue_suffix: "failed-steps".into(),
            new_event_queue_suffix: "events".into(),
            pg_connection_string: std::env::var("APP_USER_DATABASE_URL")
                .expect("APP_USER_DATABASE_URL must be set"),
        });

    #[cfg(feature = "control_server")]
    {
        use rust_workflow_2::{
            workers::{
                adapters::dependencies::ControlServerDependencyProvider,
                azure_adapter::senders::{
                    AzureServiceBusEventSender, AzureServiceBusNewInstanceSender,
                },
            },
            workflows::workflow_1::Workflow1,
        };

        let app_state = init_app_state::<
            MyProject,
            AzureServiceBusEventSender<MyProject>,
            AzureServiceBusNewInstanceSender<MyProject>,
        >(
            dependency_manager
                .control_server_dependencies()
                .await
                .expect("TODO: handle error"),
        )
        .await?;
        let router = Workflow1::control_router::<
            AzureServiceBusEventSender<MyProject>,
            AzureServiceBusNewInstanceSender<MyProject>,
        >()
        .await?
        .with_state(app_state);

        tokio::spawn(async move {
            if let Err(e) = serve(router).await {
                tracing::error!("Control server error: {}", e);
            }
        });
    }

    try_join!(
        #[cfg(feature = "active_step_worker")]
        active_step_worker::main::<MyProject, _, _, _, _, _>(
            dependency_manager
                .active_step_worker_dependencies()
                .await
                .expect("TODO: handle error"),
            project_workflow.clone(),
        ),
        #[cfg(feature = "new_instance_worker")]
        new_instance_worker::main::<MyProject, _, _, _>(
            dependency_manager
                .new_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::main::<MyProject, _, _, _, _>(
            dependency_manager
                .next_step_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "new_event_worker")]
        new_event_worker::main::<MyProject, _, _, _>(
            dependency_manager
                .new_event_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "completed_step_worker")]
        completed_step_worker::main::<MyProject, _, _, _>(
            dependency_manager
                .completed_step_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "failed_step_worker")]
        failed_step_worker::main::<MyProject, _, _, _>(
            dependency_manager
                .failed_step_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "failed_instance_worker")]
        failed_instance_worker::main::<MyProject, _>(
            dependency_manager
                .failed_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "completed_instance_worker")]
        completed_instance_worker::main::<MyProject, _>(
            dependency_manager
                .completed_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
    )?;

    Ok(())
}
