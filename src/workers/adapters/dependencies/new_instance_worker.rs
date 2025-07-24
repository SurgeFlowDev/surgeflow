use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistenceManager, receivers::NewInstanceReceiver, senders::NextStepSender,
    },
    workflows::Project,
};

pub struct NewInstanceWorkerDependencies<
    P,
    NextStepSenderT,
    NewInstanceReceiverT,
    PersistenceManagerT,
> where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub next_step_sender: NextStepSenderT,
    pub new_instance_receiver: NewInstanceReceiverT,
    pub persistent_step_manager: PersistenceManagerT,
    marker: PhantomData<P>,
}

impl<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>
    NewInstanceWorkerDependencies<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub fn new(
        next_step_sender: NextStepSenderT,
        new_instance_receiver: NewInstanceReceiverT,
        persistent_step_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            next_step_sender,
            new_instance_receiver,
            persistent_step_manager,
            marker: PhantomData,
        }
    }
}
