

// pub struct AzureServiceBusActiveStepWorkerDependencies<W: Workflow> {
//     #[expect(dead_code)]
//     service_bus_client: ServiceBusClient<BasicRetryPolicy>,
//     _marker: PhantomData<W>,
// }

// impl<W: Workflow> ActiveStepWorkerContext<W> for AzureServiceBusActiveStepWorkerDependencies<W> {
//     type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
//     type ActiveStepReceiver = AzureServiceBusActiveStepReceiver<W>;
//     type FailedStepSender = AzureServiceBusFailedStepSender<W>;
//     type NextStepSender = AzureServiceBusNextStepSender<W>;
//     type CompletedStepSender = AzureServiceBusCompletedStepSender<W>;
//     async fn dependencies() -> anyhow::Result<ActiveStepWorkerDependencies<W, Self>> {
//         let azure_service_bus_connection_string =
//             std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
//                 .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

//         let mut service_bus_client = ServiceBusClient::new_from_connection_string(
//             azure_service_bus_connection_string,
//             ServiceBusClientOptions::default(),
//         )
//         .await?;

//         let active_steps_queue = format!("{}-active-steps", W::NAME);
//         let failed_steps_queue = format!("{}-failed-steps", W::NAME);
//         let completed_steps_queue = format!("{}-completed-steps", W::NAME);

//         let active_step_sender =
//             AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client, &active_steps_queue)
//                 .await?;

//         let active_step_receiver = AzureServiceBusActiveStepReceiver::<W>::new(
//             &mut service_bus_client,
//             &active_steps_queue,
//         )
//         .await?;

//         let failed_step_sender =
//             AzureServiceBusFailedStepSender::<W>::new(&mut service_bus_client, &failed_steps_queue)
//                 .await?;
//         let completed_step_sender = AzureServiceBusCompletedStepSender::<W>::new(
//             &mut service_bus_client,
//             &completed_steps_queue,
//         )
//         .await?;

//         Ok(ActiveStepWorkerDependencies::new(
//             active_step_receiver,
//             active_step_sender,
//             failed_step_sender,
//             completed_step_sender,
//             Self {
//                 service_bus_client,
//                 _marker: PhantomData,
//             },
//         ))
//     }
// }
