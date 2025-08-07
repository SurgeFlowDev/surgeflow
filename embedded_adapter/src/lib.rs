use async_channel::SendError;
use aws_sdk_sqs::{
    config::http::HttpResponse, error::SdkError, operation::receive_message::ReceiveMessageError,
    operation::send_message::SendMessageError,
};
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

pub mod dependencies;
pub mod managers;
pub mod receivers;
pub mod senders;

#[derive(derive_more::Debug, thiserror::Error)]
pub enum AwsAdapterError<P: Project> {
    #[error("failed to receive message")]
    ReceiveMessageError(#[from] SdkError<ReceiveMessageError, HttpResponse>),
    #[error("failed to send step")]
    SendStepError(#[from] SendError<FullyQualifiedStep<P>>),
    #[error("failed to send instance event")]
    SendInstanceEventError(#[from] SendError<InstanceEvent<P>>),
    #[error("failed to send workflow instance")]
    SendWorkflowInstanceEventError(#[from] SendError<WorkflowInstance>),
    #[error("no messages received")]
    NoMessagesReceived,
    #[error("message without body")]
    MessageWithoutBody,
    #[error("message without recepit handle")]
    MessageWithoutReceptHandle,
    #[error("Failed to deserialize step")]
    DeserializeError(#[source] serde_json::Error),
    #[error("Failed to serialize step")]
    SerializeError(#[source] serde_json::Error),
    #[error("DynamoDB KV error")]
    DynamoKvError(#[from] managers::dynamo_kv::DynamoKvError),
    #[error("SQLx error")]
    SqlxError(#[from] sqlx::Error),
}

// pub type Result<P, T> = std::result::Result<T, AwsAdapterError<P>>;
