use crate::{
    workers::adapters::{receivers::InstanceReceiver, senders::NextStepSender},
    workflows::Workflow,
};

pub struct WorkspaceInstanceWorkerDependencies<W: Workflow, C: WorkspaceInstanceWorkerContext<W>> {
    pub next_step_sender: C::NextStepSender,
    pub instance_receiver: C::InstanceReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: WorkspaceInstanceWorkerContext<W>> WorkspaceInstanceWorkerDependencies<W, C> {
    pub fn new(
        next_step_sender: C::NextStepSender,
        instance_receiver: C::InstanceReceiver,
        context: C,
    ) -> Self {
        Self {
            next_step_sender,
            instance_receiver,
            context,
        }
    }
}

pub trait WorkspaceInstanceWorkerContext<W: Workflow>: Sized {
    type NextStepSender: NextStepSender<W>;
    //
    type InstanceReceiver: InstanceReceiver<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<WorkspaceInstanceWorkerDependencies<W, Self>>> + Send;
}
