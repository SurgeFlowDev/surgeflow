use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager,
        receivers::FailedStepReceiver, 
        senders::FailedInstanceSender
    },
    workflows::Workflow,
};

pub struct FailedStepWorkerDependencies<W, FailedStepReceiverT, FailedInstanceSenderT, PersistentStepManagerT>
where
    W: Workflow,
    FailedStepReceiverT: FailedStepReceiver<W>,
    FailedInstanceSenderT: FailedInstanceSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub failed_step_receiver: FailedStepReceiverT,
    pub failed_instance_sender: FailedInstanceSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<W>,
}

impl<W, FailedStepReceiverT, FailedInstanceSenderT, PersistentStepManagerT>
    FailedStepWorkerDependencies<W, FailedStepReceiverT, FailedInstanceSenderT, PersistentStepManagerT>
where
    W: Workflow,
    FailedStepReceiverT: FailedStepReceiver<W>,
    FailedInstanceSenderT: FailedInstanceSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub fn new(
        failed_step_receiver: FailedStepReceiverT,
        failed_instance_sender: FailedInstanceSenderT,
        persistent_step_manager: PersistentStepManagerT,
    ) -> Self {
        Self {
            failed_step_receiver,
            failed_instance_sender,
            persistent_step_manager,
            marker: PhantomData,
        }
    }
}
