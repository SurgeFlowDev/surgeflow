use crate::{
    workers::{
        adapters::dependencies::control_server::{ControlServerContext, ControlServerDependencies},
        azure_adapter::{
            managers::AzureServiceBusWorkflowInstanceManager, senders::AzureServiceBusEventSender,
        },
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
    type InstanceManager = AzureServiceBusWorkflowInstanceManager<W>;
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
        let instance_queue = format!("{}-instance", W::NAME);

        let event_sender =
            AzureServiceBusEventSender::<W>::new(&mut service_bus_client, &events_queue).await?;
        let instance_manager = AzureServiceBusWorkflowInstanceManager::<W>::new(
            &mut service_bus_client,
            &instance_queue,
        )
        .await?;

        Ok(ControlServerDependencies::new(
            event_sender,
            instance_manager,
            Self {
                service_bus_client,
                _marker: PhantomData,
            },
        ))
    }

    
}
