use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::failed_step_worker::FailedStepWorkerDependencies,
        managers::PersistentStepManager, receivers::FailedStepReceiver,
        senders::FailedInstanceSender,
    },
    workflows::Project,
};
use derive_more::Debug;

pub async fn main<P, FailedStepReceiverT, FailedInstanceSenderT, PersistentStepManagerT>(
    dependencies: FailedStepWorkerDependencies<
        P,
        FailedStepReceiverT,
        FailedInstanceSenderT,
        PersistentStepManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let mut failed_step_receiver = dependencies.failed_step_receiver;
    let mut failed_instance_sender = dependencies.failed_instance_sender;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        if let Err(err) = receive_and_process::<
            P,
            FailedStepReceiverT,
            FailedInstanceSenderT,
            PersistentStepManagerT,
        >(
            &mut failed_step_receiver,
            &mut failed_instance_sender,
            &mut persistent_step_manager,
        )
        .await
        {
            tracing::error!("Error processing failed step: {:?}", err);
        }
    }
}

async fn receive_and_process<
    P,
    FailedStepReceiverT,
    FailedInstanceSenderT,
    PersistentStepManagerT,
>(
    failed_step_receiver: &mut FailedStepReceiverT,
    failed_instance_sender: &mut FailedInstanceSenderT,
    persistent_step_manager: &mut PersistentStepManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let (step, handle) = failed_step_receiver.receive().await?;

    if let Err(err) = process::<P, FailedInstanceSenderT, PersistentStepManagerT>(
        failed_instance_sender,
        persistent_step_manager,
        step,
    )
    .await
    {
        tracing::error!("Error processing failed step: {:?}", err);
    }

    failed_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum FailedStepWorkerError<P, FailedInstanceSenderT, PersistentStepManagerT>
where
    P: Project,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    #[error("Database error occurred")]
    PersistentStepManagerError(#[source] <PersistentStepManagerT as PersistentStepManager>::Error),
    #[error("Failed to send instance: {0}")]
    SendError(#[source] <FailedInstanceSenderT as FailedInstanceSender<P>>::Error),
}

async fn process<P, FailedInstanceSenderT, PersistentStepManagerT>(
    failed_instance_sender: &mut FailedInstanceSenderT,
    persistent_step_manager: &mut PersistentStepManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<(), FailedStepWorkerError<P, FailedInstanceSenderT, PersistentStepManagerT>>
where
    P: Project,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!(
        "received failed step for instance: {}",
        step.instance.external_id
    );

    persistent_step_manager
        .set_step_status(step.step_id, 5)
        .await
        .map_err(FailedStepWorkerError::PersistentStepManagerError)?;

    failed_instance_sender
        .send(&step.instance)
        .await
        .map_err(FailedStepWorkerError::SendError)?;

    Ok(())
}
