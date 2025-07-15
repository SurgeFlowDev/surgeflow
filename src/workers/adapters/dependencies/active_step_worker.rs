use crate::{
    workers::adapters::{
        receivers::ActiveStepReceiver,
        senders::{ActiveStepSender, FailedStepSender, NextStepSender},
    },
    workflows::Workflow,
};

pub struct ActiveStepWorkerDependencies<W: Workflow, C: ActiveStepWorkerContext<W>> {
    pub active_step_receiver: C::ActiveStepReceiver,
    pub active_step_sender: C::ActiveStepSender,
    pub next_step_sender: C::NextStepSender,
    pub failed_step_sender: C::FailedStepSender,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: ActiveStepWorkerContext<W>> ActiveStepWorkerDependencies<W, C> {
    pub fn new(
        active_step_receiver: C::ActiveStepReceiver,
        active_step_sender: C::ActiveStepSender,
        next_step_sender: C::NextStepSender,
        failed_step_sender: C::FailedStepSender,
        context: C,
    ) -> Self {
        Self {
            active_step_receiver,
            active_step_sender,
            next_step_sender,
            failed_step_sender,
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

    // let mut active_step_receiver = RabbitMqActiveStepReceiver::<W>::new(&mut session).await?;
    // let mut active_step_sender = RabbitMqActiveStepSender::<W>::new(&mut session).await?;
    // let mut next_step_sender = RabbitMqNextStepSender::<W>::new(&mut session).await?;
    // let mut failed_step_sender = RabbitMqFailedStepSender::<W>::new(&mut session).await?;
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ActiveStepWorkerDependencies<W, Self>>> + Send;
}
