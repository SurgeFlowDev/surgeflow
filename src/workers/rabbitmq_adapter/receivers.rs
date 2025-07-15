use crate::{
    WorkflowInstance,
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::receivers::{
        ActiveStepReceiver, EventReceiver, InstanceReceiver, NextStepReceiver,
    },
    workflows::Workflow,
};
use fe2o3_amqp::{Receiver, link::delivery::DeliveryInfo, session::SessionHandle};
use std::marker::PhantomData;
use uuid::Uuid;

// TODO: fields should be pub?
#[derive(Debug)]
pub struct RabbitMqInstanceReceiver<W: Workflow>(pub Receiver, pub PhantomData<W>);

impl<W: Workflow> InstanceReceiver<W> for RabbitMqInstanceReceiver<W> {
    type Handle = DeliveryInfo;
    async fn receive(&mut self) -> anyhow::Result<(WorkflowInstance, DeliveryInfo)> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        Ok((event, msg.into()))
    }

    async fn accept(&mut self, handle: DeliveryInfo) -> anyhow::Result<()> {
        self.0.accept(handle).await.map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct RabbitMqEventReceiver<W: Workflow>(pub Receiver, pub PhantomData<W>);

impl<W: Workflow> EventReceiver<W> for RabbitMqEventReceiver<W> {
    type Handle = DeliveryInfo;
    async fn receive(&mut self) -> anyhow::Result<(InstanceEvent<W>, DeliveryInfo)> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        Ok((event, msg.into()))
    }

    async fn accept(&mut self, handle: DeliveryInfo) -> anyhow::Result<()> {
        self.0.accept(handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> RabbitMqEventReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-events", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
}
#[derive(Debug)]
pub struct RabbitMqNextStepReceiver<W: Workflow>(pub Receiver, pub PhantomData<W>);

impl<W: Workflow> NextStepReceiver<W> for RabbitMqNextStepReceiver<W> {
    type Handle = DeliveryInfo;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        Ok((event, msg.into()))
    }

    async fn accept(&mut self, handle: DeliveryInfo) -> anyhow::Result<()> {
        self.0.accept(handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> RabbitMqNextStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-next-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Debug)]
pub struct RabbitMqActiveStepReceiver<W: Workflow>(pub Receiver, pub PhantomData<W>);

impl<W: Workflow> ActiveStepReceiver<W> for RabbitMqActiveStepReceiver<W> {
    type Handle = DeliveryInfo;
    async fn receive(&mut self) -> anyhow::Result<(FullyQualifiedStep<W::Step>, Self::Handle)> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        Ok((event, msg.into()))
    }

    async fn accept(&mut self, handle: DeliveryInfo) -> anyhow::Result<()> {
        self.0.accept(handle).await.map_err(Into::into)
    }
}

impl<W: Workflow> RabbitMqActiveStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-next-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
}
