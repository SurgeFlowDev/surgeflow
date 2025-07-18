use sqlx::{PgConnection, PgPool, query};

use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerContext, managers::StepsAwaitingEventManager,
        receivers::NextStepReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};
use std::any::TypeId;

pub async fn main<W: Workflow, D: NextStepWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;

    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = next_step_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;

        if let Err(err) = process::<W, D>(
            &mut active_step_sender,
            &mut steps_awaiting_event_manager,
            &mut tx,
            step,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }

        tx.commit().await?;
        next_step_receiver.accept(handle).await?;
    }
}

async fn process<W: Workflow, D: NextStepWorkerContext<W>>(
    active_step_sender: &mut D::ActiveStepSender,
    steps_awaiting_event_manager: &mut D::StepsAwaitingEventManager,
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W::Step>,
) -> anyhow::Result<()> {
    tracing::info!("received next step for instance: {}", step.instance_id);
    query!(
        r#"
        UPDATE latest_workflow_steps SET "status" = $1
        WHERE "workflow_instance_id" = $2
        "#,
        2,
        i32::from(step.instance_id)
    )
    .execute(conn)
    .await?;

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
        steps_awaiting_event_manager
            .put_step(FullyQualifiedStep {
                instance_id: step.instance_id,
                step: step.step,
                event: None,
                retry_count: 0,
            })
            .await?;
    }

    Ok(())
}
