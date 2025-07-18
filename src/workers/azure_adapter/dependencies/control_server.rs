use crate::{
    workers::{
        adapters::dependencies::control_server::{ControlServerContext, ControlServerDependencies},
        azure_adapter::senders::AzureServiceBusEventSender,
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};

use std::marker::PhantomData;

pub struct AzureServiceBusControlServerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ControlServerContext<W> for AzureServiceBusControlServerDependencies<W> {
    type EventSender = AzureServiceBusEventSender<W>;
    async fn dependencies() -> anyhow::Result<ControlServerDependencies<W, Self>> {
        let azure_service_bus_connection_string =
            std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");
            
        let mut service_bus_client = ServiceBusClient::new_from_connection_string(
            azure_service_bus_connection_string,
            ServiceBusClientOptions::default(),
        )
        .await?;

        let events_queue = format!("{}-events", W::NAME);

        let event_sender =
            AzureServiceBusEventSender::<W>::new(&mut service_bus_client, &events_queue)
                .await?;

        Ok(ControlServerDependencies::new(
            event_sender,
            Self {
                service_bus_client,
                _marker: PhantomData,
            },
        ))
    }
}
