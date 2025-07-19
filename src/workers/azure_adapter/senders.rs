use azservicebus::{
    ServiceBusClient, ServiceBusSender, ServiceBusSenderOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
};
use std::marker::PhantomData;

use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::{
        managers::WorkflowInstance,
        senders::{
            ActiveStepSender, CompletedStepSender, EventSender, FailedStepSender, InstanceSender, NextStepSender
        },
    },
    workflows::Workflow,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AzureServiceBusNextStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> NextStepSender<W> for AzureServiceBusNextStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using json, could use bincode in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusNextStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct AzureServiceBusActiveStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepSender<W> for AzureServiceBusActiveStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using json, could use bincode in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusActiveStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender,
            _marker: PhantomData,
        })
    }
}

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct AzureServiceBusFailedStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> FailedStepSender<W> for AzureServiceBusFailedStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using json, could use bincode in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusFailedStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender,
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
        // TODO: using json, could use bincode in production
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

#[derive(Debug)]
pub struct AzureServiceBusInstanceSender<W: Workflow> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> InstanceSender<W> for AzureServiceBusInstanceSender<W> {
    async fn send(&self, step: &WorkflowInstance) -> anyhow::Result<()> {
        // TODO: using json, could use bincode in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(step)?)
            .await?;

        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusInstanceSender<W> {
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

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct AzureServiceBusCompletedStepSender<W: Workflow> {
    sender: ServiceBusSender,
    _marker: PhantomData<W>,
}

impl<W: Workflow> CompletedStepSender<W> for AzureServiceBusCompletedStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using json, could use bincode in production
        self.sender.send_message(serde_json::to_vec(&step)?).await?;
        Ok(())
    }
}

impl<W: Workflow> AzureServiceBusCompletedStepSender<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let sender = service_bus_client
            .create_sender(queue_name, ServiceBusSenderOptions::default())
            .await?;

        Ok(Self {
            sender,
            _marker: PhantomData,
        })
    }
}