use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::workspace_instance_worker::WorkspaceInstanceWorkerContext,
        receivers::InstanceReceiver, senders::NextStepSender,
    },
    workflows::Workflow,
};

pub async fn main<W: Workflow, C: WorkspaceInstanceWorkerContext<W>>() -> anyhow::Result<()> {
    let dependencies = C::dependencies().await?;

    let mut instance_receiver = dependencies.instance_receiver;
    let mut next_step_sender = dependencies.next_step_sender;

    loop {
        let (instance, handle) = instance_receiver.receive().await?;

        let entrypoint = FullyQualifiedStep {
            instance_id: instance.id,
            step: W::entrypoint(),
            event: None,
            retry_count: 0,
        };

        if let Err(err) = next_step_sender.send(entrypoint).await {
            tracing::error!("Failed to send next step: {:?}", err);
            continue;
        }

        instance_receiver.accept(handle).await?;
    }
}
