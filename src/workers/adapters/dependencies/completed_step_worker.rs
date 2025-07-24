use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistenceManager, receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::Project,
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
    pub persistent_step_manager: PersistenceManagerT,
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
        persistent_step_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            completed_step_receiver,
            next_step_sender,
            persistent_step_manager,
            marker: PhantomData,
        }
    }
}
