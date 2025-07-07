use crate::event::{Event, Immediate, InstanceEvent};
use crate::step::StepError;
use crate::step::{Step, StepWithSettings};
use crate::step::{StepResult, StepSettings, WorkflowStep};
use crate::workflows::{Workflow, WorkflowEvent, WorkflowInstanceId};
use crate::{AppState, MyError, WorkflowInstance};
use aide::axum::ApiRouter;
use axum::Json;
use axum::extract::State;
use axum_typed_routing::{TypedApiRouter, api_route};
use derive_more::{From, TryInto};
use macros::step;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

pub async fn control_router() -> anyhow::Result<ApiRouter<Arc<AppState<Workflow1>>>> {
    let router = ApiRouter::new()
        .typed_api_route(post_workflow_instance)
        .typed_api_route(post_workflow_event);

    Ok(router)
}

#[api_route(POST "/workflow/workflow_1" {
    summary: "Create instance",
    description: "Create instance",
    id: "post-workflow-instance",
    tags: ["workflow-1"],
    hidden: false
})]
pub async fn post_workflow_instance(
    State(state): State<Arc<AppState<Workflow1>>>,
) -> Result<Json<WorkflowInstance>, MyError> {
    tracing::info!("creating instance...");
    let mut tx = state.sqlx_pool.begin().await.unwrap();
    let res = state
        .workflow_instance_manager
        .create_instance(&mut tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    Ok(Json(res))
}

#[api_route(POST "/workflow/workflow_1/{instance_id}/event" {
    summary: "Send event",
    description: "Send event",
    id: "post-event",
    tags: ["workflow-1"],
    hidden: false
})]
pub async fn post_workflow_event<W: Workflow>(
    instance_id: WorkflowInstanceId,
    State(state): State<Arc<AppState<W>>>,
    Json(event): Json<W::Event>,
) {
    state
        .event_sender
        .send(InstanceEvent { event, instance_id })
        .await
        .unwrap();
}

#[derive(Debug, Clone, JsonSchema)]
pub struct Workflow1 {}

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

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
pub enum Workflow1Step {
    Step0(Step0),
    Step1(Step1),
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

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow1Event {
    Event0(Event0),
    Event1(Event1),
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

// boilerplate ended

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
