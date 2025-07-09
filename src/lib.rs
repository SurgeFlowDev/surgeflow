use std::marker::PhantomData;

use aide::OperationIo;
use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;
use fe2o3_amqp::Sender;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, query_as};
use tokio::sync::Mutex;

use crate::{
    event::EventSender,
    workflows::{Workflow, WorkflowId, WorkflowInstanceId},
};

pub mod event;
pub mod step;
pub mod workflows;

pub struct AppState<W: Workflow> {
    pub event_sender: EventSender<W>,
    pub workflow_instance_manager: WorkflowInstanceManager<W>,
    pub sqlx_pool: PgPool,
}

#[derive(Debug)]
pub struct WorkflowInstanceManager<W: Workflow> {
    pub sender: Mutex<Sender>,
    pub _marker: PhantomData<W>,
}

pub struct WorkflowInstanceRecord {
    pub id: i32,
    pub workflow_id: i32,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WorkflowInstance {
    pub id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
}

impl<W: Workflow> WorkflowInstanceManager<W> {
    pub async fn create_instance(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<WorkflowInstance> {
        let res = query_as!(
            WorkflowInstanceRecord,
            r#"
            INSERT INTO workflow_instances ("workflow_id")
            SELECT "id"
            FROM workflows
            WHERE "name" = $1
            RETURNING "id", "workflow_id";
        "#,
            W::NAME
        )
        .fetch_one(conn)
        .await?;

        let res = res.try_into()?;
        let msg = serde_json::to_string(&res)?;
        self.sender.lock().await.send(msg).await?;
        Ok(res)
    }
}

impl TryFrom<WorkflowInstanceRecord> for WorkflowInstance {
    type Error = WorkflowInstanceError;

    fn try_from(value: WorkflowInstanceRecord) -> Result<Self, Self::Error> {
        Ok(WorkflowInstance {
            id: value.id.into(),
            workflow_id: value.workflow_id.into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkflowInstanceError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}
