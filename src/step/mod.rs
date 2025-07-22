use fe2o3_amqp::{Receiver, Sender, session::SessionHandle};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt, marker::PhantomData};
use uuid::Uuid;

use crate::{
    event::Event,
    workers::adapters::managers::WorkflowInstance,
    workflows::{StepId, Workflow, WorkflowInstanceId},
};
use derive_more::Debug;

pub type StepResult<W> = Result<Option<StepWithSettings<W>>, StepError>;

pub trait Step:
    Serialize
    + for<'a> Deserialize<'a>
    + fmt::Debug
    + Into<<Self::Workflow as Workflow>::Step>
    + Send
    + Clone
{
    type Event: Event<Workflow = Self::Workflow>;
    type Workflow: Workflow;

    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<<Self::Workflow as Workflow>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<Self::Workflow>>, StepError>,
    > + Send;
}

pub trait WorkflowStep: Step + Sync {
    /// Returns the TypeId of the event type associated with this step.
    fn variant_event_type_id(&self) -> TypeId;
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("Unknown step error")]
    Unknown,
    #[error("couldn't convert event")]
    WrongEventType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepWithSettings<W: Workflow> {
    pub step: W::Step,
    pub settings: StepSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct StepSettings {
    pub max_retries: u32,
    // pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FullyQualifiedStep<W: Workflow> {
    // TODO: should probably just be a WorkflowInstanceId
    pub instance: WorkflowInstance,
    pub step_id: StepId,
    #[serde(bound = "")]
    pub step: StepWithSettings<W>,

    /// Eventful steps can be initialized without an event, but it will be set when the step is triggered.
    pub event: Option<W::Event>,
    pub retry_count: u32,

    pub previous_step_id: Option<StepId>,
    #[serde(bound = "")]
    pub next_step: Option<StepWithSettings<W>>,
}
