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
    async fn enqueue(
        &self,
        instance_id: InstanceId,
        step: FullyQualifiedStep,
    ) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(&self, instance_id: InstanceId) -> anyhow::Result<FullyQualifiedStep> {
        todo!()
    }
}

struct WaitingForEventStepQueue {}
impl WaitingForEventStepQueue {
    async fn enqueue(&self, instance_id: InstanceId, step: StepWithSettings) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(&self, instance_id: InstanceId) -> anyhow::Result<StepWithSettings> {
        todo!()
    }
}

struct DelayedStepQueue {}
impl DelayedStepQueue {
    async fn enqueue(
        &self,
        instance_id: InstanceId,
        step: FullyQualifiedStep,
    ) -> anyhow::Result<()> {
        todo!()
    }

    // this queue won't have a dequeue method. steps from this queue will be automatically moved to active or waiting for event queues when the timeout expires.
}

struct CompletedStepQueue {}
impl CompletedStepQueue {
    async fn enqueue(&self, step: FullyQualifiedStep) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(&self, instance_id: InstanceId) -> anyhow::Result<FullyQualifiedStep> {
        todo!()
    }
}

mod runner {
    use crate::{
        event::WorkflowEvent,
        step::{Step, StepSettings},
    };

    use super::*;

    pub async fn complete_workflow(instance_id: InstanceId) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn handle_step(
        FullyQualifiedStep {
            event,
            step:
                StepWithSettings {
                    step,
                    settings: StepSettings { .. },
                },
            retry_count,
        }: FullyQualifiedStep,
        instance_id: InstanceId,
        waiting_for_event_step_queue: &WaitingForEventStepQueue,
        active_step_queue: &ActiveStepQueue,
        delayed_step_queue: &DelayedStepQueue,
    ) -> anyhow::Result<()> {
        let next_step = step.run_raw(event).await?;

        if let Some(next_step) = next_step {
            next_step
                .step
                .enqueue(
                    instance_id,
                    next_step.settings,
                    active_step_queue,
                    waiting_for_event_step_queue,
                    delayed_step_queue,
                )
                .await?;
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
            .enqueue(
                instance_id,
                FullyQualifiedStep {
                    step: fqstep,
                    event: Some(event),
                    retry_count: 0,
                },
            )
            .await?;

        Ok(())
    }
}

struct Workflow0 {}
