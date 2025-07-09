use crate::event::{Event, Immediate, InstanceEvent};
use crate::step::StepError;
use crate::step::{Step, StepWithSettings};
use crate::step::{StepResult, StepSettings, WorkflowStep};
use crate::workflows::{Workflow, WorkflowEvent, WorkflowInstanceId};
use crate::{AppState, ArcAppState, WorkflowInstance};
use aide::axum::ApiRouter;
use aide::axum::routing::{ApiMethodRouter, post_with};
use aide::transform::TransformOperation;
use aide::{NoApi, OperationIo};
use axum::Json;
use axum::extract::{FromRef, State};
use axum_extra::routing::TypedPath;

use derive_more::{From, TryInto};
use macros::step;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use std::any::TypeId;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

#[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
#[typed_path("/workflow/workflow_1/{instance_id}/event")]
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

#[derive(TypedPath, Deserialize, JsonSchema, OperationIo)]
#[typed_path("/workflow/workflow_1")]
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
    #[status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)]
    CouldntCreateInstance,
}

pub type Tx = axum_sqlx_tx::Tx<sqlx::Postgres>;
pub type TxState = axum_sqlx_tx::State<Postgres>;
pub type TxLayer = axum_sqlx_tx::Layer<Postgres, axum_sqlx_tx::Error>;

impl<W: Workflow> FromRef<ArcAppState<W>> for TxState {
    fn from_ref(input: &ArcAppState<W>) -> Self {
        input.0.sqlx_tx_state.clone()
    }
}

pub trait WorkflowControl: Workflow {
    fn control_router() -> anyhow::Result<ApiRouter<Arc<AppState<Self>>>>;

    fn post_workflow_event_api_route() -> (&'static str, ApiMethodRouter<Arc<AppState<Self>>>) {
        (
            PostWorkflowEvent::PATH,
            post_with(
                async |PostWorkflowEvent { instance_id }: PostWorkflowEvent,
                       state: State<Arc<AppState<Self>>>,
                       Json(event): Json<<Self as Workflow>::Event>|
                       -> Result<(), PostWorkflowEventError> {
                    state
                        .event_sender
                        .send(InstanceEvent { event, instance_id })
                        .await
                        .map_err(|_| PostWorkflowEventError::CouldntQueueEventMessage)?;
                    Ok(())
                },
                |op| {
                    op.description("Send event")
                        .summary("Send event")
                        .id("post-event")
                        .tag(Self::NAME)
                        .hidden(false)
                },
            ),
        )
    }

    fn post_workflow_instance_api_route()
    -> (&'static str, ApiMethodRouter<ArcAppState<Self>>) {
        (
            PostWorkflowInstance::PATH,
            post_with(
                async |
                        _: PostWorkflowInstance,
                        NoApi(mut tx): NoApi<Tx>,
                        State(ArcAppState(state)): State<ArcAppState<Self>>
                        |
                       -> Result<Json<WorkflowInstance>, PostWorkflowInstanceError> {
                            tracing::info!("creating instance...");
                            let res = state
                                .workflow_instance_manager
                                .create_instance(&mut tx)
                                .await
                                .map_err(|_| PostWorkflowInstanceError::CouldntCreateInstance)?;
                            Ok(Json(res))
                },
                |op| {
                    op.description("Create instance")
                        .summary("Create instance")
                        .id("post-workflow-instance")
                        .tag(Self::NAME)
                        .hidden(false)
                },
            ),
        )
    }
}

impl WorkflowControl for Workflow1 {
    fn control_router() -> anyhow::Result<ApiRouter<Arc<AppState<Workflow1>>>> {
        todo!()
        // let post_workflow_event_api_route = Self::post_workflow_event_api_route();
        // let post_workflow_instance_api_route = Self::post_workflow_instance_api_route();
        // let router = ApiRouter::new()
        //     .api_route(
        //         post_workflow_instance_api_route.0,
        //         post_workflow_instance_api_route.1,
        //     )
        //     .api_route(
        //         post_workflow_event_api_route.0,
        //         post_workflow_event_api_route.1,
        //     );

        // Ok(router)
    }
}

impl WorkflowStep for Workflow1Step {
    fn variant_event_type_id(&self) -> TypeId {
        match self {
            Workflow1Step::Step0(_) => TypeId::of::<<Step0 as Step>::Event>(),
            Workflow1Step::Step1(_) => TypeId::of::<<Step1 as Step>::Event>(),
        }
    }
}

impl Step for Workflow1Step {
    type Event = Workflow1Event;
    type Workflow = Workflow1;
    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<Workflow1Event>,
    ) -> Result<Option<StepWithSettings<Self>>, StepError> {
        match self {
            Workflow1Step::Step0(step) => Step::run_raw(step, wf, event).await,
            Workflow1Step::Step1(step) => Step::run_raw(step, wf, event).await,
        }
    }
}

impl WorkflowEvent for Workflow1Event {
    fn variant_type_id(&self) -> TypeId {
        match self {
            Self::Event0(_) => TypeId::of::<Event0>(),
            Self::Event1(_) => TypeId::of::<Event1>(),
        }
    }
}

impl Event for Workflow1Event {
    type Workflow = Workflow1;
}

impl Event for Event0 {
    type Workflow = Workflow1;
}

impl From<Event0> for Option<<Workflow1 as Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow1Event::Event0(val))
    }
}

impl From<Step1> for Option<StepWithSettings<<<Step1 as Step>::Workflow as Workflow>::Step>> {
    fn from(step: Step1) -> Self {
        Some(StepWithSettings {
            step: step.into(),
            settings: StepSettings { max_retries: 0 },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
pub enum Workflow1Step {
    Step0(Step0),
    Step1(Step1),
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow1Event {
    Event0(Event0),
    Event1(Event1),
}

impl Workflow for Workflow1 {
    type Event = Workflow1Event;
    type Step = Workflow1Step;
    const NAME: &'static str = "workflow_1";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

// boilerplate ended

#[derive(Debug, Clone, JsonSchema)]
pub struct Workflow1 {}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {
    test_string: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event1 {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(
        &self,
        #[expect(unused_variables)] wf: Workflow1,
        // event: Event0,
    ) -> StepResult<Workflow1> {
        tracing::error!("Running Step0, Workflow1");

        // return the next step to run
        Ok(Some(StepWithSettings {
            step: Step1 {}.into(),
            settings: StepSettings { max_retries: 3 },
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step1 {}

static DEV_COUNT: AtomicUsize = AtomicUsize::new(0);

#[step]
impl Step1 {
    #[expect(unused_variables)]
    #[run]
    async fn run(&self, wf: Workflow1, event: Event0) -> StepResult<Workflow1> {
        tracing::error!(
            "Running Step1, Workflow1, event.test_string: {}",
            event.test_string
        );
        let dev_count = DEV_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if dev_count == 3 {
            return Ok(None);
        }
        Err(StepError::Unknown)
    }
}
