use std::marker::PhantomData;

use crate::{
    workers::adapters::{receivers::NewInstanceReceiver, senders::NextStepSender},
    workflows::Project,
};

pub struct NewInstanceWorkerDependencies<P, NextStepSenderT, NewInstanceReceiverT>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
{
    pub next_step_sender: NextStepSenderT,
    pub new_instance_receiver: NewInstanceReceiverT,
    marker: PhantomData<P>,
}

impl<P, NextStepSenderT, NewInstanceReceiverT>
    NewInstanceWorkerDependencies<P, NextStepSenderT, NewInstanceReceiverT>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
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
