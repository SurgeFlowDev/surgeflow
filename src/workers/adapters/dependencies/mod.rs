use crate::{
    workers::adapters::{
        dependencies::{
            active_step_worker::ActiveStepWorkerDependencies,
            completed_instance_worker::CompletedInstanceWorkerDependencies,
            completed_step_worker::CompletedStepWorkerDependencies,
            failed_instance_worker::FailedInstanceWorkerDependencies,
            failed_step_worker::FailedStepWorkerDependencies,
            new_event_worker::NewEventWorkerDependencies,
            new_instance_worker::NewInstanceWorkerDependencies,
            next_step_worker::NextStepWorkerDependencies,
        },
        managers::{PersistentStepManager, StepsAwaitingEventManager},
        receivers::{
            ActiveStepReceiver, CompletedInstanceReceiver, CompletedStepReceiver, EventReceiver,
            FailedInstanceReceiver, FailedStepReceiver, NewInstanceReceiver, NextStepReceiver,
        },
        senders::{
            ActiveStepSender, CompletedStepSender, FailedInstanceSender, FailedStepSender,
            NextStepSender,
        },
    },
    workflows::Workflow,
};

pub mod control_server;

pub mod active_step_worker;
pub mod completed_instance_worker;
pub mod completed_step_worker;
pub mod failed_instance_worker;
pub mod failed_step_worker;
pub mod new_event_worker;
pub mod new_instance_worker;
pub mod next_step_worker;

pub trait ActiveStepWorkerDependencyProvider<W: Workflow> {
    type ActiveStepReceiver: ActiveStepReceiver<W>;
    type ActiveStepSender: ActiveStepSender<W>;
    type FailedStepSender: FailedStepSender<W>;
    type CompletedStepSender: CompletedStepSender<W>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn active_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            ActiveStepWorkerDependencies<
                W,
                Self::ActiveStepReceiver,
                Self::ActiveStepSender,
                Self::FailedStepSender,
                Self::CompletedStepSender,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait CompletedInstanceWorkerDependencyProvider<W: Workflow> {
    type CompletedInstanceReceiver: CompletedInstanceReceiver<W>;
    type Error: Send + Sync + 'static;

    fn completed_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedInstanceWorkerDependencies<W, Self::CompletedInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait CompletedStepWorkerDependencyProvider<W: Workflow> {
    type CompletedStepReceiver: CompletedStepReceiver<W>;
    type NextStepSender: NextStepSender<W>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn completed_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedStepWorkerDependencies<
                W,
                Self::CompletedStepReceiver,
                Self::NextStepSender,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait FailedInstanceWorkerDependencyProvider<W: Workflow> {
    type FailedInstanceReceiver: FailedInstanceReceiver<W>;
    type Error: Send + Sync + 'static;

    fn failed_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            FailedInstanceWorkerDependencies<W, Self::FailedInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait FailedStepWorkerDependencyProvider<W: Workflow> {
    type FailedStepReceiver: FailedStepReceiver<W>;
    type FailedInstanceSender: FailedInstanceSender<W>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn failed_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            FailedStepWorkerDependencies<
                W,
                Self::FailedStepReceiver,
                Self::FailedInstanceSender,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait NewEventWorkerDependencyProvider<W: Workflow> {
    type ActiveStepSender: ActiveStepSender<W>;
    type EventReceiver: EventReceiver<W>;
    type StepsAwaitingEventManager: StepsAwaitingEventManager<W>;
    type Error: Send + Sync + 'static;

    fn new_event_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewEventWorkerDependencies<
                W,
                Self::ActiveStepSender,
                Self::EventReceiver,
                Self::StepsAwaitingEventManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait NewInstanceWorkerDependencyProvider<W: Workflow> {
    type NextStepSender: NextStepSender<W>;
    type NewInstanceReceiver: NewInstanceReceiver<W>;
    type Error: Send + Sync + 'static;

    fn new_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewInstanceWorkerDependencies<W, Self::NextStepSender, Self::NewInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait NextStepWorkerDependencyProvider<W: Workflow> {
    type NextStepReceiver: NextStepReceiver<W>;
    type ActiveStepSender: ActiveStepSender<W>;
    type StepsAwaitingEventManager: StepsAwaitingEventManager<W>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn next_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NextStepWorkerDependencies<
                W,
                Self::NextStepReceiver,
                Self::ActiveStepSender,
                Self::StepsAwaitingEventManager,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait DependencyManager<W: Workflow>:
    Sized
    + ActiveStepWorkerDependencyProvider<W>
    + CompletedInstanceWorkerDependencyProvider<W>
    + CompletedStepWorkerDependencyProvider<W>
    + FailedInstanceWorkerDependencyProvider<W>
    + FailedStepWorkerDependencyProvider<W>
    + NewEventWorkerDependencyProvider<W>
    + NewInstanceWorkerDependencyProvider<W>
    + NextStepWorkerDependencyProvider<W>
{
    type Error: Send + Sync + 'static;
}
