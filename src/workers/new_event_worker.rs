use crate::{
    event::InstanceEvent,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::adapters::{
        dependencies::new_event_worker::NewEventWorkerContext, managers::StepsAwaitingEventManager,
        receivers::EventReceiver, senders::ActiveStepSender,
    },
    workflows::{Workflow, WorkflowEvent},
};

pub async fn main<W: Workflow, C: NewEventWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = C::dependencies().await?;

    let mut active_step_sender = dependencies.active_step_sender;
    let mut event_receiver = dependencies.event_receiver;
    let mut steps_awaiting_event = dependencies.steps_awaiting_event_manager;

    loop {
        let (InstanceEvent { event, instance_id }, handle) = event_receiver.receive().await?;
        let step = steps_awaiting_event.get_step(instance_id).await?;
        let Some(step) = step else {
            tracing::info!("No step awaiting event for instance {}", instance_id);
            continue;
        };
        if step.step.step.variant_event_type_id() == event.variant_type_id() {
            steps_awaiting_event.delete_step(instance_id).await?;
        } else {
            tracing::info!(
                "Step {:?} is not waiting for event {:?}",
                step.step,
                event.variant_type_id()
            );
            continue;
        }
        active_step_sender
            .send(FullyQualifiedStep {
                instance_id: step.instance_id,
                step: step.step,
                event: Some(event),
                retry_count: 0,
            })
            .await?;
        event_receiver.accept(handle).await?;
    }
}
