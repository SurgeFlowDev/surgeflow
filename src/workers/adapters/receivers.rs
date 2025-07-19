use std::error::Error;

use crate::{
    event::InstanceEvent, step::FullyQualifiedStep, workers::adapters::managers::WorkflowInstance,
    workflows::Workflow,
};

// Steps

pub trait NextStepReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<W::Step>, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ActiveStepReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<W::Step>, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Events

pub trait EventReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(InstanceEvent<W>, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Instances

pub trait NewInstanceReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedInstanceReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedInstanceReceiver<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(&mut self, handle: Self::Handle) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
