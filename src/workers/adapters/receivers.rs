use crate::{
    event::InstanceEvent, step::FullyQualifiedStep, workers::adapters::managers::WorkflowInstance,
    workflows::Workflow,
};

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

pub trait NextStepReceiver<W: Workflow>: Sized {
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = anyhow::Result<()>> + Send;
}
pub trait ActiveStepReceiver<W: Workflow>: Sized {
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = anyhow::Result<()>> + Send;
}
