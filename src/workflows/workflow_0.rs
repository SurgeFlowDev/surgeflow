use std::any::TypeId;

use crate::Workflow;
use crate::event::{Event, Immediate};
use crate::step::{StepResult, StepSettings};
use crate::{StepError, StepWithSettings, Workflow0, Workflow0Step, step::Step};
use derive_more::{From, TryInto};
use macros::step;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow0Event {
    Event0(Event0),
}

impl WorkflowEvent for Workflow0Event {
    fn variant_type_id(&self) -> TypeId {
        match self {
            Self::Event0(_) => TypeId::of::<Event0>(),
        }
    }
}

pub trait WorkflowEvent: Event + Debug + JsonSchema + Send {
    fn variant_type_id(&self) -> TypeId;
}

impl Event for Workflow0Event {
    type Workflow = Workflow0;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {}

impl Event for Event0 {
    type Workflow = Workflow0;
}

impl From<Event0> for Option<<Workflow0 as crate::Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow0Event::Event0(val))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(
        &self,
        #[expect(unused_variables)] wf: Workflow0,
        // event: Event0,
    ) -> StepResult<Workflow0> {
        tracing::info!("Running Step0");

        // return the next step to run
        Ok(Step1 {}.into())
    }
}

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
    #[expect(unused_variables)]
    #[run]
    async fn run(&self, wf: Workflow0, event: Event0) -> StepResult<Workflow0> {
        tracing::info!("Running Step1");
        // let dev_count = DEV_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // if dev_count == 3 {
        //     return Ok(None);
        // }
        // Err(StepError::Unknown)
        Ok(None)
    }
}
