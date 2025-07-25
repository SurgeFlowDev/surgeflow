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
    use rust_workflow_2::workers::aws_adapter::dependencies::{
        AzureAdapterConfig, AzureDependencyManager,
    };

    let mut dependency_manager: AzureDependencyManager =
        AzureDependencyManager::new(AzureAdapterConfig {
            new_instance_queue_url: std::env::var("NEW_INSTANCE_QUEUE_URL")
                .expect("NEW_INSTANCE_QUEUE_URL must be set"),
            next_step_queue_url: std::env::var("NEXT_STEP_QUEUE_URL")
                .expect("NEXT_STEP_QUEUE_URL must be set"),
            completed_instance_queue_url: std::env::var("COMPLETED_INSTANCE_QUEUE_URL")
                .expect("COMPLETED_INSTANCE_QUEUE_URL must be set"),
            completed_step_queue_url: std::env::var("COMPLETED_STEP_QUEUE_URL")
                .expect("COMPLETED_STEP_QUEUE_URL must be set"),
            active_step_queue_url: std::env::var("ACTIVE_STEP_QUEUE_URL")
                .expect("ACTIVE_STEP_QUEUE_URL must be set"),
            failed_instance_queue_url: std::env::var("FAILED_INSTANCE_QUEUE_URL")
                .expect("FAILED_INSTANCE_QUEUE_URL must be set"),
            failed_step_queue_url: std::env::var("FAILED_STEP_QUEUE_URL")
                .expect("FAILED_STEP_QUEUE_URL must be set"),
            new_event_queue_url: std::env::var("NEW_EVENT_QUEUE_URL")
                .expect("NEW_EVENT_QUEUE_URL must be set"),
            pg_connection_string: std::env::var("APP_USER_DATABASE_URL")
                .expect("APP_USER_DATABASE_URL must be set"),
            dynamodb_table_name: std::env::var("DYNAMODB_TABLE_NAME")
                .expect("DYNAMODB_TABLE_NAME must be set"),
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
