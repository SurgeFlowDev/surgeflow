use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::workflows::{
    Event, Project, ProjectEvent, ProjectStep, ProjectWorkflow, Step, Workflow, WorkflowEvent,
    WorkflowStep,
};

#[derive(Clone)]
pub enum MyProject {
    Workflow1(Workflow1),
}

#[derive(Clone)]
pub struct Workflow1;

impl From<Workflow1> for MyProject {
    fn from(workflow: Workflow1) -> Self {
        MyProject::Workflow1(workflow)
    }
}

impl Workflow for Workflow1 {
    type Project = MyProject;

    type Event = Workflow1Event;

    type Step = Workflow1Step;

    const NAME: &'static str = "workflow_1";

    fn entrypoint() -> crate::step::StepWithSettings<Self::Project> {
        todo!()
    }
}

impl Project for MyProject {
    type Step = MyProjectStep;

    type Event = MyProjectEvent;

    type Workflow = Self;
}

impl ProjectWorkflow for MyProject {
    type Project = Self;

    fn entrypoint() -> crate::step::StepWithSettings<Self::Project> {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyProjectStep {
    Workflow1(Workflow1Step),
}

impl ProjectStep for MyProjectStep {
    type Project = MyProject;

    fn is_event<T: super::Event + 'static>(&self) -> bool {
        todo!()
    }

    fn is_project_event<T: ProjectEvent + 'static>(&self, event: &T) -> bool {
        todo!()
    }

    fn run_raw(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: Option<<Self::Project as Project>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<
            Option<crate::step::StepWithSettings<Self::Project>>,
            crate::step::StepError,
        >,
    > + Send {
        async move { todo!() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow1Step {
    Step0(Step0),
}

impl WorkflowStep for Workflow1Step {
    type Workflow = Workflow1;

    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<<Self::Workflow as Workflow>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<
            Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
            crate::step::StepError,
        >,
    > + Send {
        async move { todo!() }
    }

    fn is_event<T: Event + 'static>(&self) -> bool {
        todo!()
    }

    fn is_workflow_event<T: WorkflowEvent + 'static>(&self, event: &T) -> bool {
        todo!()
    }
}

impl From<Workflow1Step> for MyProjectStep {
    fn from(step: Workflow1Step) -> Self {
        MyProjectStep::Workflow1(step)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step0;

impl From<Step0> for Workflow1Step {
    fn from(step: Step0) -> Self {
        Workflow1Step::Step0(step)
    }
}

impl Step for Step0 {
    type Event = Event0;

    type Workflow = Workflow1;

    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<
            Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
            crate::step::StepError,
        >,
    > + Send {
        async move { todo!() }
    }
}

// events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyProjectEvent {
    Workflow1(Workflow1Event),
}

impl ProjectEvent for MyProjectEvent {
    type Project = MyProject;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow1Event {
    Event0(Event0),
}

impl From<Workflow1Event> for MyProjectEvent {
    fn from(event: Workflow1Event) -> Self {
        MyProjectEvent::Workflow1(event)
    }
}

impl WorkflowEvent for Workflow1Event {
    type Workflow = Workflow1;

    fn is_event<T: Event + 'static>(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event0;

impl From<Event0> for Workflow1Event {
    fn from(event: Event0) -> Self {
        Workflow1Event::Event0(event)
    }
}

impl Event for Event0 {}
