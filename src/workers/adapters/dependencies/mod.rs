use std::error::Error;

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
        managers::StepsAwaitingEventManager,
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

pub trait DependencyManager: Sized {
    type Error: /* Error + */ Send + Sync + 'static;

    fn active_step_worker_dependencies<
        W,
        ActiveStepReceiverT,
        ActiveStepSenderT,
        FailedStepSenderT,
        CompletedStepSenderT,
    >(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            ActiveStepWorkerDependencies<
                W,
                ActiveStepReceiverT,
                ActiveStepSenderT,
                FailedStepSenderT,
                CompletedStepSenderT,
            >,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        ActiveStepReceiverT: ActiveStepReceiver<W>,
        ActiveStepSenderT: ActiveStepSender<W>,
        FailedStepSenderT: FailedStepSender<W>,
        CompletedStepSenderT: CompletedStepSender<W>;

    fn completed_instance_worker_dependencies<W, CompletedInstanceReceiverT>(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedInstanceWorkerDependencies<W, CompletedInstanceReceiverT>,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        CompletedInstanceReceiverT: CompletedInstanceReceiver<W>;

    fn completed_step_worker_dependencies<W, CompletedStepReceiverT, NextStepSenderT>(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            CompletedStepWorkerDependencies<W, CompletedStepReceiverT, NextStepSenderT>,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        CompletedStepReceiverT: CompletedStepReceiver<W>,
        NextStepSenderT: NextStepSender<W>;

    fn failed_instance_worker_dependencies<W, FailedInstanceReceiverT>(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<FailedInstanceWorkerDependencies<W, FailedInstanceReceiverT>, Self::Error>,
    > + Send
    where
        W: Workflow,
        FailedInstanceReceiverT: FailedInstanceReceiver<W>;

    fn failed_step_worker_dependencies<W, FailedStepReceiverT, FailedInstanceSenderT>(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            FailedStepWorkerDependencies<W, FailedStepReceiverT, FailedInstanceSenderT>,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        FailedStepReceiverT: FailedStepReceiver<W>,
        FailedInstanceSenderT: FailedInstanceSender<W>;

    fn new_event_worker_dependencies<
        W,
        ActiveStepSenderT,
        EventReceiverT,
        StepsAwaitingEventManagerT,
    >(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewEventWorkerDependencies<
                W,
                ActiveStepSenderT,
                EventReceiverT,
                StepsAwaitingEventManagerT,
            >,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        ActiveStepSenderT: ActiveStepSender<W>,
        EventReceiverT: EventReceiver<W>,
        StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>;

    fn new_instance_worker_dependencies<W, NextStepSenderT, NewInstanceReceiverT>(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NewInstanceWorkerDependencies<W, NextStepSenderT, NewInstanceReceiverT>,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        NextStepSenderT: NextStepSender<W>,
        NewInstanceReceiverT: NewInstanceReceiver<W>;

    fn next_step_worker_dependencies<
        W,
        NextStepReceiverT,
        ActiveStepSenderT,
        StepsAwaitingEventManagerT,
    >(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<
            NextStepWorkerDependencies<
                W,
                NextStepReceiverT,
                ActiveStepSenderT,
                StepsAwaitingEventManagerT,
            >,
            Self::Error,
        >,
    > + Send
    where
        W: Workflow,
        NextStepReceiverT: NextStepReceiver<W>,
        ActiveStepSenderT: ActiveStepSender<W>,
        StepsAwaitingEventManagerT: StepsAwaitingEventManager<W>;
}
