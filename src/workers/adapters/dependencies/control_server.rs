use crate::{
    workers::adapters::{managers::WorkflowInstanceManager, senders::EventSender},
    workflows::Workflow,
};

pub struct ControlServerDependencies<W: Workflow, C: ControlServerContext<W>> {
    pub event_sender: C::EventSender,
    pub instance_manager: C::InstanceManager,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: ControlServerContext<W>> ControlServerDependencies<W, C> {
    pub fn new(
        event_sender: C::EventSender,
        instance_manager: C::InstanceManager,
        context: C,
    ) -> Self {
        Self {
            event_sender,
            instance_manager,
            context,
        }
    }
}

pub trait ControlServerContext<W: Workflow>: Sized {
    type EventSender: EventSender<W>;
    //
    type InstanceManager: WorkflowInstanceManager<W>;
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ControlServerDependencies<W, Self>>> + Send;
}
