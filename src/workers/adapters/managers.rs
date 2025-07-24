use std::error::Error;

use crate::{
    step::FullyQualifiedStep,
    workflows::{Project, WorkflowInstanceId},
};

pub trait StepsAwaitingEventManager<P: Project>: Sized {
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

mod persistent_step_manager {
    use std::fmt::{Debug, Display};

    use crate::workflows::{StepId, Project, WorkflowInstanceId};

    pub trait PersistentStepManager {
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
    }
}

pub use persistent_step_manager::PersistentStepManager;

mod workflow_instance_manager {
    use std::error::Error;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use sqlx::PgConnection;

    use crate::workflows::{Project, WorkflowId, WorkflowInstanceId};

    #[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
    pub struct WorkflowInstance {
        pub external_id: WorkflowInstanceId,
        pub workflow_id: WorkflowId,
    }

    pub trait WorkflowInstanceManager<P: Project> {
        type Error: Error + Send + Sync + 'static;
        fn create_instance(
            &self,
            conn: &mut PgConnection,
        ) -> impl Future<Output = Result<WorkflowInstance, Self::Error>> + Send;
    }
}

pub use workflow_instance_manager::WorkflowInstance;
pub use workflow_instance_manager::WorkflowInstanceManager;
