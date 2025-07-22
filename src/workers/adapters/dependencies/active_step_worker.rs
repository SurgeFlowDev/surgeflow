use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, CompletedStepSender, FailedStepSender},
    },
    workflows::Workflow,
};

pub struct ActiveStepWorkerDependencies<
    W,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistentStepManagerT,
> where
    W: Workflow,
    ActiveStepReceiverT: ActiveStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub active_step_receiver: ActiveStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub failed_step_sender: FailedStepSenderT,
    pub completed_step_sender: CompletedStepSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    _marker: PhantomData<W>,
}

impl<W, ActiveStepReceiverT, ActiveStepSenderT, FailedStepSenderT, CompletedStepSenderT, PersistentStepManagerT>
    ActiveStepWorkerDependencies<
        W,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
        PersistentStepManagerT,
    >
where
    W: Workflow,
    ActiveStepReceiverT: ActiveStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub fn new(
        active_step_receiver: ActiveStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        failed_step_sender: FailedStepSenderT,
        completed_step_sender: CompletedStepSenderT,
        persistent_step_manager: PersistentStepManagerT,
    ) -> Self {
        Self {
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            persistent_step_manager,
            _marker: PhantomData,
        }
    }
}
