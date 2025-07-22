use std::marker::PhantomData;

use crate::{
    workers::adapters::{receivers::NewInstanceReceiver, senders::NextStepSender},
    workflows::Workflow,
};

pub struct NewInstanceWorkerDependencies<W, NextStepSenderT, NewInstanceReceiverT>
where
    W: Workflow,
    NextStepSenderT: NextStepSender<W>,
    NewInstanceReceiverT: NewInstanceReceiver<W>,
{
    pub next_step_sender: NextStepSenderT,
    pub new_instance_receiver: NewInstanceReceiverT,
    marker: PhantomData<W>,
}

impl<W, NextStepSenderT, NewInstanceReceiverT>
    NewInstanceWorkerDependencies<W, NextStepSenderT, NewInstanceReceiverT>
where
    W: Workflow,
    NextStepSenderT: NextStepSender<W>,
    NewInstanceReceiverT: NewInstanceReceiver<W>,
{
    pub fn new(
        next_step_sender: NextStepSenderT,
        new_instance_receiver: NewInstanceReceiverT,
    ) -> Self {
        Self {
            next_step_sender,
            new_instance_receiver,
            marker: PhantomData,
        }
    }
}
