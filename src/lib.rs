use anyhow::Context;
use fe2o3_amqp::{Receiver, Sender};
use futures::{TryStreamExt, stream::StreamExt};
use schemars::JsonSchema;
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    hash::Hash,
    sync::Arc,
    time::Duration,
};
use tikv_client::RawClient;

use derive_more::{Display, From, Into, TryInto};
use lapin::{
    BasicProperties, Channel, Consumer,
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions},
    types::FieldTable,
};
use serde::{Deserialize, Serialize, Serializer};

pub mod event;
pub mod step;

use step::{StepError, WorkflowStep};

use crate::{
    event::{Event, Workflow0Event},
    step::{FullyQualifiedStep, Step, StepSettings, StepWithSettings, step_0::Step0},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowInstanceId(i32);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowId(i32);
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowName(String);

impl AsRef<str> for WorkflowName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct ActiveStepQueue {
    // pub queues: HashMap<InstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
    // pub pub_channel: Channel,
    pub sender: Sender,
    pub receiver: Receiver,
}

impl ActiveStepQueue {
    async fn enqueue<S: step::Step>(&mut self, step: FullyQualifiedStep<S>) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(&step)?;
        self.sender.send(&payload).await?;

        Ok(())
    }
    pub async fn dequeue<S: step::Step>(&mut self) -> anyhow::Result<FullyQualifiedStep<S>> {
        let delivery = self.receiver.recv::<Vec<u8>>().await.unwrap();
        self.receiver.accept(&delivery).await.unwrap();

        let data: FullyQualifiedStep<S> = serde_json::from_slice(delivery.body())?;

        Ok(data)
    }
}

#[derive(Clone)]
pub struct WaitingForEventStepQueue {
    // pub queues: HashMap<InstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
    pub queues: RawClient,
}
impl WaitingForEventStepQueue {
    // pub async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
    //     let instance_id = step.instance_id;
    //     self.queues.entry(instance_id).or_default().push_back(step);
    //     Ok(())
    // }
    pub async fn enqueue<S: step::Step>(
        &mut self,
        step: FullyQualifiedStep<S>,
    ) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        let payload = serde_json::to_vec(&step)?;
        self.queues
            .put(format!("instance_{}", instance_id.0), payload)
            .await?;
        Ok(())
    }

    async fn dequeue<S: step::Step>(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<S>> {
        let value = self
            .queues
            .get(format!("instance_{}", instance_id.0))
            .await?
            .context("no event")?;
        let data: FullyQualifiedStep<S> = serde_json::from_slice(&value)?;
        Ok(data)
    }
}

// struct DelayedStepQueue {}
// impl DelayedStepQueue {
//     async fn enqueue(&self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
//         todo!()
//     }

//     // this queue won't have a dequeue method. steps from this queue will be automatically moved to active or waiting for event queues when the timeout expires.
// }

pub struct CompletedStepQueue {
    pub queues: HashMap<WorkflowInstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
}
impl CompletedStepQueue {
    async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        self.queues.entry(instance_id).or_default().push_back(step);
        Ok(())
    }
    async fn dequeue(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        if let Some(queue) = self.queues.get_mut(&instance_id) {
            if let Some(step) = queue.pop_front() {
                return Ok(step);
            }
        }
        Err(anyhow::anyhow!(
            "No completed steps found for instance ID: {:?}",
            instance_id
        ))
    }
}

pub struct Ctx {
    pub active: ActiveStepQueue,
    pub waiting: WaitingForEventStepQueue,
}

#[derive(Debug, Clone)]
pub struct Workflow0 {}

impl Workflow for Workflow0 {
    type Event = Workflow0Event;
    type Step = WorkflowStep;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

pub trait Workflow: Clone {
    // + WorkflowEvent<Self>
    type Event: Event<Workflow = Self> + JsonSchema;
    type Step: Step<Workflow = Self, Event = Self::Event>;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<Self::Step>;
}

pub trait WorkflowExt: Workflow {
    fn name() -> WorkflowName {
        String::from(Self::NAME).into()
    }
}

impl<T: Workflow> WorkflowExt for T {}
