// TODO: review this file. a senders/receivers/managers are receiveing &str queue names when they should receive String to avoid one extra allocation

use aws_config::SdkConfig;
use aws_credential_types::Credentials;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_sqs::{Client as SqsClient, config::SharedCredentialsProvider};
use aws_types::region::Region;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    workers::{
        adapters::dependencies::{
            ActiveStepWorkerDependencyProvider, CompletedInstanceWorkerDependencyProvider,
            CompletedStepWorkerDependencyProvider, ControlServerDependencyProvider,
            DependencyManager, FailedInstanceWorkerDependencyProvider,
            FailedStepWorkerDependencyProvider, NewEventWorkerDependencyProvider,
            NewInstanceWorkerDependencyProvider, NextStepWorkerDependencyProvider,
            completed_instance_worker::CompletedInstanceWorkerDependencies,
            completed_step_worker::CompletedStepWorkerDependencies,
            control_server::ControlServerDependencies,
        },
        aws_adapter::{
            managers::{AzurePersistenceManager, AzureServiceBusStepsAwaitingEventManager},
            receivers::{
                AzureServiceBusActiveStepReceiver, AzureServiceBusCompletedInstanceReceiver,
                AzureServiceBusCompletedStepReceiver, AzureServiceBusEventReceiver,
                AzureServiceBusFailedInstanceReceiver, AzureServiceBusFailedStepReceiver,
                AzureServiceBusNewInstanceReceiver, AzureServiceBusNextStepReceiver,
            },
            senders::{
                AzureServiceBusActiveStepSender, AzureServiceBusCompletedStepSender,
                AzureServiceBusEventSender, AzureServiceBusFailedInstanceSender,
                AzureServiceBusFailedStepSender, AzureServiceBusNewInstanceSender,
                AzureServiceBusNextStepSender,
            },
        },
    },
    workflows::Project,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AzureAdapterConfig {
    /// The URL for the new instance queue
    pub new_instance_queue_url: String,
    /// The URL for the next step queue
    pub next_step_queue_url: String,
    /// The URL for the completed instance queue
    pub completed_instance_queue_url: String,
    /// The URL for the completed step queue
    pub completed_step_queue_url: String,
    /// The URL for the active step queue
    pub active_step_queue_url: String,
    /// The URL for the failed instance queue
    pub failed_instance_queue_url: String,
    /// The URL for the failed step queue
    pub failed_step_queue_url: String,
    /// The URL for the new event queue
    pub new_event_queue_url: String,
    /// The connection string for the PostgreSQL database
    /// This is used for persistent step management
    pub pg_connection_string: String,
    /// The DynamoDB table name for steps awaiting events
    pub dynamodb_table_name: String,
    /// AWS access key
    pub aws_secret_access_key: String,
    /// AWS access key ID
    pub aws_access_key_id: String,
    /// AWS region
    pub aws_region: String,
}

#[derive(Debug)]
pub struct AzureDependencyManager {
    sqs_client: Option<SqsClient>,
    dynamo_client: Option<DynamoClient>,
    sqlx_pool: Option<PgPool>,
    config: AzureAdapterConfig,
    sdk_config: Option<SdkConfig>,
}

impl AzureDependencyManager {
    pub fn new(config: AzureAdapterConfig) -> Self {
        Self {
            sqs_client: None,
            dynamo_client: None,
            sqlx_pool: None,
            config,
            sdk_config: None,
        }
    }

    fn get_sdk_config(&mut self) -> &SdkConfig {
        if self.sdk_config.is_none() {
            let credentials = Credentials::from_keys(
                self.config.aws_access_key_id.clone(),
                self.config.aws_secret_access_key.clone(),
                None,
            );

            self.sdk_config = Some(
                aws_config::SdkConfig::builder()
                    .credentials_provider(SharedCredentialsProvider::new(credentials))
                    .region(Region::new(self.config.aws_region.clone()))
                    .build(),
            );
        }

        self.sdk_config.as_ref().unwrap()
    }
}

impl AzureDependencyManager {
    async fn sqs_client(&mut self) -> &SqsClient {
        if self.sqs_client.is_none() {
            let config = self.get_sdk_config();
            self.sqs_client = Some(SqsClient::new(config));
        }
        self.sqs_client.as_ref().unwrap()
    }

    async fn dynamo_client(&mut self) -> &DynamoClient {
        if self.dynamo_client.is_none() {
            let config = self.get_sdk_config();
            self.dynamo_client = Some(DynamoClient::new(config));
        }
        self.dynamo_client.as_ref().unwrap()
    }

    async fn sqlx_pool(&mut self) -> &mut PgPool {
        if self.sqlx_pool.is_none() {
            self.sqlx_pool = Some(
                PgPool::connect(&self.config.pg_connection_string)
                    .await
                    .expect("Failed to connect to PostgreSQL database"),
            );
        }
        self.sqlx_pool.as_mut().unwrap()
    }
}

impl<P: Project> CompletedInstanceWorkerDependencyProvider<P> for AzureDependencyManager {
    type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<P>;
    type Error = anyhow::Error;

    async fn completed_instance_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<CompletedInstanceWorkerDependencies<P, Self::CompletedInstanceReceiver>>
    {
        let sqs_client = self.sqs_client().await.clone();

        let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
            sqs_client,
            self.config.completed_instance_queue_url.clone(),
        )
        .await?;

        Ok(CompletedInstanceWorkerDependencies::new(instance_receiver))
    }
}

impl<P: Project> CompletedStepWorkerDependencyProvider<P> for AzureDependencyManager {
    type CompletedStepReceiver = AzureServiceBusCompletedStepReceiver<P>;
    type NextStepSender = AzureServiceBusNextStepSender<P>;
    type PersistenceManager = AzurePersistenceManager;
    type Error = anyhow::Error;

    async fn completed_step_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<
        CompletedStepWorkerDependencies<
            P,
            Self::CompletedStepReceiver,
            Self::NextStepSender,
            Self::PersistenceManager,
        >,
    > {
        let sqs_client = self.sqs_client().await.clone();

        let completed_step_receiver = AzureServiceBusCompletedStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.completed_step_queue_url.clone(),
        )
        .await?;

        let next_step_sender = AzureServiceBusNextStepSender::<P>::new(
            sqs_client,
            self.config.next_step_queue_url.clone(),
        )
        .await?;

        let persistence_manager = AzurePersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(CompletedStepWorkerDependencies::new(
            completed_step_receiver,
            next_step_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> ActiveStepWorkerDependencyProvider<P> for AzureDependencyManager {
    type ActiveStepReceiver = AzureServiceBusActiveStepReceiver<P>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<P>;
    type FailedStepSender = AzureServiceBusFailedStepSender<P>;
    type CompletedStepSender = AzureServiceBusCompletedStepSender<P>;
    type PersistenceManager = AzurePersistenceManager;
    type Error = anyhow::Error;

    async fn active_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::active_step_worker::ActiveStepWorkerDependencies<
            P,
            Self::ActiveStepReceiver,
            Self::ActiveStepSender,
            Self::FailedStepSender,
            Self::CompletedStepSender,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().await.clone();

        let active_step_receiver = AzureServiceBusActiveStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.active_step_queue_url.clone(),
        )
        .await?;

        let active_step_sender = AzureServiceBusActiveStepSender::<P>::new(
            sqs_client.clone(),
            self.config.active_step_queue_url.clone(),
        )
        .await?;

        let failed_step_sender = AzureServiceBusFailedStepSender::<P>::new(
            sqs_client.clone(),
            self.config.failed_step_queue_url.clone(),
        )
        .await?;

        let completed_step_sender = AzureServiceBusCompletedStepSender::<P>::new(
            sqs_client,
            self.config.completed_step_queue_url.clone(),
        )
        .await?;

        let persistence_manager = AzurePersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::active_step_worker::ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> FailedInstanceWorkerDependencyProvider<P> for AzureDependencyManager {
    type FailedInstanceReceiver = AzureServiceBusFailedInstanceReceiver<P>;
    type Error = anyhow::Error;

    async fn failed_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_instance_worker::FailedInstanceWorkerDependencies<P, Self::FailedInstanceReceiver>,
        Self::Error,
    >{
        let sqs_client = self.sqs_client().await.clone();

        let failed_instance_receiver = AzureServiceBusFailedInstanceReceiver::<P>::new(
            sqs_client,
            self.config.failed_instance_queue_url.clone(),
        )
        .await?;

        Ok(crate::workers::adapters::dependencies::failed_instance_worker::FailedInstanceWorkerDependencies::new(
            failed_instance_receiver,
        ))
    }
}

impl<P: Project> FailedStepWorkerDependencyProvider<P> for AzureDependencyManager {
    type FailedStepReceiver = AzureServiceBusFailedStepReceiver<P>;
    type FailedInstanceSender = AzureServiceBusFailedInstanceSender<P>;
    type PersistenceManager = AzurePersistenceManager;
    type Error = anyhow::Error;

    async fn failed_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_step_worker::FailedStepWorkerDependencies<
            P,
            Self::FailedStepReceiver,
            Self::FailedInstanceSender,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().await.clone();

        let failed_step_receiver = AzureServiceBusFailedStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.failed_step_queue_url.clone(),
        )
        .await?;

        let failed_instance_sender = AzureServiceBusFailedInstanceSender::<P>::new(
            sqs_client,
            self.config.failed_instance_queue_url.clone(),
        )
        .await?;

        let persistence_manager = AzurePersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::failed_step_worker::FailedStepWorkerDependencies::new(
            failed_step_receiver,
            failed_instance_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> NewEventWorkerDependencyProvider<P> for AzureDependencyManager {
    type ActiveStepSender = AzureServiceBusActiveStepSender<P>;
    type EventReceiver = AzureServiceBusEventReceiver<P>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<P>;
    type Error = anyhow::Error;

    async fn new_event_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_event_worker::NewEventWorkerDependencies<
            P,
            Self::ActiveStepSender,
            Self::EventReceiver,
            Self::StepsAwaitingEventManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().await.clone();
        let dynamo_client = self.dynamo_client().await.clone();

        let event_receiver = AzureServiceBusEventReceiver::<P>::new(
            sqs_client.clone(),
            self.config.new_event_queue_url.clone(),
        )
        .await?;

        let active_step_sender = AzureServiceBusActiveStepSender::<P>::new(
            sqs_client,
            self.config.active_step_queue_url.clone(),
        )
        .await?;

        let steps_awaiting_event_manager = AzureServiceBusStepsAwaitingEventManager::<P>::new(
            dynamo_client,
            self.config.dynamodb_table_name.clone(),
        );

        Ok(crate::workers::adapters::dependencies::new_event_worker::NewEventWorkerDependencies::new(
            active_step_sender,
            event_receiver,
            steps_awaiting_event_manager,
        ))
    }
}

impl<P: Project> NewInstanceWorkerDependencyProvider<P> for AzureDependencyManager {
    type NextStepSender = AzureServiceBusNextStepSender<P>;
    type NewInstanceReceiver = AzureServiceBusNewInstanceReceiver<P>;
    type PersistenceManager = AzurePersistenceManager;
    type Error = anyhow::Error;

    async fn new_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_instance_worker::NewInstanceWorkerDependencies<
            P,
            Self::NextStepSender,
            Self::NewInstanceReceiver,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().await.clone();

        let new_instance_receiver = AzureServiceBusNewInstanceReceiver::<P>::new(
            sqs_client.clone(),
            self.config.new_instance_queue_url.clone(),
        )
        .await?;

        let next_step_sender = AzureServiceBusNextStepSender::<P>::new(
            sqs_client,
            self.config.next_step_queue_url.clone(),
        )
        .await?;

        let persistence_manager = AzurePersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::new_instance_worker::NewInstanceWorkerDependencies::new(
            next_step_sender,
            new_instance_receiver,
            persistence_manager,
        ))
    }
}

impl<P: Project> NextStepWorkerDependencyProvider<P> for AzureDependencyManager {
    type NextStepReceiver = AzureServiceBusNextStepReceiver<P>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<P>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<P>;
    type PersistenceManager = AzurePersistenceManager;
    type Error = anyhow::Error;

    async fn next_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::next_step_worker::NextStepWorkerDependencies<
            P,
            Self::NextStepReceiver,
            Self::ActiveStepSender,
            Self::StepsAwaitingEventManager,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().await.clone();
        let dynamo_client = self.dynamo_client().await.clone();

        let next_step_receiver = AzureServiceBusNextStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.next_step_queue_url.clone(),
        )
        .await?;

        let active_step_sender = AzureServiceBusActiveStepSender::<P>::new(
            sqs_client,
            self.config.active_step_queue_url.clone(),
        )
        .await?;

        let steps_awaiting_event_manager = AzureServiceBusStepsAwaitingEventManager::<P>::new(
            dynamo_client,
            self.config.dynamodb_table_name.clone(),
        );

        let persistence_manager = AzurePersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::next_step_worker::NextStepWorkerDependencies::new(
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            persistence_manager,
        ))
    }
}

impl<P: Project> ControlServerDependencyProvider<P> for AzureDependencyManager {
    type EventSender = AzureServiceBusEventSender<P>;
    type NewInstanceSender = AzureServiceBusNewInstanceSender<P>;
    type Error = anyhow::Error;

    async fn control_server_dependencies(
        &mut self,
    ) -> Result<ControlServerDependencies<P, Self::EventSender, Self::NewInstanceSender>, Self::Error>
    {
        let sqs_client = self.sqs_client().await.clone();

        let event_sender = AzureServiceBusEventSender::<P>::new(
            sqs_client.clone(),
            self.config.new_event_queue_url.clone(),
        )
        .await?;

        let new_instance_sender = AzureServiceBusNewInstanceSender::<P>::new(
            sqs_client,
            self.config.new_instance_queue_url.clone(),
        )
        .await?;

        Ok(ControlServerDependencies::new(
            event_sender,
            new_instance_sender,
        ))
    }
}

impl<P: Project> DependencyManager<P> for AzureDependencyManager {
    type Error = anyhow::Error;
}
