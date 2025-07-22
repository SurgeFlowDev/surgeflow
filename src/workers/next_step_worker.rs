use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::next_step_worker::NextStepWorkerDependencies,
        managers::StepsAwaitingEventManager, receivers::NextStepReceiver,
        senders::ActiveStepSender,
    },
    workflows::Workflow,
};
use derive_more::Debug;
use sqlx::{PgConnection, PgPool, query};
use std::any::TypeId;
use uuid::Uuid;

pub async fn main<W, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT>(
    dependencies: NextStepWorkerDependencies<
        W,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
    >,
) -> anyhow::Result<()>
where
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;
    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let mut pool = PgPool::connect(&connection_string).await?;

    loop {
        if let Err(err) = receive_and_process::<
            W,
            NextStepReceiverT,
            ActiveStepSenderT,
            StepsAwaitingEventManagerT,
        >(
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

async fn receive_and_process<W, NextStepReceiverT, ActiveStepSenderT, StepsAwaitingEventManagerT>(
    next_step_receiver: &mut NextStepReceiverT,
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    pool: &mut PgPool,
) -> anyhow::Result<()>
where
    W: Workflow,
    NextStepReceiverT: NextStepReceiver<W>,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    let (step, handle) = next_step_receiver.receive().await?;
    let mut tx = pool.begin().await?;

    if let Err(err) = process::<W, ActiveStepSenderT, StepsAwaitingEventManagerT>(
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
enum NextStepWorkerError<W, ActiveStepSenderT, StepsAwaitingEventManagerT>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] <ActiveStepSenderT as ActiveStepSender<W>>::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] <StepsAwaitingEventManagerT as StepsAwaitingEventManager<W>>::Error),
}

async fn process<W, ActiveStepSenderT, StepsAwaitingEventManagerT>(
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    conn: &mut PgConnection,
    step: FullyQualifiedStep<W>,
) -> Result<(), NextStepWorkerError<W, ActiveStepSenderT, StepsAwaitingEventManagerT>>
where
    W: Workflow,
    ActiveStepSenderT: ActiveStepSender<W>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>,
{
    tracing::info!(
        "received next step for instance: {}",
        step.instance.external_id
    );

    let json_step =
        serde_json::to_value(&step.step.step).expect("TODO: handle serialization error");

    query!(
        r#"
        INSERT INTO workflow_steps ("workflow_instance_external_id", "external_id", "step")
        VALUES ($1, $2, $3)
        "#,
        Uuid::from(step.instance.external_id),
        Uuid::from(step.step_id),
        json_step
    )
    .execute(conn)
    .await?;

    if step.step.step.variant_event_type_id() == TypeId::of::<Immediate<W>>() {
        active_step_sender
            .send(step)
            .await
            .map_err(NextStepWorkerError::SendActiveStepError)?;
    } else {
        steps_awaiting_event_manager
            .put_step(step)
            .await
            .map_err(NextStepWorkerError::AwaitEventError)?;
    }

    Ok(())
}
