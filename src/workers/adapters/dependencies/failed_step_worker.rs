use crate::{workers::adapters::receivers::FailedStepReceiver, workflows::Workflow};

pub struct FailedStepWorkerDependencies<W: Workflow, C: FailedStepWorkerContext<W>> {
    pub failed_step_receiver: C::FailedStepReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: FailedStepWorkerContext<W>> FailedStepWorkerDependencies<W, C> {
    pub fn new(failed_step_receiver: C::FailedStepReceiver, context: C) -> Self {
        Self {
            failed_step_receiver,
            context,
        }
    }
}

pub trait FailedStepWorkerContext<W: Workflow>: Sized {
    type FailedStepReceiver: FailedStepReceiver<W>;

    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<FailedStepWorkerDependencies<W, Self>>> + Send;
}
