use adapter_types::{
    dependencies::new_instance_worker::NewInstanceWorkerDependencies, managers::PersistenceManager,
    receivers::NewInstanceReceiver, senders::NextStepSender,
};
use surgeflow_types::{FullyQualifiedStep, Project, ProjectWorkflow, StepId, WorkflowInstance};

async fn process<P, NextStepSenderT, PersistenceManagerT>(
    next_step_sender: &mut NextStepSenderT,
    persistence_manager: &mut PersistenceManagerT,
    instance: WorkflowInstance,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let workflow_name = instance.workflow_name.clone();
    persistence_manager
        .insert_instance(instance.clone())
        .await
        .expect("TODO: handle error inserting instance");

    let entrypoint = FullyQualifiedStep {
        instance,
        step: <<P as Project>::Workflow as ProjectWorkflow>::entrypoint(workflow_name),
        retry_count: 0,
        step_id: StepId::new(),
        event: None,
        previous_step_id: None,
        next_step: None,
    };

    next_step_sender.send(entrypoint).await?;

    Ok(())
}

pub async fn main<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>(
    dependencies: NewInstanceWorkerDependencies<
        P,
        NextStepSenderT,
        NewInstanceReceiverT,
        PersistenceManagerT,
    >,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let instance_receiver = dependencies.new_instance_receiver;
    let next_step_sender = dependencies.next_step_sender;
    let persistence_manager = dependencies.persistence_manager;

    loop {
        tracing::info!("Waiting for new instance...");
        if let Err(err) = receive_and_process::<
            P,
            NextStepSenderT,
            NewInstanceReceiverT,
            PersistenceManagerT,
        >(&instance_receiver, &next_step_sender, &persistence_manager)
        .await
        {
            tracing::error!("Error processing new instance: {:?}", err);
        }
    }
}

async fn receive_and_process<P, NextStepSenderT, NewInstanceReceiverT, PersistenceManagerT>(
    instance_receiver: &NewInstanceReceiverT,
    next_step_sender: &NextStepSenderT,
    persistence_manager: &PersistenceManagerT,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    NewInstanceReceiverT: NewInstanceReceiver<P>,
    PersistenceManagerT: PersistenceManager<P>,
{
    let mut instance_receiver = instance_receiver.clone();

    let (step, handle) = instance_receiver.receive().await?;
    let next_step_sender = next_step_sender.clone();
    let persistence_manager = persistence_manager.clone();

    tokio::spawn(async move {
        if let Err(err) = process(
            &mut next_step_sender.clone(),
            &mut persistence_manager.clone(),
            step,
        )
        .await
        {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        tracing::debug!("acknowledging new instance");
        instance_receiver
            .accept(handle)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to acknowledge new instance: {:?}", e);
            })
            .unwrap();
        tracing::debug!("acknowledged new instance");
    });
    Ok(())
}
