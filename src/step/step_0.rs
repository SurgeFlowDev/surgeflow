use crate::{
    StepError, StepWithSettings, Workflow0, WorkflowStep,
    event::{Event, WorkflowEvent, event_0::Event0},
    step::{Step, Step1, StepSettings},
};
use macros::step;
use serde::{Deserialize, Serialize, de};

#[derive(Debug, Serialize, Deserialize)]
pub struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(
        &self,
        wf: Workflow0,
        event: Event0,
    ) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError> {
        println!("Running Step0 with event: {:?}", event);
        Ok(Some(StepWithSettings {
            step: WorkflowStep::Step1(Step1 {}),
            settings: StepSettings {
                max_retry_count: 1,
                // delay: None,
            },
        }))
    }
}
