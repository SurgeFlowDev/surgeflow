use crate::{
    workers::adapters::{
        dependencies::failed_instance_worker::FailedInstanceWorkerDependencies,
        managers::WorkflowInstance, receivers::FailedInstanceReceiver,
    },
    workflows::Project,
};

async fn process(instance: WorkflowInstance) -> anyhow::Result<()> {
    tracing::info!("Failed instance: {:?}", instance);

    Ok(())
}

pub async fn main<P, FailedInstanceReceiverT>(
    dependencies: FailedInstanceWorkerDependencies<P, FailedInstanceReceiverT>,
) -> anyhow::Result<()>
where
    P: Project,
    FailedInstanceReceiverT: FailedInstanceReceiver<P>,
{
    let mut failed_instance_receiver = dependencies.failed_instance_receiver;

    loop {
        let Ok((step, handle)) = failed_instance_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };

        if let Err(err) = process(step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        failed_instance_receiver.accept(handle).await?;
    }
}
