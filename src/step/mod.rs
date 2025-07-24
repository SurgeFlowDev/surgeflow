use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    workers::adapters::managers::WorkflowInstance,
    workflows::{Project, StepId, Workflow, WorkflowEvent},
};
use derive_more::Debug;

pub type StepResult<W> = Result<Option<StepWithSettings<W>>, StepError>;

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("Unknown step error")]
    Unknown,
    #[error("couldn't convert event")]
    WrongEventType,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct StepSettings {
    pub max_retries: u32,
    // pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct OldFullyQualifiedStep<W: Workflow> {
//     // TODO: should probably just be a WorkflowInstanceId
//     pub instance: WorkflowInstance,
//     pub step_id: StepId,
//     #[serde(bound = "")]
//     pub step: StepWithSettings<W>,

//     /// Eventful steps can be initialized without an event, but it will be set when the step is triggered.
//     pub event: Option<W::Event>,
//     pub retry_count: u32,

//     pub previous_step_id: Option<StepId>,
//     #[serde(bound = "")]
//     pub next_step: Option<StepWithSettings<W>>,
// }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FullyQualifiedStep<P: Project> {
    // TODO: should probably just be a WorkflowInstanceId
    pub instance: WorkflowInstance,
    pub step_id: StepId,
    #[serde(bound = "")]
    pub step: StepWithSettings<P>,

    /// Eventful steps can be initialized without an event, but it will be set when the step is triggered.
    pub event: Option<P::Event>,
    pub retry_count: u32,

    pub previous_step_id: Option<StepId>,
    #[serde(bound = "")]
    pub next_step: Option<StepWithSettings<P>>,
}

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct OldStepWithSettings<W: Workflow> {
//     pub step: W::Step,
//     pub settings: StepSettings,
// }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepWithSettings<P: Project> {
    pub step: P::Step,
    pub settings: StepSettings,
}
