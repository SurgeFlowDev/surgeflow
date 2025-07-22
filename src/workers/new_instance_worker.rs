use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::new_instance_worker::NewInstanceWorkerDependencies,
        managers::WorkflowInstance, receivers::NewInstanceReceiver, senders::NextStepSender,
    },
    workflows::{StepId, Workflow},
};

async fn process<W, NextStepSenderT>(
    next_step_sender: &mut NextStepSenderT,

    instance: WorkflowInstance,
) -> anyhow::Result<()>
where
    W: Workflow,
    NextStepSenderT: NextStepSender<W>,
{
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

pub async fn main<W, NextStepSenderT, NewInstanceReceiverT>(
    dependencies: NewInstanceWorkerDependencies<W, NextStepSenderT, NewInstanceReceiverT>,
) -> anyhow::Result<()>
where
    W: Workflow,
    NextStepSenderT: NextStepSender<W>,
    NewInstanceReceiverT: NewInstanceReceiver<W>,
{
    let mut instance_receiver = dependencies.new_instance_receiver;
    let mut next_step_sender = dependencies.next_step_sender;

    loop {
        let Ok((step, handle)) = instance_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };

        if let Err(err) = process(&mut next_step_sender, step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        instance_receiver.accept(handle).await?;
    }
}
