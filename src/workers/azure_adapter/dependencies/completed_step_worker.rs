use std::marker::PhantomData;

use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};

use crate::{
    workers::{
        adapters::dependencies::completed_step_worker::{
            CompletedStepWorkerContext, CompletedStepWorkerDependencies,
        },
        azure_adapter::receivers::AzureServiceBusCompletedStepReceiver,
    },
    workflows::Workflow,
};

pub struct AzureServiceBusCompletedStepWorkerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> CompletedStepWorkerContext<W>
    for AzureServiceBusCompletedStepWorkerDependencies<W>
{
    type CompletedStepReceiver = AzureServiceBusCompletedStepReceiver<W>;

    async fn dependencies() -> anyhow::Result<CompletedStepWorkerDependencies<W, Self>> {
        let azure_service_bus_connection_string =
            std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
                .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

        let mut service_bus_client = ServiceBusClient::new_from_connection_string(
            azure_service_bus_connection_string,
            ServiceBusClientOptions::default(),
        )
        .await?;

        let completed_steps_queue = format!("{}-completed-steps", W::NAME);

        let completed_step_receiver = AzureServiceBusCompletedStepReceiver::<W>::new(
            &mut service_bus_client,
            &completed_steps_queue,
        )
        .await?;

        Ok(CompletedStepWorkerDependencies::new(
            completed_step_receiver,
            Self {
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
