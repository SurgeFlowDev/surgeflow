use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager, receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::Project,
};

pub struct CompletedStepWorkerDependencies<
    P,
    CompletedStepReceiverT,
    NextStepSenderT,
    PersistentStepManagerT,
> where
    P: Project,
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub completed_step_receiver: CompletedStepReceiverT,
    pub next_step_sender: NextStepSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<P>,
}

impl<P: Project, CompletedStepReceiverT, NextStepSenderT, PersistentStepManagerT>
    CompletedStepWorkerDependencies<
        P,
        CompletedStepReceiverT,
        NextStepSenderT,
        PersistentStepManagerT,
    >
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub fn new(
        completed_step_receiver: CompletedStepReceiverT,
        next_step_sender: NextStepSenderT,
        persistent_step_manager: PersistentStepManagerT,
    ) -> Self {
        Self {
            completed_step_receiver,
            next_step_sender,
            persistent_step_manager,
            marker: PhantomData,
        }
    }
}
