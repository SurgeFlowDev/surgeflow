use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::{
        adapters::{
            managers::WorkflowInstance,
            receivers::{
                ActiveStepReceiver, CompletedInstanceReceiver, CompletedStepReceiver,
                EventReceiver, FailedInstanceReceiver, FailedStepReceiver, NewInstanceReceiver,
                NextStepReceiver,
            },
        },
        azure_adapter::AzureAdapterError,
    },
    workflows::Workflow,
};
use azservicebus::{
    ServiceBusClient, ServiceBusReceivedMessage, ServiceBusReceiver, ServiceBusReceiverOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt, receiver::DeadLetterOptions,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct AzureServiceBusCompletedInstanceReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> CompletedInstanceReceiver<W> for AzureServiceBusCompletedInstanceReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<W: Workflow> AzureServiceBusCompletedInstanceReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver,
            _marker: PhantomData,
        })
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusFailedInstanceReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> FailedInstanceReceiver<W> for AzureServiceBusFailedInstanceReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<W: Workflow> AzureServiceBusFailedInstanceReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver,
            _marker: PhantomData,
        })
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusNewInstanceReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> NewInstanceReceiver<W> for AzureServiceBusNewInstanceReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<W: Workflow> AzureServiceBusNewInstanceReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct AzureServiceBusEventReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> EventReceiver<W> for AzureServiceBusEventReceiver<W> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(InstanceEvent<W>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
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
            receiver,
            _marker: PhantomData,
        })
    }
}
#[derive(Debug)]
pub struct AzureServiceBusNextStepReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> NextStepReceiver<W> for AzureServiceBusNextStepReceiver<W> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(
        &mut self,
    ) -> Result<(FullyQualifiedStep<W>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
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
            receiver,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusCompletedStepReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> CompletedStepReceiver<W> for AzureServiceBusCompletedStepReceiver<W> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(
        &mut self,
    ) -> Result<(FullyQualifiedStep<W>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<W: Workflow> AzureServiceBusCompletedStepReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusFailedStepReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> FailedStepReceiver<W> for AzureServiceBusFailedStepReceiver<W> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(
        &mut self,
    ) -> Result<(FullyQualifiedStep<W>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<W: Workflow> AzureServiceBusFailedStepReceiver<W> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusActiveStepReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepReceiver<W> for AzureServiceBusActiveStepReceiver<W> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(
        &mut self,
    ) -> Result<(FullyQualifiedStep<W>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .receive_message()
            .await
            .map_err(AzureAdapterError::ServiceBusError)?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(
            msg.body().map_err(AzureAdapterError::AmqpMessageError)?,
        ) {
            Ok(event) => event,
            Err(e) => {
                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await
                    .map_err(AzureAdapterError::ServiceBusError)?;

                return Err(AzureAdapterError::DeserializeError(e));
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
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
            receiver,
            _marker: PhantomData,
        })
    }
}
