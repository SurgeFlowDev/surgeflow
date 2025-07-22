use std::marker::PhantomData;

use crate::{
    workers::adapters::{receivers::CompletedStepReceiver, senders::NextStepSender},
    workflows::Workflow,
};

pub struct CompletedStepWorkerDependencies<W, CompletedStepReceiverT, NextStepSenderT>
where
    W: Workflow,
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
{
    pub completed_step_receiver: CompletedStepReceiverT,
    pub next_step_sender: NextStepSenderT,
    marker: PhantomData<W>,
}

impl<W: Workflow, CompletedStepReceiverT, NextStepSenderT>
    CompletedStepWorkerDependencies<W, CompletedStepReceiverT, NextStepSenderT>
where
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
{
    pub fn new(
        completed_step_receiver: CompletedStepReceiverT,
        next_step_sender: NextStepSenderT,
    ) -> Self {
        Self {
            completed_step_receiver,
            next_step_sender,
            marker: PhantomData,
        }
    }
}
