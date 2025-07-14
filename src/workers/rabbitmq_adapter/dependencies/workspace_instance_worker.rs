use std::marker::PhantomData;

use fe2o3_amqp::{
    Connection, Receiver, Sender, Session, connection::ConnectionHandle, session::SessionHandle,
};
use uuid::Uuid;

use crate::{
    workers::{
        adapters::dependencies::workspace_instance_worker::{
            WorkspaceInstanceWorkerContext, WorkspaceInstanceWorkerDependencies,
        },
        rabbitmq_adapter::{receivers::RabbitMqInstanceReceiver, senders::RabbitMqNextStepSender},
    },
    workflows::Workflow,
};

pub struct RabbitMqWorkspaceInstanceWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> WorkspaceInstanceWorkerContext<W>
    for RabbitMqWorkspaceInstanceWorkerDependencies<W, (), ()>
{
    type NextStepSender = RabbitMqNextStepSender<W>;
    type InstanceReceiver = RabbitMqInstanceReceiver<W>;
    async fn dependencies() -> anyhow::Result<WorkspaceInstanceWorkerDependencies<W, Self>> {
        let mut fe2o3_connection =
            Connection::open("control-connection-3", "amqp://guest:guest@127.0.0.1:5672").await?;
        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let next_step_sender = {
            let addr = format!("{}-next-steps", W::NAME);
            let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
            let sender: Sender = Sender::attach(&mut fe2o3_session, link_name, addr).await?;
            RabbitMqNextStepSender(sender, PhantomData)
        };

        let instance_receiver = {
            let receiver = Receiver::attach(
                &mut fe2o3_session,
                format!("{}-instances-receiver-1", W::NAME),
                format!("{}-instances", W::NAME),
            )
            .await?;
            RabbitMqInstanceReceiver(receiver, PhantomData)
        };

        Ok(WorkspaceInstanceWorkerDependencies::new(
            next_step_sender,
            instance_receiver,
            Self {
                fe2o3_connection,
                fe2o3_session,
                phantom: PhantomData,
            },
        ))
    }
}
