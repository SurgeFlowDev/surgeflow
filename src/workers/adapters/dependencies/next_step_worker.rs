use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::StepsAwaitingEventManager, receivers::NextStepReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};

pub struct NextStepWorkerDependencies<
    W,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
> where
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    pub next_step_receiver: NextStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    marker: PhantomData<W>,
}

impl<W: Workflow, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT>
    NextStepWorkerDependencies<W, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT>
where
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    pub fn new(
        next_step_receiver: NextStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        steps_awaiting_event_manager: StepsAwaitingEventManagerT,
    ) -> Self {
        Self {
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            marker: PhantomData,
        }
    }
}
