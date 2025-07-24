use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager, receivers::FailedStepReceiver,
        senders::FailedInstanceSender,
    },
    workflows::Project,
};

pub struct FailedStepWorkerDependencies<
    P,
    FailedStepReceiverT,
    FailedInstanceSenderT,
    PersistentStepManagerT,
> where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub failed_step_receiver: FailedStepReceiverT,
    pub failed_instance_sender: FailedInstanceSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<P>,
}

impl<P, FailedStepReceiverT, FailedInstanceSenderT, PersistentStepManagerT>
    FailedStepWorkerDependencies<
        P,
        FailedStepReceiverT,
        FailedInstanceSenderT,
        PersistentStepManagerT,
    >
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
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
