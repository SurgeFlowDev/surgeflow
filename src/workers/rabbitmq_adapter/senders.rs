use std::marker::PhantomData;

use fe2o3_amqp::{Sender, session::SessionHandle};
use uuid::Uuid;

use crate::{
    step::FullyQualifiedStep,
    workers::adapters::senders::{ActiveStepSender, FailedStepSender, NextStepSender},
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
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
}
