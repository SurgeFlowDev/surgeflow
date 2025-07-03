use crate::{
    StepError, StepWithSettings, Workflow0, WorkflowStep,
    event::{Workflow0Event, event_0::Event0},
    step::{Step, Step1, StepSettings},
};
use macros::step;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(
        &self,
        wf: Workflow0,
        event: Event0,
    ) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError> {
        // println!("Running Step0 with event: {:?}", event);
        tracing::info!("Running Step0");
        Ok(Some(StepWithSettings {
            step: WorkflowStep::Step1(Step1 {}),
            settings: StepSettings {
                max_retries: 0,
                // delay: None,
            },
        }))
    }
}
