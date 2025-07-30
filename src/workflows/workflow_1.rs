use derive_more::{From, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    event::Immediate,
    step::{StepSettings, WorkflowStepWithSettings},
    workflows::{
        Event, MyProject, MyProjectEvent, Step, TryFromRef, Workflow, WorkflowEvent, WorkflowStep,
    },
};

///////////////

#[derive(Clone, Debug)]
pub struct Workflow1 {}

impl Workflow for Workflow1 {
    type Project = MyProject;

    type Event = Workflow1Event;

    type Step = Workflow1Step;

    const NAME: &'static str = "workflow_1";

    fn entrypoint() -> crate::step::WorkflowStepWithSettings<Self> {
        WorkflowStepWithSettings {
            step: Workflow1Step::Step0(Step0),
            settings: crate::step::StepSettings { max_retries: 3 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TryInto, From)]
pub enum Workflow1Step {
    Step0(Step0),
    Step1(Step1),
}

#[derive(thiserror::Error, Debug)]
pub enum Workflow1StepError {
    #[error("Step0 error: {0}")]
    Step0(<Step0 as Step>::Error),
    #[error("Step1 error: {0}")]
    Step1(<Step1 as Step>::Error),
}



impl WorkflowStep for Workflow1Step {
    type Workflow = Workflow1;
    type Error = Workflow1StepError;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: <Self::Workflow as Workflow>::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<crate::step::WorkflowStepWithSettings<Self::Workflow>>, crate::step::StepError>
    {
        match self {
            Workflow1Step::Step0(step) => step.run_raw(wf, event.try_into().unwrap()).await,
            Workflow1Step::Step1(step) => step.run_raw(wf, event.try_into().unwrap()).await,
        }
    }

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow1Step::Step0(_) => <Step0 as Step>::Event::is::<T>(),
            Workflow1Step::Step1(_) => <Step1 as Step>::Event::is::<T>(),
        }
    }

    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool {
        match self {
            Workflow1Step::Step0(_) => event.is_event::<<Step0 as Step>::Event>(),
            Workflow1Step::Step1(_) => event.is_event::<<Step1 as Step>::Event>(),
        }
    }
}

///// Steps

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step0;

#[derive(thiserror::Error, Debug)]
pub enum Step0Error {
    // TODO
    #[error("Step0 error")]
    Unknown,
}

impl Step for Step0 {
    type Event = Event0;
    type Workflow = Workflow1;
    type Error = Step0Error;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<WorkflowStepWithSettings<Self::Workflow>>, crate::step::StepError> {
        tracing::info!("Running Step0 in Workflow1");
        Ok(Some(WorkflowStepWithSettings {
            // TODO: this should just return Workflow1Step, right? It shouldn't be able to return steps from other workflows
            step: Workflow1Step::Step1(Step1),
            settings: StepSettings { max_retries: 3 },
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step1;

#[derive(thiserror::Error, Debug)]
pub enum Step1Error {
    // TODO
    #[error("Step1 error")]
    Unknown,
}

impl Step for Step1 {
    type Event = Immediate;
    type Workflow = Workflow1;
    type Error = Step1Error;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
    ) -> Result<Option<crate::step::WorkflowStepWithSettings<Self::Workflow>>, crate::step::StepError>
    {
        tracing::info!("Running Step1 in Workflow1");
        Ok(None)
    }
}

// events

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow1Event {
    Event0(Event0),
    #[serde(skip)]
    Immediate(Immediate),
}

impl WorkflowEvent for Workflow1Event {
    type Workflow = Workflow1;

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow1Event::Event0(_) => Event0::is::<T>(),
            Workflow1Event::Immediate(_) => Immediate::is::<T>(),
        }
    }
}

impl TryFrom<Workflow1Event> for Immediate {
    type Error = ();

    fn try_from(event: Workflow1Event) -> Result<Self, Self::Error> {
        match event {
            Workflow1Event::Immediate(immediate) => Ok(immediate),
            _ => Err(()),
        }
    }
}

impl From<Immediate> for Workflow1Event {
    fn from(immediate: Immediate) -> Self {
        Workflow1Event::Immediate(immediate)
    }
}

impl TryFrom<Workflow1Event> for Event0 {
    type Error = ();

    fn try_from(event: Workflow1Event) -> Result<Self, Self::Error> {
        if let Workflow1Event::Event0(event0) = event {
            Ok(event0)
        } else {
            Err(())
        }
    }
}

impl TryFrom<MyProjectEvent> for Workflow1Event {
    type Error = ();

    fn try_from(event: MyProjectEvent) -> Result<Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow1(workflow_event) => Ok(workflow_event),
            MyProjectEvent::Immediate(immediate) => Ok(Workflow1Event::Immediate(immediate)),
            _ => Err(()),
        }
    }
}

impl TryFromRef<MyProjectEvent> for Workflow1Event {
    type Error = ();

    fn try_from_ref(event: &MyProjectEvent) -> Result<&Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow1(workflow_event) => Ok(workflow_event),
            MyProjectEvent::Immediate(_) => Err(()),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event0 {}

impl From<Event0> for Workflow1Event {
    fn from(event: Event0) -> Self {
        Workflow1Event::Event0(event)
    }
}

impl Event for Event0 {}
