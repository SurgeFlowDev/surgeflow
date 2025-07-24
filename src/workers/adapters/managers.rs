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

mod persistence_manager {
    use std::fmt::{Debug, Display};

    use crate::{workers::adapters::managers::WorkflowInstance, workflows::{Project, StepId, WorkflowInstanceId}};

    // TODO: should these take references instead of ownership?
    pub trait PersistenceManager {
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

pub use persistence_manager::PersistenceManager;

// TODO: this should be moved to a more appropriate place
mod workflow_instance_manager {
    use crate::workflows::WorkflowInstanceId;
    use derive_more::{From, Into};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
    pub struct WorkflowInstance {
        pub external_id: WorkflowInstanceId,
        pub workflow_name: WorkflowName,
    }
    #[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, From, Into)]
    #[serde(transparent)]
    pub struct WorkflowName(String);

    impl From<&str> for WorkflowName {
        fn from(value: &str) -> Self {
            Self(value.to_string())
        }
    }
}

pub use workflow_instance_manager::WorkflowInstance;
pub use workflow_instance_manager::WorkflowName;
