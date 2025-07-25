use std::marker::PhantomData;

use super::AzureAdapterError;
use crate::{
    step::FullyQualifiedStep,
    workers::adapters::managers::StepsAwaitingEventManager,
    workflows::{Project, WorkflowInstanceId},
};

pub mod dynamo_kv {

    use aws_sdk_dynamodb::{
        Client,
        error::SdkError,
        operation::{delete_item::DeleteItemError, put_item::PutItemError},
        types::{AttributeValue, ReturnValue},
    };
    use aws_sdk_sqs::config::http::HttpResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, thiserror::Error)]
    pub enum DynamoKvError {
        #[error("Item not found")]
        NotFound,
        #[error("Unexpected type for attribute value")]
        UnexpectedType,
        #[error("Put item error: {0}")]
        PutItem(#[from] SdkError<PutItemError, HttpResponse>),
        #[error("Delete item error: {0}")]
        DeleteItem(#[from] SdkError<DeleteItemError, HttpResponse>),
        #[error("Serialization error: {0}")]
        SerializeError(#[source] serde_json::Error),
        #[error("Deserialization error: {0}")]
        DeserializeError(#[source] serde_json::Error),
    }

    pub async fn put_item<V: Serialize>(
        client: &Client,
        table_name: String,
        key: &[u8],
        value: V,
    ) -> Result<(), DynamoKvError> {
        client
            .put_item()
            .table_name(table_name)
            .item("PK", AttributeValue::B(key.into()))
            .item(
                "Value",
                AttributeValue::B(
                    serde_json::to_vec(&value)
                        .map_err(DynamoKvError::SerializeError)?
                        .into(),
                ),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_and_delete_item<V: for<'de> Deserialize<'de>>(
        client: &Client,
        table_name: String,
        key: &[u8],
    ) -> Result<Option<V>, DynamoKvError> {
        let resp = client
            .delete_item()
            .table_name(table_name)
            .key("PK", AttributeValue::B(key.into()))
            .return_values(ReturnValue::AllOld)
            .send()
            .await?;

        if let Some(mut attrs) = resp.attributes {
            if let Some(old_val) = attrs.remove("Value") {
                if let AttributeValue::B(ref blob) = old_val {
                    return Ok(Some(
                        serde_json::from_slice(blob.as_ref())
                            .map_err(DynamoKvError::DeserializeError)?,
                    ));
                } else {
                    return Err(DynamoKvError::UnexpectedType);
                }
            }
        }

        Ok(None)
    }
}

#[derive(Clone)]
pub struct AzureServiceBusStepsAwaitingEventManager<P: Project> {
    dynamo_client: aws_sdk_dynamodb::Client,
    table_name: String,
    _phantom: PhantomData<P>,
}

impl<P: Project> AzureServiceBusStepsAwaitingEventManager<P> {
    // TODO: should be private
    pub fn new(dynamo_client: aws_sdk_dynamodb::Client, table_name: String) -> Self {
        Self {
            dynamo_client,
            table_name,
            _phantom: PhantomData,
        }
    }

    fn make_key(instance_id: WorkflowInstanceId) -> [u8; 16] {
        instance_id.into()
    }
}

impl<P: Project> StepsAwaitingEventManager<P> for AzureServiceBusStepsAwaitingEventManager<P> {
    type Error = AzureAdapterError;
    async fn get_step(
        &mut self,
        instance_id: WorkflowInstanceId,
    ) -> Result<Option<FullyQualifiedStep<P>>, Self::Error> {
        let key = Self::make_key(instance_id);
        match dynamo_kv::get_and_delete_item::<FullyQualifiedStep<P>>(
            &self.dynamo_client,
            self.table_name.clone(),
            &key,
        )
        .await
        {
            Ok(item) => Ok(item),
            Err(dynamo_kv::DynamoKvError::NotFound) => Ok(None),
            Err(e) => Err(AzureAdapterError::DynamoKvError(e)),
        }
    }

    async fn delete_step(&mut self, instance_id: WorkflowInstanceId) -> Result<(), Self::Error> {
        let key = Self::make_key(instance_id);
        dynamo_kv::get_and_delete_item::<FullyQualifiedStep<P>>(
            &self.dynamo_client,
            self.table_name.clone(),
            &key,
        )
        .await
        .map_err(AzureAdapterError::DynamoKvError)?;
        Ok(())
    }

    async fn put_step(&mut self, step: FullyQualifiedStep<P>) -> Result<(), Self::Error> {
        let key = Self::make_key(step.instance.external_id);
        dynamo_kv::put_item(&self.dynamo_client, self.table_name.clone(), &key, step)
            .await
            .map_err(AzureAdapterError::DynamoKvError)?;
        Ok(())
    }
}

mod persistence_manager {
    use sqlx::{PgPool, query};
    use uuid::Uuid;

    use crate::{
        workers::adapters::managers::{PersistenceManager, WorkflowInstance},
        workflows::{Project, StepId, WorkflowInstanceId},
    };

    #[derive(Clone)]
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
                Uuid::from(workflow_instance.external_id),
                String::from(workflow_instance.workflow_name)
            )
            .fetch_one(&self.sqlx_pool)
            .await?;

            Ok(workflow_instance.external_id)
        }
    }
}
pub use persistence_manager::AzurePersistenceManager;
