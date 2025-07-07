use schemars::JsonSchema;
use std::{any::TypeId, hash::Hash};

use derive_more::{Display, From, Into, TryInto};
use serde::{Deserialize, Serialize};

pub mod event;
pub mod step;
pub mod workflows;

use step::StepError;

use crate::{
    event::Event,
    step::{Step, StepSettings, StepWithSettings, WorkflowStep},
    workflows::workflow_0::{Step0, Step1, Workflow0Event, WorkflowEvent},
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

#[derive(Debug, Clone, JsonSchema)]
pub struct Workflow0 {}

impl Workflow for Workflow0 {
    type Event = Workflow0Event;
    type Step = Workflow0Step;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
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

// step

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
pub enum Workflow0Step {
    Step0(Step0),
    Step1(Step1),
}
impl WorkflowStep for Workflow0Step {
    fn variant_event_type_id(&self) -> TypeId {
        match self {
            Workflow0Step::Step0(_) => TypeId::of::<<Step0 as Step>::Event>(),
            Workflow0Step::Step1(_) => TypeId::of::<<Step1 as Step>::Event>(),
        }
    }
}

impl Step for Workflow0Step {
    type Event = Workflow0Event;
    type Workflow = Workflow0;
    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<Workflow0Event>,
    ) -> Result<Option<StepWithSettings<Self>>, StepError> {
        match self {
            Workflow0Step::Step0(step) => Step::run_raw(step, wf, event).await,
            Workflow0Step::Step1(step) => Step::run_raw(step, wf, event).await,
        }
    }
}
