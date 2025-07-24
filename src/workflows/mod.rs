use crate::event::Event;
use crate::step::{FullyQualifiedStep, StepError};
use crate::workers::adapters::managers::WorkflowInstance;
// use crate::workflows::workflow_0::Workflow0;
use crate::{
    AppState, ArcAppState, WorkflowInstanceManager,
    event::InstanceEvent,
    step::StepWithSettings,
    workers::adapters::{dependencies::control_server::ControlServerContext, senders::EventSender},
};
use aide::{NoApi, OperationIo, axum::ApiRouter};
use axum::{
    Json,
    extract::{FromRef, State},
    http::StatusCode,
};
use axum_extra::routing::TypedPath;
use derive_more::{Display, From, Into};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use std::fmt::{self, Debug};
use std::{marker::PhantomData, sync::Arc};
use uuid::Uuid;

// pub mod workflow_0;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema,
)]
pub struct WorkflowInstanceId(Uuid);

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

pub type Tx = axum_sqlx_tx::Tx<sqlx::Postgres>;
pub type TxState = axum_sqlx_tx::State<Postgres>;
pub type TxLayer = axum_sqlx_tx::Layer<Postgres, axum_sqlx_tx::Error>;

impl<P: Project, E: EventSender<P>, M: WorkflowInstanceManager<P>> FromRef<ArcAppState<P, E, M>>
    for TxState
{
    fn from_ref(input: &ArcAppState<P, E, M>) -> Self {
        input.0.sqlx_tx_state.clone()
    }
}

pub async fn init_app_state<P: Project, D: ControlServerContext<P>>(
    sqlx_tx_state: TxState,
) -> anyhow::Result<ArcAppState<P, D::EventSender, D::InstanceManager>> {
    let dependencies = D::dependencies().await?;
    let event_sender = dependencies.event_sender;
    let workflow_instance_manager = dependencies.instance_manager;

    Ok(ArcAppState(Arc::new(AppState {
        event_sender,
        workflow_instance_manager,
        sqlx_tx_state,
        _marker: PhantomData,
    })))
}

pub trait WorkflowControl: Workflow {
    fn control_router<
        E: EventSender<Self::Project> + Send + Sync + 'static,
        M: WorkflowInstanceManager<Self::Project> + Send + Sync + 'static,
    >() -> impl Future<Output = anyhow::Result<ApiRouter<ArcAppState<Self::Project, E, M>>>> + Send
    {
        async {
            let post_workflow_event_api_route = Self::post_workflow_event_api_route::<E, M>();
            let post_workflow_instance_api_route = Self::post_workflow_instance_api_route::<E, M>();

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
        M: WorkflowInstanceManager<Self::Project> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self::Project, E, M>> {
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
            M: WorkflowInstanceManager<T::Project>,
        >(
            PostWorkflowEvent { instance_id }: PostWorkflowEvent,
            State(ArcAppState(state)): State<ArcAppState<T::Project, E, M>>,
            Json(event): Json<T::Event>,
        ) -> Result<(), PostWorkflowEventError> {
            state
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
        M: WorkflowInstanceManager<Self::Project> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self::Project, E, M>> {
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
            M: WorkflowInstanceManager<T::Project>,
        >(
            _: PostWorkflowInstance,
            NoApi(mut tx): NoApi<Tx>,
            State(ArcAppState(state)): State<ArcAppState<T::Project, E, M>>,
        ) -> Result<Json<WorkflowInstance>, PostWorkflowInstanceError> {
            tracing::info!("creating instance...");
            let res = state
                .workflow_instance_manager
                .create_instance(&mut tx)
                .await
                .map_err(|_| PostWorkflowInstanceError::CouldntCreateInstance)?;
            Ok(Json(res))
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

pub trait Project: Sized + Send + Sync + 'static {
    type Step: ProjectStep<Project = Self>;
    type Event: ProjectEvent<Project = Self>;
    type Workflow: ProjectWorkflow<Project = Self>;
}

pub trait ProjectStep:
    Sized + Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>
{
    type Project: Project<Step = Self>;

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_project_event<T: ProjectEvent + 'static>(&self, event: &T) -> bool;
    fn run_raw(
        &self,
        wf: <Self::Project as Project>::Workflow,
        event: Option<<Self::Project as Project>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<Self::Project>>, StepError>,
    > + Send;
}
pub trait ProjectEvent:
    Sized + Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>
{
    type Project: Project<Event = Self>;
}
pub trait ProjectWorkflow:
    Sized + Send + Sync + 'static + Clone + Serialize + for<'de> Deserialize<'de>
{
    type Project: Project<Workflow = Self>;

    fn entrypoint() -> StepWithSettings<Self::Project>;
}

///////////////////////////////////////////////////////////////////////////////////////////

pub trait Workflow:
    Clone + Send + Sync + 'static + Into<<Self::Project as Project>::Workflow>
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
{
    type Workflow: Workflow<Step = Self>;
    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<<Self::Workflow as Workflow>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Project>>, StepError>,
    > + Send;

    fn is_event<T: Event + 'static>(&self) -> bool;
    fn is_workflow_event<T: WorkflowEvent + 'static>(&self, event: &T) -> bool;
}

pub trait WorkflowEvent:
    Serialize + for<'de> Deserialize<'de> + Debug + Send + Clone
    + Into<<<Self::Workflow as Workflow>::Project as Project>::Event>
    // JsonSchema should probably be implemented as an extension trait. JsonSchema doesn't matter for the 
    // inner workings of the workflow, but it is useful for API documentation
     + JsonSchema
{
    type Workflow: Workflow<Event = Self>;
    fn is_event<T: Event + 'static>(&self) -> bool;
}
