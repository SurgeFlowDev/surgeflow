use crate::{
    workers::{
        adapters::dependencies::control_server::{ControlServerContext, ControlServerDependencies},
        azure_adapter::{
            managers::AzureServiceBusWorkflowInstanceManager, senders::AzureServiceBusEventSender,
        },
    },
    workflows::Project,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};

use std::marker::PhantomData;

pub struct AzureServiceBusControlServerDependencies<P: Project> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    workflow_name: String,
    _marker: PhantomData<P>,
}

impl<P: Project> AzureServiceBusControlServerDependencies<P> {
    pub fn new(service_bus_client: ServiceBusClient<BasicRetryPolicy>, workflow_name: String) -> Self {
        Self {
            service_bus_client,
            workflow_name,
            _marker: PhantomData,
        }
    }
    
    pub async fn with_workflow_name(workflow_name: String) -> anyhow::Result<Self> {
        let azure_service_bus_connection_string =
            std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

        let service_bus_client = ServiceBusClient::new_from_connection_string(
            azure_service_bus_connection_string,
            ServiceBusClientOptions::default(),
        )
        .await?;

        Ok(Self::new(service_bus_client, workflow_name))
    }
}

impl<P: Project> ControlServerContext<P> for AzureServiceBusControlServerDependencies<P> {
    type EventSender = AzureServiceBusEventSender<P>;
    type InstanceManager = AzureServiceBusWorkflowInstanceManager<P>;
    
    async fn dependencies() -> anyhow::Result<ControlServerDependencies<P, Self>> {
        // This approach won't work because we can't get the workflow name from a static method.
        // We need to refactor this to pass the workflow name explicitly.
        todo!("This needs to be refactored to accept workflow name as parameter")
    }
}
