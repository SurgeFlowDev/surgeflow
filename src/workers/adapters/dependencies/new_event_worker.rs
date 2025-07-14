use crate::{
    workers::adapters::{
        managers::StepsAwaitingEventManager, receivers::EventReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};

pub struct NewEventWorkerDependencies<W: Workflow, C: NewEventWorkerContext<W>> {
    pub active_step_sender: C::ActiveStepSender,
    pub event_receiver: C::EventReceiver,
    pub steps_awaiting_event_manager: C::StepsAwaitingEventManager,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: NewEventWorkerContext<W>> NewEventWorkerDependencies<W, C> {
    pub fn new(
        active_step_sender: C::ActiveStepSender,
        event_receiver: C::EventReceiver,
        steps_awaiting_event_manager: C::StepsAwaitingEventManager,
        context: C,
    ) -> Self {
        Self {
            active_step_sender,
            event_receiver,
            steps_awaiting_event_manager,
            context,
        }
    }
}

pub trait NewEventWorkerContext<W: Workflow>: Sized {
    type ActiveStepSender: ActiveStepSender<W>;
    //
    type EventReceiver: EventReceiver<W>;
    //
    type StepsAwaitingEventManager: StepsAwaitingEventManager<W>;
    fn dependencies()
    -> impl Future<Output = anyhow::Result<NewEventWorkerDependencies<W, Self>>> + Send;
}
