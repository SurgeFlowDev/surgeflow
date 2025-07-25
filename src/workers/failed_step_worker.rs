use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::failed_step_worker::FailedStepWorkerDependencies,
        managers::PersistenceManager, receivers::FailedStepReceiver, senders::FailedInstanceSender,
    },
    workflows::Project,
};
use derive_more::Debug;

pub async fn main<P, FailedStepReceiverT, FailedInstanceSenderT, PersistenceManagerT>(
    dependencies: FailedStepWorkerDependencies<
        P,
        FailedStepReceiverT,
        FailedInstanceSenderT,
        PersistenceManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    let failed_step_receiver = dependencies.failed_step_receiver;
    let failed_instance_sender = dependencies.failed_instance_sender;
    let persistence_manager = dependencies.persistence_manager;

    loop {
        if let Err(err) = receive_and_process::<
            P,
            FailedStepReceiverT,
            FailedInstanceSenderT,
            PersistenceManagerT,
        >(
            &failed_step_receiver,
            &failed_instance_sender,
            &persistence_manager,
        )
        .await
        {
            tracing::error!("Error processing failed step: {:?}", err);
        }
    }
}

async fn receive_and_process<P, FailedStepReceiverT, FailedInstanceSenderT, PersistenceManagerT>(
    failed_step_receiver: &FailedStepReceiverT,
    failed_instance_sender: &FailedInstanceSenderT,
    persistence_manager: &PersistenceManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    let mut failed_step_receiver = failed_step_receiver.clone();

    let (step, handle) = failed_step_receiver.receive().await?;
    let failed_instance_sender = failed_instance_sender.clone();
    let persistence_manager = persistence_manager.clone();

    let block = async move {
        if let Err(err) = process::<P, FailedInstanceSenderT, PersistenceManagerT>(
            failed_instance_sender,
            persistence_manager,
            step,
        )
        .await
        {
            tracing::error!("Error processing failed step: {:?}", err);
        }

        tracing::info!("acknowledging failed step for instance");
        failed_step_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge failed step: {:?}", e);
            })
            .unwrap();
        tracing::info!("acknowledged failed step for instance");
    };
    tokio::spawn(block);
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum FailedStepWorkerError<P, FailedInstanceSenderT, PersistenceManagerT>
where
    P: Project,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    #[error("Database error occurred")]
    PersistenceManagerError(#[source] <PersistenceManagerT as PersistenceManager>::Error),
    #[error("Failed to send instance: {0}")]
    SendError(#[source] <FailedInstanceSenderT as FailedInstanceSender<P>>::Error),
}

async fn process<P, FailedInstanceSenderT, PersistenceManagerT>(
    failed_instance_sender: FailedInstanceSenderT,
    persistence_manager: PersistenceManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<(), FailedStepWorkerError<P, FailedInstanceSenderT, PersistenceManagerT>>
where
    P: Project,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    tracing::info!(
        "received failed step for instance: {}",
        step.instance.external_id
    );

    persistence_manager
        .set_step_status(step.step_id, 5)
        .await
        .map_err(FailedStepWorkerError::PersistenceManagerError)?;

    failed_instance_sender
        .send(&step.instance)
        .await
        .map_err(FailedStepWorkerError::SendError)?;

    Ok(())
}
