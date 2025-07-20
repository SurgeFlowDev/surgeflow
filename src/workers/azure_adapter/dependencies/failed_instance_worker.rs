use crate::{
    workers::{
        adapters::dependencies::failed_instance_worker::{
            FailedInstanceWorkerContext, FailedInstanceWorkerDependencies,
        },
        azure_adapter::receivers::AzureServiceBusFailedInstanceReceiver,
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use std::marker::PhantomData;

pub struct AzureServiceBusFailedInstanceWorkerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> FailedInstanceWorkerContext<W>
    for AzureServiceBusFailedInstanceWorkerDependencies<W>
{
    type FailedInstanceReceiver = AzureServiceBusFailedInstanceReceiver<W>;

    async fn dependencies() -> anyhow::Result<FailedInstanceWorkerDependencies<W, Self>> {
        let azure_service_bus_connection_string =
            std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

        let mut service_bus_client = ServiceBusClient::new_from_connection_string(
            azure_service_bus_connection_string,
            ServiceBusClientOptions::default(),
        )
        .await?;

        let failed_instances_queue = format!("{}-failed-instances", W::NAME);

        let instance_receiver =
            AzureServiceBusFailedInstanceReceiver::new(&mut service_bus_client, &failed_instances_queue)
                .await?;

        Ok(FailedInstanceWorkerDependencies::new(
            instance_receiver,
            Self {
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
