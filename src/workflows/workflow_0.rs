use crate::event::{Event, Immediate};
use crate::step::StepError;
use crate::step::{Step, StepWithSettings};
use crate::step::{StepResult, StepSettings, WorkflowStep};
use crate::workflows::{Workflow, WorkflowEvent};
use derive_more::{From, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

//////////////////////// #[workflow], or #[derive(Workflow)] generated boilerplate /////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, JsonSchema)]
pub struct Workflow0 {}

impl WorkflowStep<Workflow0> for Workflow0Step {
    async fn run_raw(
        &self,
        wf: Workflow0,
        event: Option<Workflow0Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<StepWithSettings<Workflow0>>, StepError> {
        match self {
            Workflow0Step::Step0(step) => step.run_raw(wf, event.try_into().unwrap()).await,
            Workflow0Step::Step1(step) => step.run_raw(wf, event.try_into().unwrap()).await,
        }
    }

    fn matches_event_type<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow0Step::Step0(_) => <Step0 as Step>::Event::matches_type::<T>(),
            Workflow0Step::Step1(_) => <Step1 as Step>::Event::matches_type::<T>(),
        }
    }

    fn matches_workflow_event_type<T: WorkflowEvent + 'static>(&self, workflow_event: &T) -> bool {
        match self {
            Workflow0Step::Step0(_) => workflow_event.matches_type::<<Step0 as Step>::Event>(),
            Workflow0Step::Step1(_) => workflow_event.matches_type::<<Step1 as Step>::Event>(),
        }
    }
}

impl TryFrom<Option<Workflow0Event>> for Event0 {
    type Error = anyhow::Error;

    fn try_from(value: Option<Workflow0Event>) -> Result<Self, Self::Error> {
        match value {
            Some(Workflow0Event::Event0(event)) => Ok(event),
            _ => Err(anyhow::anyhow!("Invalid event type for Event0")),
        }
    }
}

impl TryFrom<Option<Workflow0Event>> for Event1 {
    type Error = anyhow::Error;

    fn try_from(value: Option<Workflow0Event>) -> Result<Self, Self::Error> {
        match value {
            Some(Workflow0Event::Event1(event)) => Ok(event),
            _ => Err(anyhow::anyhow!("Invalid event type for Event1")),
        }
    }
}

impl TryFrom<Option<Workflow0Event>> for Immediate {
    type Error = anyhow::Error;

    fn try_from(value: Option<Workflow0Event>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Immediate {}),
            _ => Err(anyhow::anyhow!("Invalid event type for Immediate")),
        }
    }
}

impl WorkflowEvent for Workflow0Event {
    fn matches_type<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow0Event::Event0(_) => Event0::matches_type::<T>(),
            Workflow0Event::Event1(_) => Event1::matches_type::<T>(),
        }
    }
}

impl From<Event0> for Option<<Workflow0 as Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow0Event::Event0(val))
    }
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
pub enum Workflow0Step {
    Step0(Step0),
    Step1(Step1),
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow0Event {
    Event0(Event0),
    Event1(Event1),
}

impl Workflow for Workflow0 {
    type Event = Workflow0Event;
    type Step = Workflow0Step;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl Event for Event0 {}

impl From<Step1> for Option<StepWithSettings<<Step1 as Step>::Workflow>> {
    fn from(step: Step1) -> Self {
        Some(StepWithSettings {
            step: step.into(),
            settings: StepSettings { max_retries: 0 },
        })
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {
    test_string: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event1 {}

impl Event for Event1 {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

impl Step for Step0 {
    type Event = Immediate;
    type Workflow = Workflow0;

    async fn run_raw(
        &self,
        #[expect(unused_variables)] wf: Workflow0,
        _event: Immediate,
    ) -> StepResult<Workflow0> {
        tracing::error!("Running Step0, Workflow0");

        // return the next step to run
        // Ok(None)

        Ok(Some(StepWithSettings {
            step: Step1 {}.into(),
            settings: StepSettings { max_retries: 3 },
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step1 {}

impl Step for Step1 {
    type Event = Event0;
    type Workflow = Workflow0;

    async fn run_raw(
        &self,
        #[expect(unused_variables)] wf: Workflow0,
        event: Event0,
    ) -> StepResult<Workflow0> {
        tracing::error!("Running Step1, Workflow0");

        Ok(None)
    }
}
