// use crate::{
//     workers::{
//         adapters::dependencies::new_event_worker::{
//             NewEventWorkerContext, NewEventWorkerDependencies,
//         },
//         azure_adapter::{
//             managers::AzureServiceBusStepsAwaitingEventManager,
//             receivers::AzureServiceBusEventReceiver, senders::AzureServiceBusActiveStepSender,
//         },
//     },
//     workflows::Workflow,
// };
// use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
// use azure_data_cosmos::CosmosClient;

// use std::marker::PhantomData;

// pub struct AzureServiceBusNewEventWorkerDependencies<W: Workflow> {
//     #[expect(dead_code)]
//     service_bus_client: ServiceBusClient<BasicRetryPolicy>,
//     phantom: PhantomData<W>,
// }

// impl<W: Workflow> NewEventWorkerContext<W> for AzureServiceBusNewEventWorkerDependencies<W> {
//     type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
//     type EventReceiver = AzureServiceBusEventReceiver<W>;
//     type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
//     async fn dependencies() -> anyhow::Result<NewEventWorkerDependencies<W, Self>> {
//         let azure_service_bus_connection_string =
//             std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
//                 .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

//         let mut service_bus_client = ServiceBusClient::new_from_connection_string(
//             azure_service_bus_connection_string,
//             ServiceBusClientOptions::default(),
//         )
//         .await?;

//         let active_steps_queue = format!("{}-active-steps", W::NAME);
//         let events_queue = format!("{}-events", W::NAME);

//         let active_step_sender =
//             AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client, &active_steps_queue)
//                 .await?;

//         let cosmos_connection_string = std::env::var("COSMOS_CONNECTION_STRING")
//             .expect("COSMOS_CONNECTION_STRING must be set");

//         let cosmos_client =
//             CosmosClient::with_connection_string(cosmos_connection_string.into(), None)?;
//         let steps_awaiting_event_manager =
//             AzureServiceBusStepsAwaitingEventManager::<W>::new(&cosmos_client);

//         let event_receiver =
//             AzureServiceBusEventReceiver::<W>::new(&mut service_bus_client, &events_queue).await?;

//         Ok(NewEventWorkerDependencies::new(
//             active_step_sender,
//             event_receiver,
//             steps_awaiting_event_manager,
//             Self {
//                 service_bus_client,
//                 phantom: PhantomData,
//             },
//         ))
//     }
// }
