use crate::{
    event::{event_0::Event0, Immediate, Workflow0Event}, step::{Step, StepResult, StepSettings, Workflow0Step}, StepError, StepWithSettings, Workflow, Workflow0
};
use macros::step;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step1 {}

impl From<Step1> for Option<StepWithSettings<<<Step1 as Step>::Workflow as Workflow>::Step>> {
    fn from(step: Step1) -> Self {
        Some(StepWithSettings {
            step: step.into(),
            settings: StepSettings { max_retries: 0 },
        })
    }
}

// static DEV_COUNT: AtomicUsize = AtomicUsize::new(0);

#[step]
impl Step1 {
    #[run]
    async fn run(
        &self,
        #[expect(unused_variables)]
        wf: Workflow0,
        event: Event0,
    ) -> StepResult<Workflow0> {
        tracing::info!("Running Step1");
        // let dev_count = DEV_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // if dev_count == 3 {
        //     return Ok(None);
        // }
        // Err(StepError::Unknown)
        Ok(None)
    }
}
