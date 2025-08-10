use std::convert;

use adapter_types::{
    dependencies::active_step_worker::ActiveStepWorkerDependencies,
    managers::PersistenceManager,
    receivers::ActiveStepReceiver,
    senders::{ActiveStepSender, CompletedStepSender, FailedStepSender},
};
use anyhow::Context;
use surgeflow_types::{__Step, __Workflow, FullyQualifiedStep, Immediate, Project};

async fn process<
    P,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistenceManagerT,
>(
    wf: <P as Project>::Workflow,
    active_step_sender: &mut ActiveStepSenderT,
    failed_step_sender: &mut FailedStepSenderT,
    completed_step_sender: &mut CompletedStepSenderT,
    persistence_manager: &mut PersistenceManagerT,
    mut step: FullyQualifiedStep<P>,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    tracing::debug!("Received new step");
    persistence_manager
        .set_step_status(step.step_id, 3)
        .await
        .context("TODO: handle error")?;

    let event = step
        .event
        .clone()
        .unwrap_or(P::Event::from(Immediate).into());

    let next_step = step.step.step.run(wf.clone(), event).await;
    step.retry_count += 1;
    if let Ok(next_step) = next_step {
        completed_step_sender
            .send(FullyQualifiedStep { next_step, ..step })
            .await?;
    } else {
        // tracing::debug!("Failed to run step: {:?}", step.step);

        if step.retry_count <= step.step.settings.max_retries {
            tracing::debug!("Retrying step. Retry count: {}", step.retry_count);
            active_step_sender.send(step).await?;
        } else {
            tracing::debug!("Max retries reached for step: {}", step.step_id);
            failed_step_sender.send(step).await?;
        }
    }

    Ok(())
}

pub async fn main<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistenceManagerT,
>(
    dependencies: ActiveStepWorkerDependencies<
        P,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
        PersistenceManagerT,
    >,
    project: P,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let active_step_receiver = dependencies.active_step_receiver;
    let active_step_sender = dependencies.active_step_sender;
    let failed_step_sender = dependencies.failed_step_sender;
    let completed_step_sender = dependencies.completed_step_sender;
    let persistence_manager = dependencies.persistence_manager;

    loop {
        tracing::info!("Waiting for active step...");
        if let Err(err) = receive_and_process::<
            P,
            ActiveStepReceiverT,
            ActiveStepSenderT,
            FailedStepSenderT,
            CompletedStepSenderT,
            PersistenceManagerT,
        >(
            &active_step_receiver,
            &active_step_sender,
            &failed_step_sender,
            &completed_step_sender,
            &persistence_manager,
            &project,
        )
        .await
        {
            tracing::error!("Error processing active step: {:?}", err);
        }
    }
}

async fn receive_and_process<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistenceManagerT,
>(
    active_step_receiver: &ActiveStepReceiverT,
    active_step_sender: &ActiveStepSenderT,
    failed_step_sender: &FailedStepSenderT,
    completed_step_sender: &CompletedStepSenderT,
    persistence_manager: &PersistenceManagerT,
    project: &P,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let mut active_step_receiver = active_step_receiver.clone();

    let (step, handle) = active_step_receiver.receive().await?;
    let active_step_sender = active_step_sender.clone();
    let failed_step_sender = failed_step_sender.clone();
    let completed_step_sender = completed_step_sender.clone();
    let persistence_manager = persistence_manager.clone();
    let project = project.clone();

    tokio::spawn(async move {
        let wf = project.workflow_for_step(&step.step.step);

        if let Err(err) = process::<
            P,
            ActiveStepSenderT,
            FailedStepSenderT,
            CompletedStepSenderT,
            PersistenceManagerT,
        >(
            wf.clone(),
            &mut active_step_sender.clone(),
            &mut failed_step_sender.clone(),
            &mut completed_step_sender.clone(),
            &mut persistence_manager.clone(),
            step,
        )
        .await
        {
            tracing::error!("Error processing active step: {:?}", err);
        }

        tracing::debug!("acknowledging active step for instance");
        active_step_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge active step: {:?}", e);
            })
            .unwrap();
        tracing::debug!("acknowledged active step for instance");
    });
    Ok(())
}
