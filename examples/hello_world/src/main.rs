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
use adapter_types::dependencies::DependencyManager;
use anyhow::Context;
use aws_adapter::dependencies::AwsAdapterConfig;
use aws_adapter::dependencies::AwsDependencyManager;
use config::{Config, Environment};
use surgeflow::workers::active_step_worker;
use surgeflow::workers::completed_instance_worker;
use surgeflow::workers::completed_step_worker;
use surgeflow::workers::control_server;
use surgeflow::workers::failed_instance_worker;
use surgeflow::workers::failed_step_worker;
use surgeflow::workers::new_event_worker;
use surgeflow::workers::new_instance_worker;
use surgeflow::workers::next_step_worker;
use surgeflow::workflows::MyProject;
use surgeflow::workflows::workflow_1::Workflow1;
use surgeflow::workflows::workflow_2::Workflow2;
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
use surgeflow_types::Project;
use tokio::try_join;
use tracing::Level;

fn get_aws_adapter_config() -> anyhow::Result<AwsAdapterConfig> {
    let config = Config::builder()
        // .add_source(File::with_name("config/default")) // e.g. default.toml
        // .add_source(File::with_name("config/local").required(false))
        .add_source(Environment::with_prefix("SURGEFLOW"))
        .build()?;

    let config = config
        .try_deserialize::<AwsAdapterConfig>()
        .context("Failed to deserialize AwsAdapterConfig from environment variables")?;

    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let config = get_aws_adapter_config()?;

    let dependency_manager = AwsDependencyManager::new(config);

    let project = MyProject {
        workflow_1: Workflow1 {},
        workflow_2: Workflow2 {},
    };

    main_handler(project, dependency_manager).await?;
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
async fn main_handler<P: Project, D>(project: P, mut dependency_manager: D) -> anyhow::Result<()>
where
    D: DependencyManager<P>,
{
    try_join!(
        #[cfg(feature = "control_server")]
        control_server::main::<P, _, _>(
            dependency_manager
                .control_server_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get control server dependencies"))
        ),
        #[cfg(feature = "active_step_worker")]
        active_step_worker::main::<P, _, _, _, _, _>(
            dependency_manager
                .active_step_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get active step worker dependencies")),
            project.clone(),
        ),
        #[cfg(feature = "new_instance_worker")]
        new_instance_worker::main::<P, _, _, _>(
            dependency_manager
                .new_instance_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get new instance worker dependencies"))
        ),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::main::<P, _, _, _, _>(
            dependency_manager
                .next_step_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get next step worker dependencies"))
        ),
        #[cfg(feature = "new_event_worker")]
        new_event_worker::main::<P, _, _, _>(
            dependency_manager
                .new_event_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get new event worker dependencies"))
        ),
        #[cfg(feature = "completed_step_worker")]
        completed_step_worker::main::<P, _, _, _>(
            dependency_manager
                .completed_step_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get completed step worker dependencies"))
        ),
        #[cfg(feature = "failed_step_worker")]
        failed_step_worker::main::<P, _, _, _>(
            dependency_manager
                .failed_step_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get failed step worker dependencies"))
        ),
        #[cfg(feature = "failed_instance_worker")]
        failed_instance_worker::main::<P, _>(
            dependency_manager
                .failed_instance_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get failed instance worker dependencies"))
        ),
        #[cfg(feature = "completed_instance_worker")]
        completed_instance_worker::main::<P, _>(
            dependency_manager
                .completed_instance_worker_dependencies()
                .await
                .unwrap_or_else(|_| panic!("Failed to get completed instance worker dependencies"))
        ),
    )?;

    Ok(())
}
