use sqlx::{PgConnection, PgPool};

use crate::{
    workers::adapters::{
        dependencies::completed_instance_worker::{
            CompletedInstanceWorkerContext, CompletedInstanceWorkerDependencies,
        },
        managers::WorkflowInstance,
        receivers::CompletedInstanceReceiver,
    },
    workflows::Workflow,
};

async fn process(conn: &mut PgConnection, instance: WorkflowInstance) -> anyhow::Result<()> {
    tracing::info!("Completed instance: {:?}", instance);

    Ok(())
}

pub async fn main<W: Workflow, C: CompletedInstanceWorkerContext<W>>(
    dependencies: CompletedInstanceWorkerDependencies<W, C>,
) -> anyhow::Result<()> {
    // let dependencies = C::dependencies().await?;

    let mut completed_instance_receiver = dependencies.completed_instance_receiver;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = completed_instance_receiver.receive().await else {
            tracing::error!("Completed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;
        if let Err(err) = process(&mut tx, step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }
        tx.commit().await?;
        completed_instance_receiver.accept(handle).await?;
    }
}
