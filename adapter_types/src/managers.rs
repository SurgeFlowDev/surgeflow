use std::error::Error;
use surgeflow_types::{FullyQualifiedStep, Project, WorkflowInstanceId};

pub use persistence_manager::PersistenceManager;

pub trait StepsAwaitingEventManager<P: Project>: Sized + Send + 'static + Clone {
    type Error: Error + Send + Sync + 'static;
    fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl Future<Output = Result<Option<FullyQualifiedStep<P>>, Self::Error>> + Send;
    fn delete_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn put_step(
        &mut self,
        step: FullyQualifiedStep<P>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

mod persistence_manager {
    use std::fmt::{Debug, Display};

    use surgeflow_types::{Project, StepId, WorkflowInstance, WorkflowInstanceId};

    // TODO: should these take references instead of ownership?
    pub trait PersistenceManager: Sized + Send + 'static + Clone {
        type Error: Send + Sync + 'static + Debug + Display;
        fn set_step_status(
            &self,
            step_id: StepId,
            status: i32,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;

        fn insert_step<P: Project>(
            &self,
            workflow_instance_id: WorkflowInstanceId,
            step_id: StepId,
            step: &P::Step,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;

        fn insert_step_output<P: Project>(
            &self,
            step_id: StepId,
            output: Option<&P::Step>,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;

        fn insert_instance(
            &self,
            workflow_instance: WorkflowInstance,
        ) -> impl Future<Output = Result<WorkflowInstanceId, Self::Error>> + Send;
    }
}
