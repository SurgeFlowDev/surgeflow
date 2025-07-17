use std::{marker::PhantomData, sync::Arc};

use azservicebus::{
    ServiceBusClient, ServiceBusClientOptions, ServiceBusMessage, ServiceBusRetryPolicy,
    ServiceBusSender, ServiceBusSenderOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
};
use azure_core::{HttpClient, RetryPolicy};
use azure_messaging_servicebus::{
    prelude::QueueClient,
    service_bus::{SendMessageOptions, SettableBrokerProperties},
};
use fe2o3_amqp::{Sender, session::SessionHandle};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::senders::{ActiveStepSender, EventSender, FailedStepSender, NextStepSender},
    workflows::Workflow,
};

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct RabbitMqNextStepSender<W: Workflow>(pub Sender, pub PhantomData<W>);

impl<W: Workflow> NextStepSender<W> for RabbitMqNextStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqNextStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-next-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
}

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct RabbitMqActiveStepSender<W: Workflow>(pub Sender, pub PhantomData<W>);

impl<W: Workflow> ActiveStepSender<W> for RabbitMqActiveStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqActiveStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
}

// TODO: fields should not be pub?
#[derive(Debug)]
pub struct RabbitMqFailedStepSender<W: Workflow>(pub Sender, pub PhantomData<W>);

impl<W: Workflow> FailedStepSender<W> for RabbitMqFailedStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

impl<W: Workflow> RabbitMqFailedStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-failed-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
}

#[derive(Debug)]
pub struct AzureServiceBusEventSender<W: Workflow> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> EventSender<W> for AzureServiceBusEventSender<W> {
    async fn send(&self, step: InstanceEvent<W>) -> anyhow::Result<()> {
        let mut sender = self.sender.lock().await;
        sender.send_message(serde_json::to_vec(&step)?).await?;

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
