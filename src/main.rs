use aide::{
    OperationIo,
    axum::{ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{
    Extension, ServiceExt,
    extract::{Json, Request, State},
    http::StatusCode,
};
use axum_thiserror::ErrorStatus;
use axum_typed_routing::{TypedApiRouter, api_route};
use fe2o3_amqp::{Receiver, Sender, Session};
use futures::lock::Mutex;
use rust_workflow_2::{
    Workflow, Workflow0, WorkflowExt, WorkflowId, WorkflowInstanceId, WorkflowName,
    event::{EventReceiver, EventSender, Immediate, InstanceEvent, WorkflowEvent},
    step::{
        ActiveStepReceiver, ActiveStepSender, FailedStepSender, FullyQualifiedStep,
        NextStepReceiver, NextStepSender, Step, StepsAwaitingEventManager, WorkflowStep,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, query_as};
use std::{any::TypeId, sync::Arc};
use tikv_client::RawClient;
use tokio::try_join;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

#[derive(Debug)]
struct WorkflowInstanceManager {
    // TODO: could probably be a const param if they allowed &str
    workflow_name: WorkflowName,
    sender: Mutex<Sender>,
}

impl WorkflowInstanceManager {
    async fn create_instance(&self, conn: &mut PgConnection) -> anyhow::Result<WorkflowInstance> {
        let res = query_as!(
            WorkflowInstanceRecord,
            r#"
            INSERT INTO workflow_instances ("workflow_id")
            SELECT "id"
            FROM workflows
            WHERE "name" = $1
            RETURNING "id", "workflow_id";
        "#,
            self.workflow_name.as_ref()
        )
        .fetch_one(conn)
        .await?;

        let res = res.try_into()?;
        let msg = serde_json::to_string(&res)?;
        self.sender.lock().await.send(msg).await?;
        Ok(res)
    }
}

struct WorkflowInstanceRecord {
    pub id: i32,
    pub workflow_id: i32,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct WorkflowInstance {
    pub id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
}

#[derive(Debug, thiserror::Error)]
enum WorkflowInstanceError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl TryFrom<WorkflowInstanceRecord> for WorkflowInstance {
    type Error = WorkflowInstanceError;

    fn try_from(value: WorkflowInstanceRecord) -> Result<Self, Self::Error> {
        Ok(WorkflowInstance {
            id: value.id.into(),
            workflow_id: value.workflow_id.into(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error, ErrorStatus, OperationIo)]
enum MyError {
    #[error("unkown")]
    #[status(StatusCode::CONFLICT)]
    Unkown,
}

#[api_route(POST "/workflow/workflow_0" {
    summary: "Create workflow Instance",
    description: "Create workflow Instance",
    id: "post-workflow-instance",
    tags: ["workflow-instance"],
    hidden: false
})]
async fn post_workflow_instance<W: Workflow>(
    State(state): State<Arc<AppState<W>>>,
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
async fn post_workflow_event<W: Workflow>(
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

struct AppState<W: Workflow> {
    event_sender: EventSender<W>,
    workflow_instance_manager: WorkflowInstanceManager,
    sqlx_pool: PgPool,
}

async fn workspace_instance_worker<W: Workflow>() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let mut instance_receiver = Receiver::attach(
        &mut session,
        "workflow-0-instances-receiver-1",
        "workflow-0-instances",
    )
    .await?;

    let next_step_sender = NextStepSender::<W>::new(&mut session).await?;

    loop {
        let Ok(msg) = instance_receiver.recv::<String>().await else {
            continue;
        };
        tracing::info!("Received instance message");
        let Ok(instance) = serde_json::from_str::<WorkflowInstance>(msg.body()) else {
            continue;
        };

        instance_receiver.accept(msg).await?;

        let entrypoint = FullyQualifiedStep {
            instance_id: instance.id,
            step: W::entrypoint(),
            event: None,
            retry_count: 0,
        };

        if let Err(err) = next_step_sender.send(entrypoint).await {
            tracing::error!("Failed to send next step: {:?}", err);
            continue;
        }
    }
}

async fn handle_event_new<W: Workflow>() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;
    let steps_awaiting_event =
        StepsAwaitingEventManager::<W>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);
    let event_receiver = EventReceiver::<W>::new(&mut session).await?;

    loop {
        let Ok(InstanceEvent { event, instance_id }) = event_receiver.recv().await else {
            continue;
        };
        let step = steps_awaiting_event.get_step(instance_id).await?;
        let Some(step) = step else {
            tracing::info!("No step awaiting event for instance {}", instance_id);
            return Ok(());
        };
        if step.step.step.variant_event_type_id() == event.variant_type_id() {
            steps_awaiting_event.delete_step(instance_id).await?;
        } else {
            tracing::info!(
                "Step {:?} is not waiting for event {:?}",
                step.step,
                event.variant_type_id()
            );
            return Ok(());
        }
        active_step_sender
            .send(FullyQualifiedStep {
                instance_id: step.instance_id,
                step: step.step,
                event: Some(event),
                retry_count: 0,
            })
            .await?;
    }
}

async fn next_step_worker<W: Workflow + 'static>() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;
    let mut session = Session::begin(&mut connection).await?;

    let next_step_receiver = NextStepReceiver::<W>::new(&mut session).await?;
    let active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;

    let steps_awaiting_event =
        StepsAwaitingEventManager::<W>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);

    loop {
        let Ok(step) = next_step_receiver.recv().await else {
            continue;
        };

        if step.step.step.variant_event_type_id() == TypeId::of::<Immediate<W>>() {
            active_step_sender
                .send(FullyQualifiedStep {
                    instance_id: step.instance_id,
                    step: step.step,
                    event: None,
                    retry_count: 0,
                })
                .await?;
        } else {
            steps_awaiting_event
                .put_step(FullyQualifiedStep {
                    instance_id: step.instance_id,
                    step: step.step,
                    event: None,
                    retry_count: 0,
                })
                .await?;
        }
    }
}

async fn active_step_worker<W: Workflow>(wf: W) -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;
    let mut session = Session::begin(&mut connection).await?;
    let active_step_receiver = ActiveStepReceiver::<W>::new(&mut session).await?;
    let active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;
    let next_step_sender = NextStepSender::<W>::new(&mut session).await?;

    let failed_step_sender = FailedStepSender::<W>::new(&mut session).await?;

    // let succeeded_step_sender = SucceededStepSender::<Workflow0>::new(&mut session).await?;

    loop {
        let Ok(mut step) = active_step_receiver.recv().await else {
            continue;
        };
        tracing::info!("Received new step");
        let next_step = step
            .step
            .step
            .run_raw(wf.clone(), step.event.clone())
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to run step: {:?}", e);
            });
        step.retry_count += 1;
        if let Ok(next_step) = next_step {
            // TODO: maybe use `succeeded-step-sender` and push the old step into it? and handle workflow completion there
            if let Some(next_step) = next_step {
                next_step_sender
                    .send(FullyQualifiedStep {
                        instance_id: step.instance_id,
                        step: next_step,
                        event: None,
                        retry_count: 0,
                    })
                    .await?;
            } else {
                tracing::info!("Instance {} completed", step.instance_id);
            }
        } else {
            tracing::info!("Failed to run step: {:?}", step.step);

            if step.retry_count <= step.step.settings.max_retries {
                tracing::info!("Retrying step. Retry count: {}", step.retry_count);
                active_step_sender.send(step).await?;
            } else {
                tracing::info!("Max retries reached for step: {:?}", step.step);
                // TODO: push into "step failed" queue.
                // TODO: and maybe "workflow failed" queue. though this could be done in the "step failed" queue consumer
                failed_step_sender.send(step).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // TODO: refactor this into a function
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;
    let sqlx_pool = PgPool::connect("postgres://workflow:workflow@localhost:5432/workflow").await?;

    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let instance_sender = Sender::attach(
        &mut session,
        "workflow-0-instances-sender-1",
        "workflow-0-instances",
    )
    .await?;

    let event_sender = EventSender::<Workflow0>::new(&mut session).await?;

    let app_state = Arc::new(AppState {
        event_sender,
        workflow_instance_manager: WorkflowInstanceManager {
            workflow_name: Workflow0::name(),
            sender: instance_sender.into(),
        },
        sqlx_pool,
    });

    let mut api = base_open_api();

    let router0 = control_server::<Workflow0>().await?;

    // // test that we can use the same control server for multiple workflows
    // let router1 = control_server::<Workflow0>().await?;

    let router = ApiRouter::new()
        .merge(router0)
        // .merge(router1)
        .with_state(app_state.clone());

    let router = if cfg!(debug_assertions) {
        let router = router
            .typed_api_route(serve_api)
            .route("/docs", Scalar::new("/openapi.json").axum_route());
        router.finish_api(&mut api).layer(Extension(api))
    } else {
        router.into()
    };
    let router = NormalizePathLayer::trim_trailing_slash().layer(router);
    let router = ServiceExt::<Request>::into_make_service(router);
    // TODO: end here


    try_join!(
        workspace_instance_worker::<Workflow0>(),
        active_step_worker(Workflow0 {}),
        next_step_worker::<Workflow0>(),
        handle_event_new::<Workflow0>(),
        async {
            axum::serve(listener, router).await?;
            Ok(())
        }
    )?;

    Ok(())
}

async fn control_server<W: Workflow>() -> anyhow::Result<ApiRouter<Arc<AppState<W>>>> {
    let router = ApiRouter::new()
        .typed_api_route(post_workflow_instance)
        .typed_api_route(post_workflow_event);

    Ok(router)
}

pub fn base_open_api() -> OpenApi {
    OpenApi {
        info: Info {
            description: Some("API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    }
}
#[api_route(GET "/openapi.json" {
    summary: "OpenAPI Spec",
    hidden: false
})]
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}
