use crate::Workflow;
use crate::step::{Immediate, StepResult};
use crate::{
    StepError, StepWithSettings, Workflow0, WorkflowStep,
    event::Workflow0Event,
    step::{Step, Step1},
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
        // event: Event0,
    ) -> StepResult<Workflow0> {
        tracing::info!("Running Step0");

        // return the next step to run
        Step1 {}.into()
    }
}
