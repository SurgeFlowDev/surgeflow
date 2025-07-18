use crate::workers::adapters::managers::WorkflowInstance;
use crate::{
    AppState, ArcAppState, WorkflowInstanceManager,
    event::{Event, InstanceEvent},
    step::{Step, StepWithSettings, WorkflowStep},
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
use std::fmt::Debug;
use std::{any::TypeId, marker::PhantomData, sync::Arc};

pub mod workflow_0;
pub mod workflow_1;

pub trait WorkflowEvent: Event + Debug + JsonSchema + Send {
    fn variant_type_id(&self) -> TypeId;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowInstanceId(i32);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowId(i32);
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into, JsonSchema, Display,
)]
pub struct WorkflowName(String);

impl AsRef<str> for WorkflowName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub trait Workflow: Clone + Send + Sync + 'static {
    type Event: Event<Workflow = Self> + JsonSchema + WorkflowEvent;
    type Step: Step<Workflow = Self, Event = Self::Event> + WorkflowStep;
    const NAME: &'static str;

    fn entrypoint() -> StepWithSettings<Self::Step>;
}

pub trait WorkflowExt: Workflow {
    fn name() -> WorkflowName {
        String::from(Self::NAME).into()
    }
}

impl<T: Workflow> WorkflowExt for T {}

pub type Tx = axum_sqlx_tx::Tx<sqlx::Postgres>;
pub type TxState = axum_sqlx_tx::State<Postgres>;
pub type TxLayer = axum_sqlx_tx::Layer<Postgres, axum_sqlx_tx::Error>;

impl<W: Workflow, E: EventSender<W>, M: WorkflowInstanceManager<W>> FromRef<ArcAppState<W, E, M>>
    for TxState
{
    fn from_ref(input: &ArcAppState<W, E, M>) -> Self {
        input.0.sqlx_tx_state.clone()
    }
}

pub async fn init_app_state<W: Workflow, D: ControlServerContext<W>>(
    sqlx_tx_state: TxState,
) -> anyhow::Result<ArcAppState<W, D::EventSender, D::InstanceManager>> {
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
        E: EventSender<Self> + Send + Sync + 'static,
        M: WorkflowInstanceManager<Self> + Send + Sync + 'static,
    >() -> impl Future<Output = anyhow::Result<ApiRouter<ArcAppState<Self, E, M>>>> + Send {
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
        E: EventSender<Self> + Send + Sync + 'static,
        M: WorkflowInstanceManager<Self> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self, E, M>> {
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
        async fn handler<T: Workflow, E: EventSender<T>, M: WorkflowInstanceManager<T>>(
            PostWorkflowEvent { instance_id }: PostWorkflowEvent,
            State(ArcAppState(state)): State<ArcAppState<T, E, M>>,
            Json(event): Json<<T as Workflow>::Event>,
        ) -> Result<(), PostWorkflowEventError> {
            state
                .event_sender
                .send(InstanceEvent { event, instance_id })
                .await
                .map_err(|_| PostWorkflowEventError::CouldntQueueEventMessage)?;
            Ok(())
        }

        ApiRouter::new().typed_post_with(handler, |op| {
            op.description("Send event")
                .summary("Send event")
                .id("post-event")
                .tag(Self::NAME)
                .hidden(false)
        })
    }

    fn post_workflow_instance_api_route<
        E: EventSender<Self> + Send + Sync + 'static,
        M: WorkflowInstanceManager<Self> + Send + Sync + 'static,
    >() -> ApiRouter<ArcAppState<Self, E, M>> {
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
        async fn handler<T: Workflow, E: EventSender<T>, M: WorkflowInstanceManager<T>>(
            _: PostWorkflowInstance,
            NoApi(mut tx): NoApi<Tx>,
            State(ArcAppState(state)): State<ArcAppState<T, E, M>>,
        ) -> Result<Json<WorkflowInstance>, PostWorkflowInstanceError> {
            tracing::info!("creating instance...");
            let res = state
                .workflow_instance_manager
                .create_instance(&mut tx)
                .await
                .map_err(|_| PostWorkflowInstanceError::CouldntCreateInstance)?;
            Ok(Json(res))
        }
        ApiRouter::new().typed_post_with(handler, |op| {
            op.description("Create instance")
                .summary("Create instance")
                .id("post-workflow-instance")
                .tag(Self::NAME)
                .hidden(false)
        })
    }
}

impl<T: Workflow> WorkflowControl for T {}
