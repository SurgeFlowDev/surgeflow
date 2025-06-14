use serde::{Deserialize, Serialize};

use crate::{
    ActiveStepQueue, Event, InstanceId, StepError, StepWithSettings, StepWithSettingsAndEvent,
    WaitingForEventStepQueue,
    step::{Step, StepSettings},
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Step1 {}

impl Step1 {
    async fn run(&self) -> Result<Option<StepWithSettings>, StepError> {
        Ok(None)
    }
}
impl Step for Step1 {
    async fn run_raw(&self, event: Option<Event>) -> Result<Option<StepWithSettings>, StepError> {
        self.run().await
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
        active_step_queue
            .enqueue(
                instance_id,
                StepWithSettingsAndEvent {
                    fqstep: StepWithSettings {
                        step: self.into(),
                        settings,
                    },
                    event: None,
                },
            )
            .await?;
        Ok(())
    }
}
