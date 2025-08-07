use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::PersistenceManager, receivers::NewInstanceReceiver, senders::NextStepSender,
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
    PersistenceManagerT: PersistenceManager<P>,
{
    pub next_step_sender: NextStepSenderT,
    pub new_instance_receiver: NewInstanceReceiverT,
    pub persistence_manager: PersistenceManagerT,
    marker: PhantomData<P>,
}

impl<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>
    NewInstanceWorkerDependencies<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    pub fn new(
        next_step_sender: NextStepSenderT,
        new_instance_receiver: NewInstanceReceiverT,
        persistence_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            next_step_sender,
            new_instance_receiver,
            persistence_manager,
            marker: PhantomData,
        }
    }
}
