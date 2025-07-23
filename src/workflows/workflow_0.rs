use crate::event::{Event, Immediate};
use crate::step::StepError;
use crate::step::{Step, StepWithSettings};
use crate::step::{StepResult, StepSettings, WorkflowStep};
use crate::workflows::{AsWorkflowEventType, Workflow, WorkflowEvent, WorkflowEventType};
use derive_more::{From, TryInto};
use macros::step;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::fmt::Debug;

impl WorkflowStep<Workflow0> for Workflow0Step {
    fn run_raw(
        &self,
        wf: Workflow0,
        event: Option<Workflow0Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<Output = Result<Option<StepWithSettings<Workflow0>>, StepError>> + Send
    {
        async move {
            match self {
                Workflow0Step::Step0(step) => Step::run_raw(step, wf, event).await,
                Workflow0Step::Step1(step) => Step::run_raw(step, wf, event).await,
            }
        }
    }
}

// impl Step for Workflow0Step {
//     type Event = Workflow0Event;
//     type Workflow = Workflow0;
//     async fn run_raw(
//         &self,
//         wf: Self::Workflow,
//         event: Option<Workflow0Event>,
//     ) -> Result<Option<StepWithSettings<Self::Workflow>>, StepError> {
//         match self {
//             Workflow0Step::Step0(step) => Step::run_raw(step, wf, event).await,
//             Workflow0Step::Step1(step) => Step::run_raw(step, wf, event).await,
//         }
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    Event(TypeId),
    Immediate,
}

impl WorkflowEventType for EventType {}

impl WorkflowEvent for Workflow0Event {}

impl AsWorkflowEventType for Workflow0Event {
    fn as_event_type(&self) -> EventType {
        match self {
            Workflow0Event::Event0(_) => EventType::Event(TypeId::of::<Event0>()),
            Workflow0Event::Event1(_) => EventType::Event(TypeId::of::<Event1>()),
        }
    }
}

impl AsWorkflowEventType for Workflow0Step {
    fn as_event_type(&self) -> EventType {
        match self {
            Workflow0Step::Step0(step) => step.as_event_type(),
            Workflow0Step::Step1(step) => step.as_event_type(),
        }
    }
}

impl<T: Step> AsWorkflowEventType for T {
    fn as_event_type(&self) -> EventType {
        let raw_event_type = TypeId::of::<<T as Step>::Event>();
        if raw_event_type == TypeId::of::<Immediate>() {
            return EventType::Immediate;
        }
        EventType::Event(raw_event_type)
    }
}

impl Event for Workflow0Event {}

impl Event for Event0 {}

impl From<Event0> for Option<<Workflow0 as Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow0Event::Event0(val))
    }
}

impl From<Step1> for Option<StepWithSettings<<Step1 as Step>::Workflow>> {
    fn from(step: Step1) -> Self {
        Some(StepWithSettings {
            step: step.into(),
            settings: StepSettings { max_retries: 0 },
        })
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
    type EventType = EventType;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

// boilerplate ended

#[derive(Debug, Clone, JsonSchema)]
pub struct Workflow0 {}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {
    test_string: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event1 {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

pub struct Step0Marker {}

impl Step for Step0 {
    type Event = Immediate;
    type Workflow = Workflow0;

    async fn run_raw(
        &self,
        #[expect(unused_variables)] wf: Workflow0,
        event: Option<Workflow0Event>,
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

#[step]
impl Step1 {
    #[expect(unused_variables)]
    #[run]
    async fn run(&self, wf: Workflow0, event: Event0) -> StepResult<Workflow0> {
        tracing::error!(
            "Running Step1, Workflow0, event.test_string: {}",
            event.test_string
        );
        Ok(None)
    }
}
