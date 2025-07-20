use fe2o3_amqp::{Receiver, Sender, session::SessionHandle};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug, marker::PhantomData};
use uuid::Uuid;

use crate::{
    event::Event,
    workflows::{StepId, Workflow, WorkflowInstanceId},
};

pub type StepResult<W> = Result<Option<StepWithSettings<<W as Workflow>::Step>>, StepError>;

pub trait Step:
    Serialize
    + for<'a> Deserialize<'a>
    + Debug
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
        Output = Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Step>>, StepError>,
    > + Send;
}

pub trait WorkflowStep: Step {
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
pub struct StepWithSettings<S: Debug + Step + for<'a> Deserialize<'a>> {
    #[serde(bound = "")]
    pub step: S,
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
pub struct FullyQualifiedStep<S: Debug + Step + for<'a> Deserialize<'a>> {
    pub instance_id: WorkflowInstanceId,
    pub step_id: StepId,
    #[serde(bound = "")]
    pub step: StepWithSettings<S>,
    pub event: Option<<S::Workflow as Workflow>::Event>,
    pub retry_count: u32,
}