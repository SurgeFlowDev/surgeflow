use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize, Serializer};

mod event;
mod step;

use step::{StepError, WorkflowStep};

use crate::step::{FullyQualifiedStep, StepWithSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct InstanceId(i32);

struct ActiveStepQueue {}

impl ActiveStepQueue {
    async fn enqueue(&self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(
        &self,
        instance_id: InstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        todo!()
    }
}

struct WaitingForEventStepQueue {}
impl WaitingForEventStepQueue {
    async fn enqueue(&self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(
        &self,
        instance_id: InstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        todo!()
    }
}

struct DelayedStepQueue {}
impl DelayedStepQueue {
    async fn enqueue(&self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        todo!()
    }

    // this queue won't have a dequeue method. steps from this queue will be automatically moved to active or waiting for event queues when the timeout expires.
}

struct CompletedStepQueue {}
impl CompletedStepQueue {
    async fn enqueue(&self, step: FullyQualifiedStep<WorkflowStep>) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(
        &self,
        instance_id: InstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<WorkflowStep>> {
        todo!()
    }
}

mod runner {
    use std::any::TypeId;

    use crate::{
        event::{Immediate, WorkflowEvent},
        step::{Step, StepSettings},
    };

    use super::*;

    pub async fn complete_workflow(instance_id: InstanceId) -> anyhow::Result<()> {
        todo!()
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
                            delay,
                        },
                },
            retry_count,
        }: FullyQualifiedStep<WorkflowStep>,
        waiting_for_event_step_queue: &WaitingForEventStepQueue,
        active_step_queue: &ActiveStepQueue,
        delayed_step_queue: &DelayedStepQueue,
    ) -> anyhow::Result<()> {
        let Ok(next_step) = step.run_raw(event).await else {
            // If the step fails, we can either retry it or complete the workflow
            if retry_count < max_retry_count {
                let next_step = FullyQualifiedStep {
                    instance_id,
                    step: StepWithSettings {
                        step,
                        settings: StepSettings {
                            max_retry_count,
                            delay,
                        },
                    },
                    event: None,
                    retry_count: retry_count + 1,
                };
                active_step_queue.enqueue(next_step).await?;
                return Ok(());
            } else {
                // If we reached the max retry count, we can complete the workflow
                complete_workflow(instance_id).await?;
                return Ok(());
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
                return Ok(());
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
        }
        Ok(())
    }

    pub async fn handle_event(
        instance_id: InstanceId,
        event: WorkflowEvent,
        waiting_for_step_queue: &WaitingForEventStepQueue,
        active_step_queue: &ActiveStepQueue,
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

struct Workflow0 {}
