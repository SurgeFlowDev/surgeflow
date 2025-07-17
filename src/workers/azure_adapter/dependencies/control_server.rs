use crate::{
    workers::{
        adapters::dependencies::control_server::{ControlServerContext, ControlServerDependencies},
        azure_adapter::senders::AzureServiceBusEventSender,
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;

pub struct AzureServiceBusControlServerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> ControlServerContext<W> for AzureServiceBusControlServerDependencies<W, (), ()> {
    type EventSender = AzureServiceBusEventSender<W>;
    async fn dependencies() -> anyhow::Result<ControlServerDependencies<W, Self>> {
        let mut fe2o3_connection = fe2o3_amqp::Connection::open(
            "control-connection-control-server",
            "amqp://guest:guest@127.0.0.1:5672",
        )
        .await?;

        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let mut service_bus_client = ServiceBusClient::with_custom_retry_policy::<BasicRetryPolicy>()
            .new_from_connection_string(
                &std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")?,
                ServiceBusClientOptions::default(),
            ).await?;

        let queue_name = format!("{}-events", W::NAME);
        let event_sender = AzureServiceBusEventSender::<W>::new(&mut service_bus_client, &queue_name).await?;

        Ok(ControlServerDependencies::new(
            event_sender,
            Self {
                fe2o3_connection,
                fe2o3_session,
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
