use crate::{workers::adapters::receivers::CompletedStepReceiver, workflows::Workflow};

pub struct CompletedStepWorkerDependencies<W: Workflow, C: CompletedStepWorkerContext<W>> {
    pub completed_step_receiver: C::CompletedStepReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: CompletedStepWorkerContext<W>> CompletedStepWorkerDependencies<W, C> {
    pub fn new(completed_step_receiver: C::CompletedStepReceiver, context: C) -> Self {
        Self {
            completed_step_receiver,
            context,
        }
    }
}

pub trait CompletedStepWorkerContext<W: Workflow>: Sized {
    type CompletedStepReceiver: CompletedStepReceiver<W>;

    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<CompletedStepWorkerDependencies<W, Self>>> + Send;
}
