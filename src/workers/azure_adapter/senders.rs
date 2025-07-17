use azservicebus::{
    ServiceBusClient, ServiceBusSender, ServiceBusSenderOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
};
use std::marker::PhantomData;

use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::senders::{ActiveStepSender, EventSender, FailedStepSender, NextStepSender},
    workflows::Workflow,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct RabbitMqNextStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> NextStepSender<W> for RabbitMqNextStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqNextStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender: sender,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct RabbitMqActiveStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepSender<W> for RabbitMqActiveStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqActiveStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender: sender,
            _marker: PhantomData,
        })
    }
}

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct RabbitMqFailedStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> FailedStepSender<W> for RabbitMqFailedStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqFailedStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender: sender,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct AzureServiceBusEventSender<W: Workflow> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> EventSender<W> for AzureServiceBusEventSender<W> {
    async fn send(&self, step: InstanceEvent<W>) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(&step)?)
            .await?;

        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusEventSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender: Mutex::new(sender),
            _marker: PhantomData,
        })
    }
}
