use std::marker::PhantomData;

use crate::{
    workers::adapters::{
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
> where
    W: Workflow,
    ActiveStepReceiverT: ActiveStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
{
    pub active_step_receiver: ActiveStepReceiverT,
    pub active_step_sender: ActiveStepSenderT,
    pub failed_step_sender: FailedStepSenderT,
    pub completed_step_sender: CompletedStepSenderT,
    _marker: PhantomData<W>,
}

impl<W, ActiveStepReceiverT, ActiveStepSenderT, FailedStepSenderT, CompletedStepSenderT>
    ActiveStepWorkerDependencies<
        W,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
    >
where
    W: Workflow,
    ActiveStepReceiverT: ActiveStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    FailedStepSenderT: FailedStepSender<W>,
    CompletedStepSenderT: CompletedStepSender<W>,
{
    pub fn new(
        active_step_receiver: ActiveStepReceiverT,
        active_step_sender: ActiveStepSenderT,
        failed_step_sender: FailedStepSenderT,
        completed_step_sender: CompletedStepSenderT,
    ) -> Self {
        Self {
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            _marker: PhantomData,
        }
    }
}
