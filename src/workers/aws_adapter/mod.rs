use aws_sdk_sqs::{
    config::http::HttpResponse, error::SdkError, operation::receive_message::ReceiveMessageError,
    operation::send_message::SendMessageError,
};

// pub mod dependencies;
// pub mod managers;
pub mod receivers;
pub mod senders;

#[derive(Debug, thiserror::Error)]
pub enum AzureAdapterError {
    #[error("failed to receive message")]
    ReceiveMessageError(#[from] SdkError<ReceiveMessageError, HttpResponse>),
    #[error("failed to send message")]
    SendMessageError(#[from] SdkError<SendMessageError, HttpResponse>),
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
}
