use crate::{
    workers::{
        adapters::dependencies::new_event_worker::{
            NewEventWorkerContext, NewEventWorkerDependencies,
        },
        rabbitmq_adapter::{
            managers::RabbitMqStepsAwaitingEventManager, receivers::RabbitMqEventReceiver,
            senders::RabbitMqActiveStepSender,
        },
    },
    workflows::Workflow,
};
use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;
use tikv_client::RawClient;

pub struct RabbitMqNewEventWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> NewEventWorkerContext<W> for RabbitMqNewEventWorkerDependencies<W, (), ()> {
    type ActiveStepSender = RabbitMqActiveStepSender<W>;
    type EventReceiver = RabbitMqEventReceiver<W>;
    type StepsAwaitingEventManager = RabbitMqStepsAwaitingEventManager<W>;
    async fn dependencies() -> anyhow::Result<NewEventWorkerDependencies<W, Self>> {
        let mut fe2o3_connection = fe2o3_amqp::Connection::open(
            "control-connection-2",
            "amqp://guest:guest@127.0.0.1:5672",
        )
        .await?;

        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let active_step_sender = RabbitMqActiveStepSender::<W>::new(&mut fe2o3_session).await?;

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
                phantom: PhantomData,
            },
        ))
    }
}
