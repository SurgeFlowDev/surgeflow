use crate::{
    event::InstanceEvent, step::FullyQualifiedStep, workers::adapters::managers::WorkflowInstance,
    workflows::Workflow,
};

pub trait NextStepSender<W: Workflow>: Sized {
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait ActiveStepSender<W: Workflow>: Sized {
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait FailedStepSender<W: Workflow>: Sized {
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait EventSender<W: Workflow>: Sized {
    fn send(&self, event: InstanceEvent<W>) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait InstanceSender<InstanceSenderW: Workflow>: Sized {
    fn send(&self, event: &WorkflowInstance) -> impl Future<Output = anyhow::Result<()>> + Send;
}
