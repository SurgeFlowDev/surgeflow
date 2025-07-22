
// pub struct AzureServiceBusCompletedInstanceWorkerDependencies<W: Workflow> {
//     #[expect(dead_code)]
//     service_bus_client: ServiceBusClient<BasicRetryPolicy>,
//     phantom: PhantomData<W>,
// }

// impl<W: Workflow> CompletedInstanceWorkerContext<W>
//     for AzureServiceBusCompletedInstanceWorkerDependencies<W>
// {
//     type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<W>;

//     async fn dependencies() -> anyhow::Result<CompletedInstanceWorkerDependencies<W, Self>> {
//         let azure_service_bus_connection_string =
//             std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")
//                 .expect("AZURE_SERVICE_BUS_CONNECTION_STRING must be set");

//         let mut service_bus_client = ServiceBusClient::new_from_connection_string(
//             azure_service_bus_connection_string,
//             ServiceBusClientOptions::default(),
//         )
//         .await?;

//         let completed_instance_queue = format!("{}-completed-instances", W::NAME);

//         let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
//             &mut service_bus_client,
//             &completed_instance_queue,
//         )
//         .await?;

//         Ok(CompletedInstanceWorkerDependencies::new(
//             instance_receiver,
//             Self {
//                 service_bus_client,
//                 phantom: PhantomData,
//             },
//         ))
//     }
// }
