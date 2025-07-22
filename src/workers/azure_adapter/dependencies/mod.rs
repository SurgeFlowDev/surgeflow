use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use azure_data_cosmos::CosmosClient;
use sqlx::PgPool;

use crate::{
    workers::{
        adapters::dependencies::{
            ActiveStepWorkerDependencyProvider, CompletedInstanceWorkerDependencyProvider,
            CompletedStepWorkerDependencyProvider, DependencyManager,
            FailedInstanceWorkerDependencyProvider, FailedStepWorkerDependencyProvider,
            NewEventWorkerDependencyProvider, NewInstanceWorkerDependencyProvider,
            NextStepWorkerDependencyProvider,
            completed_instance_worker::CompletedInstanceWorkerDependencies,
            completed_step_worker::CompletedStepWorkerDependencies,
        },
        azure_adapter::{
            managers::{AzurePersistentStepManager, AzureServiceBusStepsAwaitingEventManager},
            receivers::{
                AzureServiceBusActiveStepReceiver, AzureServiceBusCompletedInstanceReceiver,
                AzureServiceBusCompletedStepReceiver, AzureServiceBusEventReceiver,
                AzureServiceBusFailedInstanceReceiver, AzureServiceBusFailedStepReceiver,
                AzureServiceBusNewInstanceReceiver, AzureServiceBusNextStepReceiver,
            },
            senders::{
                AzureServiceBusActiveStepSender, AzureServiceBusCompletedStepSender,
                AzureServiceBusFailedInstanceSender, AzureServiceBusFailedStepSender,
                AzureServiceBusNextStepSender,
            },
        },
    },
    workflows::Workflow,
};

pub mod active_step_worker;
pub mod completed_instance_worker;
pub mod completed_step_worker;
pub mod control_server;
pub mod failed_instance_worker;
pub mod failed_step_worker;
pub mod new_event_worker;
pub mod new_instance_worker;
pub mod next_step_worker;

#[derive(Debug)]
pub struct AzureAdapterConfig {
    pub service_bus_connection_string: String,
    pub cosmos_connection_string: String,
    /// The suffix for the new instance queue
    pub new_instance_queue_suffix: String,
    /// The suffix for the next step queue
    pub next_step_queue_suffix: String,
    /// The suffix for the completed instance queue
    pub completed_instance_queue_suffix: String,
    /// The suffix for the completed step queue
    pub completed_step_queue_suffix: String,
    /// The suffix for the active step queue
    pub active_step_queue_suffix: String,
    /// The suffix for the failed instance queue
    pub failed_instance_queue_suffix: String,
    /// The suffix for the failed step queue
    pub failed_step_queue_suffix: String,
    /// The suffix for the new event queue
    pub new_event_queue_suffix: String,
}

#[derive(Debug)]
pub struct AzureDependencyManager {
    service_bus_client: Option<ServiceBusClient<BasicRetryPolicy>>,
    cosmos_client: Option<CosmosClient>,
    sqlx_pool: Option<PgPool>,
    config: AzureAdapterConfig,
}

impl AzureDependencyManager {
    pub fn new(config: AzureAdapterConfig) -> Self {
        Self {
            service_bus_client: None,
            cosmos_client: None,
            sqlx_pool: None,
            config,
        }
    }
}

impl AzureDependencyManager {
    async fn service_bus_client(&mut self) -> &mut ServiceBusClient<BasicRetryPolicy> {
        if self.service_bus_client.is_none() {
            self.service_bus_client = Some(
                ServiceBusClient::new_from_connection_string(
                    self.config.service_bus_connection_string.clone(),
                    ServiceBusClientOptions::default(),
                )
                .await
                .expect("TODO: handle error properly"),
            );
        }
        self.service_bus_client
            .as_mut()
            .expect("TODO: handle error properly")
    }

    fn cosmos_client(&mut self) -> &CosmosClient {
        if self.cosmos_client.is_none() {
            self.cosmos_client = Some(
                CosmosClient::with_connection_string(
                    self.config.cosmos_connection_string.clone().into(),
                    None,
                )
                .expect("TODO: handle error properly"),
            );
        }
        self.cosmos_client
            .as_ref()
            .expect("TODO: handle error properly")
    }

    async fn sqlx_pool(&mut self) -> &mut PgPool {
        if self.sqlx_pool.is_none() {
            let connection_string =
                std::env::var("APP_USER_DATABASE_URL").expect("APP_USER_DATABASE_URL must be set");
            self.sqlx_pool = Some(
                PgPool::connect(&connection_string)
                    .await
                    .expect("TODO: handle error properly"),
            );
        }
        self.sqlx_pool
            .as_mut()
            .expect("TODO: handle error properly")
    }
}

impl<W: Workflow> CompletedInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn completed_instance_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<CompletedInstanceWorkerDependencies<W, Self::CompletedInstanceReceiver>>
    {
        let completed_instance_queue = format!("{}-completed-instances", W::NAME);

        let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
            self.service_bus_client().await,
            &completed_instance_queue,
        )
        .await?;

        Ok(CompletedInstanceWorkerDependencies::new(instance_receiver))
    }
}

impl<W: Workflow> CompletedStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type CompletedStepReceiver = AzureServiceBusCompletedStepReceiver<W>;
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type PersistentStepManager = AzurePersistentStepManager;
    type Error = anyhow::Error;

    async fn completed_step_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<
        CompletedStepWorkerDependencies<
            W,
            Self::CompletedStepReceiver,
            Self::NextStepSender,
            Self::PersistentStepManager,
        >,
    > {
        let completed_steps_queue = format!("{}-completed-steps", W::NAME);
        let next_steps_queue = format!("{}-next-steps", W::NAME);

        let completed_step_receiver = AzureServiceBusCompletedStepReceiver::<W>::new(
            self.service_bus_client().await,
            &completed_steps_queue,
        )
        .await?;

        let next_step_sender = AzureServiceBusNextStepSender::<W>::new(
            self.service_bus_client().await,
            &next_steps_queue,
        )
        .await?;

        let persistent_step_manager =
            AzurePersistentStepManager::new(self.sqlx_pool().await.clone());

        Ok(CompletedStepWorkerDependencies::new(
            completed_step_receiver,
            next_step_sender,
            persistent_step_manager,
        ))
    }
}

impl<W: Workflow> ActiveStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type ActiveStepReceiver = AzureServiceBusActiveStepReceiver<W>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type FailedStepSender = AzureServiceBusFailedStepSender<W>;
    type CompletedStepSender = AzureServiceBusCompletedStepSender<W>;
    type PersistentStepManager = AzurePersistentStepManager;
    type Error = anyhow::Error;

    async fn active_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::active_step_worker::ActiveStepWorkerDependencies<
            W,
            Self::ActiveStepReceiver,
            Self::ActiveStepSender,
            Self::FailedStepSender,
            Self::CompletedStepSender,
            Self::PersistentStepManager,
        >,
        Self::Error,
    > {
        let active_steps_queue = format!("{}{}", W::NAME, self.config.active_step_queue_suffix);
        let failed_steps_queue = format!("{}{}", W::NAME, self.config.failed_step_queue_suffix);
        let completed_steps_queue =
            format!("{}{}", W::NAME, self.config.completed_step_queue_suffix);
        let service_bus_client = self.service_bus_client().await;

        let active_step_receiver =
            AzureServiceBusActiveStepReceiver::<W>::new(service_bus_client, &active_steps_queue)
                .await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<W>::new(service_bus_client, &active_steps_queue)
                .await?;

        let failed_step_sender =
            AzureServiceBusFailedStepSender::<W>::new(service_bus_client, &failed_steps_queue)
                .await?;

        let completed_step_sender = AzureServiceBusCompletedStepSender::<W>::new(
            service_bus_client,
            &completed_steps_queue,
        )
        .await?;

        let persistent_step_manager =
            AzurePersistentStepManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::active_step_worker::ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            persistent_step_manager,
        ))
    }
}

impl<W: Workflow> FailedInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type FailedInstanceReceiver = AzureServiceBusFailedInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn failed_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_instance_worker::FailedInstanceWorkerDependencies<W, Self::FailedInstanceReceiver>,
        Self::Error,
    >{
        let failed_instances_queue =
            format!("{}{}", W::NAME, self.config.failed_instance_queue_suffix);

        let failed_instance_receiver = AzureServiceBusFailedInstanceReceiver::<W>::new(
            self.service_bus_client().await,
            &failed_instances_queue,
        )
        .await?;

        Ok(crate::workers::adapters::dependencies::failed_instance_worker::FailedInstanceWorkerDependencies::new(
            failed_instance_receiver,
        ))
    }
}

impl<W: Workflow> FailedStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type FailedStepReceiver = AzureServiceBusFailedStepReceiver<W>;
    type FailedInstanceSender = AzureServiceBusFailedInstanceSender<W>;
    type PersistentStepManager = AzurePersistentStepManager;
    type Error = anyhow::Error;

    async fn failed_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::failed_step_worker::FailedStepWorkerDependencies<
            W,
            Self::FailedStepReceiver,
            Self::FailedInstanceSender,
            Self::PersistentStepManager,
        >,
        Self::Error,
    > {
        let failed_steps_queue = format!("{}{}", W::NAME, self.config.failed_step_queue_suffix);
        let failed_instances_queue =
            format!("{}{}", W::NAME, self.config.failed_instance_queue_suffix);
        let service_bus_client = self.service_bus_client().await;

        let failed_step_receiver =
            AzureServiceBusFailedStepReceiver::<W>::new(service_bus_client, &failed_steps_queue)
                .await?;

        let failed_instance_sender = AzureServiceBusFailedInstanceSender::<W>::new(
            service_bus_client,
            &failed_instances_queue,
        )
        .await?;

        let persistent_step_manager =
            AzurePersistentStepManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::failed_step_worker::FailedStepWorkerDependencies::new(
            failed_step_receiver,
            failed_instance_sender,
            persistent_step_manager,
        ))
    }
}

impl<W: Workflow> NewEventWorkerDependencyProvider<W> for AzureDependencyManager {
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type EventReceiver = AzureServiceBusEventReceiver<W>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
    type Error = anyhow::Error;

    async fn new_event_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_event_worker::NewEventWorkerDependencies<
            W,
            Self::ActiveStepSender,
            Self::EventReceiver,
            Self::StepsAwaitingEventManager,
        >,
        Self::Error,
    > {
        let new_event_queue = format!("{}{}", W::NAME, self.config.new_event_queue_suffix);
        let active_steps_queue = format!("{}{}", W::NAME, self.config.active_step_queue_suffix);
        let service_bus_client = self.service_bus_client().await;

        let event_receiver =
            AzureServiceBusEventReceiver::<W>::new(service_bus_client, &new_event_queue).await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<W>::new(service_bus_client, &active_steps_queue)
                .await?;

        let cosmos_client = self.cosmos_client();

        let steps_awaiting_event_manager =
            AzureServiceBusStepsAwaitingEventManager::<W>::new(cosmos_client);

        Ok(crate::workers::adapters::dependencies::new_event_worker::NewEventWorkerDependencies::new(
            active_step_sender,
            event_receiver,
            steps_awaiting_event_manager,
        ))
    }
}

impl<W: Workflow> NewInstanceWorkerDependencyProvider<W> for AzureDependencyManager {
    type NextStepSender = AzureServiceBusNextStepSender<W>;
    type NewInstanceReceiver = AzureServiceBusNewInstanceReceiver<W>;
    type Error = anyhow::Error;

    async fn new_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::new_instance_worker::NewInstanceWorkerDependencies<
            W,
            Self::NextStepSender,
            Self::NewInstanceReceiver,
        >,
        Self::Error,
    > {
        let new_instance_queue = format!("{}{}", W::NAME, self.config.new_instance_queue_suffix);
        let next_steps_queue = format!("{}{}", W::NAME, self.config.next_step_queue_suffix);
        let service_bus_client = self.service_bus_client().await;

        let new_instance_receiver =
            AzureServiceBusNewInstanceReceiver::<W>::new(service_bus_client, &new_instance_queue)
                .await?;

        let next_step_sender =
            AzureServiceBusNextStepSender::<W>::new(service_bus_client, &next_steps_queue).await?;

        Ok(crate::workers::adapters::dependencies::new_instance_worker::NewInstanceWorkerDependencies::new(
            next_step_sender,
            new_instance_receiver,
        ))
    }
}

impl<W: Workflow> NextStepWorkerDependencyProvider<W> for AzureDependencyManager {
    type NextStepReceiver = AzureServiceBusNextStepReceiver<W>;
    type ActiveStepSender = AzureServiceBusActiveStepSender<W>;
    type StepsAwaitingEventManager = AzureServiceBusStepsAwaitingEventManager<W>;
    type PersistentStepManager = AzurePersistentStepManager;
    type Error = anyhow::Error;

    async fn next_step_worker_dependencies(
        &mut self,
    ) -> Result<
        crate::workers::adapters::dependencies::next_step_worker::NextStepWorkerDependencies<
            W,
            Self::NextStepReceiver,
            Self::ActiveStepSender,
            Self::StepsAwaitingEventManager,
            Self::PersistentStepManager,
        >,
        Self::Error,
    > {
        let next_steps_queue = format!("{}{}", W::NAME, self.config.next_step_queue_suffix);
        let active_steps_queue = format!("{}{}", W::NAME, self.config.active_step_queue_suffix);
        let service_bus_client = self.service_bus_client().await;

        let next_step_receiver =
            AzureServiceBusNextStepReceiver::<W>::new(service_bus_client, &next_steps_queue)
                .await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<W>::new(service_bus_client, &active_steps_queue)
                .await?;

        let cosmos_client = self.cosmos_client();

        let steps_awaiting_event_manager =
            AzureServiceBusStepsAwaitingEventManager::<W>::new(cosmos_client);

        let persistent_step_manager =
            AzurePersistentStepManager::new(self.sqlx_pool().await.clone());

        Ok(crate::workers::adapters::dependencies::next_step_worker::NextStepWorkerDependencies::new(
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            persistent_step_manager,
        ))
    }
}

impl<W: Workflow> DependencyManager<W> for AzureDependencyManager {
    type Error = anyhow::Error;
}
