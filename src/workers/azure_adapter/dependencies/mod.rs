use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use azure_data_cosmos::CosmosClient;

use crate::{
    workers::{
        adapters::dependencies::{
            DependencyManager, completed_instance_worker::CompletedInstanceWorkerDependencies,
            completed_step_worker::CompletedStepWorkerDependencies,
        },
        azure_adapter::{
            receivers::{
                AzureServiceBusCompletedInstanceReceiver, AzureServiceBusCompletedStepReceiver,
            },
            senders::AzureServiceBusNextStepSender,
        },
    },
    workflows::Workflow,
};

pub mod active_step_worker;
pub mod completed_instance_worker;
pub mod completed_step_worker;
pub mod control_server;
pub mod failed_instance_worker;
pub mod failed_step_worker;
pub mod new_event_worker;
pub mod new_instance_worker;
pub mod next_step_worker;

#[derive(Debug, Default)]
pub struct AzureDependencyManager {
    service_bus_client: Option<ServiceBusClient<BasicRetryPolicy>>,
    cosmos_client: Option<CosmosClient>,
}

impl AzureDependencyManager {
    async fn service_bus_client(&mut self) -> &mut ServiceBusClient<BasicRetryPolicy> {
        if self.service_bus_client.is_none() {
            let connection_string = std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");
            self.service_bus_client = Some(
                ServiceBusClient::new_from_connection_string(
                    connection_string,
                    ServiceBusClientOptions::default(),
                )
                .await
                .expect("Failed to create Service Bus client"),
            );
        }
        self.service_bus_client.as_mut().unwrap()
    }

    fn cosmos_client(&mut self) -> &CosmosClient {
        if self.cosmos_client.is_none() {
            let connection_string = std::env::var("COSMOS_CONNECTION_STRING")
                .expect("COSMOS_CONNECTION_STRING must be set");
            self.cosmos_client = Some(
                CosmosClient::with_connection_string(connection_string.into(), None)
                    .expect("Failed to create Cosmos client"),
            );
        }
        self.cosmos_client.as_ref().unwrap()
    }
}

impl DependencyManager for AzureDependencyManager {
    type Error = anyhow::Error;
    async fn completed_instance_worker_dependencies<W: Workflow>(
        &mut self,
    ) -> anyhow::Result<CompletedInstanceWorkerDependencies<W, ()>> {
        let completed_instance_queue = format!("{}-completed-instances", W::NAME);

        let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
            self.service_bus_client().await,
            &completed_instance_queue,
        )
        .await?;

        Ok(CompletedInstanceWorkerDependencies::new(instance_receiver))
    }
    async fn completed_step_worker_dependencies<W: Workflow>(
        &mut self,
    ) -> anyhow::Result<CompletedStepWorkerDependencies<W, ()>> {
        let completed_steps_queue = format!("{}-completed-steps", W::NAME);
        let next_steps_queue = format!("{}-next-steps", W::NAME);

        let completed_step_receiver = AzureServiceBusCompletedStepReceiver::<W>::new(
            self.service_bus_client().await,
            &completed_steps_queue,
        )
        .await?;

        let next_step_sender = AzureServiceBusNextStepSender::<W>::new(
            self.service_bus_client().await,
            &next_steps_queue,
        )
        .await?;

        Ok(CompletedStepWorkerDependencies::new(
            completed_step_receiver,
            next_step_sender,
        ))
    }
}

// impl<W: Workflow> CompletedInstanceWorkerContext<W> for () {
//     type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<W>;

//     fn dependencies()
//     -> impl Future<Output = anyhow::Result<CompletedInstanceWorkerDependencies<W, Self>>> + Send
//     {
//         async { todo!() }
//     }
// }

// impl<W: Workflow> CompletedStepWorkerContext<W> for () {
//     type CompletedStepReceiver = AzureServiceBusCompletedStepReceiver<W>;
//     type NextStepSender = AzureServiceBusNextStepSender<W>;
//     fn dependencies()
//     -> impl Future<Output = anyhow::Result<CompletedStepWorkerDependencies<W, Self>>> + Send {
//         async { todo!() }
//     }
// }
