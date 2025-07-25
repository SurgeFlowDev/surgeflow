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
        aws_adapter::AzureAdapterError,
    },
    workflows::Project,
};
use aws_sdk_sqs::Client;
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AzureServiceBusCompletedInstanceReceiver<P: Project> {
    receiver: Client,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceReceiver<P> for AzureServiceBusCompletedInstanceReceiver<P> {
    type Handle = String;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .map(|vec| vec.into_iter().next())
            .flatten()
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;
        // TODO: using json, could use bincode in production
        let event =
            match serde_json::from_str(&msg.body.ok_or(AzureAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AzureAdapterError::DeserializeError(e));
                }
            };
        Ok((event, handle))
    }

    async fn accept(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.receiver
            .delete_message()
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedInstanceReceiver<P> {
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
pub struct AzureServiceBusFailedInstanceReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceReceiver<P> for AzureServiceBusFailedInstanceReceiver<P> {
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

impl<P: Project> AzureServiceBusFailedInstanceReceiver<P> {
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
pub struct AzureServiceBusNewInstanceReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceReceiver<P> for AzureServiceBusNewInstanceReceiver<P> {
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

impl<P: Project> AzureServiceBusNewInstanceReceiver<P> {
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
pub struct AzureServiceBusEventReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> EventReceiver<P> for AzureServiceBusEventReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(InstanceEvent<P>, Self::Handle), Self::Error> {
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

impl<P: Project> AzureServiceBusEventReceiver<P> {
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
pub struct AzureServiceBusNextStepReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepReceiver<P> for AzureServiceBusNextStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
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

impl<P: Project> AzureServiceBusNextStepReceiver<P> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: String,
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
pub struct AzureServiceBusCompletedStepReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepReceiver<P> for AzureServiceBusCompletedStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
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

impl<P: Project> AzureServiceBusCompletedStepReceiver<P> {
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

#[derive(Debug, Clone)]
pub struct AzureServiceBusFailedStepReceiver<P: Project> {
    receiver: Arc<Mutex<ServiceBusReceiver>>,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepReceiver<P> for AzureServiceBusFailedStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let msg = self
            .receiver
            .lock()
            .await
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
                    .lock()
                    .await
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
            .lock()
            .await
            .complete_message(handle)
            .await
            .map_err(AzureAdapterError::ServiceBusError)
    }
}

impl<P: Project> AzureServiceBusFailedStepReceiver<P> {
    pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
        service_bus_client: &mut ServiceBusClient<RP>,
        queue_name: &str,
    ) -> anyhow::Result<Self> {
        let receiver = service_bus_client
            .create_receiver_for_queue(queue_name, ServiceBusReceiverOptions::default())
            .await?;

        Ok(Self {
            receiver: Mutex::new(receiver).into(),
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct AzureServiceBusActiveStepReceiver<P: Project> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepReceiver<P> for AzureServiceBusActiveStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
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

impl<P: Project> AzureServiceBusActiveStepReceiver<P> {
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
