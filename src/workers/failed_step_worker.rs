use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::failed_step_worker::FailedStepWorkerContext,
        managers::{StepsAwaitingEventManager, WorkflowInstance},
        receivers::FailedStepReceiver,
        senders::{ActiveStepSender, FailedInstanceSender},
    },
    workflows::Workflow,
};
use derive_more::Debug;
use sqlx::{PgConnection, PgPool, query};
use uuid::Uuid;

pub async fn main<W: Workflow, D: FailedStepWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut failed_step_receiver = dependencies.failed_step_receiver;
    let mut failed_instance_sender = dependencies.failed_instance_sender;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let mut pool = PgPool::connect(&connection_string).await?;

    loop {
        if let Err(err) = receive_and_process::<W, D>(
            &mut failed_step_receiver,
            &mut failed_instance_sender,
            &mut pool,
        )
        .await
        {
            tracing::error!("Error processing failed step: {:?}", err);
        }
    }
}

async fn receive_and_process<W: Workflow, D: FailedStepWorkerContext<W>>(
    failed_step_receiver: &mut D::FailedStepReceiver,
    failed_instance_sender: &mut D::FailedInstanceSender,
    pool: &mut PgPool,
) -> anyhow::Result<()> {
    let (step, handle) = failed_step_receiver.receive().await?;
    let mut tx = pool.begin().await?;

    if let Err(err) = process::<W, D>(failed_instance_sender, &mut tx, step).await {
        tracing::error!("Error processing failed step: {:?}", err);
    }

    tx.commit().await?;
    failed_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum FailedStepWorkerError<W: Workflow, D: FailedStepWorkerContext<W>> {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send instance: {0}")]
    SendError(#[source] <D::FailedInstanceSender as FailedInstanceSender<W>>::Error),
}

async fn process<W: Workflow, D: FailedStepWorkerContext<W>>(
    failed_instance_sender: &mut D::FailedInstanceSender,
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W>,
) -> Result<(), FailedStepWorkerError<W, D>> {
    tracing::info!(
        "received failed step for instance: {}",
        step.instance.external_id
    );

    query!(
        r#"
        UPDATE workflow_steps SET "status" = $1
        WHERE "external_id" = $2
        "#,
        5,
        Uuid::from(step.step_id)
    )
    .execute(conn)
    .await?;

    failed_instance_sender
        .send(&step.instance)
        .await
        .map_err(FailedStepWorkerError::SendError)?;

    Ok(())
}
