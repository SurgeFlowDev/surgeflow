use azservicebus::{
    ServiceBusClient, ServiceBusSender, ServiceBusSenderOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
};
use std::marker::PhantomData;

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
        azure_adapter::AzureAdapterError,
    },
    workflows::Project,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AzureServiceBusNextStepSender<P: Project> {
    sender: ServiceBusSender,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepSender<P> for AzureServiceBusNextStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .send_message(serde_json::to_vec(&step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusNextStepSender<P> {
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
pub struct AzureServiceBusActiveStepSender<P: Project> {
    sender: ServiceBusSender,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepSender<P> for AzureServiceBusActiveStepSender<P> {
    type Error = AzureAdapterError;
    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .send_message(serde_json::to_vec(&step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusActiveStepSender<P> {
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
pub struct AzureServiceBusFailedStepSender<P: Project> {
    sender: ServiceBusSender,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepSender<P> for AzureServiceBusFailedStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .send_message(serde_json::to_vec(&step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedStepSender<P> {
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
pub struct AzureServiceBusEventSender<P: Project> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<P>,
}

impl<P: Project> EventSender<P> for AzureServiceBusEventSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: InstanceEvent<P>) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(&step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusEventSender<P> {
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
pub struct AzureServiceBusNewInstanceSender<P: Project> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceSender<P> for AzureServiceBusNewInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusNewInstanceSender<P> {
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

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusFailedInstanceSender<P: Project> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceSender<P> for AzureServiceBusFailedInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedInstanceSender<P> {
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

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusCompletedInstanceSender<P: Project> {
    sender: Mutex<ServiceBusSender>,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceSender<P> for AzureServiceBusCompletedInstanceSender<P> {
    type Error = AzureAdapterError;

    async fn send(&self, step: &WorkflowInstance) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .lock()
            .await
            .send_message(serde_json::to_vec(step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedInstanceSender<P> {
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

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
#[derive(Debug)]
pub struct AzureServiceBusCompletedStepSender<P: Project> {
    sender: ServiceBusSender,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepSender<P> for AzureServiceBusCompletedStepSender<P> {
    type Error = AzureAdapterError;

    async fn send(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        // TODO: using json, could use bincode in production
        self.sender
            .send_message(serde_json::to_vec(&step).map_err(AzureAdapterError::SerializeError)?)
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;
        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedStepSender<P> {
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
