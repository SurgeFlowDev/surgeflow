use std::sync::atomic::AtomicUsize;

use crate::{
    StepError, StepWithSettings, Workflow0,
    event::{Immediate, WorkflowEvent},
    step::{Step, WorkflowStep},
};
use macros::step;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step1 {}

static DEV_COUNT: AtomicUsize = AtomicUsize::new(0);

#[step]
impl Step1 {
    #[run]
    async fn run(
        &self,
        wf: Workflow0,
    ) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError> {
        println!("Running Step1");
        let dev_count = DEV_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if dev_count == 3 {
            return Ok(None);
        }
        Err(StepError::Unknown)
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
