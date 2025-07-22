use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use azure_data_cosmos::CosmosClient;

use crate::{
    workers::{
        adapters::dependencies::{
            ActiveStepWorkerDependencyProvider, CompletedInstanceWorkerDependencyProvider,
            CompletedStepWorkerDependencyProvider, DependencyManager,
            FailedInstanceWorkerDependencyProvider, FailedStepWorkerDependencyProvider,
            NewEventWorkerDependencyProvider, NewInstanceWorkerDependencyProvider,
            NextStepWorkerDependencyProvider,
            completed_instance_worker::CompletedInstanceWorkerDependencies,
            completed_step_worker::CompletedStepWorkerDependencies,
        },
        azure_adapter::{
            managers::AzureServiceBusStepsAwaitingEventManager,
            receivers::{
                AzureServiceBusActiveStepReceiver, AzureServiceBusCompletedInstanceReceiver,
                AzureServiceBusCompletedStepReceiver, AzureServiceBusEventReceiver,
                AzureServiceBusFailedInstanceReceiver, AzureServiceBusFailedStepReceiver,
                AzureServiceBusNewInstanceReceiver, AzureServiceBusNextStepReceiver,
            },
            senders::{
                AzureServiceBusActiveStepSender, AzureServiceBusCompletedStepSender,
                AzureServiceBusFailedInstanceSender, AzureServiceBusFailedStepSender,
                AzureServiceBusNextStepSender,
            },
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

impl<W: Workflow> CompletedInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn completed_instance_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<CompletedInstanceWorkerDependencies<W, Self::CompletedInstanceReceiver>>
    {
        let completed_instance_queue = format!("{}-completed-instances", W::NAME);

        let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
            self.service_bus_client().await,
            &completed_instance_queue,
        )
        .await?;

        Ok(CompletedInstanceWorkerDependencies::new(instance_receiver))
    }
}

impl<W: Workflow> CompletedStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type CompletedStepReceiver = AzureServiceBusCompletedStepReceiver<W>;
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type Error = anyhow::Error;

    async fn completed_step_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<
        CompletedStepWorkerDependencies<W, Self::CompletedStepReceiver, Self::NextStepSender>,
    > {
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

impl<W: Workflow> ActiveStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type ActiveStepReceiver = AzureServiceBusActiveStepReceiver<W>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type FailedStepSender = AzureServiceBusFailedStepSender<W>;
    type CompletedStepSender = AzureServiceBusCompletedStepSender<W>;
    type Error = anyhow::Error;

    async fn active_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::active_step_worker::ActiveStepWorkerDependencies<
            W,
            Self::ActiveStepReceiver,
            Self::ActiveStepSender,
            Self::FailedStepSender,
            Self::CompletedStepSender,
        >,
        Self::Error,
    > {
        todo!()
    }
}

impl<W: Workflow> FailedInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type FailedInstanceReceiver = AzureServiceBusFailedInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn failed_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_instance_worker::FailedInstanceWorkerDependencies<W, Self::FailedInstanceReceiver>,
        Self::Error,
    >{
        todo!()
    }
}

impl<W: Workflow> FailedStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type FailedStepReceiver = AzureServiceBusFailedStepReceiver<W>;
    type FailedInstanceSender = AzureServiceBusFailedInstanceSender<W>;
    type Error = anyhow::Error;

    async fn failed_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_step_worker::FailedStepWorkerDependencies<
            W,
            Self::FailedStepReceiver,
            Self::FailedInstanceSender,
        >,
        Self::Error,
    > {
        todo!()
    }
}

impl<W: Workflow> NewEventWorkerDependencyProvider<W> for AzureDependencyManager {
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type EventReceiver = AzureServiceBusEventReceiver<W>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
    type Error = anyhow::Error;

    async fn new_event_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_event_worker::NewEventWorkerDependencies<
            W,
            Self::ActiveStepSender,
            Self::EventReceiver,
            Self::StepsAwaitingEventManager,
        >,
        Self::Error,
    > {
        todo!()
    }
}

impl<W: Workflow> NewInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type NewInstanceReceiver = AzureServiceBusNewInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn new_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_instance_worker::NewInstanceWorkerDependencies<
            W,
            Self::NextStepSender,
            Self::NewInstanceReceiver,
        >,
        Self::Error,
    > {
        todo!()
    }
}

impl<W: Workflow> NextStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type NextStepReceiver = AzureServiceBusNextStepReceiver<W>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
    type Error = anyhow::Error;

    async fn next_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::next_step_worker::NextStepWorkerDependencies<
            W,
            Self::NextStepReceiver,
            Self::ActiveStepSender,
            Self::StepsAwaitingEventManager,
        >,
        Self::Error,
    > {
        todo!()
    }
}

impl<W: Workflow> DependencyManager<W> for AzureDependencyManager {
    type Error = anyhow::Error;
}
