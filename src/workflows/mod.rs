use crate::event::Immediate;
use crate::step::StepError;
use crate::workers::adapters::dependencies::control_server::ControlServerDependencies;
use crate::workers::adapters::managers::{WorkflowInstance, WorkflowName};
use crate::workers::adapters::senders::NewInstanceSender;
use crate::workflows::workflow_1::{Workflow1, Workflow1Event, Workflow1Step};
use crate::workflows::workflow_2::{Workflow2, Workflow2Event, Workflow2Step};
use crate::{
    AppState, ArcAppState, event::InstanceEvent, step::StepWithSettings,
    workers::adapters::senders::EventSender,
};
use aide::{OperationIo, axum::ApiRouter};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;
use derive_more::{Display, From, Into, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::fmt::{self, Debug};
use std::{marker::PhantomData, sync::Arc};
use uuid::Uuid;

pub mod workflow_1;
pub mod workflow_2;

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

pub async fn init_app_state<
    P: Project,
    EventSenderT: EventSender<P>,
    NewInstanceSenderT: NewInstanceSender<P>,
>(
    dependencies: ControlServerDependencies<P, EventSenderT, NewInstanceSenderT>,
) -> anyhow::Result<ArcAppState<P, EventSenderT, NewInstanceSenderT>> {
    Ok(ArcAppState(Arc::new(AppState {
        dependencies,

        _marker: PhantomData,
    })))
}

pub trait WorkflowControl: Workflow {
    fn control_router<
        E: EventSender<Self::Project> + Send + Sync + 'static,
        N: NewInstanceSender<Self::Project> + Send + Sync + 'static,
    >() -> impl Future<Output = anyhow::Result<ApiRouter<ArcAppState<Self::Project, E, N>>>> + Send
    {
        async {
            let post_workflow_event_api_route = Self::post_workflow_event_api_route::<E, N>();
            let post_workflow_instance_api_route = Self::post_workflow_instance_api_route::<E, N>();

            let router = ApiRouter::new().nest(
                &format!("/workflow/{}", Self::NAME),
                ApiRouter::new()
                    .merge(post_workflow_instance_api_route)
                    .merge(post_workflow_event_api_route),
            );
            Ok(router)
        }
    }

    fn post_workflow_event_api_route<
        E: EventSender<Self::Project> + Send + Sync + 'static,
        N: NewInstanceSender<Self::Project> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self::Project, E, N>> {
        #[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
        #[typed_path("/{instance_id}/event")]
        pub struct PostWorkflowEvent {
            instance_id: WorkflowInstanceId,
        }

        #[derive(
            Debug,
            Serialize,
            Deserialize,
            JsonSchema,
            Clone,
            thiserror::Error,
            axum_thiserror::ErrorStatus,
            OperationIo,
        )]
        enum PostWorkflowEventError {
            #[error("could not queue event message")]
            #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
            CouldntQueueEventMessage,
        }

        // more readable than a closure
        async fn handler<
            T: Workflow,
            E: EventSender<T::Project>,
            N: NewInstanceSender<T::Project>,
        >(
            PostWorkflowEvent { instance_id }: PostWorkflowEvent,
            State(ArcAppState(state)): State<ArcAppState<T::Project, E, N>>,
            Json(event): Json<T::Event>,
        ) -> Result<(), PostWorkflowEventError> {
            state
                .dependencies
                .event_sender
                .send(InstanceEvent {
                    event: event.into(),
                    instance_id,
                })
                .await
                .map_err(|_| PostWorkflowEventError::CouldntQueueEventMessage)?;
            Ok(())
        }

        ApiRouter::new().typed_post_with(handler::<Self, _, _>, |op| {
            op.description("Send event")
                .summary("Send event")
                .id("post-event")
                .tag(Self::NAME)
                .hidden(false)
        })
    }

    fn post_workflow_instance_api_route<
        E: EventSender<Self::Project> + Send + Sync + 'static,
        N: NewInstanceSender<Self::Project> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self::Project, E, N>> {
        #[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
        #[typed_path("/")]
        pub struct PostWorkflowInstance;

        #[derive(
            Debug,
            Serialize,
            Deserialize,
            JsonSchema,
            Clone,
            thiserror::Error,
            axum_thiserror::ErrorStatus,
            OperationIo,
        )]
        enum PostWorkflowInstanceError {
            #[error("could not create instance")]
            #[status(StatusCode::INTERNAL_SERVER_ERROR)]
            CouldntCreateInstance,
        }

        // more readable than a closure
        async fn handler<
            T: Workflow,
            E: EventSender<T::Project>,
            N: NewInstanceSender<T::Project>,
        >(
            _: PostWorkflowInstance,
            State(ArcAppState(state)): State<ArcAppState<T::Project, E, N>>,
        ) -> Result<Json<WorkflowInstanceId>, PostWorkflowInstanceError> {
            tracing::debug!("creating instance...");
            let external_id = WorkflowInstanceId::new();
            state
                .dependencies
                .new_instance_sender
                .send(&WorkflowInstance {
                    external_id,
                    workflow_name: T::NAME.into(),
                })
                .await
                .map_err(|_| PostWorkflowInstanceError::CouldntCreateInstance)?;

            Ok(Json(external_id))
        }
        ApiRouter::new().typed_post_with(handler::<Self, _, _>, |op| {
            op.description("Create instance")
                .summary("Create instance")
                .id("post-workflow-instance")
                .tag(Self::NAME)
                .hidden(false)
        })
    }
}

impl<T: Workflow> WorkflowControl for T {}

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

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_project_event(&self, event: &<Self::Project as Project>::Event) -> bool;
    fn run_raw(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: <Self::Project as Project>::Event,
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<Self::Project>>, StepError>,
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

    // TODO: this should be based on some sort of enum, not a string
    fn entrypoint(workflow_name: WorkflowName) -> StepWithSettings<Self::Project>;

    fn control_router<
        NewEventSenderT: EventSender<Self::Project>,
        NewInstanceSenderT: NewInstanceSender<Self::Project>,
    >() -> impl Future<
        Output = anyhow::Result<
            ApiRouter<ArcAppState<Self::Project, NewEventSenderT, NewInstanceSenderT>>,
        >,
    > + Send;
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

    fn entrypoint() -> StepWithSettings<Self::Project>;
}

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
    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: <Self::Workflow as Workflow>::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Project>>, StepError>,
    > + Send;

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool;
}

pub trait WorkflowEvent:
    Serialize
    + for<'de> Deserialize<'de>
    + Debug
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

    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Project>>, StepError>,
    > + Send;
}

pub trait Event:
    Serialize + for<'a> Deserialize<'a> + Clone + Debug + Send + JsonSchema + 'static
{
    // move to extension trait so it can't be overridden
    fn is<T: Event + 'static>() -> bool {
        TypeId::of::<Self>() == TypeId::of::<T>()
    }
}

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

#[derive(Clone, From, TryInto, Debug)]
pub enum MyProjectWorkflow {
    Workflow1(Workflow1),
    Workflow2(Workflow2),
}

impl ProjectWorkflow for MyProjectWorkflow {
    type Project = MyProject;

    // TODO: this should be based on some sort of enum, not a string
    fn entrypoint(workflow_name: WorkflowName) -> crate::step::StepWithSettings<Self::Project> {
        if workflow_name == Workflow1::NAME.into() {
            Workflow1::entrypoint()
        } else if workflow_name == Workflow2::NAME.into() {
            Workflow2::entrypoint()
        } else {
            panic!("Unknown workflow name");
        }
    }

    async fn control_router<
        NewEventSenderT: crate::workers::adapters::senders::EventSender<Self::Project>,
        NewInstanceSenderT: crate::workers::adapters::senders::NewInstanceSender<Self::Project>,
    >() -> anyhow::Result<
        ApiRouter<crate::ArcAppState<Self::Project, NewEventSenderT, NewInstanceSenderT>>,
    > {
        let workflow_1_router =
            Workflow1::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;

        let workflow_2_router =
            Workflow2::control_router::<NewEventSenderT, NewInstanceSenderT>().await?;

        Ok(ApiRouter::new()
            .merge(workflow_1_router)
            .merge(workflow_2_router))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, From, TryInto)]
pub enum MyProjectStep {
    Workflow1(Workflow1Step),
    Workflow2(Workflow2Step),
}

impl ProjectStep for MyProjectStep {
    type Project = MyProject;

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
    ) -> Result<Option<crate::step::StepWithSettings<Self::Project>>, crate::step::StepError> {
        match self {
            MyProjectStep::Workflow1(step) => {
                step.run_raw(wf.try_into().unwrap(), event.try_into().unwrap())
                    .await
            }
            MyProjectStep::Workflow2(step) => {
                step.run_raw(wf.try_into().unwrap(), event.try_into().unwrap())
                    .await
            }
        }
    }
}
