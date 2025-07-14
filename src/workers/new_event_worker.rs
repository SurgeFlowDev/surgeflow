use fe2o3_amqp::Session;
use tikv_client::RawClient;

use crate::{
    event::{EventReceiver, InstanceEvent},
    step::{ActiveStepSender, FullyQualifiedStep, WorkflowStep},
    workers::{
        adapters::managers::StepsAwaitingEventManager,
        rabbitmq_adapter::managers::RabbitMqStepsAwaitingEventManager,
    },
    workflows::{Workflow, WorkflowEvent},
};

pub async fn main<W: Workflow>() -> anyhow::Result<()> {
    let mut connection =
        fe2o3_amqp::Connection::open("control-connection-2", "amqp://guest:guest@127.0.0.1:5672")
            .await?;

    let mut session = Session::begin(&mut connection).await?;

    let mut active_step_sender = ActiveStepSender::<W>::new(&mut session).await?;

    let steps_awaiting_event =
        RabbitMqStepsAwaitingEventManager::<W>::new(RawClient::new(vec!["127.0.0.1:2379"]).await?);

    let mut event_receiver = EventReceiver::<W>::new(&mut session).await?;

    loop {
        let Ok(InstanceEvent { event, instance_id }) = event_receiver.recv().await else {
            continue;
        };
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
    }
}
