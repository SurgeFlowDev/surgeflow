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
    type Workflow: __Workflow<Self>
        + Serialize
        + for<'de> Deserialize<'de>
        + JsonSchema
        + fmt::Debug;
    // type Error: std::error::Error + Send + Sync + 'static;

    fn workflow_for_step(
        &self,
        step: &<Self::Workflow as __Workflow<Self>>::Step,
    ) -> Self::Workflow;

    /// Given any workflow type, return the corresponding Project level workflow
    fn workflow<T: __Workflow<Self>>() -> Self::Workflow;

    //  {

    //     // <T as  __Workflow<Self>>::project_workflow()
    // }
}

pub trait __Workflow<P: Project>: Clone + Send + Sync + 'static
where
    <Self::Step as __Step<P, Self>>::Event: From<Immediate>,
{
    type Step: __Step<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::Step>
        + TryFrom<<P::Workflow as __Workflow<P>>::Step>;

    // const NAME: &'static str;

    // TODO: make an entrypoint without &self on a new BareWorkflow trait
    fn entrypoint(&self) -> StepWithSettings<P>;

    // TODO: we should allow a "&self" receiver here, and then create a "Workflow" trait that restricts it to not have the &self receiver
    fn name(&self) -> &'static str;
}

pub trait Workflow<P: Project>: __Workflow<P>
where
    <<Self as Workflow<P>>::Step as __Step<P, Self>>::Event: From<Immediate>,
{
    type Step: __Step<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::Step>
        + TryFrom<<P::Workflow as __Workflow<P>>::Step>;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<P>;
}

impl<P: Project, W: Workflow<P>> __Workflow<P> for W {
    type Step = <W as Workflow<P>>::Step;
    fn entrypoint(&self) -> StepWithSettings<P> {
        <W as Workflow<P>>::entrypoint()
    }
    fn name(&self) -> &'static str {
        W::NAME
    }
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

pub trait __Step<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + fmt::Debug + Send + Sync + Clone
{
    type Event: __Event<P, W>
        + 'static
        + Into<<W::Step as __Step<P, W>>::Event>
        + TryFrom<<W::Step as __Step<P, W>>::Event>
        + TryFromRef<<W::Step as __Step<P, W>>::Event>
        + Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>
        + TryFromRef<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>;
    type Error: Error
        + Send
        + Sync
        + 'static
        + Into<<W::Step as __Step<P, W>>::Error>
        + TryFrom<<W::Step as __Step<P, W>>::Error>
        + Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>;

    fn run(
        &self,
        wf: W,
        event: Self::Event,
    ) -> impl Future<Output = Result<Option<StepWithSettings<P>>, <Self as __Step<P, W>>::Error>> + Send;
    fn value_has_event_value(&self, e: &Self::Event) -> bool;
}

pub trait __Event<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + Clone + fmt::Debug + Send + JsonSchema + 'static + Send + Sync
{
    fn value_is<WInner: __Workflow<P>, T: __Event<P, WInner> + 'static>(&self) -> bool;
}

// pub trait Event<P: Project, W: Workflow<P>>: __Event<P, W> {}

// impl<E: Event<P, W>, P: Project, W: Workflow<P>> __Event<P, W> for E {
//     // TODO: this functions should live in an extension trait, so that this blanket implementation can implement
//     // that extension trait and not the full __Workflow trait
//     fn value_is<WInner: Workflow<P>, T: __Event<P, WInner> + 'static>(&self) -> bool {
//         TypeId::of::<Self>() == TypeId::of::<T>()
//     }
// }

////////////////////////////////////////////////

pub type StepResult<P, W, S> = Result<Option<StepWithSettings<P>>, <S as __Step<P, W>>::Error>;

#[builder]
pub fn next_step<P: Project>(
    #[builder(into, start_fn)] step: <P::Workflow as __Workflow<P>>::Step,
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
    pub event: Option<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>,
    pub retry_count: u32,

    pub previous_step_id: Option<StepId>,
    #[serde(bound = "")]
    pub next_step: Option<StepWithSettings<P>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepWithSettings<P: Project> {
    pub step: <P::Workflow as __Workflow<P>>::Step,
    pub settings: StepSettings,
}

// TODO: implement the inverse TryFrom?

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl<P: Project, W: __Workflow<P>> __Event<P, W> for Immediate {
    fn value_is<WInner: __Workflow<P>, T: __Event<P, WInner> + 'static>(&self) -> bool {
        // TODO: Event trait
        TypeId::of::<Self>() == TypeId::of::<T>()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: <<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event,
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

impl TryFromRef<Immediate> for Immediate {
    // TODO: error type
    type Error = String;

    fn try_from_ref(value: &Immediate) -> Result<&Self, Self::Error> {
        Ok(value)
    }
}
