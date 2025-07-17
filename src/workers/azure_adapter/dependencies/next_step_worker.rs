use std::marker::PhantomData;

use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use fe2o3_amqp::{Connection, Session, connection::ConnectionHandle, session::SessionHandle};
use tikv_client::RawClient;

use crate::{
    workers::{
        adapters::dependencies::next_step_worker::{
            NextStepWorkerContext, NextStepWorkerDependencies,
        },
        azure_adapter::senders::AzureServiceBusActiveStepSender,
        rabbitmq_adapter::{
            managers::RabbitMqStepsAwaitingEventManager, receivers::RabbitMqNextStepReceiver,
        },
    },
    workflows::Workflow,
};

pub struct AzureServiceBusNextStepWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> NextStepWorkerContext<W> for AzureServiceBusNextStepWorkerDependencies<W, (), ()> {
    type NextStepReceiver = RabbitMqNextStepReceiver<W>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type StepsAwaitingEventManager = RabbitMqStepsAwaitingEventManager<W>;
    async fn dependencies() -> anyhow::Result<NextStepWorkerDependencies<W, Self>> {
        let mut fe2o3_connection =
            Connection::open("control-connection-3", "amqp://guest:guest@127.0.0.1:5672").await?;
        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let mut service_bus_client = ServiceBusClient::with_custom_retry_policy::<BasicRetryPolicy>()
            .new_from_connection_string(
                &std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")?,
                ServiceBusClientOptions::default(),
            ).await?;

        let next_step_receiver = RabbitMqNextStepReceiver::<W>::new(&mut fe2o3_session).await?;
        let active_step_sender = AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client).await?;

        let steps_awaiting_event_manager = RabbitMqStepsAwaitingEventManager::<W>::new(
            RawClient::new(vec!["127.0.0.1:2379"]).await?,
        );

        Ok(NextStepWorkerDependencies::new(
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            Self {
                fe2o3_connection,
                fe2o3_session,
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
