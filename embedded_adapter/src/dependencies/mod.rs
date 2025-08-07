use async_channel::{Receiver, Sender};
// TODO: review this file. a senders/receivers/managers are receiveing &str queue names when they should receive String to avoid one extra allocation
use aws_config::SdkConfig;
use aws_credential_types::Credentials;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_sqs::{Client as SqsClient, config::SharedCredentialsProvider};
use aws_types::region::Region;
use serde::Deserialize;
use sqlx::PgPool;

use adapter_types::dependencies::{
    ActiveStepWorkerDependencyProvider, CompletedInstanceWorkerDependencyProvider,
    CompletedStepWorkerDependencyProvider, ControlServerDependencyProvider, DependencyManager,
    FailedInstanceWorkerDependencyProvider, FailedStepWorkerDependencyProvider,
    NewEventWorkerDependencyProvider, NewInstanceWorkerDependencyProvider,
    NextStepWorkerDependencyProvider, active_step_worker::ActiveStepWorkerDependencies,
    completed_instance_worker::CompletedInstanceWorkerDependencies,
    completed_step_worker::CompletedStepWorkerDependencies,
    control_server::ControlServerDependencies,
    failed_instance_worker::FailedInstanceWorkerDependencies,
    failed_step_worker::FailedStepWorkerDependencies, new_event_worker::NewEventWorkerDependencies,
    new_instance_worker::NewInstanceWorkerDependencies,
    next_step_worker::NextStepWorkerDependencies,
};
use surgeflow_types::{FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance};

use super::{
    AwsAdapterError,
    managers::{AwsPersistenceManager, AwsSqsStepsAwaitingEventManager},
    receivers::{
        AwsSqsActiveStepReceiver, AwsSqsCompletedInstanceReceiver, AwsSqsCompletedStepReceiver,
        AwsSqsEventReceiver, AwsSqsFailedInstanceReceiver, AwsSqsFailedStepReceiver,
        AwsSqsNewInstanceReceiver, AwsSqsNextStepReceiver,
    },
    senders::{
        AwsSqsActiveStepSender, AwsSqsCompletedStepSender, AwsSqsEventSender,
        AwsSqsFailedInstanceSender, AwsSqsFailedStepSender, AwsSqsNewInstanceSender,
        AwsSqsNextStepSender,
    },
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AwsAdapterConfig {
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
pub struct AwsDependencyManager<P: Project> {
    sqs_client: Option<SqsClient>,
    //
    step_channel: Option<(
        Sender<FullyQualifiedStep<P>>,
        Receiver<FullyQualifiedStep<P>>,
    )>,
    //
    instance_event_channel: Option<(Sender<InstanceEvent<P>>, Receiver<InstanceEvent<P>>)>,
    //
    workflow_instance_channel: Option<(Sender<WorkflowInstance>, Receiver<WorkflowInstance>)>,
    //
    dynamo_client: Option<DynamoClient>,
    sqlx_pool: Option<PgPool>,
    config: AwsAdapterConfig,
    sdk_config: Option<SdkConfig>,
}

impl<P: Project> AwsDependencyManager<P> {
    pub fn new(config: AwsAdapterConfig) -> Self {
        Self {
            sqs_client: None,
            dynamo_client: None,
            sqlx_pool: None,
            config,
            sdk_config: None,
            step_channel: None,
            instance_event_channel: None,
            workflow_instance_channel: None,
        }
    }
}

impl<P: Project> AwsDependencyManager<P> {
    fn step_sender(&mut self) -> &Sender<FullyQualifiedStep<P>> {
        if self.step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.step_channel = Some((sender, receiver));
        }
        &self.step_channel.as_ref().unwrap().0
    }
    fn step_receiver(&mut self) -> &Receiver<FullyQualifiedStep<P>> {
        if self.step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.step_channel = Some((sender, receiver));
        }
        &self.step_channel.as_ref().unwrap().1
    }
    fn instance_event_sender(&mut self) -> &Sender<InstanceEvent<P>> {
        if self.instance_event_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.instance_event_channel = Some((sender, receiver));
        }
        &self.instance_event_channel.as_ref().unwrap().0
    }
    fn instance_event_receiver(&mut self) -> &Receiver<InstanceEvent<P>> {
        if self.instance_event_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.instance_event_channel = Some((sender, receiver));
        }
        &self.instance_event_channel.as_ref().unwrap().1
    }
    fn workflow_instance_sender(&mut self) -> &Sender<WorkflowInstance> {
        if self.workflow_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.workflow_instance_channel = Some((sender, receiver));
        }
        &self.workflow_instance_channel.as_ref().unwrap().0
    }
    fn workflow_instance_receiver(&mut self) -> &Receiver<WorkflowInstance> {
        if self.workflow_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.workflow_instance_channel = Some((sender, receiver));
        }
        &self.workflow_instance_channel.as_ref().unwrap().1
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

        self.sdk_config
            .as_ref()
            .expect("sdk_config is set in this function. this should never happen")
    }
    fn sqs_client(&mut self) -> &SqsClient {
        if self.sqs_client.is_none() {
            let config = self.get_sdk_config();
            self.sqs_client = Some(SqsClient::new(config));
        }
        self.sqs_client.as_ref().unwrap()
    }

    fn dynamo_client(&mut self) -> &DynamoClient {
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

impl<P: Project> CompletedInstanceWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type CompletedInstanceReceiver = AwsSqsCompletedInstanceReceiver<P>;
    type Error = AwsAdapterError<P>;

    async fn completed_instance_worker_dependencies(
        &mut self,
    ) -> Result<CompletedInstanceWorkerDependencies<P, Self::CompletedInstanceReceiver>, Self::Error>
    {
        let sqs_client = self.sqs_client();

        let instance_receiver = AwsSqsCompletedInstanceReceiver::new(
            sqs_client.clone(),
            self.config.completed_instance_queue_url.clone(),
        );

        Ok(CompletedInstanceWorkerDependencies::new(instance_receiver))
    }
}

impl<P: Project> CompletedStepWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type CompletedStepReceiver = AwsSqsCompletedStepReceiver<P>;
    type NextStepSender = AwsSqsNextStepSender<P>;
    type PersistenceManager = AwsPersistenceManager;
    type Error = AwsAdapterError<P>;

    async fn completed_step_worker_dependencies(
        &mut self,
    ) -> Result<
        CompletedStepWorkerDependencies<
            P,
            Self::CompletedStepReceiver,
            Self::NextStepSender,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().clone();
        let step_sender = self.step_sender().clone();


        let completed_step_receiver = AwsSqsCompletedStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.completed_step_queue_url.clone(),
        );

        let next_step_sender =
            AwsSqsNextStepSender::<P>::new(step_sender, self.config.next_step_queue_url.clone());

        let persistence_manager = AwsPersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(CompletedStepWorkerDependencies::new(
            completed_step_receiver,
            next_step_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> ActiveStepWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type ActiveStepReceiver = AwsSqsActiveStepReceiver<P>;
    type ActiveStepSender = AwsSqsActiveStepSender<P>;
    type FailedStepSender = AwsSqsFailedStepSender<P>;
    type CompletedStepSender = AwsSqsCompletedStepSender<P>;
    type PersistenceManager = AwsPersistenceManager;
    type Error = AwsAdapterError<P>;

    async fn active_step_worker_dependencies(
        &mut self,
    ) -> Result<
        ActiveStepWorkerDependencies<
            P,
            Self::ActiveStepReceiver,
            Self::ActiveStepSender,
            Self::FailedStepSender,
            Self::CompletedStepSender,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().clone();
        let step_sender = self.step_sender().clone();

        let active_step_receiver = AwsSqsActiveStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.active_step_queue_url.clone(),
        );

        let active_step_sender = AwsSqsActiveStepSender::<P>::new(
            step_sender.clone(),
            self.config.active_step_queue_url.clone(),
        );

        let failed_step_sender = AwsSqsFailedStepSender::<P>::new(
            step_sender.clone(),
            self.config.failed_step_queue_url.clone(),
        );

        let completed_step_sender = AwsSqsCompletedStepSender::<P>::new(
            step_sender,
            self.config.completed_step_queue_url.clone(),
        );

        let persistence_manager = AwsPersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(ActiveStepWorkerDependencies::new(
            active_step_receiver,
            active_step_sender,
            failed_step_sender,
            completed_step_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> FailedInstanceWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type FailedInstanceReceiver = AwsSqsFailedInstanceReceiver<P>;
    type Error = AwsAdapterError<P>;

    async fn failed_instance_worker_dependencies(
        &mut self,
    ) -> Result<FailedInstanceWorkerDependencies<P, Self::FailedInstanceReceiver>, Self::Error>
    {
        let sqs_client = self.sqs_client().clone();

        let failed_instance_receiver = AwsSqsFailedInstanceReceiver::<P>::new(
            sqs_client,
            self.config.failed_instance_queue_url.clone(),
        );

        Ok(FailedInstanceWorkerDependencies::new(
            failed_instance_receiver,
        ))
    }
}

impl<P: Project> FailedStepWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type FailedStepReceiver = AwsSqsFailedStepReceiver<P>;
    type FailedInstanceSender = AwsSqsFailedInstanceSender<P>;
    type PersistenceManager = AwsPersistenceManager;
    type Error = AwsAdapterError<P>;

    async fn failed_step_worker_dependencies(
        &mut self,
    ) -> Result<
        FailedStepWorkerDependencies<
            P,
            Self::FailedStepReceiver,
            Self::FailedInstanceSender,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().clone();
        let workflow_instance_sender = self.workflow_instance_sender().clone();

        let failed_step_receiver = AwsSqsFailedStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.failed_step_queue_url.clone(),
        );

        let failed_instance_sender = AwsSqsFailedInstanceSender::<P>::new(
            workflow_instance_sender,
            self.config.failed_instance_queue_url.clone(),
        );

        let persistence_manager = AwsPersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(FailedStepWorkerDependencies::new(
            failed_step_receiver,
            failed_instance_sender,
            persistence_manager,
        ))
    }
}

impl<P: Project> NewEventWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type ActiveStepSender = AwsSqsActiveStepSender<P>;
    type EventReceiver = AwsSqsEventReceiver<P>;
    type StepsAwaitingEventManager = AwsSqsStepsAwaitingEventManager<P>;
    type Error = AwsAdapterError<P>;

    async fn new_event_worker_dependencies(
        &mut self,
    ) -> Result<
        NewEventWorkerDependencies<
            P,
            Self::ActiveStepSender,
            Self::EventReceiver,
            Self::StepsAwaitingEventManager,
        >,
        Self::Error,
    > {
        let step_sender = self.step_sender().clone();
        let sqs_client = self.sqs_client().clone();
        let dynamo_client = self.dynamo_client().clone();

        let event_receiver = AwsSqsEventReceiver::<P>::new(
            sqs_client.clone(),
            self.config.new_event_queue_url.clone(),
        );
        let active_step_sender =
            AwsSqsActiveStepSender::<P>::new(step_sender, self.config.active_step_queue_url.clone());

        let steps_awaiting_event_manager = AwsSqsStepsAwaitingEventManager::<P>::new(
            dynamo_client,
            self.config.dynamodb_table_name.clone(),
        );

        Ok(NewEventWorkerDependencies::new(
            active_step_sender,
            event_receiver,
            steps_awaiting_event_manager,
        ))
    }
}

impl<P: Project> NewInstanceWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type NextStepSender = AwsSqsNextStepSender<P>;
    type NewInstanceReceiver = AwsSqsNewInstanceReceiver<P>;
    type PersistenceManager = AwsPersistenceManager;
    type Error = AwsAdapterError<P>;

    async fn new_instance_worker_dependencies(
        &mut self,
    ) -> Result<
        NewInstanceWorkerDependencies<
            P,
            Self::NextStepSender,
            Self::NewInstanceReceiver,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let sqs_client = self.sqs_client().clone();
        let step_sender = self.step_sender().clone();

        let new_instance_receiver = AwsSqsNewInstanceReceiver::<P>::new(
            sqs_client.clone(),
            self.config.new_instance_queue_url.clone(),
        );

        let next_step_sender =
            AwsSqsNextStepSender::<P>::new(step_sender, self.config.next_step_queue_url.clone());

        let persistence_manager = AwsPersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(NewInstanceWorkerDependencies::new(
            next_step_sender,
            new_instance_receiver,
            persistence_manager,
        ))
    }
}

impl<P: Project> NextStepWorkerDependencyProvider<P> for AwsDependencyManager<P> {
    type NextStepReceiver = AwsSqsNextStepReceiver<P>;
    type ActiveStepSender = AwsSqsActiveStepSender<P>;
    type StepsAwaitingEventManager = AwsSqsStepsAwaitingEventManager<P>;
    type PersistenceManager = AwsPersistenceManager;
    type Error = AwsAdapterError<P>;

    async fn next_step_worker_dependencies(
        &mut self,
    ) -> Result<
        NextStepWorkerDependencies<
            P,
            Self::NextStepReceiver,
            Self::ActiveStepSender,
            Self::StepsAwaitingEventManager,
            Self::PersistenceManager,
        >,
        Self::Error,
    > {
        let step_sender = self.step_sender().clone();
        let sqs_client = self.sqs_client().clone();
        let dynamo_client = self.dynamo_client().clone();

        let next_step_receiver = AwsSqsNextStepReceiver::<P>::new(
            sqs_client.clone(),
            self.config.next_step_queue_url.clone(),
        );

        let active_step_sender =
            AwsSqsActiveStepSender::<P>::new(step_sender, self.config.active_step_queue_url.clone());

        let steps_awaiting_event_manager = AwsSqsStepsAwaitingEventManager::<P>::new(
            dynamo_client,
            self.config.dynamodb_table_name.clone(),
        );

        let persistence_manager = AwsPersistenceManager::new(self.sqlx_pool().await.clone());

        Ok(NextStepWorkerDependencies::new(
            next_step_receiver,
            active_step_sender,
            steps_awaiting_event_manager,
            persistence_manager,
        ))
    }
}

impl<P: Project> ControlServerDependencyProvider<P> for AwsDependencyManager<P> {
    type EventSender = AwsSqsEventSender<P>;
    type NewInstanceSender = AwsSqsNewInstanceSender<P>;
    type Error = AwsAdapterError<P>;

    async fn control_server_dependencies(
        &mut self,
    ) -> Result<ControlServerDependencies<P, Self::EventSender, Self::NewInstanceSender>, Self::Error>
    {
        let instance_event_sender = self.instance_event_sender().clone();
        let workflow_instance_sender = self.workflow_instance_sender().clone();

        let event_sender = AwsSqsEventSender::<P>::new(
            instance_event_sender,
            self.config.new_event_queue_url.clone(),
        );

        let new_instance_sender = AwsSqsNewInstanceSender::<P>::new(
            workflow_instance_sender,
            self.config.new_instance_queue_url.clone(),
        );

        Ok(ControlServerDependencies::new(
            event_sender,
            new_instance_sender,
        ))
    }
}

impl<P: Project> DependencyManager<P> for AwsDependencyManager<P> {
    type Error = AwsAdapterError<P>;
}
