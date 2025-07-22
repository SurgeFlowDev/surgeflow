use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerDependencies,
        managers::{PersistentStepManager, StepsAwaitingEventManager},
        receivers::NextStepReceiver,
        senders::ActiveStepSender,
    },
    workflows::Workflow,
};
use derive_more::Debug;
use std::any::TypeId;

pub async fn main<
    W,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistentStepManagerT,
>(
    dependencies: NextStepWorkerDependencies<
        W,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistentStepManagerT,
    >,
) -> anyhow::Result<()>
where
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        if let Err(err) = receive_and_process::<
            W,
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
    W,
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
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    let (step, handle) = next_step_receiver.receive().await?;

    if let Err(err) =
        process::<W, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>(
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
enum NextStepWorkerError<W, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    #[error("Database error occurred")]
    DatabaseError(#[source] <PersistentStepManagerT as PersistentStepManager>::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] <ActiveStepSenderT as ActiveStepSender<W>>::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] <StepsAwaitingEventManagerT as StepsAwaitingEventManager<W>>::Error),
}

async fn process<W, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>(
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistent_step_manager: &mut PersistentStepManagerT,
    step: FullyQualifiedStep<W>,
) -> Result<
    (),
    NextStepWorkerError<W, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>,
>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!(
        "received next step for instance: {}",
        step.instance.external_id
    );

    persistent_step_manager
        .insert_step::<W>(step.instance.external_id, step.step_id, &step.step.step)
        .await
        .map_err(NextStepWorkerError::DatabaseError)?;

    if step.step.step.variant_event_type_id() == TypeId::of::<Immediate<W>>() {
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
