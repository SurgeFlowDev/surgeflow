use std::marker::PhantomData;

use crate::{
    workers::adapters::{receivers::FailedStepReceiver, senders::FailedInstanceSender},
    workflows::Workflow,
};

pub struct FailedStepWorkerDependencies<W, FailedStepReceiverT, FailedInstanceSenderT>
where
    W: Workflow,
    FailedStepReceiverT: FailedStepReceiver<W>,
    FailedInstanceSenderT: FailedInstanceSender<W>,
{
    pub failed_step_receiver: FailedStepReceiverT,
    pub failed_instance_sender: FailedInstanceSenderT,
    marker: PhantomData<W>,
}

impl<W, FailedStepReceiverT, FailedInstanceSenderT>
    FailedStepWorkerDependencies<W, FailedStepReceiverT, FailedInstanceSenderT>
where
    W: Workflow,
    FailedStepReceiverT: FailedStepReceiver<W>,
    FailedInstanceSenderT: FailedInstanceSender<W>,
{
    pub fn new(
        failed_step_receiver: FailedStepReceiverT,
        failed_instance_sender: FailedInstanceSenderT,
    ) -> Self {
        Self {
            failed_step_receiver,
            failed_instance_sender,
            marker: PhantomData,
        }
    }
}
