use crate::{
    workers::{
        adapters::dependencies::active_step_worker::{
            ActiveStepWorkerContext, ActiveStepWorkerDependencies,
        },
        azure_adapter::senders::{AzureServiceBusActiveStepSender, AzureServiceBusFailedStepSender, AzureServiceBusNextStepSender},
        rabbitmq_adapter::{
            receivers::RabbitMqActiveStepReceiver,
        },
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use fe2o3_amqp::{Connection, Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;

pub struct AzureServiceBusActiveStepWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> ActiveStepWorkerContext<W> for AzureServiceBusActiveStepWorkerDependencies<W, (), ()> {
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type ActiveStepReceiver = RabbitMqActiveStepReceiver<W>;
    type FailedStepSender = AzureServiceBusFailedStepSender<W>;
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    async fn dependencies() -> anyhow::Result<ActiveStepWorkerDependencies<W, Self>> {
        let mut fe2o3_connection =
            Connection::open("control-connection-5", "amqp://guest:guest@127.0.0.1:5672").await?;
        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let mut service_bus_client = ServiceBusClient::with_custom_retry_policy::<BasicRetryPolicy>()
            .new_from_connection_string(
                &std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")?,
                ServiceBusClientOptions::default(),
            ).await?;

        let active_step_sender = AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client).await?;

        let active_step_receiver = RabbitMqActiveStepReceiver::<W>::new(&mut fe2o3_session).await?;
        let next_step_sender = AzureServiceBusNextStepSender::<W>::new(&mut service_bus_client).await?;

        let failed_step_sender = AzureServiceBusFailedStepSender::<W>::new(&mut service_bus_client).await?;

        Ok(ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            next_step_sender,
            failed_step_sender,
            Self {
                fe2o3_connection,
                fe2o3_session,
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
