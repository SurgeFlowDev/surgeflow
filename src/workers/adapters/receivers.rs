use crate::{WorkflowInstance, event::InstanceEvent, workflows::Workflow};

pub trait InstanceReceiver<W: Workflow>: Sized {
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<(WorkflowInstance, Self::Handle)>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait EventReceiver<W: Workflow>: Sized {
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<(InstanceEvent<W>, Self::Handle)>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = anyhow::Result<()>> + Send;
}
