use crate::{
    workers::adapters::{
        managers::StepsAwaitingEventManager, receivers::NextStepReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};

pub struct NextStepWorkerDependencies<W: Workflow, C: NextStepWorkerContext<W>> {
    pub next_step_receiver: C::NextStepReceiver,
    pub active_step_sender: C::ActiveStepSender,
    pub steps_awaiting_event_manager: C::StepsAwaitingEventManager,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: NextStepWorkerContext<W>> NextStepWorkerDependencies<W, C> {
    pub fn new(
        next_step_receiver: C::NextStepReceiver,
        active_step_sender: C::ActiveStepSender,
        steps_awaiting_event_manager: C::StepsAwaitingEventManager,
        context: C,
    ) -> Self {
        Self {
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            context,
        }
    }
}

pub trait NextStepWorkerContext<W: Workflow>: Sized {
    type NextStepReceiver: NextStepReceiver<W>;
    //
    type ActiveStepSender: ActiveStepSender<W>;
    //
    type StepsAwaitingEventManager: StepsAwaitingEventManager<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<NextStepWorkerDependencies<W, Self>>> + Send;
}
