use azservicebus::amqp::error::RawAmqpMessageError;

pub mod dependencies;
pub mod managers;
pub mod receivers;
pub mod senders;

#[derive(Debug, thiserror::Error)]
pub enum AzureAdapterError {
    #[error("Service Bus error")]
    ServiceBusError(#[source] typespec::Error),
    #[error("CosmosDB error")]
    CosmosDbError(#[source] typespec::Error),
    #[error("Failed to serialize step")]
    SerializeError(#[source] serde_json::Error),
     #[error("Failed to deserialize step")]
     DeserializeError(#[source] serde_json::Error),
    #[error("AMQP message error")]
    AmqpMessageError(#[source] RawAmqpMessageError),
    #[error("Database error")]
    DatabaseError(#[source] sqlx::Error),
}
