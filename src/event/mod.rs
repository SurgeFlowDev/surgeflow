use derive_more::{From, TryInto};
use fe2o3_amqp::{Receiver, Sender};
use futures::lock::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

use event_0::Event0;

use crate::{Workflow, Workflow0, WorkflowInstanceId};

pub mod event_0;

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema)]
pub enum WorkflowEvent {
    Event0(Event0),
}

impl Event for WorkflowEvent {
    type Workflow = Workflow0;
}

pub trait Event: Serialize + for<'a> Deserialize<'a> {
    type Workflow: Workflow;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Immediate<W: Workflow>(PhantomData<W>);

impl<W: Workflow> Event for Immediate<W> {
    type Workflow = W;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InstanceEvent<W: Workflow> {
    #[serde(bound = "")]
    pub event: W::Event,
    pub instance_id: WorkflowInstanceId,
}

#[derive(Debug)]
pub struct EventReceiver<W: Workflow>(Mutex<Receiver>, PhantomData<W>);

impl<W: Workflow> EventReceiver<W> {
    pub fn new(receiver: Receiver) -> Self {
        Self(Mutex::new(receiver), PhantomData::default())
    }
    pub async fn recv(&self) -> anyhow::Result<W::Event> {
        let mut receiver = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let event = receiver.recv::<String>().await?;
        let event = serde_json::from_str(event.body())?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct EventSender<W: Workflow>(Mutex<Sender>, PhantomData<W>);

impl<W: Workflow> EventSender<W> {
    pub fn new(sender: Sender) -> Self {
        Self(Mutex::new(sender), PhantomData::default())
    }
    pub async fn send(&self, event: InstanceEvent<W>) -> anyhow::Result<()> {
        let mut sender = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&event)?;
        sender.send(event).await?;
        Ok(())
    }
}
