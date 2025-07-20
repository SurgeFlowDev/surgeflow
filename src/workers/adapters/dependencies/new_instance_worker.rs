use crate::{
    workers::adapters::{receivers::NewInstanceReceiver, senders::NextStepSender},
    workflows::Workflow,
};

pub struct NewInstanceWorkerDependencies<W: Workflow, C: NewInstanceWorkerContext<W>> {
    pub next_step_sender: C::NextStepSender,
    pub instance_receiver: C::InstanceReceiver,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: NewInstanceWorkerContext<W>> NewInstanceWorkerDependencies<W, C> {
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

pub trait NewInstanceWorkerContext<W: Workflow>: Sized {
    type NextStepSender: NextStepSender<W>;
    //
    type InstanceReceiver: NewInstanceReceiver<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<NewInstanceWorkerDependencies<W, Self>>> + Send;
}
