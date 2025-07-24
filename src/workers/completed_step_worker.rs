use crate::workers::adapters::managers::PersistentStepManager;
use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::completed_step_worker::CompletedStepWorkerDependencies,
        receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::{Project, StepId},
};
use derive_more::Debug;

pub async fn main<P: Project, CompletedStepReceiverT, NextStepSenderT, PersistentStepManagerT>(
    dependencies: CompletedStepWorkerDependencies<
        P,
        CompletedStepReceiverT,
        NextStepSenderT,
        PersistentStepManagerT,
    >,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let mut completed_step_receiver = dependencies.completed_step_receiver;
    let mut next_step_sender = dependencies.next_step_sender;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        if let Err(err) = receive_and_process(
            &mut completed_step_receiver,
            &mut next_step_sender,
            &mut persistent_step_manager,
        )
        .await
        {
            tracing::error!("Error processing completed step: {:?}", err);
        }
    }
}

async fn receive_and_process<
    P: Project,
    CompletedStepReceiverT,
    NextStepSenderT,
    PersistentStepManagerT,
>(
    completed_step_receiver: &mut CompletedStepReceiverT,
    next_step_sender: &mut NextStepSenderT,
    persistent_step_manager: &mut PersistentStepManagerT,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    let (step, handle) = completed_step_receiver.receive().await?;

    if let Err(err) = process(next_step_sender, persistent_step_manager, step).await {
        tracing::error!("Error processing completed step: {:?}", err);
    }

    completed_step_receiver.accept(handle).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum CompletedStepWorkerError<P: Project, NextStepSenderT: NextStepSender<P>> {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send next step")]
    SendNextStepError(#[source] <NextStepSenderT as NextStepSender<P>>::Error),
}

async fn process<P, NextStepSenderT, PersistentStepManagerT>(
    next_step_sender: &mut NextStepSenderT,
    persistent_step_manager: &mut PersistentStepManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<(), CompletedStepWorkerError<P, NextStepSenderT>>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    PersistentStepManagerT: PersistentStepManager,
{
    tracing::info!(
        "received completed step for instance: {}",
        step.instance.external_id
    );

    persistent_step_manager
        .set_step_status(step.step_id, 4)
        .await
        .expect("TODO: handle error");
    // query!(
    //     r#"
    //     UPDATE workflow_steps SET "status" = $1
    //     WHERE "external_id" = $2
    //     "#,
    //     4,
    //     Uuid::from(step.step_id)
    // )
    // .execute(conn.as_mut())
    // .await?;

    let next_step = step.next_step;

    if let Some(next_step) = next_step {
        persistent_step_manager
            .insert_step_output::<P>(step.step_id, Some(&next_step.step))
            .await
            .expect("TODO: handle error");

        next_step_sender
            .send(FullyQualifiedStep {
                instance: step.instance,
                step_id: StepId::new(),
                step: next_step,
                event: None,
                retry_count: 0,
                previous_step_id: Some(step.step_id),
                next_step: None,
            })
            .await
            .map_err(CompletedStepWorkerError::SendNextStepError)?;
    } else {
        // TODO: push to instance completed queue ?
        tracing::info!("Instance {} completed", step.instance.external_id);

        persistent_step_manager
            .insert_step_output::<P>(step.step_id, None)
            .await
            .expect("TODO: handle error");
    }

    Ok(())
}
