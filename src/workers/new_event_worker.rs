use crate::{
    event::InstanceEvent,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::new_event_worker::NewEventWorkerContext, managers::StepsAwaitingEventManager,
        receivers::EventReceiver, senders::ActiveStepSender,
    },
    workflows::{Workflow, WorkflowEvent, WorkflowInstanceId},
};

pub async fn main<W: Workflow, C: NewEventWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = C::dependencies().await?;

    let mut active_step_sender = dependencies.active_step_sender;
    let mut event_receiver = dependencies.event_receiver;
    let mut steps_awaiting_event = dependencies.steps_awaiting_event_manager;

    loop {
        let (instance_event, handle) = event_receiver.receive().await?;
        tracing::debug!(
            "Received event {:?} for instance {}",
            instance_event.event.variant_type_id(),
            instance_event.instance_id
        );
        process::<W, C>(
            instance_event,
            &mut active_step_sender,
            &mut steps_awaiting_event,
        )
        .await?;

        event_receiver.accept(handle).await?;
    }
}

async fn process<W: Workflow, C: NewEventWorkerContext<W>>(
    InstanceEvent { event, instance_id }: InstanceEvent<W>,
    active_step_sender: &mut C::ActiveStepSender,
    steps_awaiting_event: &mut C::StepsAwaitingEventManager,
) -> anyhow::Result<()> {
    let step = steps_awaiting_event.get_step(instance_id).await?;
    let Some(step) = step else {
        tracing::info!("No step awaiting event for instance {}", instance_id);
        return Ok(());
    };
    if step.step.step.variant_event_type_id() == event.variant_type_id() {
        steps_awaiting_event.delete_step(instance_id).await?;
    } else {
        tracing::info!(
            "Step {:?} is not waiting for event {:?}",
            step.step,
            event.variant_type_id()
        );
        return Ok(());
    }
    active_step_sender
        .send(FullyQualifiedStep {
            instance_id: step.instance_id,
            step: step.step,
            event: Some(event),
            retry_count: 0,
        })
        .await?;

    Ok(())
}
