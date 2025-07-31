use aide::{OperationIo, axum::ApiRouter};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;
use derive_more::{Display, From, Into, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::error::Error;
use std::fmt::{self, Debug};
use std::{marker::PhantomData, sync::Arc};
use surgeflow_types::{
    Event, Immediate, Project, ProjectEvent, ProjectStep, ProjectStepWithSettings, ProjectWorkflow, StepError, TryAsRef, Workflow, WorkflowName, WorkflowStep
};
use uuid::Uuid;

use crate::workflows::workflow_1::{Workflow1, Workflow1Event, Workflow1Step};
use crate::workflows::workflow_2::{Workflow2, Workflow2Event, Workflow2Step};

pub mod workflow_1;
pub mod workflow_2;

////////////////// Project Implementations //////////////////

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum MyProjectEvent {
    Workflow1(Workflow1Event),
    Workflow2(Workflow2Event),
    #[serde(skip)]
    Immediate(Immediate),
}

impl ProjectEvent for MyProjectEvent {
    type Project = MyProject;
}

impl TryFrom<MyProjectEvent> for Immediate {
    type Error = ();

    fn try_from(event: MyProjectEvent) -> Result<Self, Self::Error> {
        match event {
            MyProjectEvent::Immediate(immediate) => Ok(immediate),
            _ => Err(()),
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

#[derive(Clone, From, Debug, TryInto)]
pub enum MyProjectWorkflow {
    Workflow1(Workflow1),
    Workflow2(Workflow2),
}

impl ProjectWorkflow for MyProjectWorkflow {
    type Project = MyProject;

    // TODO: this should be based on some sort of enum, not a string
    fn entrypoint(workflow_name: WorkflowName) -> ProjectStepWithSettings<Self::Project> {
        if workflow_name == Workflow1::NAME.into() {
            Workflow1::entrypoint().into()
        } else if workflow_name == Workflow2::NAME.into() {
            Workflow2::entrypoint().into()
        } else {
            panic!("Unknown workflow name");
        }
    }

    // TODO
    // async fn control_router<
    //     NewEventSenderT: crate::workers::adapters::senders::EventSender<Self::Project>,
    //     NewInstanceSenderT: crate::workers::adapters::senders::NewInstanceSender<Self::Project>,
    // >() -> anyhow::Result<
    //     ApiRouter<crate::ArcAppState<Self::Project, NewEventSenderT, NewInstanceSenderT>>,
    // > {
    //     let workflow_1_router =
    //         Workflow1::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;

    //     let workflow_2_router =
    //         Workflow2::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;

    //     Ok(ApiRouter::new()
    //         .merge(workflow_1_router)
    //         .merge(workflow_2_router))
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize, From, TryInto)]
pub enum MyProjectStep {
    Workflow1(Workflow1Step),
    Workflow2(Workflow2Step),
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

    fn is_project_event(&self, event: &<Self::Project as Project>::Event) -> bool {
        match self {
            MyProjectStep::Workflow1(step) => step.is_workflow_event(event.try_as_ref().unwrap()),
            MyProjectStep::Workflow2(step) => step.is_workflow_event(event.try_as_ref().unwrap()),
        }
    }

    async fn run_raw(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: <Self::Project as Project>::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<ProjectStepWithSettings<Self::Project>>, StepError> {
        match self {
            MyProjectStep::Workflow1(step) => {
                let step = step
                    .run_raw(wf.try_into().unwrap(), event.try_into().unwrap())
                    .await?;
                Ok(step.map(Into::into))
            }
            MyProjectStep::Workflow2(step) => {
                let step = step
                    .run_raw(wf.try_into().unwrap(), event.try_into().unwrap())
                    .await?;
                Ok(step.map(Into::into))
            }
        }
    }
}
