use crate::{
    workers::{
        adapters::dependencies::new_instance_worker::{
            NewInstanceWorkerContext, NewInstanceWorkerDependencies,
        },
        azure_adapter::{
            receivers::AzureServiceBusNewInstanceReceiver, senders::AzureServiceBusNextStepSender,
        },
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use std::marker::PhantomData;

pub struct AzureServiceBusNewInstanceWorkerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> NewInstanceWorkerContext<W> for AzureServiceBusNewInstanceWorkerDependencies<W> {
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type InstanceReceiver = AzureServiceBusNewInstanceReceiver<W>;
    async fn dependencies() -> anyhow::Result<NewInstanceWorkerDependencies<W, Self>> {
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
            AzureServiceBusNewInstanceReceiver::new(&mut service_bus_client, &instance_queue)
                .await?;

        Ok(NewInstanceWorkerDependencies::new(
            next_step_sender,
            instance_receiver,
            Self {
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
