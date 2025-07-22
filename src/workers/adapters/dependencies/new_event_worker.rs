use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::StepsAwaitingEventManager, receivers::EventReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};

pub struct NewEventWorkerDependencies<
    W,
    ActiveStepSenderT,
    EventReceiverT,
    StepsAwaitingEventManagerT,
> where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    EventReceiverT: EventReceiver<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    pub active_step_sender: ActiveStepSenderT,
    pub event_receiver: EventReceiverT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    marker: PhantomData<W>,
}

impl<W, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>
    NewEventWorkerDependencies<W, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    EventReceiverT: EventReceiver<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
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
