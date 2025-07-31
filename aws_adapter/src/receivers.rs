use aws_sdk_sqs::Client;
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

use adapter_types::receivers::{
    ActiveStepReceiver, CompletedInstanceReceiver, CompletedStepReceiver, EventReceiver,
    FailedInstanceReceiver, FailedStepReceiver, NewInstanceReceiver, NextStepReceiver,
};
use std::marker::PhantomData;

use crate::AwsAdapterError;

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedInstanceReceiver<P> for AwsSqsCompletedInstanceReceiver<P> {
    type Handle = String;
    type Error = AwsAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsCompletedInstanceReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedInstanceReceiver<P> for AwsSqsFailedInstanceReceiver<P> {
    type Handle = String;
    type Error = AwsAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsFailedInstanceReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

///////////////////////////////////////
///////////////////////////////////////
///////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsNewInstanceReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NewInstanceReceiver<P> for AwsSqsNewInstanceReceiver<P> {
    type Handle = String;
    type Error = AwsAdapterError;
    async fn receive(&mut self) -> Result<(WorkflowInstance, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsNewInstanceReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwsSqsEventReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> EventReceiver<P> for AwsSqsEventReceiver<P> {
    type Error = AwsAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(InstanceEvent<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsEventReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}
#[derive(Debug, Clone)]
pub struct AwsSqsNextStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> NextStepReceiver<P> for AwsSqsNextStepReceiver<P> {
    type Error = AwsAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsNextStepReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsCompletedStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> CompletedStepReceiver<P> for AwsSqsCompletedStepReceiver<P> {
    type Error = AwsAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsCompletedStepReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsFailedStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> FailedStepReceiver<P> for AwsSqsFailedStepReceiver<P> {
    type Error = AwsAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsFailedStepReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AwsSqsActiveStepReceiver<P: Project> {
    receiver: Client,
    queue_url: String,
    _marker: PhantomData<P>,
}

impl<P: Project> ActiveStepReceiver<P> for AwsSqsActiveStepReceiver<P> {
    type Error = AwsAdapterError;
    type Handle = String;
    async fn receive(&mut self) -> Result<(FullyQualifiedStep<P>, Self::Handle), Self::Error> {
        let out = self
            .receiver
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(AwsAdapterError::ReceiveMessageError)?;

        let msg = out
            .messages
            .and_then(|vec| vec.into_iter().next())
            .ok_or(AwsAdapterError::NoMessagesReceived)?;

        let handle = msg
            .receipt_handle
            .ok_or(AwsAdapterError::MessageWithoutReceptHandle)?;

        let event =
            match serde_json::from_str(&msg.body.ok_or(AwsAdapterError::MessageWithoutBody)?) {
                Ok(event) => event,
                Err(e) => {
                    // TODO: handle dead-lettering

                    return Err(AwsAdapterError::DeserializeError(e));
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

impl<P: Project> AwsSqsActiveStepReceiver<P> {
    pub fn new(client: Client, queue_url: String) -> Self {
        Self {
            receiver: client,
            queue_url,
            _marker: PhantomData,
        }
    }
}
