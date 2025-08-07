use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::PersistenceManager, receivers::FailedStepReceiver, senders::FailedInstanceSender,
};

pub struct FailedStepWorkerDependencies<
    P,
    FailedStepReceiverT,
    FailedInstanceSenderT,
    PersistenceManagerT,
> where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    pub failed_step_receiver: FailedStepReceiverT,
    pub failed_instance_sender: FailedInstanceSenderT,
    pub persistence_manager: PersistenceManagerT,
    marker: PhantomData<P>,
}

impl<P, FailedStepReceiverT, FailedInstanceSenderT, PersistenceManagerT>
    FailedStepWorkerDependencies<P, FailedStepReceiverT, FailedInstanceSenderT, PersistenceManagerT>
where
    P: Project,
    FailedStepReceiverT: FailedStepReceiver<P>,
    FailedInstanceSenderT: FailedInstanceSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    pub fn new(
        failed_step_receiver: FailedStepReceiverT,
        failed_instance_sender: FailedInstanceSenderT,
        persistence_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            failed_step_receiver,
            failed_instance_sender,
            persistence_manager,
            marker: PhantomData,
        }
    }
}
