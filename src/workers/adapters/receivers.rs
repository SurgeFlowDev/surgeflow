use std::error::Error;

use crate::{
    event::InstanceEvent, step::FullyQualifiedStep, workers::adapters::managers::WorkflowInstance,
    workflows::Project,
};

// Steps

pub trait NextStepReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ActiveStepReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedStepReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedStepReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Events

pub trait EventReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(InstanceEvent<P>, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Instances

pub trait NewInstanceReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedInstanceReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedInstanceReceiver<P: Project>: Sized {
    type Error: Error + Send + Sync + 'static;
    type Handle;
    fn receive(
        &mut self,
    ) -> impl Future<Output = Result<(WorkflowInstance, Self::Handle), Self::Error>> + Send;
    fn accept(
        &mut self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
