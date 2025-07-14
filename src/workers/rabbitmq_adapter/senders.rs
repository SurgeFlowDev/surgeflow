use std::marker::PhantomData;

use fe2o3_amqp::Sender;

use crate::{
    step::FullyQualifiedStep, workers::adapters::senders::NextStepSender, workflows::Workflow,
};

// TODO: fields should be pub?
#[derive(Debug)]
pub struct RabbitMqNextStepSender<W: Workflow>(pub Sender, pub PhantomData<W>);

impl<W: Workflow> NextStepSender<W> for RabbitMqNextStepSender<W> {
    async fn send(
        &mut self,
        step: FullyQualifiedStep<<W as Workflow>::Step>,
    ) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}
