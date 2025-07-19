use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerContext, managers::StepsAwaitingEventManager,
        receivers::NextStepReceiver, senders::ActiveStepSender,
    },
    workflows::Workflow,
};
use derive_more::Debug;
use sqlx::{PgConnection, PgPool, query};
use std::any::TypeId;
use uuid::Uuid;

pub async fn main<W: Workflow, D: NextStepWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;

    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let mut pool = PgPool::connect(&connection_string).await?;

    loop {
        if let Err(err) = receive_and_process::<W, D>(
            &mut next_step_receiver,
            &mut active_step_sender,
            &mut steps_awaiting_event_manager,
            &mut pool,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }
    }
}

async fn receive_and_process<W: Workflow, D: NextStepWorkerContext<W>>(
    next_step_receiver: &mut D::NextStepReceiver,
    active_step_sender: &mut D::ActiveStepSender,
    steps_awaiting_event_manager: &mut D::StepsAwaitingEventManager,
    pool: &mut PgPool,
) -> anyhow::Result<()> {
    let (step, handle) = next_step_receiver.receive().await?;
    let mut tx = pool.begin().await?;

    if let Err(err) = process::<W, D>(
        active_step_sender,
        steps_awaiting_event_manager,
        &mut tx,
        step,
    )
    .await
    {
        tracing::error!("Error processing next step: {:?}", err);
    }

    tx.commit().await?;
    next_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum NextStepWorkerError<W: Workflow, D: NextStepWorkerContext<W>> {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] <D::ActiveStepSender as ActiveStepSender<W>>::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] <D::StepsAwaitingEventManager as StepsAwaitingEventManager<W>>::Error),
}

async fn process<W: Workflow, D: NextStepWorkerContext<W>>(
    active_step_sender: &mut D::ActiveStepSender,
    steps_awaiting_event_manager: &mut D::StepsAwaitingEventManager,
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W::Step>,
) -> Result<(), NextStepWorkerError<W, D>> {
    tracing::info!("received next step for instance: {}", step.instance_id);

    query!(
        r#"
        INSERT INTO workflow_steps ("workflow_instance_external_id", "external_id")
        VALUES ($1, $2)
        "#,
        Uuid::from(step.instance_id),
        Uuid::from(step.step_id)
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
                step_id: step.step_id,
            })
            .await
            .map_err(NextStepWorkerError::SendActiveStepError)?;
    } else {
        steps_awaiting_event_manager
            .put_step(FullyQualifiedStep {
                instance_id: step.instance_id,
                step: step.step,
                event: None,
                retry_count: 0,
                step_id: step.step_id,
            })
            .await
            .map_err(NextStepWorkerError::AwaitEventError)?;
    }

    Ok(())
}
