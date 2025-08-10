use bon::builder;
use derive_more::{Debug, Display, From, Into};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::error::Error;
use std::fmt::{self};
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema,
)]
pub struct WorkflowInstanceId(Uuid);

impl From<WorkflowInstanceId> for [u8; 16] {
    fn from(val: WorkflowInstanceId) -> Self {
        val.0.into_bytes()
    }
}

impl fmt::Display for WorkflowInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

impl WorkflowInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema,
)]
pub struct StepId(Uuid);

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

impl StepId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowId(i32);

///////////////////////////////////////////////////////////////////////////////////////////

pub trait Project: Sized + Send + Sync + 'static + Clone {
    type Step: __Step<Self, Self::Workflow, Event = Self::Event, Error = Self::Error>;
    type Event: __Event<Self, Self::Workflow>
        + From<<Self::Workflow as __Workflow<Self>>::Event>
        + From<Immediate>;
    type Workflow: __Workflow<Self, Error = Self::Error>
        + Serialize
        + for<'de> Deserialize<'de>
        + JsonSchema
        + fmt::Debug;
    type Error: std::error::Error + Send + Sync + 'static;

    fn workflow_for_step(&self, step: &Self::Step) -> Self::Workflow;

    /// Given any workflow type, return the corresponding Project level workflow
    fn workflow<T: Workflow<Self>>() -> Self::Workflow;
}

///////////////////////////////////////////////////////////////////////////////////////////

pub trait Workflow<P: Project>:
    __Workflow<
        P,
        Event = <Self as Workflow<P>>::Event,
        Step = <Self as Workflow<P>>::Step,
        Error = <Self as Workflow<P>>::Error,
    >
{
    type Event: __Event<P, Self>
        + From<<<Self as Workflow<P>>::Step as __Step<P, Self>>::Event>
        + Into<P::Event>;
    type Step: __Step<P, Self, Event = <Self as Workflow<P>>::Event, Error = <Self as Workflow<P>>::Error>;
    type Error: Error + Send + Sync + 'static;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<P>;
}

impl<P: Project, W: Workflow<P>> __Workflow<P> for W {
    type Event = <W as Workflow<P>>::Event;
    type Step = <W as Workflow<P>>::Step;
    type Error = <W as Workflow<P>>::Error;
    const NAME: &'static str = <W as Workflow<P>>::NAME;

    // TODO: this functions should live in an extension trait, so that this blanket implementation can implement
    // that extension trait and not the full __Workflow trait
    fn entrypoint(&self) -> StepWithSettings<P> {
        <W as Workflow<P>>::entrypoint()
    }
}

pub trait __Workflow<P: Project>: Clone + Send + Sync + 'static {
    // type Project: Project;
    type Event: __Event<P, Self> + From<<Self::Step as __Step<P, Self>>::Event> + Into<P::Event>;
    type Step: __Step<P, Self, Event = Self::Event, Error = Self::Error>;
    type Error: std::error::Error + Send + Sync + 'static;
    const NAME: &'static str;

    // TODO: make an entrypoint without &self on a new BareWorkflow trait
    fn entrypoint(&self) -> StepWithSettings<P>;
}

#[derive(thiserror::Error, Debug)]
pub enum SurgeflowWorkflowStepError<E> {
    #[error("Step error: {0}")]
    StepError(E),
    #[error("Converting workflow event to event failed: {0}")]
    ConvertingWorkflowEventToEvent(#[from] ConvertingWorkflowEventToEventError),
    #[error("Converting workflow step to step failed: {0}")]
    ConvertingWorkflowStepToStep(#[from] ConvertingWorkflowStepToStepError),
}

#[derive(thiserror::Error, Debug)]
pub enum SurgeflowProjectStepError<E> {
    #[error("Workflow step error: {0}")]
    WorkflowStepError(SurgeflowWorkflowStepError<E>),
    #[error("Converting project event to workflow event failed: {0}")]
    ConvertingProjectEventToWorkflowEvent(#[from] ConvertingProjectEventToWorkflowEventError),
    #[error("Converting project workflow to workflow failed: {0}")]
    ConvertingProjectWorkflowToWorkflow(#[from] ConvertingProjectWorkflowToWorkflowError),
    #[error("Converting project step to workflow step failed: {0}")]
    ConvertingProjectStepToWorkflowStep(#[from] ConvertingProjectStepToWorkflowStepError),
}

impl<E1, E2: From<E1>> From<SurgeflowWorkflowStepError<E1>> for SurgeflowProjectStepError<E2> {
    fn from(error: SurgeflowWorkflowStepError<E1>) -> Self {
        match error {
            SurgeflowWorkflowStepError::StepError(e) => {
                SurgeflowProjectStepError::WorkflowStepError(SurgeflowWorkflowStepError::StepError(
                    e.into(),
                ))
            }
            SurgeflowWorkflowStepError::ConvertingWorkflowStepToStep(e) => {
                SurgeflowProjectStepError::WorkflowStepError(
                    SurgeflowWorkflowStepError::ConvertingWorkflowStepToStep(e),
                )
            }
            SurgeflowWorkflowStepError::ConvertingWorkflowEventToEvent(e) => {
                SurgeflowProjectStepError::WorkflowStepError(
                    SurgeflowWorkflowStepError::ConvertingWorkflowEventToEvent(e),
                )
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("converting workflow step to step failed")]
pub struct ConvertingWorkflowStepToStepError;

#[derive(thiserror::Error, Debug)]
#[error("converting project step to workflow step failed")]
pub struct ConvertingProjectStepToWorkflowStepError;

#[derive(thiserror::Error, Debug)]
#[error("converting project event to workflow event failed")]
pub struct ConvertingWorkflowEventToEventError;

#[derive(thiserror::Error, Debug)]
#[error("converting project workflow to workflow failed")]
pub struct ConvertingProjectWorkflowToWorkflowError;

#[derive(thiserror::Error, Debug)]
#[error("converting project event to workflow event failed")]
pub struct ConvertingProjectEventToWorkflowEventError;

pub trait TryFromRef<T: ?Sized> {
    type Error;
    fn try_from_ref(value: &T) -> Result<&Self, Self::Error>;
}

pub trait TryAsRef<T: ?Sized> {
    type Error;
    fn try_as_ref(&self) -> Result<&T, Self::Error>;
}

impl<T: ?Sized, U: TryFromRef<T>> TryAsRef<U> for T {
    type Error = U::Error;

    fn try_as_ref(&self) -> Result<&U, Self::Error> {
        U::try_from_ref(self)
    }
}

//////////////////////////////////

pub trait Step<P: Project, W: Workflow<P>>:
    __Step<P, W, Event = <Self as Step<P, W>>::Event, Error = <Self as Step<P, W>>::Error>
{
    type Event: Event<P, W> + 'static;
    type Error: Error + Send + Sync + 'static;

    // TODO: waiting on https://rust-lang.github.io/rfcs/3654-return-type-notation.html
    // then can add a blanket implementation for `__Step`
    // fn run(&self, wf: W, event: <Self as Step<P, W>>::Event) -> <Self as __Step<P, W>>::run(..);
}

pub trait __Step<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + fmt::Debug + Send + Sync + Clone
{
    type Event: __Event<P, W> + 'static;
    type Error: Error + Send + Sync + 'static;

    fn run(
        &self,
        wf: W,
        event: Self::Event,
    ) -> impl Future<Output = Result<Option<StepWithSettings<P>>, <Self as __Step<P, W>>::Error>> + Send;
    fn value_has_event_value<T: __Event<P, W> + 'static>(&self, e: &T) -> bool {
        e.value_is::<Self::Event>()
    }
}

pub trait __Event<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + Clone + fmt::Debug + Send + JsonSchema + 'static + Send + Sync
{
    fn value_is<T: __Event<P, W> + 'static>(&self) -> bool;
}

pub trait Event<P: Project, W: Workflow<P>>: __Event<P, W> {}

impl<E: Event<P, W>, P: Project, W: Workflow<P>> __Event<P, W> for E {
    // TODO: this functions should live in an extension trait, so that this blanket implementation can implement
    // that extension trait and not the full __Workflow trait
    fn value_is<T: __Event<P, W> + 'static>(&self) -> bool {
        TypeId::of::<Self>() == TypeId::of::<T>()
    }
}

////////////////////////////////////////////////

pub type StepResult<P, W, S> = Result<Option<StepWithSettings<P>>, <S as __Step<P, W>>::Error>;

#[builder]
pub fn next_step<P: Project>(
    #[builder(into, start_fn)] step: P::Step,
    max_retries: u32,
) -> StepWithSettings<P> {
    StepWithSettings {
        step,
        settings: StepSettings { max_retries },
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct StepSettings {
    pub max_retries: u32,
    // TODO
    // pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FullyQualifiedStep<P: Project> {
    // TODO: should probably just be a WorkflowInstanceId
    pub instance: WorkflowInstance<P>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepWithSettings<P: Project> {
    pub step: P::Step,
    pub settings: StepSettings,
}

// TODO: implement the inverse TryFrom?

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl<P: Project, W: Workflow<P>> Event<P, W> for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: P::Event,
    pub instance_id: WorkflowInstanceId,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct WorkflowInstance<P: Project> {
    pub external_id: WorkflowInstanceId,
    // pub workflow_name: WorkflowName,
    pub workflow: P::Workflow,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, From, Into, PartialEq, Eq)]
#[serde(transparent)]
pub struct WorkflowName(String);

impl From<&str> for WorkflowName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
