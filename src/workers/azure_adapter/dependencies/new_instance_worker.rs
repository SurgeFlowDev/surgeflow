use crate::{
    workers::{
        adapters::dependencies::new_instance_worker::{
            WorkspaceInstanceWorkerContext, WorkspaceInstanceWorkerDependencies,
        },
        azure_adapter::{
            receivers::AzureServiceBusInstanceReceiver, senders::AzureServiceBusNextStepSender,
        },
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use std::marker::PhantomData;

pub struct AzureServiceBusWorkspaceInstanceWorkerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> WorkspaceInstanceWorkerContext<W>
    for AzureServiceBusWorkspaceInstanceWorkerDependencies<W>
{
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type InstanceReceiver = AzureServiceBusInstanceReceiver<W>;
    async fn dependencies() -> anyhow::Result<WorkspaceInstanceWorkerDependencies<W, Self>> {
        let azure_service_bus_connection_string =
            std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

        let mut service_bus_client = ServiceBusClient::new_from_connection_string(
            azure_service_bus_connection_string,
            ServiceBusClientOptions::default(),
        )
        .await?;

        let next_steps_queue = format!("{}-next-steps", W::NAME);

        let instance_queue = format!("{}-instance", W::NAME);

        let next_step_sender =
            AzureServiceBusNextStepSender::new(&mut service_bus_client, &next_steps_queue).await?;

        let instance_receiver =
            AzureServiceBusInstanceReceiver::new(&mut service_bus_client, &instance_queue).await?;

        Ok(WorkspaceInstanceWorkerDependencies::new(
            next_step_sender,
            instance_receiver,
            Self {
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
