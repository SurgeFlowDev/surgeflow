use serde::{Deserialize, Serialize, de};

use crate::{
    StepError, StepWithSettings, WorkflowStep,
    event::{Event, WorkflowEvent, event_0::Event0},
    step::{Step, Step1, StepSettings},
};
use macros::step;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(&self, event: Event0) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError> {
        Ok(Some(StepWithSettings {
            step: WorkflowStep::Step1(Step1 {}),
            settings: StepSettings {
                max_retry_count: 0,
                delay: None,
            },
        }))
    }
}
