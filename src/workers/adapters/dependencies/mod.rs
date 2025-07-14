use crate::{
    workers::adapters::dependencies::{
        new_event_worker::{NewEventWorkerContext, NewEventWorkerDependencies},
        workspace_instance_worker::{
            WorkspaceInstanceWorkerContext, WorkspaceInstanceWorkerDependencies,
        },
    },
    workflows::Workflow,
};

pub mod new_event_worker;
pub mod workspace_instance_worker;

pub trait Dependencies<W: Workflow>: Sized {
    type WorkspaceInstanceWorkerDependenciesT: WorkspaceInstanceWorkerContext<W>;
    type NewEventWorkerDependenciesT: NewEventWorkerContext<W>;
}

pub trait DependencyConstructors<W: Workflow>: Sized + Dependencies<W> {
    fn workspace_instance_worker_dependencies(
        &mut self,
    ) -> impl Future<
        Output = anyhow::Result<
            WorkspaceInstanceWorkerDependencies<W, Self::WorkspaceInstanceWorkerDependenciesT>,
        >,
    > + Send;
    fn new_event_worker_dependencies(
        &mut self,
    ) -> impl Future<
        Output = anyhow::Result<NewEventWorkerDependencies<W, Self::NewEventWorkerDependenciesT>>,
    > + Send;
}

impl<W: Workflow, T: Dependencies<W> + std::marker::Send> DependencyConstructors<W> for T {
    async fn workspace_instance_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<
        WorkspaceInstanceWorkerDependencies<W, Self::WorkspaceInstanceWorkerDependenciesT>,
    > {
        Ok(Self::WorkspaceInstanceWorkerDependenciesT::dependencies().await?)
    }
    async fn new_event_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<NewEventWorkerDependencies<W, Self::NewEventWorkerDependenciesT>> {
        Ok(Self::NewEventWorkerDependenciesT::dependencies().await?)
    }
}
