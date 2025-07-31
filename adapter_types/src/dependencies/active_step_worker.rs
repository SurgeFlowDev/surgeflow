use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::{
    managers::PersistenceManager,
    receivers::ActiveStepReceiver,
    senders::{ActiveStepSender, CompletedStepSender, FailedStepSender},
};

pub struct ActiveStepWorkerDependencies<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistenceManagerT,
> where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub active_step_receiver: ActiveStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub failed_step_sender: FailedStepSenderT,
    pub completed_step_sender: CompletedStepSenderT,
    pub persistence_manager: PersistenceManagerT,
    _marker: PhantomData<P>,
}

impl<
    P,
    ActiveStepReceiverT,
    ActiveStepSenderT,
    FailedStepSenderT,
    CompletedStepSenderT,
    PersistenceManagerT,
>
    ActiveStepWorkerDependencies<
        P,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
        PersistenceManagerT,
    >
where
    P: Project,
    ActiveStepReceiverT: ActiveStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    FailedStepSenderT: FailedStepSender<P>,
    CompletedStepSenderT: CompletedStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    pub fn new(
        active_step_receiver: ActiveStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        failed_step_sender: FailedStepSenderT,
        completed_step_sender: CompletedStepSenderT,
        persistence_manager: PersistenceManagerT,
    ) -> Self {
        Self {
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            persistence_manager,
            _marker: PhantomData,
        }
    }
}
