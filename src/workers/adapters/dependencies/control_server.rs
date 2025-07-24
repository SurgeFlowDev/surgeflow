use crate::{
    workers::adapters::{managers::WorkflowInstanceManager, senders::EventSender},
    workflows::Project,
};
use std::future::Future;

pub struct ControlServerDependencies<P: Project, C: ControlServerContext<P>> {
    pub event_sender: C::EventSender,
    pub instance_manager: C::InstanceManager,
    #[expect(dead_code)]
    context: C,
}

impl<P: Project, C: ControlServerContext<P>> ControlServerDependencies<P, C> {
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

pub trait ControlServerContext<P: Project>: Sized {
    type EventSender: EventSender<P>;
    //
    type InstanceManager: WorkflowInstanceManager<P>;
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ControlServerDependencies<P, Self>>> + Send;
}
