use crate::{
    workers::adapters::{receivers::CompletedStepReceiver, senders::NextStepSender},
    workflows::Workflow,
};

pub struct CompletedStepWorkerDependencies<W: Workflow, C: CompletedStepWorkerContext<W>> {
    pub completed_step_receiver: C::CompletedStepReceiver,
    pub next_step_sender: C::NextStepSender,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: CompletedStepWorkerContext<W>> CompletedStepWorkerDependencies<W, C> {
    pub fn new(
        completed_step_receiver: C::CompletedStepReceiver,
        next_step_sender: C::NextStepSender,
        context: C,
    ) -> Self {
        Self {
            completed_step_receiver,
            next_step_sender,
            context,
        }
    }
}

pub trait CompletedStepWorkerContext<W: Workflow>: Sized {
    type CompletedStepReceiver: CompletedStepReceiver<W>;
    type NextStepSender: NextStepSender<W>;

    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<CompletedStepWorkerDependencies<W, Self>>> + Send;
}
