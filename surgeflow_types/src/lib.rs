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

pub trait Project: Sized + Send + Sync + 'static + Clone
where
    <<Self::Workflow as __Workflow<Self>>::Step as __Step<Self, Self::Workflow>>::Event:
        From<Immediate>,
{
    type Workflow: __Workflow<Self>;

    ///
    fn workflow_for_step(
        &self,
        step: &<Self::Workflow as __Workflow<Self>>::Step,
    ) -> Self::Workflow;
}

pub trait __Workflow<P: Project>: Clone + Send + Sync + 'static
where
    <<Self as __Workflow<P>>::Step as __Step<P, Self>>::Event: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>,
    <<Self as __Workflow<P>>::Step as __Step<P, Self>>::Error: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>,
{
    type Step: __Step<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::Step>
        + TryFrom<<P::Workflow as __Workflow<P>>::Step>;
    type WorkflowStatic: __WorkflowStatic<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::WorkflowStatic>
        + TryFrom<<P::Workflow as __Workflow<P>>::WorkflowStatic>;
}

pub trait __WorkflowStatic<P: Project, W: __Workflow<P>>:
    Clone
    + Send
    + Sync
    + 'static
    + Serialize
    + Clone
    + Copy
    + Send
    + Sync
    + for<'a> Deserialize<'a>
    + fmt::Debug
    + JsonSchema
{
    fn entrypoint(&self) -> StepWithSettings<P>;
    fn name(&self) -> &'static str;
}

pub trait Workflow<P: Project>:
    __Workflow<
        P,
        Step = <Self as Workflow<P>>::Step,
        WorkflowStatic = <Self as Workflow<P>>::WorkflowStatic,
    >
where
    <<Self as Workflow<P>>::Step as __Step<P, Self>>::Event: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>,
    <<Self as Workflow<P>>::Step as __Step<P, Self>>::Error: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>,
{
    type WorkflowStatic: __WorkflowStatic<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::WorkflowStatic>
        + TryFrom<<P::Workflow as __Workflow<P>>::WorkflowStatic>;
    type Step: __Step<P, Self>
        + Into<<P::Workflow as __Workflow<P>>::Step>
        + TryFrom<<P::Workflow as __Workflow<P>>::Step>;
    const NAME: &'static str;
    const WORKFLOW_STATIC: <Self as __Workflow<P>>::WorkflowStatic;

    fn entrypoint() -> StepWithSettings<P>;
}

impl<P: Project, W: Workflow<P>> __Workflow<P> for W {
    type Step = <W as Workflow<P>>::Step;
    type WorkflowStatic = <W as Workflow<P>>::WorkflowStatic;
}

impl<P: Project, W: Workflow<P>> __WorkflowStatic<P, W> for <W as Workflow<P>>::WorkflowStatic {
    fn entrypoint(&self) -> StepWithSettings<P> {
        <W as Workflow<P>>::entrypoint()
    }

    fn name(&self) -> &'static str {
        <W as Workflow<P>>::NAME
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

// pub trait TryFromRef<T: ?Sized> {
//     type Error;
//     fn try_from_ref(value: &T) -> Result<&Self, Self::Error>;
// }

// pub trait TryAsRef<T: ?Sized> {
//     type Error;
//     fn try_as_ref(&self) -> Result<&T, Self::Error>;
// }

// impl<T: ?Sized, U: TryFromRef<T>> TryAsRef<U> for T {
//     type Error = U::Error;

//     fn try_as_ref(&self) -> Result<&U, Self::Error> {
//         U::try_from_ref(self)
//     }
// }

pub trait __Step<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + fmt::Debug + Send + Sync + Clone
where
    <W::Step as __Step<P, W>>::Event: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event>,
    <W::Step as __Step<P, W>>::Error: Into<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>
        + TryFrom<<<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Error>,
{
    type Event: __Event<P, W>
        + 'static
        + Into<<W::Step as __Step<P, W>>::Event>
        + TryFrom<<W::Step as __Step<P, W>>::Event>;
    type Error: Error
        + Send
        + Sync
        + 'static
        + Into<<W::Step as __Step<P, W>>::Error>
        + TryFrom<<W::Step as __Step<P, W>>::Error>;

    fn run(
        &self,
        wf: W,
        event: Self::Event,
    ) -> impl Future<Output = Result<Option<StepWithSettings<P>>, <Self as __Step<P, W>>::Error>> + Send;

    fn event_is_event(&self, event: &Self::Event) -> bool;
}

pub trait __Event<P: Project, W: __Workflow<P>>:
    Serialize + for<'a> Deserialize<'a> + Clone + fmt::Debug + Send + JsonSchema + 'static + Send + Sync
{
}

pub trait Event<P: Project, W: __Workflow<P>>: __Event<P, W> {}

impl<P: Project, W: __Workflow<P>, E: Event<P, W>> __Event<P, W> for E {}

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

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl<P: Project, W: __Workflow<P>> Event<P, W> for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: <<P::Workflow as __Workflow<P>>::Step as __Step<P, P::Workflow>>::Event,
    pub instance_id: WorkflowInstanceId,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct WorkflowInstance<P: Project> {
    pub external_id: WorkflowInstanceId,
    pub workflow: <P::Workflow as __Workflow<P>>::WorkflowStatic,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, From, Into, PartialEq, Eq)]
#[serde(transparent)]
pub struct WorkflowName(String);

impl From<&str> for WorkflowName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

// #[generic]
// mod IAmGeneric {
//     type A = generic!(); // placeholder, removed by the macro

//     impl ::core::fmt::Display for A {
//         fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
//             f.write_str("common implementation")
//         }
//     }
// }

// struct A;
// struct B;

// #[concrete(IAmGeneric)]
// mod IamConcreteA { type A = crate::A; }

// #[concrete(IAmGeneric)]
// mod IamConcreteB { type A = crate::B; }
