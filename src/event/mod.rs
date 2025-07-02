use derive_more::{From, TryInto};
use fe2o3_amqp::{session::SessionHandle, Receiver, Sender};
use futures::lock::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use event_0::Event0;

use crate::{Workflow, Workflow0, WorkflowInstanceId};

pub mod event_0;

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow0Event {
    Event0(Event0),
}

impl Workflow0Event {
    pub fn variant_type_id(&self) -> TypeId {
        match self {
            Self::Event0(_) => TypeId::of::<Event0>(),
        }
    }
}

impl Event for Workflow0Event {
    type Workflow = Workflow0;
}

pub trait Event: Serialize + for<'a> Deserialize<'a> + Clone {
    type Workflow: Workflow;
}

// pub trait WorkflowEvent<W: Workflow>: Event<Workflow = W> {
//     const NAME: &'static str;
// }
// impl WorkflowEvent<Workflow0> for Workflow0Event {
//     const NAME: &'static str = "Workflow0Event";
// }

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-events", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(Mutex::new(receiver), PhantomData::default()))
    }
    pub async fn recv(&self) -> anyhow::Result<InstanceEvent<W>> {
        let mut receiver = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let msg = receiver.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                receiver.reject(msg, None).await?;
                return Err(err);
            }
        };
        receiver.accept(msg).await?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct EventSender<W: Workflow>(Mutex<Sender>, PhantomData<W>);

impl<W: Workflow> EventSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-events", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(Mutex::new(sender), PhantomData::default()))
    }
    pub async fn send(&self, step: InstanceEvent<W>) -> anyhow::Result<()> {
        let mut sender = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        sender.send(event).await?;
        Ok(())
    }
}
