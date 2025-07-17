use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};

use crate::{
    workers::{
        adapters::dependencies::active_step_worker::{
            ActiveStepWorkerContext, ActiveStepWorkerDependencies,
        },
        azure_adapter::{
            receivers::AzureServiceBusActiveStepReceiver,
            senders::{
                AzureServiceBusActiveStepSender, AzureServiceBusFailedStepSender,
                AzureServiceBusNextStepSender,
            },
        },
    },
    workflows::Workflow,
};
use std::marker::PhantomData;

pub struct AzureServiceBusActiveStepWorkerDependencies<W: Workflow> {
    #[expect(dead_code)]
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    _marker: PhantomData<W>,
}

impl<W: Workflow> ActiveStepWorkerContext<W> for AzureServiceBusActiveStepWorkerDependencies<W> {
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type ActiveStepReceiver = AzureServiceBusActiveStepReceiver<W>;
    type FailedStepSender = AzureServiceBusFailedStepSender<W>;
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    async fn dependencies() -> anyhow::Result<ActiveStepWorkerDependencies<W, Self>> {
        let mut service_bus_client = ServiceBusClient::new_from_connection_string(
            "connection_string",
            ServiceBusClientOptions::default(),
        )
        .await?;

        let active_steps_queue = format!("{}-active-steps", W::NAME);
        let next_steps_queue = format!("{}-next-steps", W::NAME);
        let failed_steps_queue = format!("{}-failed-steps", W::NAME);

        let active_step_sender =
            AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client, &active_steps_queue)
                .await?;

        let active_step_receiver = AzureServiceBusActiveStepReceiver::<W>::new(
            &mut service_bus_client,
            &active_steps_queue,
        )
        .await?;
        let next_step_sender =
            AzureServiceBusNextStepSender::<W>::new(&mut service_bus_client, &next_steps_queue)
                .await?;

        let failed_step_sender =
            AzureServiceBusFailedStepSender::<W>::new(&mut service_bus_client, &failed_steps_queue)
                .await?;

        Ok(ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            next_step_sender,
            failed_step_sender,
            Self {
                service_bus_client,
                _marker: PhantomData,
            },
        ))
    }
}
