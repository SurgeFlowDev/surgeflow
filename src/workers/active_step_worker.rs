use crate::{
    step::{FullyQualifiedStep, Step},
    workers::adapters::{
        dependencies::active_step_worker::ActiveStepWorkerContext,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, CompletedStepSender, FailedStepSender, NextStepSender},
    },
    workflows::{StepId, Workflow},
};
use sqlx::{PgConnection, query};
use uuid::Uuid;

async fn process<W: Workflow, D: ActiveStepWorkerContext<W>>(
    wf: W,
    active_step_sender: &mut D::ActiveStepSender,
    next_step_sender: &mut D::NextStepSender,
    failed_step_sender: &mut D::FailedStepSender,
    completed_step_sender: &mut D::CompletedStepSender,
    conn: &mut PgConnection,
    mut step: FullyQualifiedStep<W::Step>,
) -> anyhow::Result<()> {
    let instance_id = step.instance_id;
    tracing::info!("Received new step");
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

    let next_step = step.step.step.run_raw(wf.clone(), step.event.clone()).await;
    step.retry_count += 1;
    if let Ok(next_step) = next_step {
        completed_step_sender.send(step).await?;
        if let Some(next_step) = next_step {
            next_step_sender
                .send(FullyQualifiedStep {
                    instance_id,
                    step: next_step,
                    event: None,
                    retry_count: 0,
                    step_id: StepId::new(),
                })
                .await?;
        } else {
            // TODO: push to instance completed queue ?
            tracing::info!("Instance {instance_id} completed");
        }
    } else {
        tracing::info!("Failed to run step: {:?}", step.step);

        if step.retry_count <= step.step.settings.max_retries {
            tracing::info!("Retrying step. Retry count: {}", step.retry_count);
            active_step_sender.send(step).await?;
        } else {
            tracing::info!("Max retries reached for step: {:?}", step.step);
            // TODO: push into "step failed" queue.
            // TODO: and maybe "workflow failed" queue. though this could be done in the "step failed" queue consumer
            failed_step_sender.send(step).await?;
        }
    }

    Ok(())
}

pub async fn main<W: Workflow, D: ActiveStepWorkerContext<W>>(wf: W) -> anyhow::Result<()> {
    tracing::info!("Active Step Worker started for workflow: {}", W::NAME);
    let dependencies = D::dependencies().await?;

    let mut active_step_receiver = dependencies.active_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut next_step_sender = dependencies.next_step_sender;
    let mut failed_step_sender = dependencies.failed_step_sender;
    let mut completed_step_sender = dependencies.completed_step_sender;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = active_step_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;

        if let Err(err) = process::<W, D>(
            wf.clone(),
            &mut active_step_sender,
            &mut next_step_sender,
            &mut failed_step_sender,
            &mut completed_step_sender,
            &mut tx,
            step,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }

        tx.commit().await?;
        active_step_receiver.accept(handle).await?;
    }
}
