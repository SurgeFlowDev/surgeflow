use crate::{
    workers::{
        adapters::dependencies::control_server::{ControlServerContext, ControlServerDependencies},
        rabbitmq_adapter::senders::RabbitMqEventSender,
    },
    workflows::Workflow,
};
use fe2o3_amqp::{Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;

pub struct RabbitMqControlServerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> ControlServerContext<W> for RabbitMqControlServerDependencies<W, (), ()> {
    type EventSender = RabbitMqEventSender<W>;
    async fn dependencies() -> anyhow::Result<ControlServerDependencies<W, Self>> {
        let mut fe2o3_connection = fe2o3_amqp::Connection::open(
            "control-connection-control-server",
            "amqp://guest:guest@127.0.0.1:5672",
        )
        .await?;

        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let event_sender = RabbitMqEventSender::<W>::new(&mut fe2o3_session).await?;

        Ok(ControlServerDependencies::new(
            event_sender,
            Self {
                fe2o3_connection,
                fe2o3_session,
                phantom: PhantomData,
            },
        ))
    }
}
