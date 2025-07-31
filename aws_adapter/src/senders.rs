use aws_sdk_sqs::Client;
use std::marker::PhantomData;
use uuid::Uuid;

use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

use adapter_types::senders::{
    ActiveStepSender, CompletedInstanceSender, CompletedStepSender, EventSender,
    FailedInstanceSender, FailedStepSender, NewInstanceSender, NextStepSender,
};

use crate::AwsAdapterError;

#[derive(Debug, Clone)]
pub struct AwsSqsNextStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepSender<P> for AwsSqsNextStepSender<P> {
    type Error = AwsAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsNextStepSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsActiveStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepSender<P> for AwsSqsActiveStepSender<P> {
    type Error = AwsAdapterError;
    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsActiveStepSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

// TODO: fields should not be pub?
#[derive(Debug, Clone)]
pub struct AwsSqsFailedStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepSender<P> for AwsSqsFailedStepSender<P> {
    type Error = AwsAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsFailedStepSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsEventSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> EventSender<P> for AwsSqsEventSender<P> {
    type Error = AwsAdapterError;

    async fn send(&self, step: InstanceEvent<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsEventSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsNewInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceSender<P> for AwsSqsNewInstanceSender<P> {
    type Error = AwsAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to send message: {:?}", e);
            })
            .map_err(AwsAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsNewInstanceSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceSender<P> for AwsSqsFailedInstanceSender<P> {
    type Error = AwsAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsFailedInstanceSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceSender<P> for AwsSqsCompletedInstanceSender<P> {
    type Error = AwsAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedInstanceSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct AwsSqsCompletedStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepSender<P> for AwsSqsCompletedStepSender<P> {
    type Error = AwsAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AwsAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AwsAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AwsSqsCompletedStepSender<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}
