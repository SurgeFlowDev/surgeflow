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

mod workflow_instance_manager {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use sqlx::PgConnection;

    use crate::workflows::{Workflow, WorkflowId, WorkflowInstanceId};

    // pub struct WorkflowInstanceRecord {
    //     pub id: i32,
    //     pub workflow_id: i32,
    // }
    // impl TryFrom<WorkflowInstanceRecord> for WorkflowInstance {
    //     type Error = WorkflowInstanceError;
    //     fn try_from(value: WorkflowInstanceRecord) -> Result<Self, Self::Error> {
    //         Ok(WorkflowInstance {
    //             id: value.id.into(),
    //             workflow_id: value.workflow_id.into(),
    //         })
    //     }
    // }
    #[derive(Debug, Deserialize, Serialize, JsonSchema)]
    pub struct WorkflowInstance {
        pub external_id: WorkflowInstanceId,
        pub workflow_id: WorkflowId,
    }

    pub trait WorkflowInstanceManager<W: Workflow> {
        fn create_instance(
            &self,
            conn: &mut PgConnection,
        ) -> impl std::future::Future<Output = anyhow::Result<WorkflowInstance>> + Send;
    }
}

pub use workflow_instance_manager::WorkflowInstance;
pub use workflow_instance_manager::WorkflowInstanceManager;
