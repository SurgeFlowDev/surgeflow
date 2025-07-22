use sqlx::{PgConnection, PgPool};

use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::failed_instance_worker::FailedInstanceWorkerContext,
        managers::WorkflowInstance, receivers::FailedInstanceReceiver,
    },
    workflows::{StepId, Workflow},
};

async fn process(conn: &mut PgConnection, instance: WorkflowInstance) -> anyhow::Result<()> {
    tracing::info!("Failed instance: {:?}", instance);

    Ok(())
}

pub async fn main<W: Workflow, C: FailedInstanceWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = C::dependencies().await?;

    let mut failed_instance_receiver = dependencies.failed_instance_receiver;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = failed_instance_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;
        if let Err(err) = process(&mut tx, step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }
        tx.commit().await?;
        failed_instance_receiver.accept(handle).await?;
    }
}
