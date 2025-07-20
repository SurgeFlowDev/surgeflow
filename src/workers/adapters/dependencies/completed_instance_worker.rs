use crate::{workers::adapters::receivers::CompletedInstanceReceiver, workflows::Workflow};

pub struct CompletedInstanceWorkerDependencies<W: Workflow, C: CompletedInstanceWorkerContext<W>> {
    pub instance_receiver: C::CompletedInstanceReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: CompletedInstanceWorkerContext<W>> CompletedInstanceWorkerDependencies<W, C> {
    pub fn new(instance_receiver: C::CompletedInstanceReceiver, context: C) -> Self {
        Self {
            instance_receiver,
            context,
        }
    }
}

pub trait CompletedInstanceWorkerContext<W: Workflow>: Sized {
    type CompletedInstanceReceiver: CompletedInstanceReceiver<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<CompletedInstanceWorkerDependencies<W, Self>>> + Send;
}
