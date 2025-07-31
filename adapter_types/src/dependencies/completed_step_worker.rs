use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::PersistenceManager, receivers::CompletedStepReceiver, senders::NextStepSender,
};

pub struct CompletedStepWorkerDependencies<
    P,
    CompletedStepReceiverT,
    NextStepSenderT,
    PersistenceManagerT,
> where
    P: Project,
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub completed_step_receiver: CompletedStepReceiverT,
    pub next_step_sender: NextStepSenderT,
    pub persistence_manager: PersistenceManagerT,
    marker: PhantomData<P>,
}

impl<P: Project, CompletedStepReceiverT, NextStepSenderT, PersistenceManagerT>
    CompletedStepWorkerDependencies<P, CompletedStepReceiverT, NextStepSenderT, PersistenceManagerT>
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub fn new(
        completed_step_receiver: CompletedStepReceiverT,
        next_step_sender: NextStepSenderT,
        persistence_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            completed_step_receiver,
            next_step_sender,
            persistence_manager,
            marker: PhantomData,
        }
    }
}
