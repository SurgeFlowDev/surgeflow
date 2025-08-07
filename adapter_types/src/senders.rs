use std::error::Error;

use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

// Steps

pub trait NextStepSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<P>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ActiveStepSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<P>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedStepSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<P>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedStepSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<P>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Events

pub trait EventSender<P: Project>: Sized + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;
    fn send(&self, event: InstanceEvent<P>)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Instances

pub trait NewInstanceSender<P: Project>: Sized + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;
    fn send(&self, event: WorkflowInstance)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedInstanceSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(&self, event: WorkflowInstance)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedInstanceSender<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn send(&self, event: WorkflowInstance)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}
