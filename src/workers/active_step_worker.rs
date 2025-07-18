use crate::{
    step::{FullyQualifiedStep, Step},
    workers::adapters::{
        dependencies::active_step_worker::ActiveStepWorkerContext,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, FailedStepSender, NextStepSender},
    },
    workflows::Workflow,
};
use anyhow::bail;

async fn process<W: Workflow>(
    wf: W,
    active_step_receiver: &mut impl ActiveStepReceiver<W>,
    active_step_sender: &mut impl ActiveStepSender<W>,
    next_step_sender: &mut impl NextStepSender<W>,
    failed_step_sender: &mut impl FailedStepSender<W>,
) -> anyhow::Result<()> {
    tracing::info!("Active Step Worker started for workflow: {}", W::NAME);
    tracing::info!("Listening for active steps...");
    let Ok((mut step, handle)) = active_step_receiver.receive().await else {
        bail!("Failed to receive active step");
    };
    tracing::info!("Received new step");
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
    active_step_receiver.accept(handle).await?;

    Ok(())
}

pub async fn main<W: Workflow, D: ActiveStepWorkerContext<W>>(wf: W) -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut active_step_receiver = dependencies.active_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut next_step_sender = dependencies.next_step_sender;
    let mut failed_step_sender = dependencies.failed_step_sender;

    loop {
        if let Err(err) = process(
            wf.clone(),
            &mut active_step_receiver,
            &mut active_step_sender,
            &mut next_step_sender,
            &mut failed_step_sender,
        )
        .await
        {
            tracing::error!("Error occurred in loop_content: {}", err);
            continue;
        }
    }
}
