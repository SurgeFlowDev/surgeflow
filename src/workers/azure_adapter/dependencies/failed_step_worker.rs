// use std::marker::PhantomData;

// use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};

// use crate::{
//     workers::{
//         adapters::dependencies::failed_step_worker::{
//             FailedStepWorkerContext, FailedStepWorkerDependencies,
//         },
//         azure_adapter::{
//             receivers::AzureServiceBusFailedStepReceiver,
//             senders::AzureServiceBusFailedInstanceSender,
//         },
//     },
//     workflows::Workflow,
// };

// pub struct AzureServiceBusFailedStepWorkerDependencies<W: Workflow> {
//     #[expect(dead_code)]
//     service_bus_client: ServiceBusClient<BasicRetryPolicy>,
//     phantom: PhantomData<W>,
// }

// impl<W: Workflow> FailedStepWorkerContext<W> for AzureServiceBusFailedStepWorkerDependencies<W> {
//     type FailedStepReceiver = AzureServiceBusFailedStepReceiver<W>;
//     type FailedInstanceSender = AzureServiceBusFailedInstanceSender<W>;
//     async fn dependencies() -> anyhow::Result<FailedStepWorkerDependencies<W, Self>> {
//         let azure_service_bus_connection_string =
//             std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
//                 .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

//         let mut service_bus_client = ServiceBusClient::new_from_connection_string(
//             azure_service_bus_connection_string,
//             ServiceBusClientOptions::default(),
//         )
//         .await?;

//         let failed_steps_queue = format!("{}-failed-steps", W::NAME);
//         let failed_instances_queue = format!("{}-failed-instances", W::NAME);

//         let failed_step_receiver = AzureServiceBusFailedStepReceiver::<W>::new(
//             &mut service_bus_client,
//             &failed_steps_queue,
//         )
//         .await?;

//         let failed_instance_sender = AzureServiceBusFailedInstanceSender::<W>::new(
//             &mut service_bus_client,
//             &failed_instances_queue,
//         )
//         .await?;

//         Ok(FailedStepWorkerDependencies::new(
//             failed_step_receiver,
//             failed_instance_sender,
//             Self {
//                 service_bus_client,
//                 phantom: PhantomData,
//             },
//         ))
//     }
// }
