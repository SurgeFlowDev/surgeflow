use std::marker::PhantomData;

use crate::{
    workers::adapters::{
        managers::PersistentStepManager,
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, CompletedStepSender, FailedStepSender},
    },
    workflows::Project,
};

pub struct ActiveStepWorkerDependencies<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistentStepManagerT,
> where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    pub active_step_receiver: ActiveStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub failed_step_sender: FailedStepSenderT,
    pub completed_step_sender: CompletedStepSenderT,
    pub persistent_step_manager: PersistentStepManagerT,
    _marker: PhantomData<P>,
}

impl<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistentStepManagerT,
>
    ActiveStepWorkerDependencies<
        P,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
        PersistentStepManagerT,
    >
where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
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
