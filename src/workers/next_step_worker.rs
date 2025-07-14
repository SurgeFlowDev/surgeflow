use std::any::TypeId;

use fe2o3_amqp::Session;
use tikv_client::RawClient;

use crate::{
    event::Immediate,
    step::{FullyQualifiedStep, WorkflowStep},
    workers::{
        adapters::{
            dependencies::next_step_worker::NextStepWorkerContext,
            managers::StepsAwaitingEventManager, receivers::NextStepReceiver,
            senders::ActiveStepSender,
        },
        rabbitmq_adapter::{
            managers::RabbitMqStepsAwaitingEventManager, receivers::RabbitMqNextStepReceiver,
            senders::RabbitMqActiveStepSender,
        },
    },
    workflows::Workflow,
};

pub async fn main<W: Workflow, D: NextStepWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = D::dependencies().await?;

    let mut next_step_receiver = dependencies.next_step_receiver;
    let mut active_step_sender = dependencies.active_step_sender;

    let mut steps_awaiting_event_manager = dependencies.steps_awaiting_event_manager;

    loop {
        let Ok((step, handle)) = next_step_receiver.receive().await else {
            continue;
        };

        if step.step.step.variant_event_type_id() == TypeId::of::<Immediate<W>>() {
            active_step_sender
                .send(FullyQualifiedStep {
                    instance_id: step.instance_id,
                    step: step.step,
                    event: None,
                    retry_count: 0,
                })
                .await?;
        } else {
            steps_awaiting_event_manager
                .put_step(FullyQualifiedStep {
                    instance_id: step.instance_id,
                    step: step.step,
                    event: None,
                    retry_count: 0,
                })
                .await?;
        }
        next_step_receiver.accept(handle);
    }
}
