// #[workflow("workflow_1")]
// impl Workflow for Workflow1 {
//     type Project = MyProject;
//     type Event = event!(Event0, ...);
//     type Step = step!(Step0, Step1);
//
//     fn entrypoint() -> WorkflowStepWithSettings<Self> {
//         WorkflowStepWithSettings {
//             step: Workflow1Step::Step0(Step0),
//             settings: StepSettings { max_retries: 3 },
//         }
//     }
// }

use aide::axum::ApiRouter;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surgeflow::senders::{EventSender, NewInstanceSender};
use surgeflow::{ArcAppState, ProjectWorkflowControl, WorkflowControl};
use surgeflow::{
    ConvertingProjectStepToWorkflowStepError, ConvertingProjectWorkflowToWorkflowError, Event,
    Immediate, Project, ProjectEvent, ProjectStep, ProjectStepWithSettings, ProjectWorkflow,
    SurgeflowProjectStepError, TryAsRef, Workflow, WorkflowName, WorkflowStep,
};

use crate::workflows::workflow_1::{Workflow1, Workflow1Event, Workflow1Step};
use crate::workflows::workflow_2::{Workflow2, Workflow2Event, Workflow2Step};

pub mod workflow_1;
pub mod workflow_2;

////////////////// Project Implementations //////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyProjectEvent {
    Workflow1(Workflow1Event),
    Workflow2(Workflow2Event),
    #[serde(skip)]
    Immediate(Immediate),
}

impl From<Workflow1Event> for MyProjectEvent {
    fn from(event: Workflow1Event) -> Self {
        MyProjectEvent::Workflow1(event)
    }
}
impl From<Workflow2Event> for MyProjectEvent {
    fn from(event: Workflow2Event) -> Self {
        MyProjectEvent::Workflow2(event)
    }
}
impl From<Immediate> for MyProjectEvent {
    fn from(immediate: Immediate) -> Self {
        MyProjectEvent::Immediate(immediate)
    }
}

impl ProjectEvent for MyProjectEvent {
    type Project = MyProject;
}

impl TryFrom<MyProjectEvent> for Immediate {
    type Error = ConvertingProjectWorkflowToWorkflowError;

    fn try_from(event: MyProjectEvent) -> Result<Self, Self::Error> {
        match event {
            MyProjectEvent::Immediate(immediate) => Ok(immediate),
            _ => Err(ConvertingProjectWorkflowToWorkflowError),
        }
    }
}

#[derive(Clone)]
pub struct MyProject {
    pub workflow_1: Workflow1,
    pub workflow_2: Workflow2,
}

impl Project for MyProject {
    type Step = MyProjectStep;
    type Event = MyProjectEvent;
    type Workflow = MyProjectWorkflow;

    fn workflow_for_step(&self, step: &Self::Step) -> Self::Workflow {
        match step {
            MyProjectStep::Workflow1(_) => MyProjectWorkflow::Workflow1(self.workflow_1.clone()),
            MyProjectStep::Workflow2(_) => MyProjectWorkflow::Workflow2(self.workflow_2.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MyProjectWorkflow {
    Workflow1(Workflow1),
    Workflow2(Workflow2),
}
impl From<Workflow1> for MyProjectWorkflow {
    fn from(workflow: Workflow1) -> Self {
        MyProjectWorkflow::Workflow1(workflow)
    }
}
impl From<Workflow2> for MyProjectWorkflow {
    fn from(workflow: Workflow2) -> Self {
        MyProjectWorkflow::Workflow2(workflow)
    }
}
impl TryFrom<MyProjectWorkflow> for Workflow1 {
    type Error = ConvertingProjectWorkflowToWorkflowError;

    fn try_from(workflow: MyProjectWorkflow) -> Result<Self, Self::Error> {
        if let MyProjectWorkflow::Workflow1(wf) = workflow {
            Ok(wf)
        } else {
            Err(ConvertingProjectWorkflowToWorkflowError)
        }
    }
}
impl TryFrom<MyProjectWorkflow> for Workflow2 {
    type Error = ConvertingProjectWorkflowToWorkflowError;

    fn try_from(workflow: MyProjectWorkflow) -> Result<Self, Self::Error> {
        if let MyProjectWorkflow::Workflow2(wf) = workflow {
            Ok(wf)
        } else {
            Err(ConvertingProjectWorkflowToWorkflowError)
        }
    }
}

impl ProjectWorkflow for MyProjectWorkflow {
    type Project = MyProject;

    // TODO: this should be based on some sort of enum, not a WorkflowName
    fn entrypoint(workflow_name: WorkflowName) -> ProjectStepWithSettings<Self::Project> {
        if workflow_name == Workflow1::NAME.into() {
            Workflow1::entrypoint().into()
        } else if workflow_name == Workflow2::NAME.into() {
            Workflow2::entrypoint().into()
        } else {
            panic!("Unknown workflow name");
        }
    }
}

impl ProjectWorkflowControl for MyProjectWorkflow {
    async fn control_router<
        NewEventSenderT: EventSender<Self::Project>,
        NewInstanceSenderT: NewInstanceSender<Self::Project>,
    >() -> anyhow::Result<ApiRouter<ArcAppState<Self::Project, NewEventSenderT, NewInstanceSenderT>>>
    {
        let workflow_1_router =
            Workflow1::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;
        let workflow_2_router =
            Workflow2::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;

        Ok(ApiRouter::new()
            .merge(workflow_1_router)
            .merge(workflow_2_router))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyProjectStep {
    Workflow1(Workflow1Step),
    Workflow2(Workflow2Step),
}

impl From<Workflow1Step> for MyProjectStep {
    fn from(step: Workflow1Step) -> Self {
        MyProjectStep::Workflow1(step)
    }
}
impl From<Workflow2Step> for MyProjectStep {
    fn from(step: Workflow2Step) -> Self {
        MyProjectStep::Workflow2(step)
    }
}
impl TryFrom<MyProjectStep> for Workflow1Step {
    type Error = ConvertingProjectStepToWorkflowStepError;

    fn try_from(step: MyProjectStep) -> Result<Self, Self::Error> {
        if let MyProjectStep::Workflow1(s) = step {
            Ok(s)
        } else {
            Err(ConvertingProjectStepToWorkflowStepError)
        }
    }
}
impl TryFrom<MyProjectStep> for Workflow2Step {
    type Error = ConvertingProjectStepToWorkflowStepError;

    fn try_from(step: MyProjectStep) -> Result<Self, Self::Error> {
        if let MyProjectStep::Workflow2(s) = step {
            Ok(s)
        } else {
            Err(ConvertingProjectStepToWorkflowStepError)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MyProjectStepError {
    #[error("Workflow1 step error: {0}")]
    Workflow1(<Workflow1Step as WorkflowStep>::Error),
    #[error("Workflow2 step error: {0}")]
    Workflow2(<Workflow2Step as WorkflowStep>::Error),
}

impl ProjectStep for MyProjectStep {
    type Project = MyProject;
    type Error = MyProjectStepError;

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            MyProjectStep::Workflow1(step) => step.is_event::<T>(),
            MyProjectStep::Workflow2(step) => step.is_event::<T>(),
        }
    }

    fn is_project_event(
        &self,
        event: &<Self::Project as Project>::Event,
    ) -> Result<bool, SurgeflowProjectStepError<Self::Error>> {
        let res = match self {
            MyProjectStep::Workflow1(step) => step.is_workflow_event(event.try_as_ref()?),
            MyProjectStep::Workflow2(step) => step.is_workflow_event(event.try_as_ref()?),
        };
        Ok(res)
    }

    async fn run(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: <Self::Project as Project>::Event,
    ) -> Result<
        Option<ProjectStepWithSettings<Self::Project>>,
        SurgeflowProjectStepError<Self::Error>,
    > {
        match self {
            MyProjectStep::Workflow1(step) => {
                let step = step.run(wf.try_into()?, event.try_into()?).await?;
                Ok(step.map(Into::into))
            }
            MyProjectStep::Workflow2(step) => {
                let step = step.run(wf.try_into()?, event.try_into()?).await?;
                Ok(step.map(Into::into))
            }
        }
    }
}
