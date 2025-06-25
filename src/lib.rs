use anyhow::Context;
use futures::{TryStreamExt, stream::StreamExt};
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    time::Duration,
};

use derive_more::{From, Into, TryInto};
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions}, types::FieldTable, BasicProperties, Channel, Consumer
};
use serde::{Deserialize, Serialize, Serializer};

pub mod event;
pub mod step;

use step::{StepError, WorkflowStep};

use crate::{
    event::{Event, WorkflowEvent},
    step::{FullyQualifiedStep, Step, StepWithSettings},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct InstanceId(i32);

pub struct ActiveStepQueue {
    // pub queues: HashMap<InstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
    pub pub_channel: Channel,
    pub sub_channel: Channel,
    pub consumer: Consumer,
}

impl ActiveStepQueue {
    // async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
    //     let instance_id = step.instance_id;
    //     self.queues.entry(instance_id).or_default().push_back(step);
    //     Ok(())
    // }
    async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(&step)?;
        self.pub_channel
            .basic_publish(
                "",
                "workflow-name",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?
            .await?;

        Ok(())
    }
    pub async fn dequeue(
        &mut self,
        instance_id: InstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        
        let next = self.consumer
            .try_next()
            .await?
            .context("channel returned none")?;

        let data: FullyQualifiedStep<WorkflowStep> = serde_json::from_slice(&next.data)?;

        next.ack(BasicAckOptions::default()).await?;

        Ok(data)
    }
    // pub async fn wait_until_dequeue(
    //     &mut self,
    //     instance_id: InstanceId,
    // ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
    //     loop {
    //         if let Some(queue) = self.queues.get_mut(&instance_id) {
    //             if let Some(step) = queue.pop_front() {
    //                 return Ok(step);
    //             }
    //         }
    //         tokio::time::sleep(Duration::from_millis(100)).await;
    //     }
    // }
}

pub struct WaitingForEventStepQueue {
    pub queues: HashMap<InstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
}
impl WaitingForEventStepQueue {
    pub async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        self.queues.entry(instance_id).or_default().push_back(step);
        Ok(())
    }
    async fn dequeue(
        &mut self,
        instance_id: InstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        if let Some(queue) = self.queues.get_mut(&instance_id) {
            if let Some(step) = queue.pop_front() {
                return Ok(step);
            }
        }
        Err(anyhow::anyhow!(
            "No waiting steps found for instance ID: {:?}",
            instance_id
        ))
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
    pub queues: HashMap<InstanceId, VecDeque<FullyQualifiedStep<WorkflowStep>>>,
}
impl CompletedStepQueue {
    async fn enqueue(&mut self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        self.queues.entry(instance_id).or_default().push_back(step);
        Ok(())
    }
    async fn dequeue(
        &mut self,
        instance_id: InstanceId,
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

pub mod runner {
    use std::any::TypeId;
    use tokio::time::{Duration, sleep};

    use crate::{
        event::{Immediate, WorkflowEvent},
        step::{Step, StepSettings},
    };

    use super::*;

    pub async fn complete_workflow(instance_id: InstanceId) -> anyhow::Result<()> {
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
        waiting_for_event_step_queue: &mut WaitingForEventStepQueue,
        active_step_queue: &mut ActiveStepQueue,
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
                active_step_queue.enqueue(next_step).await?;
                return Ok(false);
            } else {
                // If we reached the max retry count, we can complete the workflow
                complete_workflow(instance_id).await?;
                return Ok(true);
            }
        };

        if let Some(next_step) = next_step {
            if next_step.step.variant_event_type_id() != TypeId::of::<Immediate>() {
                // If the next step requires an event, enqueue it in the waiting for event queue
                waiting_for_event_step_queue
                    .enqueue(FullyQualifiedStep {
                        instance_id,
                        step: next_step,
                        event: None,
                        retry_count: 0,
                    })
                    .await?;
                return Ok(false);
            } else {
                active_step_queue
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
        instance_id: InstanceId,
        event: WorkflowEvent,
        waiting_for_step_queue: &mut WaitingForEventStepQueue,
        active_step_queue: &mut ActiveStepQueue,
    ) -> anyhow::Result<()> {
        let fqstep = waiting_for_step_queue.dequeue(instance_id).await?;
        // let event = step.try_deserialize_event(event)?;

        active_step_queue
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
}

pub trait Workflow
where
    step::WorkflowStep: From<<Self as Workflow>::Step>,
{
    type Event: Event;
    type Step: Step<Workflow = Self, Event = Self::Event>;
}
