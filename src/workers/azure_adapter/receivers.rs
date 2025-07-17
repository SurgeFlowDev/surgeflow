use crate::{
    WorkflowInstance,
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::receivers::{
        ActiveStepReceiver, EventReceiver, InstanceReceiver, NextStepReceiver,
    },
    workflows::Workflow,
};
use azservicebus::{
    ServiceBusClient, ServiceBusReceiver, ServiceBusReceiverOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
    primitives::service_bus_received_message::ServiceBusReceivedMessage,
};
use std::marker::PhantomData;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AzureServiceBusInstanceReceiver<W: Workflow> {
    receiver: Mutex<ServiceBusReceiver>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> InstanceReceiver<W> for AzureServiceBusInstanceReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(WorkflowInstance, ServiceBusReceivedMessage)> {
        let mut receiver = self.receiver.lock().await;
        let message = receiver.receive_message().await?;

        let body = message.body()?;
        let event = match serde_json::from_slice(body) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize workflow instance: {}", e);
                tracing::error!("{}", err);
                receiver.abandon_message(&message, None).await?;
                return Err(err);
            }
        };
        Ok((event, message))
    }

    async fn accept(&mut self, handle: ServiceBusReceivedMessage) -> anyhow::Result<()> {
        let mut receiver = self.receiver.lock().await;
        receiver.complete_message(&handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> AzureServiceBusInstanceReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver: Mutex::new(receiver),
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct AzureServiceBusEventReceiver<W: Workflow> {
    receiver: Mutex<ServiceBusReceiver>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> EventReceiver<W> for AzureServiceBusEventReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(InstanceEvent<W>, ServiceBusReceivedMessage)> {
        let mut receiver = self.receiver.lock().await;
        let message = receiver.receive_message().await?;

        let body = message.body()?;
        let event = match serde_json::from_slice(body) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize event: {}", e);
                tracing::error!("{}", err);
                receiver.abandon_message(&message, None).await?;
                return Err(err);
            }
        };
        Ok((event, message))
    }

    async fn accept(&mut self, handle: ServiceBusReceivedMessage) -> anyhow::Result<()> {
        let mut receiver = self.receiver.lock().await;
        receiver.complete_message(&handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> AzureServiceBusEventReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver: Mutex::new(receiver),
            _marker: PhantomData,
        })
    }
}
#[derive(Debug)]
pub struct AzureServiceBusNextStepReceiver<W: Workflow> {
    receiver: Mutex<ServiceBusReceiver>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> NextStepReceiver<W> for AzureServiceBusNextStepReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, ServiceBusReceivedMessage)> {
        let mut receiver = self.receiver.lock().await;
        let message = receiver.receive_message().await?;

        let body = message.body()?;
        let event = match serde_json::from_slice(body) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                receiver.abandon_message(&message, None).await?;
                return Err(err);
            }
        };
        Ok((event, message))
    }

    async fn accept(&mut self, handle: ServiceBusReceivedMessage) -> anyhow::Result<()> {
        let mut receiver = self.receiver.lock().await;
        receiver.complete_message(&handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> AzureServiceBusNextStepReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver: Mutex::new(receiver),
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct AzureServiceBusActiveStepReceiver<W: Workflow> {
    receiver: Mutex<ServiceBusReceiver>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepReceiver<W> for AzureServiceBusActiveStepReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, ServiceBusReceivedMessage)> {
        let mut receiver = self.receiver.lock().await;
        let message = receiver.receive_message().await?;

        let body = message.body()?;
        let event = match serde_json::from_slice(body) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                receiver.abandon_message(&message, None).await?;
                return Err(err);
            }
        };
        Ok((event, message))
    }

    async fn accept(&mut self, handle: ServiceBusReceivedMessage) -> anyhow::Result<()> {
        let mut receiver = self.receiver.lock().await;
        receiver.complete_message(&handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> AzureServiceBusActiveStepReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver: Mutex::new(receiver),
            _marker: PhantomData,
        })
    }
}
