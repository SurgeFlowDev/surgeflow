use schemars::JsonSchema;
use std::hash::Hash;

use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

pub mod event;
pub mod step;

use step::{StepError, WorkflowStep};

use crate::{
    event::{Event, Workflow0Event},
    step::{Step, StepSettings, StepWithSettings, step_0::Step0},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowInstanceId(i32);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowId(i32);
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowName(String);

impl AsRef<str> for WorkflowName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Workflow0 {}

impl Workflow for Workflow0 {
    type Event = Workflow0Event;
    type Step = WorkflowStep;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

pub trait Workflow: Clone {
    // + WorkflowEvent<Self>
    type Event: Event<Workflow = Self> + JsonSchema;
    type Step: Step<Workflow = Self, Event = Self::Event>;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<Self::Step>;
}

pub trait WorkflowExt: Workflow {
    fn name() -> WorkflowName {
        String::from(Self::NAME).into()
    }
}

impl<T: Workflow> WorkflowExt for T {}
