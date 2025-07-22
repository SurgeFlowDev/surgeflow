// use std::marker::PhantomData;

// use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
// use azure_data_cosmos::CosmosClient;

// use crate::{
//     workers::{
//         adapters::dependencies::next_step_worker::{
//             NextStepWorkerContext, NextStepWorkerDependencies,
//         },
//         azure_adapter::{
//             managers::AzureServiceBusStepsAwaitingEventManager,
//             receivers::AzureServiceBusNextStepReceiver, senders::AzureServiceBusActiveStepSender,
//         },
//     },
//     workflows::Workflow,
// };

// pub struct AzureServiceBusNextStepWorkerDependencies<W: Workflow> {
//     #[expect(dead_code)]
//     service_bus_client: ServiceBusClient<BasicRetryPolicy>,
//     phantom: PhantomData<W>,
// }

// impl<W: Workflow> NextStepWorkerContext<W> for AzureServiceBusNextStepWorkerDependencies<W> {
//     type NextStepReceiver = AzureServiceBusNextStepReceiver<W>;
//     type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
//     type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
//     async fn dependencies() -> anyhow::Result<NextStepWorkerDependencies<W, Self>> {
//         let azure_service_bus_connection_string =
//             std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
//                 .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

//         let mut service_bus_client = ServiceBusClient::new_from_connection_string(
//             azure_service_bus_connection_string,
//             ServiceBusClientOptions::default(),
//         )
//         .await?;

//         let active_steps_queue = format!("{}-active-steps", W::NAME);
//         let next_steps_queue = format!("{}-next-steps", W::NAME);

//         let next_step_receiver =
//             AzureServiceBusNextStepReceiver::<W>::new(&mut service_bus_client, &next_steps_queue)
//                 .await?;
//         let active_step_sender =
//             AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client, &active_steps_queue)
//                 .await?;

//         let cosmos_connection_string = std::env::var("COSMOS_CONNECTION_STRING")
//             .expect("COSMOS_CONNECTION_STRING must be set");

//         let cosmos_client =
//             CosmosClient::with_connection_string(cosmos_connection_string.into(), None)?;
//         let steps_awaiting_event_manager =
//             AzureServiceBusStepsAwaitingEventManager::<W>::new(&cosmos_client);

//         Ok(NextStepWorkerDependencies::new(
//             next_step_receiver,
//             active_step_sender,
//             steps_awaiting_event_manager,
//             Self {
//                 service_bus_client,
//                 phantom: PhantomData,
//             },
//         ))
//     }
// }
