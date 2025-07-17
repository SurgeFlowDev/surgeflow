use std::marker::PhantomData;

use azure_data_cosmos::{CosmosClient, PartitionKey, clients::ContainerClient};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use typespec::{error::ErrorKind, http::StatusCode};

use crate::{
    step::FullyQualifiedStep,
    workers::adapters::managers::StepsAwaitingEventManager,
    workflows::{Workflow, WorkflowInstanceId},
};

#[derive(Clone)]
pub struct AzureServiceBusAwaitingEventManager<W: Workflow> {
    container_client: ContainerClient,
    _phantom: PhantomData<W>,
}

impl<W: Workflow> AzureServiceBusAwaitingEventManager<W> {
    // TODO: should be private
    pub fn new(cosmos_client: &CosmosClient) -> Self {
        let container_client = cosmos_client
            .database_client("events")
            .container_client("events");
        Self {
            container_client,
            _phantom: PhantomData,
        }
    }
    fn make_key(instance_id: WorkflowInstanceId) -> String {
        format!("instance_{}", i32::from(instance_id))
    }
}

impl<W: Workflow> StepsAwaitingEventManager<W> for AzureServiceBusAwaitingEventManager<W> {
    async fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> anyhow::Result<Option<FullyQualifiedStep<W::Step>>> {
        let id = Self::make_key(instance_id);
        let item =
            CosmosItem::<FullyQualifiedStep<W::Step>>::read(&self.container_client, id).await?;

        Ok(item.map(|item| item.data))
    }
    async fn delete_step(&mut self, instance_id: WorkflowInstanceId) -> anyhow::Result<()> {
        let id = Self::make_key(instance_id);
        CosmosItem::<FullyQualifiedStep<W::Step>>::delete(&self.container_client, id).await?;
        Ok(())
    }
    async fn put_step(&mut self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        let id = Self::make_key(step.instance_id);
        let item = CosmosItem::new(step, id);
        item.create(&self.container_client).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct CosmosItem<T: Serialize + DeserializeOwned> {
    id: String,
    #[serde(flatten, bound = "")]
    data: T,
}

impl<T: Serialize + for<'de> Deserialize<'de>> CosmosItem<T> {
    fn new(data: T, id: String) -> Self {
        Self { id, data }
    }
    async fn read(container_client: &ContainerClient, id: String) -> anyhow::Result<Option<Self>> {
        let response = try_read_item(container_client, &id, &id).await?;
        Ok(response)
    }
    async fn delete(container_client: &ContainerClient, id: String) -> anyhow::Result<()> {
        container_client.delete_item(&id, &id, None).await?;

        Ok(())
    }
    async fn create(self, container_client: &ContainerClient) -> anyhow::Result<()> {
        let id = self.id.clone();
        container_client.create_item(id, self, None).await?;
        Ok(())
    }
}

/// Try to read an item; returns Ok(Some(item)) if found,
/// Ok(None) if it wasn\u2019t there, or Err(_) on any other failure.
pub async fn try_read_item<T>(
    container: &ContainerClient,
    pk: impl Into<PartitionKey>,
    id: &str,
) -> typespec::Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    match container.read_item::<T>(pk, id, None).await {
        // if we got a 200, deserialize its body into T
        Ok(response) => {
            let item = response.into_body().await?;
            Ok(Some(item))
        }

        // if we got a 404 Not Found, return Ok(None)
        Err(e)
            if matches!(
                e.kind(),
                ErrorKind::HttpResponse { status, .. } if *status == StatusCode::NotFound
            ) =>
        {
            Ok(None)
        }

        // any other error, propagate
        Err(e) => Err(e),
    }
}
