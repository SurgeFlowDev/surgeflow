use crate::workers::adapters::managers::PersistenceManager;
use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::completed_step_worker::CompletedStepWorkerDependencies,
        receivers::CompletedStepReceiver, senders::NextStepSender,
    },
    workflows::{Project, StepId},
};
use derive_more::Debug;

pub async fn main<P: Project, CompletedStepReceiverT, NextStepSenderT, PersistenceManagerT>(
    dependencies: CompletedStepWorkerDependencies<
        P,
        CompletedStepReceiverT,
        NextStepSenderT,
        PersistenceManagerT,
    >,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    let completed_step_receiver = dependencies.completed_step_receiver;
    let next_step_sender = dependencies.next_step_sender;
    let persistence_manager = dependencies.persistence_manager;

    loop {
        if let Err(err) = receive_and_process(
            &completed_step_receiver,
            &next_step_sender,
            &persistence_manager,
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
    PersistenceManagerT,
>(
    completed_step_receiver: &CompletedStepReceiverT,
    next_step_sender: &NextStepSenderT,
    persistence_manager: &PersistenceManagerT,
) -> anyhow::Result<()>
where
    CompletedStepReceiverT: CompletedStepReceiver<P>,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    let mut completed_step_receiver = completed_step_receiver.clone();

    let (step, handle) = completed_step_receiver.receive().await?;
    let next_step_sender = next_step_sender.clone();
    let persistence_manager = persistence_manager.clone();

    tokio::spawn(async move {
        if let Err(err) = process(&mut next_step_sender.clone(), &mut persistence_manager.clone(), step).await {
            tracing::error!("Error processing completed step: {:?}", err);
        }

        tracing::info!("acknowledging completed step for instance");
        completed_step_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge completed step: {:?}", e);
            })
            .unwrap();
        tracing::info!("acknowledged completed step for instance");
    });
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum CompletedStepWorkerError<P: Project, NextStepSenderT: NextStepSender<P>> {
    #[error("Database error occurred")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send next step")]
    SendNextStepError(#[source] <NextStepSenderT as NextStepSender<P>>::Error),
}

async fn process<P, NextStepSenderT, PersistenceManagerT>(
    next_step_sender: &mut NextStepSenderT,
    persistence_manager: &mut PersistenceManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<(), CompletedStepWorkerError<P, NextStepSenderT>>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    tracing::info!(
        "received completed step for instance: {}",
        step.instance.external_id
    );

    persistence_manager
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
        persistence_manager
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

        persistence_manager
            .insert_step_output::<P>(step.step_id, None)
            .await
            .expect("TODO: handle error");
    }

    Ok(())
}
