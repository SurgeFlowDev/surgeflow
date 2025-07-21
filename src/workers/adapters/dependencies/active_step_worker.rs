use crate::{
    workers::adapters::{
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, CompletedStepSender, FailedStepSender, NextStepSender},
    },
    workflows::Workflow,
};

pub struct ActiveStepWorkerDependencies<W: Workflow, C: ActiveStepWorkerContext<W>> {
    pub active_step_receiver: C::ActiveStepReceiver,
    pub active_step_sender: C::ActiveStepSender,
    pub failed_step_sender: C::FailedStepSender,
    pub completed_step_sender: C::CompletedStepSender,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: ActiveStepWorkerContext<W>> ActiveStepWorkerDependencies<W, C> {
    pub fn new(
        active_step_receiver: C::ActiveStepReceiver,
        active_step_sender: C::ActiveStepSender,
        failed_step_sender: C::FailedStepSender,
        completed_step_sender: C::CompletedStepSender,
        context: C,
    ) -> Self {
        Self {
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            context,
        }
    }
}

pub trait ActiveStepWorkerContext<W: Workflow>: Sized {
    type ActiveStepReceiver: ActiveStepReceiver<W>;
    //
    type ActiveStepSender: ActiveStepSender<W>;
    //
    type NextStepSender: NextStepSender<W>;
    //
    type FailedStepSender: FailedStepSender<W>;
    //
    type CompletedStepSender: CompletedStepSender<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ActiveStepWorkerDependencies<W, Self>>> + Send;
}
