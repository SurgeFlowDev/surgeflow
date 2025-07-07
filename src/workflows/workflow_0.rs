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

pub async fn control_router() -> anyhow::Result<ApiRouter<Arc<AppState<Workflow0>>>> {
    let router = ApiRouter::new()
        .typed_api_route(post_workflow_instance)
        .typed_api_route(post_workflow_event);

    Ok(router)
}

#[api_route(POST "/workflow/workflow_0" {
    summary: "Create workflow Instance",
    description: "Create workflow Instance",
    id: "post-workflow-instance",
    tags: ["workflow-instance"],
    hidden: false
})]
pub async fn post_workflow_instance(
    State(state): State<Arc<AppState<Workflow0>>>,
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

#[api_route(POST "/workflow/workflow_0/{instance_id}/event" {
    summary: "Send event",
    description: "Send event",
    id: "post-event",
    tags: ["workflow-event"],
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
pub struct Workflow0 {}

impl Workflow for Workflow0 {
    type Event = Workflow0Event;
    type Step = Workflow0Step;
    const NAME: &'static str = "workflow_0";

    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 1 },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
pub enum Workflow0Step {
    Step0(Step0),
    Step1(Step1),
}
impl WorkflowStep for Workflow0Step {
    fn variant_event_type_id(&self) -> TypeId {
        match self {
            Workflow0Step::Step0(_) => TypeId::of::<<Step0 as Step>::Event>(),
            Workflow0Step::Step1(_) => TypeId::of::<<Step1 as Step>::Event>(),
        }
    }
}

impl Step for Workflow0Step {
    type Event = Workflow0Event;
    type Workflow = Workflow0;
    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<Workflow0Event>,
    ) -> Result<Option<StepWithSettings<Self>>, StepError> {
        match self {
            Workflow0Step::Step0(step) => Step::run_raw(step, wf, event).await,
            Workflow0Step::Step1(step) => Step::run_raw(step, wf, event).await,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema, Clone)]
pub enum Workflow0Event {
    Event0(Event0),
}

impl WorkflowEvent for Workflow0Event {
    fn variant_type_id(&self) -> TypeId {
        match self {
            Self::Event0(_) => TypeId::of::<Event0>(),
        }
    }
}

impl Event for Workflow0Event {
    type Workflow = Workflow0;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {}

impl Event for Event0 {
    type Workflow = Workflow0;
}

impl From<Event0> for Option<<Workflow0 as Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow0Event::Event0(val))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step0 {}

#[step]
impl Step0 {
    #[run]
    async fn run(
        &self,
        #[expect(unused_variables)] wf: Workflow0,
        // event: Event0,
    ) -> StepResult<Workflow0> {
        tracing::info!("Running Step0, Workflow0");

        // return the next step to run
        Ok(Step1 {}.into())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step1 {}

impl From<Step1> for Option<StepWithSettings<<<Step1 as Step>::Workflow as Workflow>::Step>> {
    fn from(step: Step1) -> Self {
        Some(StepWithSettings {
            step: step.into(),
            settings: StepSettings { max_retries: 0 },
        })
    }
}

// static DEV_COUNT: AtomicUsize = AtomicUsize::new(0);

#[step]
impl Step1 {
    #[expect(unused_variables)]
    #[run]
    async fn run(&self, wf: Workflow0, event: Event0) -> StepResult<Workflow0> {
        tracing::info!("Running Step1, Workflow0");
        // let dev_count = DEV_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // if dev_count == 3 {
        //     return Ok(None);
        // }
        // Err(StepError::Unknown)
        Ok(None)
    }
}
