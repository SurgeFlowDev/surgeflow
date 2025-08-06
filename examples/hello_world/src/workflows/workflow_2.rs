use derive_more::{From, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surgeflow::{
    ConvertingProjectEventToWorkflowEventError, ConvertingProjectStepToWorkflowStepError,
    ConvertingWorkflowEventToEventError, ConvertingWorkflowStepToStepError, Event, Immediate,
    Project, ProjectStep, Step, StepSettings, SurgeflowWorkflowStepError, TryFromRef, Workflow,
    WorkflowEvent, WorkflowStep, WorkflowStepWithSettings,
};

use crate::workflows::{MyProject, MyProjectEvent};

///////////////

#[derive(Clone, Debug)]
pub struct Workflow2 {}

impl Workflow for Workflow2 {
    type Project = MyProject;

    type Event = Workflow2Event;

    type Step = Workflow2Step;

    const NAME: &'static str = "workflow_2";

    fn entrypoint() -> WorkflowStepWithSettings<Self> {
        WorkflowStepWithSettings {
            step: Workflow2Step::Step0(Step0),
            settings: StepSettings { max_retries: 3 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, From, TryInto)]
pub enum Workflow2Step {
    Step0(Step0),
    Step1(Step1),
}

#[derive(thiserror::Error, Debug)]
pub enum Workflow2StepError {
    #[error("Step0 error: {0}")]
    Step0(<Step0 as Step>::Error),
    #[error("Step1 error: {0}")]
    Step1(<Step1 as Step>::Error),
}

//////////// ProjectStep::Error <-> WorkflowStep::Error conversions

impl From<<<Workflow2 as Workflow>::Step as WorkflowStep>::Error>
    for <<<Workflow2 as Workflow>::Project as Project>::Step as ProjectStep>::Error
{
    fn from(error: <<Workflow2 as Workflow>::Step as WorkflowStep>::Error) -> Self {
        <<<Workflow2 as Workflow>::Project as Project>::Step as ProjectStep>::Error::Workflow2(
            error,
        )
    }
}

impl TryFrom<<<MyProject as Project>::Step as ProjectStep>::Error>
    for <<Workflow2 as Workflow>::Step as WorkflowStep>::Error
{
    type Error = ConvertingProjectStepToWorkflowStepError;

    fn try_from(
        error: <<<Workflow2 as Workflow>::Project as Project>::Step as ProjectStep>::Error,
    ) -> Result<Self, Self::Error> {
        type Error = <<MyProject as Project>::Step as ProjectStep>::Error;
        match error {
            Error::Workflow2(e) => Ok(e),
            _ => Err(ConvertingProjectStepToWorkflowStepError),
        }
    }
}

//////////// WorkflowStep::Error <-> Step::Error conversions

impl From<<Step0 as Step>::Error> for <<Workflow2 as Workflow>::Step as WorkflowStep>::Error {
    fn from(error: <Step0 as Step>::Error) -> Self {
        Workflow2StepError::Step0(error)
    }
}

impl From<<Step1 as Step>::Error> for <<Workflow2 as Workflow>::Step as WorkflowStep>::Error {
    fn from(error: <Step1 as Step>::Error) -> Self {
        Workflow2StepError::Step1(error)
    }
}

impl TryFrom<<<Workflow2 as Workflow>::Step as WorkflowStep>::Error> for <Step1 as Step>::Error {
    type Error = ConvertingWorkflowStepToStepError;

    fn try_from(
        error: <<Workflow2 as Workflow>::Step as WorkflowStep>::Error,
    ) -> Result<Self, Self::Error> {
        match error {
            Workflow2StepError::Step1(e) => Ok(e),
            _ => Err(ConvertingWorkflowStepToStepError),
        }
    }
}

impl TryFrom<<<Workflow2 as Workflow>::Step as WorkflowStep>::Error> for <Step0 as Step>::Error {
    type Error = ConvertingWorkflowStepToStepError;

    fn try_from(
        error: <<Workflow2 as Workflow>::Step as WorkflowStep>::Error,
    ) -> Result<Self, Self::Error> {
        match error {
            Workflow2StepError::Step0(e) => Ok(e),
            _ => Err(ConvertingWorkflowStepToStepError),
        }
    }
}

////////////

impl WorkflowStep for Workflow2Step {
    type Workflow = Workflow2;
    type Error = Workflow2StepError;

    async fn run(
        &self,
        wf: Self::Workflow,
        event: <Self::Workflow as Workflow>::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<
        Option<WorkflowStepWithSettings<Self::Workflow>>,
        SurgeflowWorkflowStepError<<Self as WorkflowStep>::Error>,
    > {
        let res = match self {
            Workflow2Step::Step0(step) => step
                .run(wf, event.try_into()?)
                .await
                .map_err(|e| SurgeflowWorkflowStepError::StepError(Workflow2StepError::Step0(e)))?,
            Workflow2Step::Step1(step) => step
                .run(wf, event.try_into()?)
                .await
                .map_err(|e| SurgeflowWorkflowStepError::StepError(Workflow2StepError::Step1(e)))?,
        };
        Ok(res)
    }

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow2Step::Step0(_) => <Step0 as Step>::Event::is::<T>(),
            Workflow2Step::Step1(_) => <Step1 as Step>::Event::is::<T>(),
        }
    }

    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool {
        match self {
            Workflow2Step::Step0(_) => event.is_event::<<Step0 as Step>::Event>(),
            Workflow2Step::Step1(_) => event.is_event::<<Step1 as Step>::Event>(),
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
    type Workflow = Workflow2;
    type Error = Step0Error;

    async fn run(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
    ) -> Result<Option<WorkflowStepWithSettings<Self::Workflow>>, <Self as Step>::Error> {
        tracing::info!("Running Step0 in Workflow2");
        Ok(Some(WorkflowStepWithSettings {
            step: Workflow2Step::Step1(Step1),
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
    type Workflow = Workflow2;
    type Error = Step1Error;

    async fn run(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
    ) -> Result<Option<WorkflowStepWithSettings<Self::Workflow>>, <Self as Step>::Error> {
        tracing::info!("Running Step1 in Workflow2");
        Ok(None)
    }
}

// events

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow2Event {
    Event0(Event0),
    #[serde(skip)]
    Immediate(Immediate),
}

impl WorkflowEvent for Workflow2Event {
    type Workflow = Workflow2;

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow2Event::Event0(_) => Event0::is::<T>(),
            Workflow2Event::Immediate(_) => Immediate::is::<T>(),
        }
    }
}

impl TryFrom<Workflow2Event> for Immediate {
    type Error = ConvertingWorkflowEventToEventError;

    fn try_from(event: Workflow2Event) -> Result<Self, Self::Error> {
        match event {
            Workflow2Event::Immediate(immediate) => Ok(immediate),
            _ => Err(ConvertingWorkflowEventToEventError),
        }
    }
}

impl From<Immediate> for Workflow2Event {
    fn from(immediate: Immediate) -> Self {
        Workflow2Event::Immediate(immediate)
    }
}

impl TryFrom<Workflow2Event> for Event0 {
    type Error = ConvertingWorkflowEventToEventError;

    fn try_from(event: Workflow2Event) -> Result<Self, Self::Error> {
        if let Workflow2Event::Event0(event0) = event {
            Ok(event0)
        } else {
            Err(ConvertingWorkflowEventToEventError)
        }
    }
}

impl TryFrom<MyProjectEvent> for Workflow2Event {
    type Error = ConvertingProjectEventToWorkflowEventError;

    fn try_from(event: MyProjectEvent) -> Result<Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow2(workflow_event) => Ok(workflow_event),
            MyProjectEvent::Immediate(immediate) => Ok(Workflow2Event::Immediate(immediate)),
            _ => Err(ConvertingProjectEventToWorkflowEventError),
        }
    }
}

impl TryFromRef<MyProjectEvent> for Workflow2Event {
    type Error = ConvertingProjectEventToWorkflowEventError;

    fn try_from_ref(event: &MyProjectEvent) -> Result<&Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow2(workflow_event) => Ok(workflow_event),
            // TODO: why is this an error?
            MyProjectEvent::Immediate(_) => Err(ConvertingProjectEventToWorkflowEventError),
            _ => Err(ConvertingProjectEventToWorkflowEventError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event0 {}

impl From<Event0> for Workflow2Event {
    fn from(event: Event0) -> Self {
        Workflow2Event::Event0(event)
    }
}

impl Event for Event0 {}
