use std::{any::TypeId, sync::Arc, time::Duration};

use axum_thiserror::ErrorStatus;
use fe2o3_amqp::{Receiver, Sender, Session};
use futures::lock::Mutex;
use lapin::{Connection, ConnectionProperties, options::QueueDeclareOptions, types::FieldTable};
use rust_workflow_2::{
    ActiveStepQueue, Ctx, WaitingForEventStepQueue, Workflow, Workflow0, WorkflowExt, WorkflowId,
    WorkflowInstanceId, WorkflowName,
    event::{
        EventReceiver, EventSender, Immediate, InstanceEvent, Workflow0Event, event_0::Event0,
    },
    runner::{handle_event, handle_step},
    step::{
        ActiveStepReceiver, ActiveStepSender, FailedStepSender, FullyQualifiedStep, Step,
        StepsAwaitingEventManager, SucceededStepSender,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, query_as};
use tikv_client::RawClient;
use tower_http::normalize_path::NormalizePathLayer;

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
use axum_typed_routing::{TypedApiRouter, api_route};

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

// #[derive(Debug)]
// struct WorkflowReceiver(Mutex<Receiver>);

// impl WorkflowReceiver {
//     fn new(receiver: Receiver) -> Self {
//         Self(Mutex::new(receiver))
//     }
//     async fn recv(&self) -> anyhow::Result<WorkflowName> {
//         let mut receiver = self.0.lock().await;

//         let event = receiver.recv::<String>().await?;
//         let event = serde_json::from_str(event.body())?;

//         Ok(event)
//     }
// }

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
async fn post_workflow_instance(
    State(state): State<Arc<AppState>>,
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
async fn post_workflow_event(
    instance_id: WorkflowInstanceId,
    State(state): State<Arc<AppState>>,
    Json(event): Json<Workflow0Event>,
) {
    state
        .event_sender
        .send(InstanceEvent { event, instance_id })
        .await
        .unwrap();
}

struct AppState {
    event_sender: EventSender<Workflow0>,
    workflow_instance_manager: WorkflowInstanceManager,
    sqlx_pool: PgPool,
}

async fn workspace_instance_worker() -> anyhow::Result<()> {
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

    let active_step_sender = ActiveStepSender::<Workflow0>::new(&mut session).await?;

    loop {
        let Ok(msg) = instance_receiver.recv::<String>().await else {
            continue;
        };
        tracing::info!("Received instance message");
        let Ok(instance) = serde_json::from_str::<WorkflowInstance>(msg.body()) else {
            continue;
        };
        // tracing::info!("{:?}", instance);
        instance_receiver.accept(msg).await?;

        let entrypoint = FullyQualifiedStep {
            instance_id: instance.id,
            step: Workflow0::entrypoint(),
            event: None,
            retry_count: 0,
        };

        if let Err(_) = active_step_sender.send(entrypoint).await {
            continue;
        }
    }
}

async fn handle_event_new() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let active_step_sender = ActiveStepSender::<Workflow0>::new(&mut session).await?;
    let steps_awaiting_event =
        StepsAwaitingEventManager::<Workflow0>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);
    let event_receiver = EventReceiver::<Workflow0>::new(&mut session).await?;

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

async fn active_step_worker(wf: Workflow0) -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;
    let mut session = Session::begin(&mut connection).await?;
    let active_step_receiver = ActiveStepReceiver::<Workflow0>::new(&mut session).await?;
    let active_step_sender = ActiveStepSender::<Workflow0>::new(&mut session).await?;

    let failed_step_sender = FailedStepSender::<Workflow0>::new(&mut session).await?;

    let succeeded_step_sender = SucceededStepSender::<Workflow0>::new(&mut session).await?;

    let steps_awaiting_event =
        StepsAwaitingEventManager::<Workflow0>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);

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
                {
                    // TODO: push to a `next-step` queue and have a worker that consumes it and handles this block of code
                    if next_step.step.variant_event_type_id()
                        == TypeId::of::<Immediate<Workflow0>>()
                    {
                        active_step_sender
                            .send(FullyQualifiedStep {
                                instance_id: step.instance_id,
                                step: next_step,
                                event: None,
                                retry_count: 0,
                            })
                            .await?;
                    } else {
                        steps_awaiting_event
                            .put_step(FullyQualifiedStep {
                                instance_id: step.instance_id,
                                step: next_step,
                                event: None,
                                retry_count: 0,
                            })
                            .await?;
                    }
                }
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
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;
    let sqlx_pool = PgPool::connect("postgres://workflow:workflow@localhost:5432/workflow").await?;

    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-1", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let instance_worker = tokio::spawn(workspace_instance_worker());

    let active_step_worker = tokio::spawn(active_step_worker(Workflow0 {}));

    let handle_event_worker = tokio::spawn(handle_event_new());

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
    let router: ApiRouter = ApiRouter::new()
        .typed_api_route(post_workflow_instance)
        .typed_api_route(post_workflow_event)
        .with_state(app_state);

    let router = if cfg!(debug_assertions) {
        let router = router
            .typed_api_route(serve_api)
            .route("/docs", Scalar::new("/openapi.json").axum_route());

        let mut api = base_open_api();
        router.finish_api(&mut api).layer(Extension(api))
    } else {
        router.into()
    };

    let router = NormalizePathLayer::trim_trailing_slash().layer(router);
    let router = ServiceExt::<Request>::into_make_service(router);

    axum::serve(listener, router).await?;
    instance_worker.await??;
    active_step_worker.await??;
    handle_event_worker.await??;
    Ok(())
}

async fn poc_main() -> anyhow::Result<()> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

    let client = RawClient::new(vec!["127.0.0.1:2379"]).await?;

    let channel_a = conn.create_channel().await?;

    channel_a
        .queue_declare(
            "workflow-name",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    ///////////////
    ///////////////
    ///////////////

    // let sub_channel = conn.create_channel().await?;
    // let consumer = sub_channel
    //     .basic_consume(
    //         "workflow-name",
    //         "my_consumer",
    //         BasicConsumeOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;

    let mut connection = fe2o3_amqp::Connection::open(
        "connection-111",                    // container id
        "amqp://guest:guest@127.0.0.1:5672", // url
    )
    .await
    .unwrap();

    let mut session = Session::begin(&mut connection).await.unwrap();

    // Create a sender
    let mut sender = Sender::attach(
        &mut session,           // Session
        "rust-sender-link-111", // link name
        "workflow-name",        // target address
    )
    .await
    .unwrap();

    // Create a receiver
    let mut receiver = Receiver::attach(
        &mut session,
        "rust-receiver-link-111", // link name
        "workflow-name",          // source address
    )
    .await
    .unwrap();

    let active_step_queue = ActiveStepQueue { sender, receiver };
    let waiting_for_step_queue = WaitingForEventStepQueue { queues: client };
    let completed_step_queue = rust_workflow_2::CompletedStepQueue {
        queues: std::collections::HashMap::new(),
    };
    let ctx = Arc::new(Mutex::new(Ctx {
        active: active_step_queue,
        waiting: waiting_for_step_queue,
    }));

    let instance_id = WorkflowInstanceId::from(0);
    let step_0 = rust_workflow_2::step::step_0::Step0 {};

    tracing::info!("lock ac");
    let mut ctx = ctx.lock().await;

    ctx.waiting
        .enqueue(rust_workflow_2::step::FullyQualifiedStep {
            instance_id,
            step: rust_workflow_2::step::StepWithSettings {
                step: rust_workflow_2::step::WorkflowStep::Step0(step_0),
                settings: rust_workflow_2::step::StepSettings {
                    max_retries: 1,
                    // delay: None,
                },
            },
            event: None,
            retry_count: 0,
        })
        .await?;

    tracing::info!("handle_event");
    handle_event(instance_id, Event0 {}.into(), &mut ctx).await?;

    tracing::info!("loop");
    loop {
        tracing::info!("lock ac");
        // let mut ac = ac.lock().await;
        tracing::info!("deq");
        let step = ctx.active.dequeue().await.unwrap();
        tracing::info!("handle next step");
        let is_completed = handle_step(step, &mut ctx).await.unwrap();

        if is_completed {
            break;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // tracing::info!("ok");

    // try_join!(handle)?;
    // sender.close().await?;
    session.end().await?;
    connection.close().await?;

    Ok(())
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
