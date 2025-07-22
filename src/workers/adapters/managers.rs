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
    ) -> impl Future<Output = Result<Option<FullyQualifiedStep<W>>, Self::Error>> + Send;
    fn delete_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn put_step(
        &mut self,
        step: FullyQualifiedStep<W>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

mod persistent_step_manager {
    use std::fmt::{Debug, Display};

    use crate::workflows::{StepId, Workflow, WorkflowInstanceId};

    pub trait PersistentStepManager {
        type Error: Send + Sync + 'static + Debug + Display;
        fn set_step_status(
            &self,
            step_id: StepId,
            status: i32,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;

        fn insert_step<W: Workflow>(
            &self,
            workflow_instance_id: WorkflowInstanceId,
            step_id: StepId,
            step: &W::Step,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;

        fn insert_step_output<W: Workflow>(
            &self,
            step_id: StepId,
            output: Option<&W::Step>,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    }
}

pub use persistent_step_manager::PersistentStepManager;

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
