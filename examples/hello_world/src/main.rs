use anyhow::Context;
use aws_adapter::dependencies::AwsAdapterConfig;
use aws_adapter::dependencies::AwsDependencyManager;
use config::{Config, Environment};
use surgeflow::main_handler;
use tracing::Level;
use workflows::MyProject;
use workflows::workflow_1::Workflow1;
use workflows::workflow_2::Workflow2;

mod workflows;

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
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config = get_aws_adapter_config()?;

    let dependency_manager = AwsDependencyManager::new(config);

    let project = MyProject {
        workflow_1: Workflow1 {},
        workflow_2: Workflow2 {},
    };

    main_handler(project, dependency_manager).await?;
    Ok(())
}
