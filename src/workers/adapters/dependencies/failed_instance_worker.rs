use crate::{workers::adapters::receivers::FailedInstanceReceiver, workflows::Workflow};

pub struct FailedInstanceWorkerDependencies<W: Workflow, C: FailedInstanceWorkerContext<W>> {
    pub instance_receiver: C::FailedInstanceReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: FailedInstanceWorkerContext<W>> FailedInstanceWorkerDependencies<W, C> {
    pub fn new(instance_receiver: C::FailedInstanceReceiver, context: C) -> Self {
        Self {
            instance_receiver,
            context,
        }
    }
}

pub trait FailedInstanceWorkerContext<W: Workflow>: Sized {
    type FailedInstanceReceiver: FailedInstanceReceiver<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<FailedInstanceWorkerDependencies<W, Self>>> + Send;
}
