use crate::{
    workers::adapters::{managers::WorkflowInstanceManager, senders::EventSender},
    workflows::Project,
};
use std::future::Future;

pub struct ControlServerDependencies<P: Project, ES: EventSender<P>, IM: WorkflowInstanceManager<P>> {
    pub event_sender: ES,
    pub instance_manager: IM,
    #[expect(dead_code)]
    _marker: std::marker::PhantomData<P>,
}

impl<P: Project, ES: EventSender<P>, IM: WorkflowInstanceManager<P>> ControlServerDependencies<P, ES, IM> {
    pub fn new(
        event_sender: ES,
        instance_manager: IM,
    ) -> Self {
        Self {
            event_sender,
            instance_manager,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait ControlServerContext<P: Project>: Sized {
    type EventSender: EventSender<P>;
    //
    type InstanceManager: WorkflowInstanceManager<P>;
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ControlServerDependencies<P, Self::EventSender, Self::InstanceManager>>> + Send;
}
