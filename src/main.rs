use aide::{
    axum::{ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{
    Extension, ServiceExt,
    extract::{Json, Request},
};
use axum_typed_routing::{TypedApiRouter, api_route};
use fe2o3_amqp::{Receiver, Sender, Session, session::SessionHandle};
use rust_workflow_2::{
    AppState, WorkflowInstance, WorkflowInstanceManager,
    event::{EventReceiver, EventSender, Immediate, InstanceEvent},
    step::{
        ActiveStepReceiver, ActiveStepSender, FailedStepSender, FullyQualifiedStep,
        NextStepReceiver, NextStepSender, Step, StepsAwaitingEventManager, WorkflowStep,
    },
    workflows::{
        Workflow, WorkflowEvent,
        workflow_0::{self, Workflow0},
        workflow_1::{self, Workflow1},
    },
};

use sqlx::PgPool;
use std::{any::TypeId, marker::PhantomData, sync::Arc};
use tikv_client::RawClient;
use tokio::{net::TcpListener, try_join};
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

async fn workspace_instance_worker<W: Workflow>() -> anyhow::Result<SessionHandle<()>> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-3", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let mut instance_receiver = Receiver::attach(
        &mut session,
        format!("{}-instances-receiver-1", W::NAME),
        format!("{}-instances", W::NAME),
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
        fe2o3_amqp::Connection::open("control-connection-2", "amqp://guest:guest@127.0.0.1:5672")
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
            continue;
        };
        if step.step.step.variant_event_type_id() == event.variant_type_id() {
            steps_awaiting_event.delete_step(instance_id).await?;
        } else {
            tracing::info!(
                "Step {:?} is not waiting for event {:?}",
                step.step,
                event.variant_type_id()
            );
            continue;
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
        fe2o3_amqp::Connection::open("control-connection-4", "amqp://guest:guest@127.0.0.1:5672")
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
        fe2o3_amqp::Connection::open("control-connection-5", "amqp://guest:guest@127.0.0.1:5672")
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

async fn init_app_state<W: Workflow>(
    sqlx_pool: PgPool,
    session: &mut SessionHandle<()>,
) -> anyhow::Result<Arc<AppState<W>>> {
    let instance_sender = Sender::attach(
        session,
        format!("{}-instances-receiver-1", W::NAME),
        format!("{}-instances", W::NAME),
    )
    .await?;

    let event_sender = EventSender::<W>::new(session).await?;

    Ok(Arc::new(AppState {
        event_sender,
        workflow_instance_manager: WorkflowInstanceManager {
            sender: instance_sender.into(),
            _marker: PhantomData,
        },
        sqlx_pool,
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;

    let sqlx_pool = PgPool::connect("postgres://workflow:workflow@localhost:5432/workflow").await?;

    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-6", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    // ISOLATED WORKFLOW 0
    let app_state_0 = init_app_state::<Workflow0>(sqlx_pool.clone(), &mut session).await?;

    let router_0 = workflow_0::control_router().await?.with_state(app_state_0);

    let handlers_0 = async {
        try_join!(
            workspace_instance_worker::<Workflow0>(),
            active_step_worker(Workflow0 {}),
            next_step_worker::<Workflow0>(),
            handle_event_new::<Workflow0>()
        )
    };
    // ISOLATED END

    // ISOLATED WORKFLOW 1
    let app_state_1 = init_app_state::<Workflow1>(sqlx_pool, &mut session).await?;

    let router_1 = workflow_1::control_router().await?.with_state(app_state_1);

    let handlers_1 = async {
        try_join!(
            workspace_instance_worker::<Workflow1>(),
            active_step_worker(Workflow1 {}),
            next_step_worker::<Workflow1>(),
            handle_event_new::<Workflow1>()
        )
    };
    // ISOLATED END

    // MERGE
    let all_handlers = async { try_join!(handlers_0, handlers_1) };
    let router = ApiRouter::new().merge(router_0).merge(router_1);
    // MERGE END

    let mut api = base_open_api();

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

    let server = async {
        axum::serve(listener, router).await?;
        Ok(())
    };

    try_join!(all_handlers, server)?;

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
