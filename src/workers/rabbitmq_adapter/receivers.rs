use crate::{
    WorkflowInstance, workers::adapters::receivers::InstanceReceiver, workflows::Workflow,
};
use fe2o3_amqp::{Receiver, link::delivery::DeliveryInfo};
use std::marker::PhantomData;

// TODO: fields should be pub?
#[derive(Debug)]
pub struct RabbitMqInstanceReceiver<W: Workflow>(pub Receiver, pub PhantomData<W>);

impl<W: Workflow> InstanceReceiver<W> for RabbitMqInstanceReceiver<W> {
    type Handle = DeliveryInfo;
    async fn receive(&mut self) -> anyhow::Result<(WorkflowInstance, DeliveryInfo)> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        Ok((event, msg.into()))
    }

    async fn accept(&mut self, handle: DeliveryInfo) -> anyhow::Result<()> {
        self.0.accept(handle).await.map_err(Into::into)
    }
}
