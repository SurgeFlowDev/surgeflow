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
    ServiceBusClient, ServiceBusReceivedMessage, ServiceBusReceiver, ServiceBusReceiverOptions,
    primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt, receiver::DeadLetterOptions,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct AzureServiceBusInstanceReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> InstanceReceiver<W> for AzureServiceBusInstanceReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(WorkflowInstance, Self::Handle)> {
        let msg = self.receiver.receive_message().await?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(msg.body()?) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);

                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await?;

                return Err(err);
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> anyhow::Result<()> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(Into::into)
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
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(InstanceEvent<W>, Self::Handle)> {
        let msg = self.receiver.receive_message().await?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(msg.body()?) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);

                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await?;

                return Err(err);
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> anyhow::Result<()> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(Into::into)
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
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)> {
        let msg = self.receiver.receive_message().await?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(msg.body()?) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);

                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await?;

                return Err(err);
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> anyhow::Result<()> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(Into::into)
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
pub struct AzureServiceBusActiveStepReceiver<W: Workflow> {
    receiver: ServiceBusReceiver,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepReceiver<W> for AzureServiceBusActiveStepReceiver<W> {
    type Handle = ServiceBusReceivedMessage;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)> {
        let msg = self.receiver.receive_message().await?;

        // TODO: using json, could use bincode in production
        let event = match serde_json::from_slice(msg.body()?) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);

                self.receiver
                    .dead_letter_message(msg, DeadLetterOptions::default())
                    .await?;

                return Err(err);
            }
        };
        Ok((event, msg))
    }

    async fn accept(&mut self, handle: Self::Handle) -> anyhow::Result<()> {
        self.receiver
            .complete_message(handle)
            .await
            .map_err(Into::into)
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
