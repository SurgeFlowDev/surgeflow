use serde::{Deserialize, Serialize};

use crate::{
    ActiveStepQueue, InstanceId, StepError, StepWithSettings, WaitingForEventStepQueue,
    WorkflowStep,
    event::{WorkflowEvent, event_0::Event0},
    step::{Step, Step1, StepSettings},
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Step0 {}

impl Step0 {
    async fn run(&self, event: Event0) -> Result<Option<StepWithSettings>, StepError> {
        // TODO
        Ok(Some(StepWithSettings {
            step: WorkflowStep::Step1(Step1 {}),
            settings: StepSettings {
                max_retry_count: 0,
                delay: None,
            },
        }))
    }
}
impl Step for Step0 {
    async fn run_raw(
        &self,
        event: Option<WorkflowEvent>,
    ) -> Result<Option<StepWithSettings>, StepError> {
        let event = if let Some(event) = event {
            event
        } else {
            return Err(StepError::Unknown);
        };
        let event = event.try_into().map_err(|_| StepError::Unknown)?;

        self.run(event).await
    }

    // each step can implement its own enqueue method, so we have to take both the active and waiting for step queues as parameters,
    // and the step will decide which queue to enqueue itself into
    async fn enqueue(
        self,
        instance_id: InstanceId,
        settings: StepSettings,
        active_step_queue: &ActiveStepQueue,
        waiting_for_step_queue: &WaitingForEventStepQueue,
    ) -> anyhow::Result<()> {
        waiting_for_step_queue
            .enqueue(
                instance_id,
                StepWithSettings {
                    // TODO
                    step: self.into(),
                    settings,
                },
            )
            .await?;
        Ok(())
    }

    // fn try_deserialize_event(event: String) -> Result<Event, EventDeserializationError> {
    //     let event: Event0 =
    //         serde_json::from_str(&event).map_err(|_| EventDeserializationError::Unknown)?;
    //     Ok(event.into())
    // }
}
