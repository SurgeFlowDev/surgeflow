use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::completed_step_worker::CompletedStepWorkerDependencies,
        receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::{StepId, Workflow},
};
use derive_more::Debug;
use sqlx::{PgConnection, PgPool, query};
use uuid::Uuid;

pub async fn main<W: Workflow, CompletedStepReceiverT, NextStepSenderT>(
    dependencies: CompletedStepWorkerDependencies<W, CompletedStepReceiverT, NextStepSenderT>,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
{
    let mut completed_step_receiver = dependencies.completed_step_receiver;
    let mut next_step_sender = dependencies.next_step_sender;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let mut pool = PgPool::connect(&connection_string).await?;

    loop {
        if let Err(err) = receive_and_process::<W, CompletedStepReceiverT, NextStepSenderT>(
            &mut completed_step_receiver,
            &mut next_step_sender,
            &mut pool,
        )
        .await
        {
            tracing::error!("Error processing completed step: {:?}", err);
        }
    }
}

async fn receive_and_process<W: Workflow, CompletedStepReceiverT, NextStepSenderT>(
    completed_step_receiver: &mut CompletedStepReceiverT,
    next_step_sender: &mut NextStepSenderT,
    pool: &mut PgPool,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
{
    let (step, handle) = completed_step_receiver.receive().await?;
    let mut tx = pool.begin().await?;

    if let Err(err) = process::<W, NextStepSenderT>(next_step_sender, &mut tx, step).await {
        tracing::error!("Error processing completed step: {:?}", err);
    }

    tx.commit().await?;
    completed_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum CompletedStepWorkerError<W: Workflow, NextStepSenderT: NextStepSender<W>> {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send next step")]
    SendNextStepError(#[source] <NextStepSenderT as NextStepSender<W>>::Error),
}

async fn process<W: Workflow, NextStepSenderT>(
    next_step_sender: &mut NextStepSenderT,
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W>,
) -> Result<(), CompletedStepWorkerError<W, NextStepSenderT>>
where
    NextStepSenderT: NextStepSender<W>,
{
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
    .execute(conn.as_mut())
    .await?;

    let next_step = step.next_step;

    if let Some(next_step) = next_step {
        let json_step =
            serde_json::to_value(&next_step.step).expect("TODO: handle serialization error");

        next_step_sender
            .send(FullyQualifiedStep {
                instance: step.instance,
                step_id: StepId::new(),
                step: next_step,
                event: None,
                retry_count: 0,
                previous_step_id: Some(step.step_id),
                next_step: None,
            })
            .await
            .map_err(CompletedStepWorkerError::SendNextStepError)?;

        query!(
            r#"
            INSERT INTO workflow_step_outputs ("workflow_step_id", "output")
            VALUES ((SELECT id FROM workflow_steps WHERE external_id = $1), $2)
            "#,
            Uuid::from(step.step_id),
            json_step
        )
        .execute(conn)
        .await?;
    } else {
        // TODO: push to instance completed queue ?
        tracing::info!("Instance {} completed", step.instance.external_id);

        query!(
            r#"
            INSERT INTO workflow_step_outputs ("workflow_step_id", "output")
            VALUES ((SELECT id FROM workflow_steps WHERE external_id = $1), NULL)
            "#,
            Uuid::from(step.step_id),
        )
        .execute(conn)
        .await?;
    }

    Ok(())
}
