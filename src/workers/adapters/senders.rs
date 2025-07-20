use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    event::InstanceEvent, step::FullyQualifiedStep, workers::adapters::managers::WorkflowInstance,
    workflows::Workflow,
};

// Steps

pub trait NextStepSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait ActiveStepSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedStepSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedStepSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Events

pub trait EventSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(&self, event: InstanceEvent<W>)
    -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// Instances

pub trait NewInstanceSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &self,
        event: &WorkflowInstance,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait CompletedInstanceSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &self,
        event: &WorkflowInstance,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait FailedInstanceSender<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn send(
        &self,
        event: &WorkflowInstance,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
