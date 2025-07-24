use rust_workflow_2::workers::active_step_worker;
use rust_workflow_2::workers::adapters::dependencies::ControlServerDependencyProvider;
use rust_workflow_2::workers::completed_instance_worker;
use rust_workflow_2::workers::completed_step_worker;
use rust_workflow_2::workers::control_server;
use rust_workflow_2::workers::failed_instance_worker;
use rust_workflow_2::workers::failed_step_worker;
use rust_workflow_2::workers::new_event_worker;
use rust_workflow_2::workers::new_instance_worker;
use rust_workflow_2::workers::next_step_worker;
use rust_workflow_2::workflows::workflow_1::MyProject;

use rust_workflow_2::workflows::Project;
use rust_workflow_2::workflows::workflow_1::Workflow1;
use tokio::try_join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    main_handler::<MyProject>(MyProject {
        workflow_1: Workflow1,
    })
    .await?;
    Ok(())
}

#[cfg(any(
    feature = "active_step_worker",
    feature = "new_instance_worker",
    feature = "next_step_worker",
    feature = "new_event_worker",
    feature = "completed_step_worker",
    feature = "failed_step_worker",
    feature = "failed_instance_worker",
    feature = "completed_instance_worker",
    feature = "control_server"
))]
async fn main_handler<P: Project>(project: P) -> anyhow::Result<()> {
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
    try_join!(
        #[cfg(feature = "control_server")]
        control_server::main::<P, _, _>(
            dependency_manager
                .control_server_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "active_step_worker")]
        active_step_worker::main::<P, _, _, _, _, _>(
            dependency_manager
                .active_step_worker_dependencies()
                .await
                .expect("TODO: handle error"),
            project.clone(),
        ),
        #[cfg(feature = "new_instance_worker")]
        new_instance_worker::main::<P, _, _, _>(
            dependency_manager
                .new_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::main::<P, _, _, _, _>(
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
        completed_step_worker::main::<P, _, _, _>(
            dependency_manager
                .completed_step_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "failed_step_worker")]
        failed_step_worker::main::<P, _, _, _>(
            dependency_manager
                .failed_step_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "failed_instance_worker")]
        failed_instance_worker::main::<P, _>(
            dependency_manager
                .failed_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
        #[cfg(feature = "completed_instance_worker")]
        completed_instance_worker::main::<P, _>(
            dependency_manager
                .completed_instance_worker_dependencies()
                .await
                .expect("TODO: handle error")
        ),
    )?;

    Ok(())
}
