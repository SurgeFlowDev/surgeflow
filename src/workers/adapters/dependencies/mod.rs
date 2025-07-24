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
    workflows::Project,
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

pub trait ActiveStepWorkerDependencyProvider<P: Project> {
    type ActiveStepReceiver: ActiveStepReceiver<P>;
    type ActiveStepSender: ActiveStepSender<P>;
    type FailedStepSender: FailedStepSender<P>;
    type CompletedStepSender: CompletedStepSender<P>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn active_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            ActiveStepWorkerDependencies<
                P,
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

pub trait CompletedInstanceWorkerDependencyProvider<P: Project> {
    type CompletedInstanceReceiver: CompletedInstanceReceiver<P>;
    type Error: Send + Sync + 'static;

    fn completed_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedInstanceWorkerDependencies<P, Self::CompletedInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait CompletedStepWorkerDependencyProvider<P: Project> {
    type CompletedStepReceiver: CompletedStepReceiver<P>;
    type NextStepSender: NextStepSender<P>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn completed_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedStepWorkerDependencies<
                P,
                Self::CompletedStepReceiver,
                Self::NextStepSender,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait FailedInstanceWorkerDependencyProvider<P: Project> {
    type FailedInstanceReceiver: FailedInstanceReceiver<P>;
    type Error: Send + Sync + 'static;

    fn failed_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            FailedInstanceWorkerDependencies<P, Self::FailedInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait FailedStepWorkerDependencyProvider<P: Project> {
    type FailedStepReceiver: FailedStepReceiver<P>;
    type FailedInstanceSender: FailedInstanceSender<P>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn failed_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            FailedStepWorkerDependencies<
                P,
                Self::FailedStepReceiver,
                Self::FailedInstanceSender,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait NewEventWorkerDependencyProvider<P: Project> {
    type ActiveStepSender: ActiveStepSender<P>;
    type EventReceiver: EventReceiver<P>;
    type StepsAwaitingEventManager: StepsAwaitingEventManager<P>;
    type Error: Send + Sync + 'static;

    fn new_event_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewEventWorkerDependencies<
                P,
                Self::ActiveStepSender,
                Self::EventReceiver,
                Self::StepsAwaitingEventManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait NewInstanceWorkerDependencyProvider<P: Project> {
    type NextStepSender: NextStepSender<P>;
    type NewInstanceReceiver: NewInstanceReceiver<P>;
    type Error: Send + Sync + 'static;

    fn new_instance_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewInstanceWorkerDependencies<P, Self::NextStepSender, Self::NewInstanceReceiver>,
            Self::Error,
        >,
    > + Send;
}

pub trait NextStepWorkerDependencyProvider<P: Project> {
    type NextStepReceiver: NextStepReceiver<P>;
    type ActiveStepSender: ActiveStepSender<P>;
    type StepsAwaitingEventManager: StepsAwaitingEventManager<P>;
    type PersistentStepManager: PersistentStepManager;
    type Error: Send + Sync + 'static;

    fn next_step_worker_dependencies(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NextStepWorkerDependencies<
                P,
                Self::NextStepReceiver,
                Self::ActiveStepSender,
                Self::StepsAwaitingEventManager,
                Self::PersistentStepManager,
            >,
            Self::Error,
        >,
    > + Send;
}

pub trait DependencyManager<P: Project>:
    Sized
    + ActiveStepWorkerDependencyProvider<P>
    + CompletedInstanceWorkerDependencyProvider<P>
    + CompletedStepWorkerDependencyProvider<P>
    + FailedInstanceWorkerDependencyProvider<P>
    + FailedStepWorkerDependencyProvider<P>
    + NewEventWorkerDependencyProvider<P>
    + NewInstanceWorkerDependencyProvider<P>
    + NextStepWorkerDependencyProvider<P>
{
    type Error: Send + Sync + 'static;
}
