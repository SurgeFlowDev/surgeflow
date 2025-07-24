use crate::{
    workers::adapters::{
        dependencies::completed_instance_worker::CompletedInstanceWorkerDependencies,
        managers::WorkflowInstance, receivers::CompletedInstanceReceiver,
    },
    workflows::Project,
};

async fn process(instance: WorkflowInstance) -> anyhow::Result<()> {
    tracing::info!("Completed instance: {:?}", instance);

    Ok(())
}

pub async fn main<P: Project, CompletedInstanceReceiverT: CompletedInstanceReceiver<P>>(
    dependencies: CompletedInstanceWorkerDependencies<P, CompletedInstanceReceiverT>,
) -> anyhow::Result<()> {
    let mut completed_instance_receiver = dependencies.completed_instance_receiver;

    loop {
        let Ok((step, handle)) = completed_instance_receiver.receive().await else {
            tracing::error!("Completed to receive next step");
            continue;
        };

        if let Err(err) = process(step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        completed_instance_receiver.accept(handle).await?;
    }
}
