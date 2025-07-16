use fe2o3_amqp::Sender;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, query_as};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    workers::adapters::senders::EventSender,
    workflows::{TxState, Workflow, WorkflowId, WorkflowInstanceId},
};

pub mod event;
pub mod step;
pub mod workers;
pub mod workflows;

pub struct AppState<W: Workflow, E: EventSender<W>> {
    pub event_sender: E,
    pub workflow_instance_manager: WorkflowInstanceManager<W>,
    pub sqlx_tx_state: TxState,
}

pub struct ArcAppState<W: Workflow, E: EventSender<W>>(pub Arc<AppState<W, E>>);

impl<W: Workflow, E: EventSender<W>> Clone for ArcAppState<W, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
// must be thread-safe
#[derive(Debug)]
pub struct WorkflowInstanceManager<W: Workflow> {
    sender: Mutex<Sender>,
    _marker: PhantomData<W>,
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
