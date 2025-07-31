use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::StepsAwaitingEventManager, receivers::EventReceiver, senders::ActiveStepSender,
};

pub struct NewEventWorkerDependencies<
    P,
    ActiveStepSenderT,
    EventReceiverT,
    StepsAwaitingEventManagerT,
> where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    EventReceiverT: EventReceiver<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
{
    pub active_step_sender: ActiveStepSenderT,
    pub event_receiver: EventReceiverT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    marker: PhantomData<P>,
}

impl<P, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>
    NewEventWorkerDependencies<P, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    EventReceiverT: EventReceiver<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
{
    pub fn new(
        active_step_sender: ActiveStepSenderT,
        event_receiver: EventReceiverT,
        steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    ) -> Self {
        Self {
            active_step_sender,
            event_receiver,
            steps_awaiting_event_manager,
            marker: PhantomData,
        }
    }
}
