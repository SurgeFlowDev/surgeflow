use crate::{
    event::Immediate,
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerDependencies,
        managers::{PersistentStepManager, StepsAwaitingEventManager},
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
    PersistentStepManagerT,
>(
    dependencies: NextStepWorkerDependencies<
        P,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistentStepManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        if let Err(err) = receive_and_process::<
            P,
            NextStepReceiverT,
            ActiveStepSenderT,
            StepsAwaitingEventManagerT,
            PersistentStepManagerT,
        >(
            &mut next_step_receiver,
            &mut active_step_sender,
            &mut steps_awaiting_event_manager,
            &mut persistent_step_manager,
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
    PersistentStepManagerT,
>(
    next_step_receiver: &mut NextStepReceiverT,
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistent_step_manager: &mut PersistentStepManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let (step, handle) = next_step_receiver.receive().await?;

    if let Err(err) =
        process::<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>(
            active_step_sender,
            steps_awaiting_event_manager,
            persistent_step_manager,
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
enum NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    #[error("Database error occurred")]
    DatabaseError(#[source] <PersistentStepManagerT as PersistentStepManager>::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] <ActiveStepSenderT as ActiveStepSender<P>>::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] <StepsAwaitingEventManagerT as StepsAwaitingEventManager<P>>::Error),
}

async fn process<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>(
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistent_step_manager: &mut PersistentStepManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<
    (),
    NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>,
>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!(
        "received next step for instance: {}",
        step.instance.external_id
    );

    persistent_step_manager
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
    // active_step_sender
    //         .send(step)
    //         .await
    //         .map_err(NextStepWorkerError::SendActiveStepError)?;

    Ok(())
}
