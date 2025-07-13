use std::marker::PhantomData;

use fe2o3_amqp::{
    Connection, Receiver, Session, connection::ConnectionHandle, session::SessionHandle,
};

use crate::{
    WorkflowInstance,
    step::{FullyQualifiedStep, NextStepSender, NextStepSenderAmqp1_0},
    workflows::Workflow,
};

pub trait WorkspaceInstanceWorkerDependencies<W: Workflow>: Sized {
    fn new() -> impl Future<Output = anyhow::Result<Self>> + Send;
    fn next_step_sender(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<impl NextStepSender<W>>> + Send;
    fn instance_receiver(&mut self) -> impl Future<Output = anyhow::Result<Receiver>> + Send;
}

pub struct RabbitMqHandleEventNewDependencies<W: Workflow> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<()>,
    fe2o3_session: SessionHandle<()>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> WorkspaceInstanceWorkerDependencies<W> for RabbitMqHandleEventNewDependencies<W> {
    async fn new() -> anyhow::Result<Self> {
        let mut fe2o3_connection =
            Connection::open("control-connection-3", "amqp://guest:guest@127.0.0.1:5672").await?;
        let fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        Ok(Self {
            fe2o3_connection,
            fe2o3_session,
            phantom: PhantomData,
        })
    }

    async fn next_step_sender(&mut self) -> anyhow::Result<impl NextStepSender<W>> {
        Ok(NextStepSenderAmqp1_0::<W>::new(&mut self.fe2o3_session).await?)
    }

    async fn instance_receiver(&mut self) -> anyhow::Result<Receiver> {
        Ok(Receiver::attach(
            &mut self.fe2o3_session,
            format!("{}-instances-receiver-1", W::NAME),
            format!("{}-instances", W::NAME),
        )
        .await?)
    }
}

pub async fn main<W: Workflow, D: WorkspaceInstanceWorkerDependencies<W>>() -> anyhow::Result<()> {
    let mut dependencies = D::new().await?;

    let mut instance_receiver = dependencies.instance_receiver().await?;
    let mut next_step_sender = dependencies.next_step_sender().await?;

    loop {
        let Ok(msg) = instance_receiver.recv::<String>().await else {
            continue;
        };
        tracing::info!("Received instance message");
        let Ok(instance) = serde_json::from_str::<WorkflowInstance>(msg.body()) else {
            continue;
        };

        instance_receiver.accept(msg).await?;

        let entrypoint = FullyQualifiedStep {
            instance_id: instance.id,
            step: W::entrypoint(),
            event: None,
            retry_count: 0,
        };

        if let Err(err) = next_step_sender.send(entrypoint).await {
            tracing::error!("Failed to send next step: {:?}", err);
            continue;
        }
    }
}
