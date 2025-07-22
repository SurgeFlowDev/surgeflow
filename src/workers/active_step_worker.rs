use crate::{
    step::{FullyQualifiedStep, Step},
    workers::adapters::{
        dependencies::active_step_worker::ActiveStepWorkerDependencies,
        managers::PersistentStepManager,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, CompletedStepSender, FailedStepSender},
    },
    workflows::Workflow,
};

async fn process<
    W,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistentStepManagerT,
>(
    wf: W,
    active_step_sender: &mut ActiveStepSenderT,
    failed_step_sender: &mut FailedStepSenderT,
    completed_step_sender: &mut CompletedStepSenderT,
    persistent_step_manager: &mut PersistentStepManagerT,
    mut step: FullyQualifiedStep<W>,
) -> anyhow::Result<()>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!("Received new step");
    persistent_step_manager
        .set_step_status(step.step_id, 3)
        .await
        .expect("TODO: handle error");

    let next_step = step.step.step.run_raw(wf.clone(), step.event.clone()).await;
    step.retry_count += 1;
    if let Ok(next_step) = next_step {
        completed_step_sender
            .send(FullyQualifiedStep { next_step, ..step })
            .await?;
    } else {
        tracing::info!("Failed to run step: {:?}", step.step);

        if step.retry_count <= step.step.settings.max_retries {
            tracing::info!("Retrying step. Retry count: {}", step.retry_count);
            active_step_sender.send(step).await?;
        } else {
            tracing::info!("Max retries reached for step: {}", step.step_id);
            failed_step_sender.send(step).await?;
        }
    }

    Ok(())
}

pub async fn main<
    W,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistentStepManagerT,
>(
    dependencies: ActiveStepWorkerDependencies<
        W,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
        PersistentStepManagerT,
    >,
    wf: W,
) -> anyhow::Result<()>
where
    W: Workflow,
    ActiveStepReceiverT: ActiveStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!("Active Step Worker started for workflow: {}", W::NAME);

    let mut active_step_receiver = dependencies.active_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut failed_step_sender = dependencies.failed_step_sender;
    let mut completed_step_sender = dependencies.completed_step_sender;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        let Ok((step, handle)) = active_step_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };

        if let Err(err) = process::<
            W,
            ActiveStepSenderT,
            FailedStepSenderT,
            CompletedStepSenderT,
            PersistentStepManagerT,
        >(
            wf.clone(),
            &mut active_step_sender,
            &mut failed_step_sender,
            &mut completed_step_sender,
            &mut persistent_step_manager,
            step,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }

        active_step_receiver.accept(handle).await?;
    }
}
