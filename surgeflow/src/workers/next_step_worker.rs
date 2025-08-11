use adapter_types::{
    dependencies::next_step_worker::NextStepWorkerDependencies,
    managers::{PersistenceManager, StepsAwaitingEventManager},
    receivers::NextStepReceiver,
    senders::ActiveStepSender,
};
use derive_more::Debug;
use surgeflow_types::{__Step, __Workflow, FullyQualifiedStep, Immediate, Project};

pub async fn main<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
>(
    dependencies: NextStepWorkerDependencies<
        P,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
        PersistenceManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let next_step_receiver = dependencies.next_step_receiver;
    let active_step_sender = dependencies.active_step_sender;
    let steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;
    let persistence_manager = dependencies.persistence_manager;

    loop {
        tracing::info!("Waiting for new step...");
        if let Err(err) = receive_and_process::<
            P,
            NextStepReceiverT,
            ActiveStepSenderT,
            StepsAwaitingEventManagerT,
            PersistenceManagerT,
        >(
            &next_step_receiver,
            &active_step_sender,
            &steps_awaiting_event_manager,
            &persistence_manager,
        )
        .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }
    }
}

async fn receive_and_process<
    P,
    NextStepReceiverT,
    ActiveStepSenderT,
    StepsAwaitingEventManagerT,
    PersistenceManagerT,
>(
    next_step_receiver: &NextStepReceiverT,
    active_step_sender: &ActiveStepSenderT,
    steps_awaiting_event_manager: &StepsAwaitingEventManagerT,
    persistence_manager: &PersistenceManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepReceiverT: NextStepReceiver<P>,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let mut next_step_receiver = next_step_receiver.clone();

    let (step, handle) = next_step_receiver.receive().await?;
    let active_step_sender = active_step_sender.clone();
    let steps_awaiting_event_manager = steps_awaiting_event_manager.clone();
    let persistence_manager = persistence_manager.clone();

    tokio::spawn(async move {
        if let Err(err) =
            process::<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>(
                &mut active_step_sender.clone(),
                &mut steps_awaiting_event_manager.clone(),
                &mut persistence_manager.clone(),
                step,
            )
            .await
        {
            tracing::error!("Error processing next step: {:?}", err);
        }

        tracing::debug!("acknowledging next step for instance");
        next_step_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge next step: {:?}", e);
            })
            .unwrap();
        tracing::debug!("acknowledged next step for instance");
    });
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    #[error("Database error occurred")]
    DatabaseError(#[source] PersistenceManagerT::Error),
    #[error("Failed to send active step")]
    SendActiveStepError(#[source] ActiveStepSenderT::Error),
    #[error("Failed to put step in awaiting event manager")]
    AwaitEventError(#[source] StepsAwaitingEventManagerT::Error),
}

async fn process<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>(
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event_manager: &mut StepsAwaitingEventManagerT,
    persistence_manager: &mut PersistenceManagerT,
    step: FullyQualifiedStep<P>,
) -> Result<
    (),
    NextStepWorkerError<P, ActiveStepSenderT, StepsAwaitingEventManagerT, PersistenceManagerT>,
>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    tracing::debug!(
        "received next step for instance: {}",
        step.instance.external_id
    );

    persistence_manager
        .insert_step(step.instance.external_id, step.step_id, &step.step.step)
        .await
        .map_err(NextStepWorkerError::DatabaseError)?;

    if step
        .step
        .step
        .value_has_event_value(&<<P::Workflow as __Workflow<P>>::Step as __Step<
            P,
            P::Workflow,
        >>::Event::from(Immediate))
    {
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
