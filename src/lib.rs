use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize, Serializer};

mod step;

use step::{StepError, WorkflowStep};

use crate::step::{StepWithSettings, StepWithSettingsAndEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct InstanceId(i32);

struct ActiveStepQueue {}

impl ActiveStepQueue {
    async fn enqueue(
        &self,
        instance_id: InstanceId,
        step: StepWithSettingsAndEvent,
    ) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(&self, instance_id: InstanceId) -> anyhow::Result<StepWithSettingsAndEvent> {
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

// struct WaitingForTimeoutStepQueue {}
// impl WaitingForTimeoutStepQueue {
//     async fn enqueue(&self, step: FullyQualifiedStep) -> anyhow::Result<()> {
//         todo!()
//     }
//     // this queue won't have a dequeue method. steps from this queue will be automatically moved to active or waiting for event queues when the timeout expires.
// }

struct CompletedStepQueue {}
impl CompletedStepQueue {
    async fn enqueue(&self, step: StepWithSettingsAndEvent) -> anyhow::Result<()> {
        todo!()
    }
    async fn dequeue(&self, instance_id: InstanceId) -> anyhow::Result<StepWithSettingsAndEvent> {
        todo!()
    }
}

mod runner {
    use crate::step::{Step, StepSettings};

    use super::*;

    pub async fn complete_workflow(instance_id: InstanceId) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn handle_step(
        StepWithSettingsAndEvent {
            event,
            fqstep:
                StepWithSettings {
                    step,
                    settings: StepSettings { .. },
                },
        }: StepWithSettingsAndEvent,
        instance_id: InstanceId,
        waiting_for_event_step_queue: &WaitingForEventStepQueue,
        active_step_queue: &ActiveStepQueue,
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
                )
                .await?;
        } else {
            complete_workflow(instance_id).await?;
        }
        Ok(())
    }

    pub async fn handle_event(
        instance_id: InstanceId,
        event: Event,
        waiting_for_step_queue: &WaitingForEventStepQueue,
        active_step_queue: &ActiveStepQueue,
    ) -> anyhow::Result<()> {
        let fqstep = waiting_for_step_queue.dequeue(instance_id).await?;
        // let event = step.try_deserialize_event(event)?;

        active_step_queue
            .enqueue(
                instance_id,
                StepWithSettingsAndEvent {
                    fqstep,
                    event: Some(event),
                },
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum EventDeserializationError {
    #[error("Unknown step error")]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event0 {}

#[derive(Debug, From, TryInto)]
enum Event {
    Event0(Event0),
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Event::Event0(inner) => inner.serialize(serializer),
        }
    }
}

struct Workflow0 {}
