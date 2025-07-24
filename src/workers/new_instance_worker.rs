use crate::workers::adapters::managers::PersistenceManager;
use crate::workflows::ProjectWorkflow;
use crate::{
    step::FullyQualifiedStep,
    workers::adapters::{
        dependencies::new_instance_worker::NewInstanceWorkerDependencies,
        managers::WorkflowInstance, receivers::NewInstanceReceiver, senders::NextStepSender,
    },
    workflows::{Project, StepId},
};

async fn process<P, NextStepSenderT, PersistenceManagerT>(
    next_step_sender: &mut NextStepSenderT,
    persistent_step_manager: &mut PersistenceManagerT,
    instance: WorkflowInstance,
) -> anyhow::Result<()>
where
    P: Project,
    NextStepSenderT: NextStepSender<P>,
    PersistenceManagerT: PersistenceManager,
{
    persistent_step_manager
        .insert_instance(instance.clone())
        .await
        .expect("TODO: handle error inserting instance");

    let entrypoint = FullyQualifiedStep {
        instance,
        step: <<P as Project>::Workflow as ProjectWorkflow>::entrypoint(),
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
    PersistenceManagerT: PersistenceManager,
{
    let mut instance_receiver = dependencies.new_instance_receiver;
    let mut next_step_sender = dependencies.next_step_sender;
    let mut persistent_step_manager = dependencies.persistent_step_manager;

    loop {
        let Ok((step, handle)) = instance_receiver.receive().await else {
            tracing::error!("Failed to receive next step");
            continue;
        };

        if let Err(err) = process(&mut next_step_sender, &mut persistent_step_manager, step).await {
            tracing::error!("Error processing workflow instance: {:?}", err);
        }

        instance_receiver.accept(handle).await?;
    }
}
