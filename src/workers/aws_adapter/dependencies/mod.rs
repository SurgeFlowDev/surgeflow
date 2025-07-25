// TODO: review this file. a senders/receivers/managers are receiveing &str queue names when they should receive String to avoid one extra allocation

use azservicebus::{ServiceBusClient, ServiceBusClientOptions, core::BasicRetryPolicy};
use azure_data_cosmos::CosmosClient;
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
    /// The connection string for the PostgreSQL database
    /// This is used for persistent step management
    pub pg_connection_string: String,
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
            let connection_string = &self.config.pg_connection_string;
            self.sqlx_pool = Some(
                PgPool::connect(connection_string)
                    .await
                    .expect("TODO: handle error properly"),
            );
        }
        self.sqlx_pool
            .as_mut()
            .expect("TODO: handle error properly")
    }
}

impl<P: Project> CompletedInstanceWorkerDependencyProvider<P> for AzureDependencyManager {
    type CompletedInstanceReceiver = AzureServiceBusCompletedInstanceReceiver<P>;
    type Error = anyhow::Error;

    async fn completed_instance_worker_dependencies(
        &mut self,
    ) -> anyhow::Result<CompletedInstanceWorkerDependencies<P, Self::CompletedInstanceReceiver>>
    {
        let completed_instance_queue = self.config.completed_instance_queue_suffix.clone();

        let instance_receiver = AzureServiceBusCompletedInstanceReceiver::new(
            self.service_bus_client().await,
            &completed_instance_queue,
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
        let completed_steps_queue = self.config.completed_step_queue_suffix.clone();
        let next_steps_queue = self.config.next_step_queue_suffix.clone();

        let completed_step_receiver = AzureServiceBusCompletedStepReceiver::<P>::new(
            self.service_bus_client().await,
            &completed_steps_queue,
        )
        .await?;

        let next_step_sender = AzureServiceBusNextStepSender::<P>::new(
            self.service_bus_client().await,
            &next_steps_queue,
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
        let active_steps_queue = self.config.active_step_queue_suffix.clone();
        let failed_steps_queue = self.config.failed_step_queue_suffix.clone();
        let completed_steps_queue = self.config.completed_step_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;

        let active_step_receiver =
            AzureServiceBusActiveStepReceiver::<P>::new(service_bus_client, &active_steps_queue)
                .await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<P>::new(service_bus_client, &active_steps_queue)
                .await?;

        let failed_step_sender =
            AzureServiceBusFailedStepSender::<P>::new(service_bus_client, &failed_steps_queue)
                .await?;

        let completed_step_sender = AzureServiceBusCompletedStepSender::<P>::new(
            service_bus_client,
            &completed_steps_queue,
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
        let failed_instances_queue = self.config.failed_instance_queue_suffix.clone();

        let failed_instance_receiver = AzureServiceBusFailedInstanceReceiver::<P>::new(
            self.service_bus_client().await,
            &failed_instances_queue,
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
        let failed_steps_queue = self.config.failed_step_queue_suffix.clone();
        let failed_instances_queue = self.config.failed_instance_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;

        let failed_step_receiver =
            AzureServiceBusFailedStepReceiver::<P>::new(service_bus_client, &failed_steps_queue)
                .await?;

        let failed_instance_sender = AzureServiceBusFailedInstanceSender::<P>::new(
            service_bus_client,
            &failed_instances_queue,
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
        let new_event_queue = self.config.new_event_queue_suffix.clone();
        let active_steps_queue = self.config.active_step_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;

        let event_receiver =
            AzureServiceBusEventReceiver::<P>::new(service_bus_client, &new_event_queue).await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<P>::new(service_bus_client, &active_steps_queue)
                .await?;

        let cosmos_client = self.cosmos_client();

        let steps_awaiting_event_manager =
            AzureServiceBusStepsAwaitingEventManager::<P>::new(cosmos_client);

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
        let new_instance_queue = self.config.new_instance_queue_suffix.clone();
        let next_steps_queue = self.config.next_step_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;

        let new_instance_receiver =
            AzureServiceBusNewInstanceReceiver::<P>::new(service_bus_client, &new_instance_queue)
                .await?;

        let next_step_sender =
            AzureServiceBusNextStepSender::<P>::new(service_bus_client, &next_steps_queue).await?;

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
        let next_steps_queue = self.config.next_step_queue_suffix.clone();
        let active_steps_queue = self.config.active_step_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;

        let next_step_receiver =
            AzureServiceBusNextStepReceiver::<P>::new(service_bus_client, next_steps_queue).await?;

        let active_step_sender =
            AzureServiceBusActiveStepSender::<P>::new(service_bus_client, &active_steps_queue)
                .await?;

        let cosmos_client = self.cosmos_client();

        let steps_awaiting_event_manager =
            AzureServiceBusStepsAwaitingEventManager::<P>::new(cosmos_client);

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
        let new_event_queue_suffix = self.config.new_event_queue_suffix.clone();
        let new_instance_queue_suffix = self.config.new_instance_queue_suffix.clone();
        let service_bus_client = self.service_bus_client().await;
        let event_sender =
            AzureServiceBusEventSender::<P>::new(service_bus_client, &new_event_queue_suffix)
                .await?;
        let new_instance_sender = AzureServiceBusNewInstanceSender::<P>::new(
            service_bus_client,
            &new_instance_queue_suffix,
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
