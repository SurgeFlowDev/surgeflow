use macros::step;
use serde::{Deserialize, Serialize};

use crate::{
    event::{self, event_0::Event0, WorkflowEvent}, step::{Step, StepSettings}, ActiveStepQueue, DelayedStepQueue, FullyQualifiedStep, InstanceId, StepError, StepWithSettings, WaitingForEventStepQueue
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Step1 {}

#[step]
impl Step1 {
    #[run]
    async fn run(&self) -> Result<Option<StepWithSettings>, StepError> {
        Ok(None)
    }
}
// impl Step for Step1 {
//     async fn run_raw(&self, event: Option<WorkflowEvent>) -> Result<Option<StepWithSettings>, StepError> {
//         self.run().await
//     }

//     // each step can implement its own enqueue method, so we have to take both the active and waiting for step queues as parameters,
//     // and the step will decide which queue to enqueue itself into
//     async fn enqueue(
//         self,
//         instance_id: InstanceId,
//         settings: StepSettings,
//         active_step_queue: &ActiveStepQueue,
//         waiting_for_step_queue: &WaitingForEventStepQueue,
//         delayed_step_queue: &DelayedStepQueue,
//     ) -> anyhow::Result<()> {
//         active_step_queue
//             .enqueue(
//                 instance_id,
//                 FullyQualifiedStep {
//                     step: StepWithSettings {
//                         step: self.into(),
//                         settings,
//                     },
//                     event: None,
//                     retry_count: 0,
//                 },
//             )
//             .await?;
//         Ok(())
//     }
// }
