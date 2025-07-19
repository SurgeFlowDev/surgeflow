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
pub struct AzureServiceBusStepsAwaitingEventManager<W: Workflow> {
    container_client: ContainerClient,
    _phantom: PhantomData<W>,
}

impl<W: Workflow> AzureServiceBusStepsAwaitingEventManager<W> {
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
        instance_id.to_string()
    }
}

impl<W: Workflow> StepsAwaitingEventManager<W> for AzureServiceBusStepsAwaitingEventManager<W> {
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

mod workflow_instance_manager {
    use std::marker::PhantomData;

    use azservicebus::{
        ServiceBusClient, primitives::service_bus_retry_policy::ServiceBusRetryPolicyExt,
    };
    use sqlx::{PgConnection, query_as};
    use uuid::Uuid;

    use crate::{
        workers::{
            adapters::{
                managers::{WorkflowInstance, WorkflowInstanceManager},
                senders::NewInstanceSender,
            },
            azure_adapter::senders::AzureServiceBusInstanceSender,
        },
        workflows::Workflow,
    };

    // must be thread-safe
    #[derive(Debug)]
    pub struct AzureServiceBusWorkflowInstanceManager<W: Workflow> {
        sender: AzureServiceBusInstanceSender<W>,
        _marker: PhantomData<W>,
    }
    impl<W: Workflow> AzureServiceBusWorkflowInstanceManager<W> {
        pub async fn new<RP: ServiceBusRetryPolicyExt + 'static>(
            service_bus_client: &mut ServiceBusClient<RP>,
            instance_queue_name: &str,
        ) -> anyhow::Result<Self> {
            let sender =
                AzureServiceBusInstanceSender::<W>::new(service_bus_client, instance_queue_name)
                    .await?;
            Ok(Self {
                sender,
                _marker: PhantomData,
            })
        }
    }

    struct WorkflowInstanceRecord {
        pub external_id: Uuid,
        pub workflow_id: i32,
    }

    impl<W: Workflow> WorkflowInstanceManager<W> for AzureServiceBusWorkflowInstanceManager<W> {
        async fn create_instance(
            &self,
            conn: &mut PgConnection,
        ) -> anyhow::Result<WorkflowInstance> {
            tracing::info!("Creating workflow instance for: {}", W::NAME);
            let res = query_as!(
                WorkflowInstanceRecord,
                r#"
            INSERT INTO workflow_instances ("workflow_id")
            SELECT "id"
            FROM workflows
            WHERE "name" = $1
            RETURNING "external_id", "workflow_id";
        "#,
                W::NAME
            )
            .fetch_one(conn)
            .await
            .inspect_err(|e| tracing::error!("Failed to create workflow instance: {:?}", e))?;

            tracing::info!(
                "Created workflow instance: {} with id: {}",
                W::NAME,
                res.external_id
            );

            let res: WorkflowInstance = res.try_into()?;

            tracing::info!(
                "Sending workflow instance: {} with id: {} to Azure Service Bus",
                W::NAME,
                res.external_id
            );

            self.sender.send(&res).await?;
            Ok(res)
        }
    }

    impl TryFrom<WorkflowInstanceRecord> for WorkflowInstance {
        type Error = WorkflowInstanceError;

        fn try_from(value: WorkflowInstanceRecord) -> Result<Self, Self::Error> {
            Ok(WorkflowInstance {
                external_id: value.external_id.into(),
                workflow_id: value.workflow_id.into(),
            })
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum WorkflowInstanceError {
        #[error("Database error")]
        Database(#[from] sqlx::Error),
    }
}

pub use workflow_instance_manager::AzureServiceBusWorkflowInstanceManager;
