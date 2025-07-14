use crate::{
    step::FullyQualifiedStep,
    workflows::{Workflow, WorkflowInstanceId},
};

pub trait StepsAwaitingEventManager<W: Workflow>: Sized {
    fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<FullyQualifiedStep<W::Step>>>> + Send;
    fn delete_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn put_step(
        &mut self,
        step: FullyQualifiedStep<W::Step>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}
