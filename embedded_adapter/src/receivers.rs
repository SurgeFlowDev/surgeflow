use async_channel::Receiver;
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

use adapter_types::receivers::{
    ActiveStepReceiver, CompletedInstanceReceiver, CompletedStepReceiver, EventReceiver,
    FailedInstanceReceiver, FailedStepReceiver, NewInstanceReceiver, NextStepReceiver,
};
use std::marker::PhantomData;

use crate::AwsAdapterError;

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedInstanceReceiver<P: Project> {
    receiver: Receiver<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceReceiver<P> for AwsSqsCompletedInstanceReceiver<P> {
    type Handle = ();
    type Error = AwsAdapterError<P>;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let workflow_instance = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        Ok((workflow_instance, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedInstanceReceiver<P> {
    pub fn new(receiver: Receiver<WorkflowInstance>) -> Self {
        Self {
            receiver,
            _marker: PhantomData,
        }
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedInstanceReceiver<P: Project> {
    receiver: Receiver<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceReceiver<P> for AwsSqsFailedInstanceReceiver<P> {
    type Handle = ();
    type Error = AwsAdapterError<P>;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let workflow_instance = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        Ok((workflow_instance, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsFailedInstanceReceiver<P> {
    pub fn new(receiver: Receiver<WorkflowInstance>) -> Self {
        Self {
            receiver,
            _marker: PhantomData,
        }
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsNewInstanceReceiver<P: Project> {
    receiver: Receiver<WorkflowInstance>,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceReceiver<P> for AwsSqsNewInstanceReceiver<P> {
    type Handle = ();
    type Error = AwsAdapterError<P>;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let workflow_instance = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((workflow_instance, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsNewInstanceReceiver<P> {
    pub fn new(receiver: Receiver<WorkflowInstance>) -> Self {
        Self {
            receiver,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsEventReceiver<P: Project> {
    receiver: Receiver<InstanceEvent<P>>,
}

impl<P: Project> EventReceiver<P> for AwsSqsEventReceiver<P> {
    type Error = AwsAdapterError<P>;
    type Handle = ();
    async fn receive(&mut self) -> Result<(InstanceEvent<P>, Self::Handle), Self::Error> {
        let event = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((event, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsEventReceiver<P> {
    pub fn new(receiver: Receiver<InstanceEvent<P>>) -> Self {
        Self { receiver }
    }
}
#[derive(Debug, Clone)]
pub struct AwsSqsNextStepReceiver<P: Project> {
    receiver: Receiver<FullyQualifiedStep<P>>,
}

impl<P: Project> NextStepReceiver<P> for AwsSqsNextStepReceiver<P> {
    type Error = AwsAdapterError<P>;
    type Handle = ();
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let step = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((step, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsNextStepReceiver<P> {
    pub fn new(receiver: Receiver<FullyQualifiedStep<P>>) -> Self {
        Self { receiver }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedStepReceiver<P: Project> {
    receiver: Receiver<FullyQualifiedStep<P>>,
}

impl<P: Project> CompletedStepReceiver<P> for AwsSqsCompletedStepReceiver<P> {
    type Error = AwsAdapterError<P>;
    type Handle = ();
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let step = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((step, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedStepReceiver<P> {
    pub fn new(receiver: Receiver<FullyQualifiedStep<P>>) -> Self {
        Self { receiver }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedStepReceiver<P: Project> {
    receiver: Receiver<FullyQualifiedStep<P>>,
}

impl<P: Project> FailedStepReceiver<P> for AwsSqsFailedStepReceiver<P> {
    type Error = AwsAdapterError<P>;
    type Handle = ();
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let step = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((step, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsFailedStepReceiver<P> {
    pub fn new(receiver: Receiver<FullyQualifiedStep<P>>) -> Self {
        Self { receiver }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsActiveStepReceiver<P: Project> {
    receiver: Receiver<FullyQualifiedStep<P>>,
}

impl<P: Project> ActiveStepReceiver<P> for AwsSqsActiveStepReceiver<P> {
    type Error = AwsAdapterError<P>;
    type Handle = ();
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let step = self
            .receiver
            .recv()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;
        Ok((step, ()))
    }

    async fn accept(&mut self, _handle: Self::Handle) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<P: Project> AwsSqsActiveStepReceiver<P> {
    pub fn new(receiver: Receiver<FullyQualifiedStep<P>>) -> Self {
        Self { receiver }
    }
}
