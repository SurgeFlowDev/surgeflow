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
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct AzureServiceBusCompletedInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceReceiver<P> for AzureServiceBusCompletedInstanceReceiver<P> {
    type Handle = String;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedInstanceReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusFailedInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceReceiver<P> for AzureServiceBusFailedInstanceReceiver<P> {
    type Handle = String;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedInstanceReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusNewInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceReceiver<P> for AzureServiceBusNewInstanceReceiver<P> {
    type Handle = String;
    type Error = AzureAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusNewInstanceReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AzureServiceBusEventReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> EventReceiver<P> for AzureServiceBusEventReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(InstanceEvent<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusEventReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}
#[derive(Debug, Clone)]
pub struct AzureServiceBusNextStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepReceiver<P> for AzureServiceBusNextStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusNextStepReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusCompletedStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepReceiver<P> for AzureServiceBusCompletedStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusCompletedStepReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusFailedStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepReceiver<P> for AzureServiceBusFailedStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusFailedStepReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AzureServiceBusActiveStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepReceiver<P> for AzureServiceBusActiveStepReceiver<P> {
    type Error = AzureAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AzureAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AzureAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AzureAdapterError::MessageWithoutReceptHandle)?;

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
            .queue_url(&self.queue_url)
            .receipt_handle(handle)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

impl<P: Project> AzureServiceBusActiveStepReceiver<P> {
    pub async fn new(client: Client, queue_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        })
    }
}
