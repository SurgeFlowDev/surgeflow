use crate::{
    step::{FullyQualifiedStep, Step},
    workers::adapters::{
        dependencies::active_step_worker::ActiveStepWorkerContext,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, FailedStepSender, NextStepSender},
    },
    workflows::Workflow,
};
use sqlx::{PgConnection, query};

async fn process<W: Workflow>(
    wf: W,

    active_step_sender: &mut impl ActiveStepSender<W>,
    next_step_sender: &mut impl NextStepSender<W>,
    failed_step_sender: &mut impl FailedStepSender<W>,
    conn: &mut PgConnection,
    mut step: FullyQualifiedStep<W::Step>,
) -> anyhow::Result<()> {
    tracing::info!("Received new step");
    query!(
        r#"
        UPDATE latest_workflow_steps SET "status" = $1
        WHERE "workflow_instance_id" = $2
        "#,
        4,
        i32::from(step.instance_id)
    )
    .execute(conn)
    .await?;

    let next_step = step.step.step.run_raw(wf.clone(), step.event.clone()).await;
    step.retry_count += 1;
    if let Ok(next_step) = next_step {
        // TODO: maybe use `succeeded-step-sender` and push the old step into it? and handle workflow completion there
        if let Some(next_step) = next_step {
            next_step_sender
                .send(FullyQualifiedStep {
                    instance_id: step.instance_id,
                    step: next_step,
                    event: None,
                    retry_count: 0,
                })
                .await?;
        } else {
            tracing::info!("Instance {} completed", step.instance_id);
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

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = active_step_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;

        if let Err(err) = process::<W>(
            wf.clone(),
            &mut active_step_sender,
            &mut next_step_sender,
            &mut failed_step_sender,
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
