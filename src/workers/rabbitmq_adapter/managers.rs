use std::marker::PhantomData;

use tikv_client::RawClient;

use crate::{
    step::FullyQualifiedStep,
    workers::adapters::managers::StepsAwaitingEventManager,
    workflows::{Workflow, WorkflowInstanceId},
};

#[derive(Clone)]
pub struct RabbitMqStepsAwaitingEventManager<W: Workflow> {
    tikv_client: RawClient,
    _phantom: PhantomData<W>,
}
impl<W: Workflow> RabbitMqStepsAwaitingEventManager<W> {
    // TODO: should be private
    pub fn new(tikv_client: RawClient) -> Self {
        Self {
            tikv_client,
            _phantom: PhantomData,
        }
    }
    fn make_key(instance_id: WorkflowInstanceId) -> String {
        format!("instance_{}", i32::from(instance_id))
    }
}

impl<W: Workflow> StepsAwaitingEventManager<W> for RabbitMqStepsAwaitingEventManager<W> {
    async fn get_step(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> anyhow::Result<Option<FullyQualifiedStep<W::Step>>> {
        let value = self.tikv_client.get(Self::make_key(instance_id)).await?;

        let data = value.map(|v| serde_json::from_slice(&v)).transpose()?;

        Ok(data)
    }
    async fn delete_step(&self, instance_id: WorkflowInstanceId) -> anyhow::Result<()> {
        self.tikv_client.delete(Self::make_key(instance_id)).await?;

        Ok(())
    }
    async fn put_step(&self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        let payload = serde_json::to_vec(&step)?;
        self.tikv_client
            .put(Self::make_key(instance_id), payload)
            .await?;
        Ok(())
    }
}
