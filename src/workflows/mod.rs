use std::any::TypeId;
use derive_more::{Display, From, Into};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::{event::Event, step::{Step, StepWithSettings, WorkflowStep}};

pub mod workflow_0;
pub mod workflow_1;

pub trait WorkflowEvent: Event + Debug + JsonSchema + Send {
    fn variant_type_id(&self) -> TypeId;
}

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

pub trait Workflow: Clone + Send + Sync + 'static {
    type Event: Event<Workflow = Self> + JsonSchema + WorkflowEvent;
    type Step: Step<Workflow = Self, Event = Self::Event> + WorkflowStep;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<Self::Step>;
}

pub trait WorkflowExt: Workflow {
    fn name() -> WorkflowName {
        String::from(Self::NAME).into()
    }
}

impl<T: Workflow> WorkflowExt for T {}