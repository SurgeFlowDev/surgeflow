use async_channel::Sender;
use std::marker::PhantomData;

use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

use adapter_types::senders::{
    ActiveStepSender, CompletedInstanceSender, CompletedStepSender, EventSender,
    FailedInstanceSender, FailedStepSender, NewInstanceSender, NextStepSender,
};

use crate::AwsAdapterError;

#[derive(Debug, Clone)]
pub struct AwsSqsNextStepSender<P: Project> {
    sender: Sender<FullyQualifiedStep<P>>,
}

impl<P: Project> NextStepSender<P> for AwsSqsNextStepSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendStepError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsNextStepSender<P> {
    pub fn new(sender: Sender<FullyQualifiedStep<P>>) -> Self {
        Self { sender }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsActiveStepSender<P: Project> {
    sender: Sender<FullyQualifiedStep<P>>,
}

impl<P: Project> ActiveStepSender<P> for AwsSqsActiveStepSender<P> {
    type Error = AwsAdapterError<P>;
    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendStepError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsActiveStepSender<P> {
    pub fn new(sender: Sender<FullyQualifiedStep<P>>) -> Self {
        Self { sender }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsFailedStepSender<P: Project> {
    sender: Sender<FullyQualifiedStep<P>>,
}

impl<P: Project> FailedStepSender<P> for AwsSqsFailedStepSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendStepError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsFailedStepSender<P> {
    pub fn new(sender: Sender<FullyQualifiedStep<P>>) -> Self {
        Self { sender }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsEventSender<P: Project> {
    sender: Sender<InstanceEvent<P>>,
}

impl<P: Project> EventSender<P> for AwsSqsEventSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&self, step: InstanceEvent<P>) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendInstanceEventError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsEventSender<P> {
    pub fn new(sender: Sender<InstanceEvent<P>>) -> Self {
        Self { sender }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsNewInstanceSender<P: Project> {
    sender: Sender<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceSender<P> for AwsSqsNewInstanceSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&self, step: WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendWorkflowInstanceEventError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsNewInstanceSender<P> {
    pub fn new(sender: Sender<WorkflowInstance>) -> Self {
        Self {
            sender,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedInstanceSender<P: Project> {
    sender: Sender<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceSender<P> for AwsSqsFailedInstanceSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&self, step: WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendWorkflowInstanceEventError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsFailedInstanceSender<P> {
    pub fn new(sender: Sender<WorkflowInstance>) -> Self {
        Self {
            sender,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedInstanceSender<P: Project> {
    sender: Sender<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceSender<P> for AwsSqsCompletedInstanceSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&self, step: WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendWorkflowInstanceEventError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedInstanceSender<P> {
    pub fn new(sender: Sender<WorkflowInstance>) -> Self {
        Self {
            sender,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedStepSender<P: Project> {
    sender: Sender<FullyQualifiedStep<P>>,
}

impl<P: Project> CompletedStepSender<P> for AwsSqsCompletedStepSender<P> {
    type Error = AwsAdapterError<P>;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send(step)
            .await
            .map_err(AwsAdapterError::SendStepError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedStepSender<P> {
    pub fn new(sender: Sender<FullyQualifiedStep<P>>) -> Self {
        Self { sender }
    }
}
