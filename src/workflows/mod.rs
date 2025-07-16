use crate::{
    event::{Event,  InstanceEvent}, step::{Step, StepWithSettings, WorkflowStep}, workers::{adapters::senders::EventSender, rabbitmq_adapter::senders::RabbitMqEventSender}, AppState, ArcAppState, WorkflowInstance, WorkflowInstanceManager
};
use aide::{NoApi, OperationIo, axum::ApiRouter};
use axum::{
    Json,
    extract::{FromRef, State},
    http::StatusCode,
};
use axum_extra::routing::TypedPath;
use derive_more::{Display, From, Into};
use fe2o3_amqp::{Sender, session::SessionHandle};
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

impl<W: Workflow> FromRef<ArcAppState<W>> for TxState {
    fn from_ref(input: &ArcAppState<W>) -> Self {
        input.0.sqlx_tx_state.clone()
    }
}

pub async fn init_app_state<W: Workflow>(
    sqlx_tx_state: TxState,
    session: &mut SessionHandle<()>,
) -> anyhow::Result<ArcAppState<W>> {
    let instance_sender = Sender::attach(
        session,
        format!("{}-instances-receiver-1", W::NAME),
        format!("{}-instances", W::NAME),
    )
    .await?;

    let event_sender = RabbitMqEventSender::<W>::new(session).await?;

    Ok(ArcAppState(Arc::new(AppState {
        event_sender,
        workflow_instance_manager: WorkflowInstanceManager {
            sender: instance_sender.into(),
            _marker: PhantomData,
        },
        sqlx_tx_state,
    })))
}

pub trait WorkflowControl: Workflow {
    fn control_router(
        sqlx_tx_state: TxState,
        session: &mut SessionHandle<()>,
    ) -> impl Future<Output = anyhow::Result<ApiRouter<()>>> + Send {
        async {
            let post_workflow_event_api_route = Self::post_workflow_event_api_route();
            let post_workflow_instance_api_route = Self::post_workflow_instance_api_route();

            let router = ApiRouter::new().nest(
                &format!("/workflow/{}", Self::NAME),
                ApiRouter::new()
                    .merge(post_workflow_instance_api_route)
                    .merge(post_workflow_event_api_route)
                    .with_state(init_app_state::<Self>(sqlx_tx_state, session).await?),
            );
            Ok(router)
        }
    }

    fn post_workflow_event_api_route() -> ApiRouter<ArcAppState<Self>> {
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
        async fn handler<T: Workflow>(
            PostWorkflowEvent { instance_id }: PostWorkflowEvent,
            State(ArcAppState(state)): State<ArcAppState<T>>,
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

    fn post_workflow_instance_api_route() -> ApiRouter<ArcAppState<Self>> {
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
        async fn handler<T: Workflow>(
            _: PostWorkflowInstance,
            NoApi(mut tx): NoApi<Tx>,
            State(ArcAppState(state)): State<ArcAppState<T>>,
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
