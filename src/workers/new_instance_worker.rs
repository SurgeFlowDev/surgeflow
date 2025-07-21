use sqlx::{PgConnection, PgPool};

use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::new_instance_worker::NewInstanceWorkerContext, managers::WorkflowInstance,
        receivers::NewInstanceReceiver, senders::NextStepSender,
    },
    workflows::{StepId, Workflow},
};

async fn process<W: Workflow, C: NewInstanceWorkerContext<W>>(
    next_step_sender: &mut C::NextStepSender,
    conn: &mut PgConnection,
    instance: WorkflowInstance,
) -> anyhow::Result<()> {
    let entrypoint = FullyQualifiedStep {
        instance,
        step: W::entrypoint(),
        retry_count: 0,
        step_id: StepId::new(),
        event: None,
        previous_step_id: None,
        next_step: None,
    };

    next_step_sender.send(entrypoint).await?;

    Ok(())
}

pub async fn main<W: Workflow, C: NewInstanceWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = C::dependencies().await?;

    let mut instance_receiver = dependencies.instance_receiver;
    let mut next_step_sender = dependencies.next_step_sender;

    let connection_string =
        std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
    let pool = PgPool::connect(&connection_string).await?;

    loop {
        let Ok((step, handle)) = instance_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };
        let mut tx = pool.begin().await?;
        if let Err(err) = process::<W, C>(&mut next_step_sender, &mut tx, step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }
        tx.commit().await?;
        instance_receiver.accept(handle).await?;
    }
}
