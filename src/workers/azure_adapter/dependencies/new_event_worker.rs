use crate::{
    workers::{
        adapters::dependencies::new_event_worker::{
            NewEventWorkerContext, NewEventWorkerDependencies,
        },
        azure_adapter::senders::AzureServiceBusActiveStepSender,
        rabbitmq_adapter::{
            managers::RabbitMqStepsAwaitingEventManager, receivers::RabbitMqEventReceiver,
        },
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;
use tikv_client::RawClient;

pub struct AzureServiceBusNewEventWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> NewEventWorkerContext<W> for AzureServiceBusNewEventWorkerDependencies<W, (), ()> {
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type EventReceiver = RabbitMqEventReceiver<W>;
    type StepsAwaitingEventManager = RabbitMqStepsAwaitingEventManager<W>;
    async fn dependencies() -> anyhow::Result<NewEventWorkerDependencies<W, Self>> {
        let mut fe2o3_connection = fe2o3_amqp::Connection::open(
            "control-connection-2",
            "amqp://guest:guest@127.0.0.1:5672",
        )
        .await?;

        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let mut service_bus_client = ServiceBusClient::with_custom_retry_policy::<BasicRetryPolicy>()
            .new_from_connection_string(
                &std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")?,
                ServiceBusClientOptions::default(),
            ).await?;

        let active_step_sender = AzureServiceBusActiveStepSender::<W>::new(&mut service_bus_client).await?;

        let steps_awaiting_event_manager = RabbitMqStepsAwaitingEventManager::<W>::new(
            RawClient::new(vec!["127.0.0.1:2379"]).await?,
        );

        let event_receiver = RabbitMqEventReceiver::<W>::new(&mut fe2o3_session).await?;

        Ok(NewEventWorkerDependencies::new(
            active_step_sender,
            event_receiver,
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
