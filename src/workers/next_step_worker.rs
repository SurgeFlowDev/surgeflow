use crate::{
    event::Immediate,
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerDependencies,
        managers::{PersistenceManager, StepsAwaitingEventManager},
        receivers::NextStepReceiver,
        senders::ActiveStepSender,
    },
    workflows::{Project, ProjectStep},
};
use derive_more::Debug;

pub async fn main<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
>(
    dependencies: NextStepWorkerDependencies<
        P,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistenceManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager,
{
    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;
    let mut persistence_manager = dependencies.persistence_manager;

    loop {
        if let Err(err) = receive_and_process::<
            P,
            NextStepReceiverT,
            ActiveStepSenderT,
            StepsAwaitingEventManagerT,
            PersistenceManagerT,
        >(
            &mut next_step_receiver,
            &mut active_step_sender,
            &mut steps_awaiting_event_manager,
            &mut persistence_manager,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }
    }
}

async fn receive_and_process<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
>(
    next_step_receiver: &mut NextStepReceiverT,
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistence_manager: &mut PersistenceManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager,
{
    let (step, handle) = next_step_receiver.receive().await?;

    if let Err(err) =
        process::<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>(
            active_step_sender,
            steps_awaiting_event_manager,
            persistence_manager,
            step,
        )
        .await
    {
        tracing::error!("Error processing next step: {:?}", err);
    }

    next_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager,
{
    #[error("Database error occurred")]
    DatabaseError(#[source] <PersistenceManagerT as PersistenceManager>::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] <ActiveStepSenderT as ActiveStepSender<P>>::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] <StepsAwaitingEventManagerT as StepsAwaitingEventManager<P>>::Error),
}

async fn process<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>(
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistence_manager: &mut PersistenceManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<
    (),
    NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>,
>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager,
{
    tracing::info!(
        "received next step for instance: {}",
        step.instance.external_id
    );

    persistence_manager
        .insert_step::<P>(step.instance.external_id, step.step_id, &step.step.step)
        .await
        .map_err(NextStepWorkerError::DatabaseError)?;

    if step.step.step.is_event::<Immediate>() {
        active_step_sender
            .send(step)
            .await
            .map_err(NextStepWorkerError::SendActiveStepError)?;
    } else {
        steps_awaiting_event_manager
            .put_step(step)
            .await
            .map_err(NextStepWorkerError::AwaitEventError)?;
    }

    Ok(())
}
