use derive_more::{Debug, Display, From, Into};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::error::Error;
use std::fmt;
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
    type Step: ProjectStep<Project = Self>;
    type Event: ProjectEvent<Project = Self>;
    type Workflow: ProjectWorkflow<Project = Self>;

    fn workflow_for_step(&self, step: &Self::Step) -> Self::Workflow;
}

pub trait ProjectStep:
    Sized + Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>
{
    type Project: Project<Step = Self>;
    type Error: Error + Send + Sync + 'static;

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_project_event(&self, event: &<Self::Project as Project>::Event) -> bool;
    fn run(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: <Self::Project as Project>::Event,
    ) -> impl std::future::Future<
        Output = Result<
            Option<ProjectStepWithSettings<Self::Project>>,
            SurgeflowProjectStepError<Self::Error>,
        >,
    > + Send;
}
pub trait ProjectEvent:
    Sized
    + Send
    + Sync
    + 'static
    + Clone
    + Serialize
    + for<'de> Deserialize<'de>
    + From<Immediate>
    + TryInto<Immediate>
{
    type Project: Project<Event = Self>;
}
pub trait ProjectWorkflow: Sized + Send + Sync + 'static + Clone {
    type Project: Project<Workflow = Self>;

    // TODO: this should be based on some sort of enum, not WorkflowName
    fn entrypoint(workflow_name: WorkflowName) -> ProjectStepWithSettings<Self::Project>;
}

///////////////////////////////////////////////////////////////////////////////////////////

pub trait Workflow:
    Clone
    + Send
    + Sync
    + 'static
    + Into<<Self::Project as Project>::Workflow>
    + TryFrom<<Self::Project as Project>::Workflow>
{
    type Project: Project;
    type Event: WorkflowEvent<Workflow = Self>;
    type Step: WorkflowStep<Workflow = Self>;
    const NAME: &'static str;

    fn entrypoint() -> WorkflowStepWithSettings<Self>;
}

#[derive(thiserror::Error, Debug)]
pub enum SurgeflowWorkflowStepError<E> {
    StepError(E),
    ConvertingWorkflowEventToEvent(#[from] ConvertingWorkflowEventToEventError),
    ConvertingWorkflowStepToStep(#[from] ConvertingWorkflowStepToStepError),
}

#[derive(thiserror::Error, Debug)]
pub enum SurgeflowProjectStepError<E> {
    WorkflowStepError(SurgeflowWorkflowStepError<E>),
    ConvertingProjectEventToWorkflowEvent(#[from] ConvertingProjectEventToWorkflowEventError),
    ConvertingProjectWorkflowToWorkflow(#[from] ConvertingProjectWorkflowToWorkflowError),
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

pub trait WorkflowStep:
    Sync
    + Serialize
    + for<'de> Deserialize<'de>
    + Clone
    + Send
    + fmt::Debug
    + Into<<<Self::Workflow as Workflow>::Project as Project>::Step>
    + TryFrom<<<Self::Workflow as Workflow>::Project as Project>::Step>
{
    type Workflow: Workflow<Step = Self>;
    type Error: Error
        + Send
        + Sync
        + 'static
        // TODO: creating a ProjectStepError/WorkflowStepError/StepError trait would make this cleaner
        // but this would come at the expense of users getting less clear error messaages when implementing these traits
        + TryFrom<<<<Self::Workflow as Workflow>::Project as Project>::Step as ProjectStep>::Error>
        + Into<<<<Self::Workflow as Workflow>::Project as Project>::Step as ProjectStep>::Error>;

    fn run(
        &self,
        wf: Self::Workflow,
        event: <Self::Workflow as Workflow>::Event,
    ) -> impl std::future::Future<
        Output = Result<
            Option<WorkflowStepWithSettings<Self::Workflow>>,
            SurgeflowWorkflowStepError<<Self as WorkflowStep>::Error>,
        >,
    > + Send;

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool;
}

pub trait WorkflowEvent:
    Serialize
    + for<'de> Deserialize<'de>
    + fmt::Debug
    + Send
    + Clone
    + Into<<<Self::Workflow as Workflow>::Project as Project>::Event>
    + TryFrom<<<Self::Workflow as Workflow>::Project as Project>::Event>
    + TryFromRef<<<Self::Workflow as Workflow>::Project as Project>::Event>
    + From<Immediate>
    + TryInto<Immediate>
    + JsonSchema
{
    type Workflow: Workflow<Event = Self>;
    fn is_event<T: Event + 'static>(&self) -> bool;
}

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

pub trait Step:
    Serialize
    + for<'a> Deserialize<'a>
    + fmt::Debug
    + Into<<Self::Workflow as Workflow>::Step>
    + TryFrom<<Self::Workflow as Workflow>::Step>
    + Send
    + Clone
{
    type Event: Event + 'static;
    type Workflow: Workflow;
    type Error: Error
        + Send
        + Sync
        + 'static
        // TODO: creating a ProjectStepError/WorkflowStepError/StepError trait would make this cleaner
        // but this would come at the expense of users getting less clear error messaages when implementing these traits
        + TryFrom<<<Self::Workflow as Workflow>::Step as WorkflowStep>::Error>
        + Into<<<Self::Workflow as Workflow>::Step as WorkflowStep>::Error>;

    fn run(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
    ) -> impl std::future::Future<
        Output = Result<Option<WorkflowStepWithSettings<Self::Workflow>>, <Self as Step>::Error>,
    > + Send;
}

pub trait Event:
    Serialize + for<'a> Deserialize<'a> + Clone + fmt::Debug + Send + JsonSchema + 'static
{
    // move to extension trait so it can't be overridden
    fn is<T: Event + 'static>() -> bool {
        TypeId::of::<Self>() == TypeId::of::<T>()
    }
}

////////////////////////////////////////////////

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
    pub instance: WorkflowInstance,
    pub step_id: StepId,
    #[serde(bound = "")]
    pub step: ProjectStepWithSettings<P>,

    /// Eventful steps can be initialized without an event, but it will be set when the step is triggered.
    pub event: Option<P::Event>,
    pub retry_count: u32,

    pub previous_step_id: Option<StepId>,
    #[serde(bound = "")]
    pub next_step: Option<ProjectStepWithSettings<P>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectStepWithSettings<P: Project> {
    pub step: P::Step,
    pub settings: StepSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowStepWithSettings<W: Workflow> {
    pub step: W::Step,
    pub settings: StepSettings,
}

impl<W: Workflow> From<WorkflowStepWithSettings<W>> for ProjectStepWithSettings<W::Project> {
    fn from(value: WorkflowStepWithSettings<W>) -> Self {
        ProjectStepWithSettings {
            step: value.step.into(),
            settings: value.settings,
        }
    }
}
// TODO: implement the inverse TryFrom?

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl Event for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: P::Event,
    pub instance_id: WorkflowInstanceId,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct WorkflowInstance {
    pub external_id: WorkflowInstanceId,
    pub workflow_name: WorkflowName,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, From, Into, PartialEq, Eq)]
#[serde(transparent)]
pub struct WorkflowName(String);

impl From<&str> for WorkflowName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
