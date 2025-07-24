use crate::{
    event::InstanceEvent,
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::new_event_worker::NewEventWorkerDependencies,
        managers::StepsAwaitingEventManager, receivers::EventReceiver, senders::ActiveStepSender,
    },
    workflows::{Project, ProjectStep},
};

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
    let mut active_step_sender = dependencies.active_step_sender;
    let mut event_receiver = dependencies.event_receiver;
    let mut steps_awaiting_event = dependencies.steps_awaiting_event_manager;

    loop {
        let (instance_event, handle) = event_receiver.receive().await?;

        process::<P, ActiveStepSenderT, StepsAwaitingEventManagerT>(
            instance_event,
            &mut active_step_sender,
            &mut steps_awaiting_event,
        )
        .await?;

        event_receiver.accept(handle).await?;
    }
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
        tracing::info!("No step awaiting event for instance {}", instance_id);
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
