use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::workflows::{
    Event, Project, ProjectEvent, ProjectStep, ProjectWorkflow, Step, TryAsRef, TryFromRef,
    Workflow, WorkflowEvent, WorkflowStep,
};

pub struct MyProject;
impl Project for MyProject {
    type Step = MyProjectStep;
    type Event = MyProjectEvent;
    type Workflow = MyProjectWorkflow;
}

#[derive(Clone)]
pub enum MyProjectWorkflow {
    Workflow1(Workflow1),
}

#[derive(Clone)]
pub struct Workflow1;

impl From<Workflow1> for MyProjectWorkflow {
    fn from(workflow: Workflow1) -> Self {
        MyProjectWorkflow::Workflow1(workflow)
    }
}

impl Workflow for Workflow1 {
    type Project = MyProject;

    type Event = Workflow1Event;

    type Step = Workflow1Step;

    const NAME: &'static str = "workflow_1";

    fn entrypoint() -> crate::step::StepWithSettings<Self::Project> {
        crate::step::StepWithSettings {
            step: MyProjectStep::Workflow1(Workflow1Step::Step0(Step0)),
            settings: crate::step::StepSettings { max_retries: 3 },
        }
    }
}

impl ProjectWorkflow for MyProjectWorkflow {
    type Project = MyProject;

    fn entrypoint() -> crate::step::StepWithSettings<Self::Project> {
        match Self::Workflow1(Workflow1) {
            MyProjectWorkflow::Workflow1(_) => Workflow1::entrypoint(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyProjectStep {
    Workflow1(Workflow1Step),
}

impl ProjectStep for MyProjectStep {
    type Project = MyProject;

    fn is_event<T: super::Event + 'static>(&self) -> bool {
        match self {
            MyProjectStep::Workflow1(step) => step.is_event::<T>(),
        }
    }

    fn is_project_event(&self, event: &<Self::Project as Project>::Event) -> bool {
        match self {
            MyProjectStep::Workflow1(step) => step.is_workflow_event(event.try_as_ref().unwrap()),
        }
    }

    async fn run_raw(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: Option<<Self::Project as Project>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<crate::step::StepWithSettings<Self::Project>>, crate::step::StepError> {
        match self {
            MyProjectStep::Workflow1(step) => {
                step.run_raw(
                    wf.try_into().unwrap(),
                    event.map(TryInto::try_into).transpose().unwrap(),
                )
                .await
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow1Step {
    Step0(Step0),
}

impl WorkflowStep for Workflow1Step {
    type Workflow = Workflow1;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<<Self::Workflow as Workflow>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<
        Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
        crate::step::StepError,
    > {
        match self {
            Workflow1Step::Step0(step) => {
                step.run_raw(wf, event.unwrap().try_into().unwrap()).await
            }
        }
    }

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow1Step::Step0(_) => <Step0 as Step>::Event::is::<T>(),
        }
    }

    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool {
        match self {
            Workflow1Step::Step0(_) => event.is_event::<<Step0 as Step>::Event>(),
        }
    }
}

impl From<Workflow1Step> for MyProjectStep {
    fn from(step: Workflow1Step) -> Self {
        MyProjectStep::Workflow1(step)
    }
}

impl TryFrom<MyProjectStep> for Workflow1Step {
    type Error = ();

    fn try_from(step: MyProjectStep) -> Result<Self, Self::Error> {
        if let MyProjectStep::Workflow1(workflow_step) = step {
            Ok(workflow_step)
        } else {
            Err(())
        }
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

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<
        Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
        crate::step::StepError,
    > {
        tracing::info!("Running Step0 in Workflow1");
        Ok(None)
    }
}

impl TryFrom<Workflow1Step> for Step0 {
    type Error = ();

    fn try_from(step: Workflow1Step) -> Result<Self, Self::Error> {
        if let Workflow1Step::Step0(step0) = step {
            Ok(step0)
        } else {
            Err(())
        }
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
        match self {
            Workflow1Event::Event0(_) => Event0::is::<T>(),
        }
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
        if let MyProjectEvent::Workflow1(workflow_event) = event {
            Ok(workflow_event)
        } else {
            Err(())
        }
    }
}

impl TryFromRef<MyProjectEvent> for Workflow1Event {
    type Error = ();

    fn try_from_ref(event: &MyProjectEvent) -> Result<&Self, Self::Error> {
        if let MyProjectEvent::Workflow1(workflow_event) = event {
            Ok(workflow_event)
        } else {
            Err(())
        }
    }
}

impl TryFrom<MyProjectWorkflow> for Workflow1 {
    type Error = ();

    fn try_from(workflow: MyProjectWorkflow) -> Result<Self, Self::Error> {
        if let MyProjectWorkflow::Workflow1(workflow1) = workflow {
            Ok(workflow1)
        } else {
            Err(())
        }
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
