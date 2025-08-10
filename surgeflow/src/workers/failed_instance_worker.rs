use adapter_types::{
    dependencies::failed_instance_worker::FailedInstanceWorkerDependencies,
    receivers::FailedInstanceReceiver,
};
use surgeflow_types::{Project, WorkflowInstance};

async fn process<P: Project>(instance: WorkflowInstance<P>) -> anyhow::Result<()> {
    tracing::debug!("Failed instance: {:?}", instance);

    Ok(())
}

pub async fn main<P, FailedInstanceReceiverT>(
    dependencies: FailedInstanceWorkerDependencies<P, FailedInstanceReceiverT>,
) -> anyhow::Result<()>
where
    P: Project,
    FailedInstanceReceiverT: FailedInstanceReceiver<P>,
{
    let failed_instance_receiver = dependencies.failed_instance_receiver;

    loop {
        if let Err(err) =
            receive_and_process::<P, FailedInstanceReceiverT>(&failed_instance_receiver).await
        {
            tracing::error!("Error processing failed instance: {:?}", err);
        }
    }
}

async fn receive_and_process<P, FailedInstanceReceiverT>(
    failed_instance_receiver: &FailedInstanceReceiverT,
) -> anyhow::Result<()>
where
    P: Project,
    FailedInstanceReceiverT: FailedInstanceReceiver<P>,
{
    let mut failed_instance_receiver = failed_instance_receiver.clone();

    let (step, handle) = failed_instance_receiver.receive().await?;

    tokio::spawn(async move {
        if let Err(err) = process::<P>(step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        tracing::debug!("acknowledging failed instance");
        failed_instance_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge failed instance: {:?}", e);
            })
            .unwrap();
        tracing::debug!("acknowledged failed instance");
    });
    Ok(())
}
