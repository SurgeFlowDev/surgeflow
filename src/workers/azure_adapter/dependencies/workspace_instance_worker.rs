use crate::{
    workers::{
        adapters::dependencies::workspace_instance_worker::{
            WorkspaceInstanceWorkerContext, WorkspaceInstanceWorkerDependencies,
        },
        azure_adapter::senders::AzureServiceBusNextStepSender,
        rabbitmq_adapter::{receivers::RabbitMqInstanceReceiver},
    },
    workflows::Workflow,
};
use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use fe2o3_amqp::{
    Connection, Receiver, Session, connection::ConnectionHandle, session::SessionHandle,
};
use std::marker::PhantomData;

pub struct AzureServiceBusWorkspaceInstanceWorkerDependencies<W: Workflow, C, S> {
    #[expect(dead_code)]
    fe2o3_connection: ConnectionHandle<C>,
    #[expect(dead_code)]
    fe2o3_session: SessionHandle<S>,
    service_bus_client: ServiceBusClient<BasicRetryPolicy>,
    phantom: PhantomData<W>,
}

impl<W: Workflow> WorkspaceInstanceWorkerContext<W>
    for AzureServiceBusWorkspaceInstanceWorkerDependencies<W, (), ()>
{
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type InstanceReceiver = RabbitMqInstanceReceiver<W>;
    async fn dependencies() -> anyhow::Result<WorkspaceInstanceWorkerDependencies<W, Self>> {
        let mut fe2o3_connection =
            Connection::open("control-connection-4", "amqp://guest:guest@127.0.0.1:5672").await?;
        let mut fe2o3_session = Session::begin(&mut fe2o3_connection).await?;

        let mut service_bus_client = ServiceBusClient::with_custom_retry_policy::<BasicRetryPolicy>()
            .new_from_connection_string(
                &std::env::var("AZURE_SERVICE_BUS_CONNECTION_STRING")?,
                ServiceBusClientOptions::default(),
            ).await?;

        let next_step_sender = AzureServiceBusNextStepSender::new(&mut service_bus_client).await?;

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
                service_bus_client,
                phantom: PhantomData,
            },
        ))
    }
}
