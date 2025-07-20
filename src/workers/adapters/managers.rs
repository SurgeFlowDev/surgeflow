use std::error::Error;

use crate::{
    step::FullyQualifiedStep,
    workflows::{Workflow, WorkflowInstanceId},
};

pub trait StepsAwaitingEventManager<W: Workflow>: Sized {
    type Error: Error + Send + Sync + 'static;
    fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl std::future::Future<Output = Result<Option<FullyQualifiedStep<W>>, Self::Error>> + Send;
    fn delete_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    fn put_step(
        &mut self,
        step: FullyQualifiedStep<W>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

mod workflow_instance_manager {
    use std::error::Error;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use sqlx::PgConnection;

    use crate::workflows::{Workflow, WorkflowId, WorkflowInstanceId};

    #[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
    pub struct WorkflowInstance {
        pub external_id: WorkflowInstanceId,
        pub workflow_id: WorkflowId,
    }

    pub trait WorkflowInstanceManager<W: Workflow> {
        type Error: Error + Send + Sync + 'static;
        fn create_instance(
            &self,
            conn: &mut PgConnection,
        ) -> impl Future<Output = Result<WorkflowInstance, Self::Error>> + Send;
    }
}

pub use workflow_instance_manager::WorkflowInstance;
pub use workflow_instance_manager::WorkflowInstanceManager;
