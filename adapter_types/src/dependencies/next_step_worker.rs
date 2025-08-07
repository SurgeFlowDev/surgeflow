use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::{PersistenceManager, StepsAwaitingEventManager},
    receivers::NextStepReceiver,
    senders::ActiveStepSender,
};

pub struct NextStepWorkerDependencies<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
> where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    pub next_step_receiver: NextStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    pub persistence_manager: PersistenceManagerT,
    marker: PhantomData<P>,
}

impl<
    P: Project,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
>
    NextStepWorkerDependencies<
        P,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistenceManagerT,
    >
where
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    pub fn new(
        next_step_receiver: NextStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        steps_awaiting_event_manager: StepsAwaitingEventManagerT,
        persistence_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            persistence_manager,
            marker: PhantomData,
        }
    }
}
