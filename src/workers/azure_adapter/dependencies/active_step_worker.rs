use crate::{
    workers::{
        adapters::dependencies::active_step_worker::{
            ActiveStepWorkerContext, ActiveStepWorkerDependencies,
        },
        rabbitmq_adapter::{
            receivers::RabbitMqActiveStepReceiver,
            senders::{RabbitMqActiveStepSender, RabbitMqFailedStepSender, RabbitMqNextStepSender},
        },
    },
    workflows::Workflow,
};
use fe2o3_amqp::{Connection, Session, connection::ConnectionHandle, session::SessionHandle};
use std::marker::PhantomData;

pub struct RabbitMqActiveStepWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> ActiveStepWorkerContext<W> for RabbitMqActiveStepWorkerDependencies<W, (), ()> {
    type ActiveStepSender = RabbitMqActiveStepSender<W>;
    type ActiveStepReceiver = RabbitMqActiveStepReceiver<W>;
    type FailedStepSender = RabbitMqFailedStepSender<W>;
    type NextStepSender = RabbitMqNextStepSender<W>;
    async fn dependencies() -> anyhow::Result<ActiveStepWorkerDependencies<W, Self>> {
        let mut fe2o3_connection =
            Connection::open("control-connection-5", "amqp://guest:guest@127.0.0.1:5672").await?;
        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let active_step_sender = RabbitMqActiveStepSender::<W>::new(&mut fe2o3_session).await?;

        let active_step_receiver = RabbitMqActiveStepReceiver::<W>::new(&mut fe2o3_session).await?;
        let next_step_sender = RabbitMqNextStepSender::<W>::new(&mut fe2o3_session).await?;

        let failed_step_sender = RabbitMqFailedStepSender::<W>::new(&mut fe2o3_session).await?;

        Ok(ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            next_step_sender,
            failed_step_sender,
            Self {
                fe2o3_connection,
                fe2o3_session,
                phantom: PhantomData,
            },
        ))
    }
}
