use adapter_types::{
    dependencies::new_event_worker::NewEventWorkerDependencies,
    managers::StepsAwaitingEventManager, receivers::EventReceiver, senders::ActiveStepSender,
};
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, ProjectStep};

pub async fn main<P, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>(
    dependencies: NewEventWorkerDependencies<
        P,
        ActiveStepSenderT,
        EventReceiverT,
        StepsAwaitingEventManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    EventReceiverT: EventReceiver<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
{
    let active_step_sender = dependencies.active_step_sender;
    let event_receiver = dependencies.event_receiver;
    let steps_awaiting_event = dependencies.steps_awaiting_event_manager;

    loop {
        if let Err(err) = receive_and_process::<
            P,
            ActiveStepSenderT,
            EventReceiverT,
            StepsAwaitingEventManagerT,
        >(&active_step_sender, &event_receiver, &steps_awaiting_event)
        .await
        {
            tracing::error!("Error processing new event: {:?}", err);
        }
    }
}

async fn receive_and_process<P, ActiveStepSenderT, EventReceiverT, StepsAwaitingEventManagerT>(
    active_step_sender: &ActiveStepSenderT,
    event_receiver: &EventReceiverT,
    steps_awaiting_event: &StepsAwaitingEventManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    EventReceiverT: EventReceiver<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
{
    let mut event_receiver = event_receiver.clone();

    let (instance_event, handle) = event_receiver.receive().await?;
    let active_step_sender = active_step_sender.clone();
    let steps_awaiting_event = steps_awaiting_event.clone();

    tokio::spawn(async move {
        if let Err(err) = process::<P, ActiveStepSenderT, StepsAwaitingEventManagerT>(
            instance_event,
            &mut active_step_sender.clone(),
            &mut steps_awaiting_event.clone(),
        )
        .await
        {
            tracing::error!("Error processing new event: {:?}", err);
        }

        tracing::debug!("acknowledging new event");
        event_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge new event: {:?}", e);
            })
            .unwrap();
        tracing::debug!("acknowledged new event");
    });
    Ok(())
}

async fn process<P, ActiveStepSenderT, StepsAwaitingEventManagerT>(
    InstanceEvent { event, instance_id }: InstanceEvent<P>,
    active_step_sender: &mut ActiveStepSenderT,
    steps_awaiting_event: &mut StepsAwaitingEventManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    ActiveStepSenderT: ActiveStepSender<P>,
    StepsAwaitingEventManagerT: StepsAwaitingEventManager<P>,
{
    let step = steps_awaiting_event.get_step(instance_id).await?;
    let Some(step) = step else {
        tracing::debug!("No step awaiting event for instance {}", instance_id);
        return Ok(());
    };
    if step.step.step.is_project_event(&event) {
        steps_awaiting_event.delete_step(instance_id).await?;
    } else {
        return Ok(());
    }
    active_step_sender
        .send(FullyQualifiedStep {
            event: Some(event),
            ..step
        })
        .await?;

    Ok(())
}
