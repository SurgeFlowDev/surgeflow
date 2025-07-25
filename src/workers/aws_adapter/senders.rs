use aws_sdk_sqs::Client;
use std::marker::PhantomData;
use uuid::Uuid;

use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::{
        adapters::{
            managers::WorkflowInstance,
            senders::{
                ActiveStepSender, CompletedInstanceSender, CompletedStepSender, EventSender,
                FailedInstanceSender, FailedStepSender, NewInstanceSender, NextStepSender,
            },
        },
        aws_adapter::AzureAdapterError,
    },
    workflows::Project,
};

#[derive(Debug, Clone)]
pub struct AzureServiceBusNextStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepSender<P> for AzureServiceBusNextStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusNextStepSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AzureServiceBusActiveStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepSender<P> for AzureServiceBusActiveStepSender<P> {
    type Error = AzureAdapterError;
    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusActiveStepSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

// TODO: fields should not be pub?
#[derive(Debug, Clone)]
pub struct AzureServiceBusFailedStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepSender<P> for AzureServiceBusFailedStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedStepSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AzureServiceBusEventSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> EventSender<P> for AzureServiceBusEventSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: InstanceEvent<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusEventSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AzureServiceBusNewInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceSender<P> for AzureServiceBusNewInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to send message: {:?}", e);
            })
            .map_err(AzureAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusNewInstanceSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusFailedInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceSender<P> for AzureServiceBusFailedInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedInstanceSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusCompletedInstanceSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceSender<P> for AzureServiceBusCompletedInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedInstanceSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct AzureServiceBusCompletedStepSender<P: Project> {
    sender: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepSender<P> for AzureServiceBusCompletedStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        self.sender
            .send_message()
            .message_group_id(Uuid::new_v4().to_string())
            .message_deduplication_id(Uuid::new_v4().to_string())
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&step).map_err(AzureAdapterError::SerializeError)?)
            .send()
            .await
            .map_err(AzureAdapterError::SendMessageError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedStepSender<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            sender: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}
