use std::sync::Arc;

use async_channel::{Receiver, Sender};
use papaya::HashMap;
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
use surgeflow_types::{
    FullyQualifiedStep, InstanceEvent, Project, WorkflowInstance, WorkflowInstanceId,
};

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
    /// The connection string for the PostgreSQL database
    /// This is used for persistent step management
    pub pg_connection_string: String,
}

#[derive(derive_more::Debug)]
pub struct AwsDependencyManager<P: Project> {
    steps_awaiting_event_map: Arc<HashMap<WorkflowInstanceId, FullyQualifiedStep<P>>>,
    next_step_channel: Option<(
        Sender<FullyQualifiedStep<P>>,
        Receiver<FullyQualifiedStep<P>>,
    )>,
    active_step_channel: Option<(
        Sender<FullyQualifiedStep<P>>,
        Receiver<FullyQualifiedStep<P>>,
    )>,
    failed_step_channel: Option<(
        Sender<FullyQualifiedStep<P>>,
        Receiver<FullyQualifiedStep<P>>,
    )>,
    completed_step_channel: Option<(
        Sender<FullyQualifiedStep<P>>,
        Receiver<FullyQualifiedStep<P>>,
    )>,
    //
    new_instance_channel: Option<(Sender<WorkflowInstance>, Receiver<WorkflowInstance>)>,
    completed_instance_channel: Option<(Sender<WorkflowInstance>, Receiver<WorkflowInstance>)>,
    failed_instance_channel: Option<(Sender<WorkflowInstance>, Receiver<WorkflowInstance>)>,
    //
    new_event_channel: Option<(Sender<InstanceEvent<P>>, Receiver<InstanceEvent<P>>)>,
    //
    sqlx_pool: Option<PgPool>,
    config: AwsAdapterConfig,
}

impl<P: Project> AwsDependencyManager<P> {
    pub fn new(config: AwsAdapterConfig) -> Self {
        Self {
            steps_awaiting_event_map: Arc::new(HashMap::new()),
            sqlx_pool: None,
            config,
            next_step_channel: None,
            active_step_channel: None,
            failed_step_channel: None,
            completed_step_channel: None,
            new_instance_channel: None,
            completed_instance_channel: None,
            failed_instance_channel: None,
            new_event_channel: None,
        }
    }
}

impl<P: Project> AwsDependencyManager<P> {
    fn steps_awaiting_event_map(&self) -> Arc<HashMap<WorkflowInstanceId, FullyQualifiedStep<P>>> {
        self.steps_awaiting_event_map.clone()
    }
    fn next_step_sender(&mut self) -> &Sender<FullyQualifiedStep<P>> {
        if self.next_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.next_step_channel = Some((sender, receiver));
        }
        &self.next_step_channel.as_ref().unwrap().0
    }
    fn next_step_receiver(&mut self) -> &Receiver<FullyQualifiedStep<P>> {
        if self.next_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.next_step_channel = Some((sender, receiver));
        }
        &self.next_step_channel.as_ref().unwrap().1
    }
    fn active_step_sender(&mut self) -> &Sender<FullyQualifiedStep<P>> {
        if self.active_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.active_step_channel = Some((sender, receiver));
        }
        &self.active_step_channel.as_ref().unwrap().0
    }
    fn active_step_receiver(&mut self) -> &Receiver<FullyQualifiedStep<P>> {
        if self.active_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.active_step_channel = Some((sender, receiver));
        }
        &self.active_step_channel.as_ref().unwrap().1
    }
    fn failed_step_sender(&mut self) -> &Sender<FullyQualifiedStep<P>> {
        if self.failed_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.failed_step_channel = Some((sender, receiver));
        }
        &self.failed_step_channel.as_ref().unwrap().0
    }
    fn failed_step_receiver(&mut self) -> &Receiver<FullyQualifiedStep<P>> {
        if self.failed_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.failed_step_channel = Some((sender, receiver));
        }
        &self.failed_step_channel.as_ref().unwrap().1
    }
    fn completed_step_sender(&mut self) -> &Sender<FullyQualifiedStep<P>> {
        if self.completed_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.completed_step_channel = Some((sender, receiver));
        }
        &self.completed_step_channel.as_ref().unwrap().0
    }
    fn completed_step_receiver(&mut self) -> &Receiver<FullyQualifiedStep<P>> {
        if self.completed_step_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.completed_step_channel = Some((sender, receiver));
        }
        &self.completed_step_channel.as_ref().unwrap().1
    }
    fn new_instance_sender(&mut self) -> &Sender<WorkflowInstance> {
        if self.new_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.new_instance_channel = Some((sender, receiver));
        }
        &self.new_instance_channel.as_ref().unwrap().0
    }
    fn new_instance_receiver(&mut self) -> &Receiver<WorkflowInstance> {
        if self.new_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.new_instance_channel = Some((sender, receiver));
        }
        &self.new_instance_channel.as_ref().unwrap().1
    }
    fn completed_instance_sender(&mut self) -> &Sender<WorkflowInstance> {
        if self.completed_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.completed_instance_channel = Some((sender, receiver));
        }
        &self.completed_instance_channel.as_ref().unwrap().0
    }
    fn completed_instance_receiver(&mut self) -> &Receiver<WorkflowInstance> {
        if self.completed_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.completed_instance_channel = Some((sender, receiver));
        }
        &self.completed_instance_channel.as_ref().unwrap().1
    }
    fn failed_instance_sender(&mut self) -> &Sender<WorkflowInstance> {
        if self.failed_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.failed_instance_channel = Some((sender, receiver));
        }
        &self.failed_instance_channel.as_ref().unwrap().0
    }
    fn failed_instance_receiver(&mut self) -> &Receiver<WorkflowInstance> {
        if self.failed_instance_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.failed_instance_channel = Some((sender, receiver));
        }
        &self.failed_instance_channel.as_ref().unwrap().1
    }
    fn new_event_sender(&mut self) -> &Sender<InstanceEvent<P>> {
        if self.new_event_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.new_event_channel = Some((sender, receiver));
        }
        &self.new_event_channel.as_ref().unwrap().0
    }
    fn new_event_receiver(&mut self) -> &Receiver<InstanceEvent<P>> {
        if self.new_event_channel.is_none() {
            let (sender, receiver) = async_channel::unbounded();
            self.new_event_channel = Some((sender, receiver));
        }
        &self.new_event_channel.as_ref().unwrap().1
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
        let completed_instance_receiver =
            AwsSqsCompletedInstanceReceiver::new(self.completed_instance_receiver().clone());

        Ok(CompletedInstanceWorkerDependencies::new(
            completed_instance_receiver,
        ))
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
        let completed_step_receiver =
            AwsSqsCompletedStepReceiver::<P>::new(self.completed_step_receiver().clone());

        let next_step_sender = AwsSqsNextStepSender::<P>::new(self.next_step_sender().clone());

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
        let active_step_receiver =
            AwsSqsActiveStepReceiver::<P>::new(self.active_step_receiver().clone());

        let active_step_sender =
            AwsSqsActiveStepSender::<P>::new(self.active_step_sender().clone());

        let failed_step_sender =
            AwsSqsFailedStepSender::<P>::new(self.failed_step_sender().clone());

        let completed_step_sender =
            AwsSqsCompletedStepSender::<P>::new(self.completed_step_sender().clone());

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
        let failed_instance_receiver =
            AwsSqsFailedInstanceReceiver::<P>::new(self.failed_instance_receiver().clone());

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
        let failed_step_receiver =
            AwsSqsFailedStepReceiver::<P>::new(self.failed_step_receiver().clone());

        let failed_instance_sender =
            AwsSqsFailedInstanceSender::<P>::new(self.failed_instance_sender().clone());

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
        let steps_awaiting_event_map = self.steps_awaiting_event_map().clone();
        let new_event_receiver = AwsSqsEventReceiver::<P>::new(self.new_event_receiver().clone());
        let active_step_sender =
            AwsSqsActiveStepSender::<P>::new(self.active_step_sender().clone());

        let steps_awaiting_event_manager =
            AwsSqsStepsAwaitingEventManager::<P>::new(steps_awaiting_event_map);

        Ok(NewEventWorkerDependencies::new(
            active_step_sender,
            new_event_receiver,
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
        let new_instance_receiver =
            AwsSqsNewInstanceReceiver::<P>::new(self.new_instance_receiver().clone());

        let next_step_sender = AwsSqsNextStepSender::<P>::new(self.next_step_sender().clone());

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
        let steps_awaiting_event_map = self.steps_awaiting_event_map().clone();

        let next_step_receiver =
            AwsSqsNextStepReceiver::<P>::new(self.next_step_receiver().clone());

        let active_step_sender =
            AwsSqsActiveStepSender::<P>::new(self.active_step_sender().clone());

        let steps_awaiting_event_manager =
            AwsSqsStepsAwaitingEventManager::<P>::new(steps_awaiting_event_map);

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
        let new_event_sender = AwsSqsEventSender::<P>::new(self.new_event_sender().clone());

        let new_instance_sender =
            AwsSqsNewInstanceSender::<P>::new(self.new_instance_sender().clone());

        Ok(ControlServerDependencies::new(
            new_event_sender,
            new_instance_sender,
        ))
    }
}

impl<P: Project> DependencyManager<P> for AwsDependencyManager<P> {
    type Error = AwsAdapterError<P>;
}
