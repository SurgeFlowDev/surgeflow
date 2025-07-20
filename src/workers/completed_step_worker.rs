use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::completed_step_worker::CompletedStepWorkerContext,
        managers::StepsAwaitingEventManager, receivers::CompletedStepReceiver,
        senders::ActiveStepSender,
    },
    workflows::Workflow,
};
use derive_more::Debug;
use sqlx::{PgConnection, PgPool, query};
use std::any::TypeId;
use uuid::Uuid;

pub async fn main<W: Workflow, D: CompletedStepWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut completed_step_receiver = dependencies.completed_step_receiver;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let mut pool = PgPool::connect(&connection_string).await?;

    loop {
        if let Err(err) = receive_and_process::<W, D>(&mut completed_step_receiver, &mut pool).await
        {
            tracing::error!("Error processing completed step: {:?}", err);
        }
    }
}

async fn receive_and_process<W: Workflow, D: CompletedStepWorkerContext<W>>(
    completed_step_receiver: &mut D::CompletedStepReceiver,
    pool: &mut PgPool,
) -> anyhow::Result<()> {
    let (step, handle) = completed_step_receiver.receive().await?;
    let mut tx = pool.begin().await?;

    if let Err(err) = process::<W>(&mut tx, step).await {
        tracing::error!("Error processing completed step: {:?}", err);
    }

    tx.commit().await?;
    completed_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum CompletedStepWorkerError {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
}

async fn process<W: Workflow>(
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W>,
) -> Result<(), CompletedStepWorkerError> {
    tracing::info!(
        "received completed step for instance: {}",
        step.instance.external_id
    );

    query!(
        r#"
        UPDATE workflow_steps SET "status" = $1
        WHERE "external_id" = $2
        "#,
        4,
        Uuid::from(step.step_id)
    )
    .execute(conn)
    .await?;

    Ok(())
}
