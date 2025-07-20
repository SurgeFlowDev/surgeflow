use crate::{
    workers::adapters::{receivers::FailedStepReceiver, senders::FailedInstanceSender},
    workflows::Workflow,
};

pub struct FailedStepWorkerDependencies<W: Workflow, C: FailedStepWorkerContext<W>> {
    pub failed_step_receiver: C::FailedStepReceiver,
    pub failed_instance_sender: C::FailedInstanceSender,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: FailedStepWorkerContext<W>> FailedStepWorkerDependencies<W, C> {
    pub fn new(
        failed_step_receiver: C::FailedStepReceiver,
        failed_instance_sender: C::FailedInstanceSender,
        context: C,
    ) -> Self {
        Self {
            failed_step_receiver,
            failed_instance_sender,
            context,
        }
    }
}

pub trait FailedStepWorkerContext<W: Workflow>: Sized {
    type FailedStepReceiver: FailedStepReceiver<W>;
    //
    type FailedInstanceSender: FailedInstanceSender<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<FailedStepWorkerDependencies<W, Self>>> + Send;
}
