use async_channel::{RecvError, SendError};
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

pub mod dependencies;
pub mod managers;
pub mod receivers;
pub mod senders;

#[derive(derive_more::Debug, thiserror::Error)]
pub enum AwsAdapterError<P: Project> {
    #[error("failed to receive message")]
    ReceiveMessageError(#[from] RecvError),
    #[error("failed to send step")]
    SendStepError(#[from] SendError<FullyQualifiedStep<P>>),
    #[error("failed to send instance event")]
    SendInstanceEventError(#[from] SendError<InstanceEvent<P>>),
    #[error("failed to send workflow instance")]
    SendWorkflowInstanceEventError(#[from] SendError<WorkflowInstance>),
    #[error("Failed to deserialize step")]
    DeserializeError(#[source] serde_json::Error),
    #[error("Failed to serialize step")]
    SerializeError(#[source] serde_json::Error),
    #[error("SQLx error")]
    SqlxError(#[from] sqlx::Error),
}
