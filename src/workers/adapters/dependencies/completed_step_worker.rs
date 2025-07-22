use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager, receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::Workflow,
};

pub struct CompletedStepWorkerDependencies<
    W,
    CompletedStepReceiverT,
    NextStepSenderT,
    PersistentStepManagerT,
> where
    W: Workflow,
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub completed_step_receiver: CompletedStepReceiverT,
    pub next_step_sender: NextStepSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    marker: PhantomData<W>,
}

impl<W: Workflow, CompletedStepReceiverT, NextStepSenderT, PersistentStepManagerT>
    CompletedStepWorkerDependencies<
        W,
        CompletedStepReceiverT,
        NextStepSenderT,
        PersistentStepManagerT,
    >
where
    CompletedStepReceiverT: CompletedStepReceiver<W>,
    NextStepSenderT: NextStepSender<W>,
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
