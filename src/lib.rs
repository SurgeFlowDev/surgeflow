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
    event::{Event, WorkflowEvent},
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

pub mod runner {
    use std::any::TypeId;
    use tokio::time::{Duration, sleep};

    use crate::{
        event::{Immediate, WorkflowEvent},
        step::{Step, StepSettings},
    };

    use super::*;

    pub async fn complete_workflow(instance_id: WorkflowInstanceId) -> anyhow::Result<()> {
        // TODO: Logic to complete the workflow, e.g., updating the database, notifying listeners, etc.
        println!("Completing workflow for instance ID: {:?}", instance_id);
        Ok(())
    }

    pub async fn handle_step(
        FullyQualifiedStep {
            event,
            instance_id,
            step:
                StepWithSettings {
                    step,
                    settings:
                        StepSettings {
                            max_retry_count,
                            // delay,
                        },
                },
            retry_count,
        }: FullyQualifiedStep<WorkflowStep>,
        ctx: &mut Ctx,
        // delayed_step_queue: &DelayedStepQueue,
    ) -> anyhow::Result<bool> {
        // TODO: Workflow0 {} shouldn't be hard coded
        let Ok(next_step) = step.run_raw(Workflow0 {}, event).await else {
            // If the step fails, we can either retry it or complete the workflow
            if retry_count < max_retry_count {
                let next_step = FullyQualifiedStep {
                    instance_id,
                    step: StepWithSettings {
                        step,
                        settings: StepSettings {
                            max_retry_count,
                            // delay,
                        },
                    },
                    event: None,
                    retry_count: retry_count + 1,
                };
                ctx.active.enqueue(next_step).await?;
                return Ok(false);
            } else {
                // If we reached the max retry count, we can complete the workflow
                complete_workflow(instance_id).await?;
                return Ok(true);
            }
        };

        if let Some(next_step) = next_step {
            if next_step.step.variant_event_type_id() != TypeId::of::<Immediate<Workflow0>>() {
                // If the next step requires an event, enqueue it in the waiting for event queue
                ctx.waiting
                    .enqueue(FullyQualifiedStep {
                        instance_id,
                        step: next_step,
                        event: None,
                        retry_count: 0,
                    })
                    .await?;
                return Ok(false);
            } else {
                ctx.active
                    .enqueue(FullyQualifiedStep {
                        instance_id,
                        step: next_step,
                        event: None,
                        retry_count: 0,
                    })
                    .await?;
            }
        } else {
            complete_workflow(instance_id).await?;
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn handle_event(
        instance_id: WorkflowInstanceId,
        event: WorkflowEvent,
        ctx: &mut Ctx,
    ) -> anyhow::Result<()> {
        tracing::info!("started handle_event");
        let fqstep = ctx.waiting.dequeue::<WorkflowStep>(instance_id).await?;
        // let event = step.try_deserialize_event(event)?;

        tracing::info!("enqueue active step");
        ctx.active
            .enqueue(FullyQualifiedStep {
                event: Some(event),
                ..fqstep
            })
            .await?;

        Ok(())
    }
}

pub struct Workflow0 {}

impl Workflow for Workflow0 {
    type Event = WorkflowEvent;
    type Step = WorkflowStep;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retry_count: 1 },
        }
    }
}

pub trait Workflow {
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
