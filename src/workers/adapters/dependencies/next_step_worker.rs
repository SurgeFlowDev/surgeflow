use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::{PersistentStepManager, StepsAwaitingEventManager},
        receivers::NextStepReceiver,
        senders::ActiveStepSender,
    },
    workflows::Project,
};

pub struct NextStepWorkerDependencies<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistentStepManagerT,
> where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub next_step_receiver: NextStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<P>,
}

impl<
    P: Project,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistentStepManagerT,
>
    NextStepWorkerDependencies<
        P,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistentStepManagerT,
    >
where
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
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
