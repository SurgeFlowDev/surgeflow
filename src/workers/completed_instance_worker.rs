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
    let completed_instance_receiver = dependencies.completed_instance_receiver;

    loop {
        if let Err(err) = receive_and_process::<P, CompletedInstanceReceiverT>(
            &completed_instance_receiver,
        )
        .await
        {
            tracing::error!("Error processing completed instance: {:?}", err);
        }
    }
}

async fn receive_and_process<P: Project, CompletedInstanceReceiverT: CompletedInstanceReceiver<P>>(
    completed_instance_receiver: &CompletedInstanceReceiverT,
) -> anyhow::Result<()> {
    let mut completed_instance_receiver = completed_instance_receiver.clone();

    let (step, handle) = completed_instance_receiver.receive().await?;

    tokio::spawn(async move {
        if let Err(err) = process(step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        tracing::info!("acknowledging completed instance");
        completed_instance_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge completed instance: {:?}", e);
            })
            .unwrap();
        tracing::info!("acknowledged completed instance");
    });
    Ok(())
}
