use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get_with},
    openapi::{Info, OpenApi},
    scalar::Scalar,
};
use axum::{
    Extension, ServiceExt,
    extract::{Json, Request},
};
use macros::my_main;
use rust_workflow_2::workers::rabbitmq_adapter::dependencies::workspace_instance_worker::RabbitMqWorkspaceInstanceWorkerDependencies;
use rust_workflow_2::workers::{
    adapters::{managers::StepsAwaitingEventManager, senders::NextStepSender},
    rabbitmq_adapter::senders::RabbitMqNextStepSender,
};
use rust_workflow_2::workers::{
    rabbitmq_adapter::managers::RabbitMqStepsAwaitingEventManager,
    workspace_instance_worker::{self},
};
use rust_workflow_2::workflows::{TxState, workflow_0::Workflow0};

use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use rust_workflow_2::{
    event::Immediate,
    step::{
        ActiveStepReceiver, ActiveStepSender, FailedStepSender, FullyQualifiedStep,
        NextStepReceiver, Step, WorkflowStep,
    },
    workflows::{Tx, TxLayer, Workflow, WorkflowControl, workflow_1::Workflow1},
};

use sqlx::PgPool;
use std::any::TypeId;
use tikv_client::RawClient;
use tokio::{net::TcpListener, try_join};
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

async fn next_step_worker<W: Workflow + 'static>() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-4", "amqp://guest:guest@127.0.0.1:5672")
            .await?;
    let mut session = Session::begin(&mut connection).await?;

    let mut next_step_receiver = NextStepReceiver::<W>::new(&mut session).await?;
    let mut active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;

    let steps_awaiting_event =
        RabbitMqStepsAwaitingEventManager::<W>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);

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
    let mut active_step_receiver = ActiveStepReceiver::<W>::new(&mut session).await?;
    let mut active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;
    // let mut next_step_sender = RabbitMqNextStepSender::<W>::new(&mut session).await?;
    let mut next_step_sender: RabbitMqNextStepSender<W> = todo!();

    let mut failed_step_sender = FailedStepSender::<W>::new(&mut session).await?;

    // let succeeded_step_sender = SucceededStepSender::<Workflow0>::new(&mut session).await?;

    loop {
        let Ok(mut step) = active_step_receiver.recv().await else {
            continue;
        };
        tracing::info!("Received new step");
        let next_step = step.step.step.run_raw(wf.clone(), step.event.clone()).await;
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

async fn serve(router: ApiRouter, sqlx_tx_layer: TxLayer) -> anyhow::Result<()> {
    let router = ApiRouter::new().merge(router).layer(sqlx_tx_layer);

    let mut api = base_open_api();

    let router = if cfg!(debug_assertions) {
        let router = router
            .api_route(
                "/openapi.json",
                get_with(serve_api, |op| op.summary("OpenAPI Spec").hidden(false)),
            )
            .route("/docs", Scalar::new("/openapi.json").axum_route());
        router.finish_api(&mut api).layer(Extension(api))
    } else {
        router.into()
    };
    let router = NormalizePathLayer::trim_trailing_slash().layer(router);
    let router = ServiceExt::<Request>::into_make_service(router);

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", 8080)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
#[cfg(feature = "control_server")]
async fn control_server_setup()
-> anyhow::Result<((TxState, TxLayer), ConnectionHandle<()>, SessionHandle<()>)> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-6", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let session = Session::begin(&mut connection).await?;

    Ok((
        Tx::setup(PgPool::connect("postgres://workflow:workflow@localhost:5432/workflow").await?),
        connection,
        session,
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    my_main!(Workflow0, Workflow1);

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

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[cfg(any(
    feature = "active_step_worker",
    feature = "new_instance_worker",
    feature = "next_step_worker",
    feature = "new_event_worker"
))]
async fn main_handler<W: Workflow>(
    #[cfg(feature = "active_step_worker")] wf: W,
) -> anyhow::Result<()> {
    use rust_workflow_2::workers::new_event_worker;

    try_join!(
        #[cfg(feature = "active_step_worker")]
        active_step_worker::<W>(wf),
        #[cfg(feature = "new_instance_worker")]
        workspace_instance_worker::main::<W, RabbitMqWorkspaceInstanceWorkerDependencies<_, _, _>>(
        ),
        #[cfg(feature = "next_step_worker")]
        next_step_worker::<W>(),
        #[cfg(feature = "new_event_worker")]
        new_event_worker::main::<W>(),
    )?;

    Ok(())
}
