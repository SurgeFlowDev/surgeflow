use std::marker::PhantomData;

use azure_data_cosmos::{CosmosClient, PartitionKey, clients::ContainerClient};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use typespec::{error::ErrorKind, http::StatusCode};

use crate::{
    step::FullyQualifiedStep,
    workers::{adapters::managers::StepsAwaitingEventManager, azure_adapter::AzureAdapterError},
    workflows::{Project, WorkflowInstanceId},
};

#[derive(Clone)]
pub struct AzureServiceBusStepsAwaitingEventManager<P: Project> {
    container_client: ContainerClient,
    _phantom: PhantomData<P>,
}

impl<P: Project> AzureServiceBusStepsAwaitingEventManager<P> {
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

impl<P: Project> StepsAwaitingEventManager<P> for AzureServiceBusStepsAwaitingEventManager<P> {
    type Error = AzureAdapterError;
    async fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> Result<Option<FullyQualifiedStep<P>>, Self::Error> {
        let id = Self::make_key(instance_id);
        let item = CosmosItem::<FullyQualifiedStep<P>>::read(&self.container_client, id).await?;

        Ok(item.map(|item| item.data))
    }
    async fn delete_step(&mut self, instance_id: WorkflowInstanceId) -> Result<(), Self::Error> {
        let id = Self::make_key(instance_id);
        CosmosItem::<FullyQualifiedStep<P>>::delete(&self.container_client, id).await?;
        Ok(())
    }
    async fn put_step(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        let id = Self::make_key(step.instance.external_id);
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
    async fn read(
        container_client: &ContainerClient,
        id: String,
    ) -> Result<Option<Self>, AzureAdapterError> {
        let response = try_read_item(container_client, &id, &id)
            .await
            .map_err(AzureAdapterError::CosmosDbError)?;
        Ok(response)
    }
    async fn delete(
        container_client: &ContainerClient,
        id: String,
    ) -> Result<(), AzureAdapterError> {
        container_client
            .delete_item(&id, &id, None)
            .await
            .map_err(AzureAdapterError::CosmosDbError)?;

        Ok(())
    }
    async fn create(self, container_client: &ContainerClient) -> Result<(), AzureAdapterError> {
        let id = self.id.clone();
        container_client
            .create_item(id, self, None)
            .await
            .map_err(AzureAdapterError::CosmosDbError)?;
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

mod persistent_step_manager {
    use sqlx::{PgPool, query};
    use uuid::Uuid;

    use crate::{
        workers::adapters::managers::{PersistenceManager, WorkflowInstance},
        workflows::{Project, StepId, WorkflowInstanceId},
    };

    pub struct AzurePersistenceManager {
        sqlx_pool: PgPool,
    }
    impl AzurePersistenceManager {
        pub fn new(sqlx_pool: PgPool) -> Self {
            Self { sqlx_pool }
        }
    }
    impl PersistenceManager for AzurePersistenceManager {
        type Error = anyhow::Error;
        async fn set_step_status(&self, step_id: StepId, status: i32) -> Result<(), anyhow::Error> {
            query!(
                r#"
        UPDATE workflow_steps SET "status" = $1
        WHERE "external_id" = $2
        "#,
                status,
                Uuid::from(step_id)
            )
            .execute(&self.sqlx_pool)
            .await?;

            Ok(())
        }

        async fn insert_step<P: Project>(
            &self,
            workflow_instance_id: WorkflowInstanceId,
            step_id: StepId,
            step: &P::Step,
        ) -> Result<(), anyhow::Error> {
            let json_step = serde_json::to_value(step).expect("TODO: handle serialization error");
            query!(
                r#"
                INSERT INTO workflow_steps ("workflow_instance_external_id", "external_id", "step")
                VALUES ($1, $2, $3)
                "#,
                Uuid::from(workflow_instance_id),
                Uuid::from(step_id),
                json_step
            )
            .execute(&self.sqlx_pool)
            .await?;

            Ok(())
        }

        async fn insert_step_output<P: Project>(
            &self,
            step_id: StepId,
            output: Option<&P::Step>,
        ) -> Result<(), anyhow::Error> {
            let output = serde_json::to_value(output).expect("TODO: handle serialization error");
            query!(
                r#"
            INSERT INTO workflow_step_outputs ("workflow_step_id", "output")
            VALUES ((SELECT id FROM workflow_steps WHERE external_id = $1), $2)
            "#,
                Uuid::from(step_id),
                output
            )
            .execute(&self.sqlx_pool)
            .await?;

            Ok(())
        }

        async fn insert_instance(
            &self,
            workflow_instance: WorkflowInstance,
        ) -> Result<WorkflowInstanceId, Self::Error> {
            query!(
                r#"
                INSERT INTO workflow_instances ("workflow_id", "external_id")
                SELECT "id", $1
                FROM workflows
                WHERE "name" = $2
                RETURNING "external_id"
                "#,
                Uuid::from(workflow_instance.external_id.clone()),
                String::from(workflow_instance.workflow_name)
            )
            .fetch_one(&self.sqlx_pool)
            .await?;

            Ok(workflow_instance.external_id)
        }
    }
}
pub use persistent_step_manager::AzurePersistenceManager;
