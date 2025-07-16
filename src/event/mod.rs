use fe2o3_amqp::{Sender, session::SessionHandle};
use futures::lock::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};
use uuid::Uuid;

use crate::workflows::{Workflow, WorkflowInstanceId};

pub trait Event: Serialize + for<'a> Deserialize<'a> + Clone {
    type Workflow: Workflow;
}

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

// must be thread-safe
#[derive(Debug)]
pub struct EventSender<W: Workflow>(Mutex<Sender>, PhantomData<W>);

impl<W: Workflow> EventSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-events", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(Mutex::new(sender), PhantomData))
    }
    pub async fn send(&self, step: InstanceEvent<W>) -> anyhow::Result<()> {
        let mut sender = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        sender.send(event).await?;
        Ok(())
    }
}
