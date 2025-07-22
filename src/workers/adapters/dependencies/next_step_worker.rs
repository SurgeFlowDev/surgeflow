use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::{PersistentStepManager, StepsAwaitingEventManager}, 
        receivers::NextStepReceiver, 
        senders::ActiveStepSender,
    },
    workflows::Workflow,
};

pub struct NextStepWorkerDependencies<
    W,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistentStepManagerT,
> where
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub next_step_receiver: NextStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<W>,
}

impl<W: Workflow, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>
    NextStepWorkerDependencies<W, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistentStepManagerT>
where
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub fn new(
        next_step_receiver: NextStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        steps_awaiting_event_manager: StepsAwaitingEventManagerT,
        persistent_step_manager: PersistentStepManagerT,
    ) -> Self {
        Self {
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            persistent_step_manager,
            marker: PhantomData,
        }
    }
}
