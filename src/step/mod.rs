mod step_0;
mod step_1;

use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug, time::Duration};
use step_0::Step0;
use step_1::Step1;

use crate::{
    ActiveStepQueue, DelayedStepQueue, InstanceId, WaitingForEventStepQueue,
    event::{Event, WorkflowEvent},
};

pub trait Step: Serialize + for<'a> Deserialize<'a> + Debug {
    type Event: Event;
    async fn run_raw(
        &self,
        event: Option<WorkflowEvent>,
    ) -> Result<Option<StepWithSettings>, StepError>;
    // async fn enqueue(
    //     self,
    //     instance_id: InstanceId,
    //     settings: StepSettings,
    //     active_step_queue: &ActiveStepQueue,
    //     waiting_for_step_queue: &WaitingForEventStepQueue,
    //     delayed_step_queue: &DelayedStepQueue,
    // ) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, From, TryInto)]
pub(crate) enum WorkflowStep {
    Step0(Step0),
    Step1(Step1),
}
impl WorkflowStep {
    /// Returns the TypeId of the event type associated with this step.
    pub fn variant_event_type_id(&self) -> TypeId {
        match self {
            WorkflowStep::Step0(_) => TypeId::of::<<Step0 as Step>::Event>(),
            WorkflowStep::Step1(_) => TypeId::of::<<Step1 as Step>::Event>(),
        }
    }
}

impl Step for WorkflowStep {
    type Event = WorkflowEvent;
    async fn run_raw(
        &self,
        event: Option<WorkflowEvent>,
    ) -> Result<Option<StepWithSettings>, StepError> {
        match self {
            WorkflowStep::Step0(step) => Step::run_raw(step, event).await,
            WorkflowStep::Step1(step) => Step::run_raw(step, event).await,
        }
    }

    // async fn enqueue(
    //     self,
    //     instance_id: InstanceId,
    //     settings: StepSettings,
    //     active_step_queue: &ActiveStepQueue,
    //     waiting_for_step_queue: &WaitingForEventStepQueue,
    //     delayed_step_queue: &DelayedStepQueue,
    // ) -> anyhow::Result<()> {
    //     match self {
    //         WorkflowStep::Step0(step) => {
    //             Step::enqueue(
    //                 step,
    //                 instance_id,
    //                 settings,
    //                 active_step_queue,
    //                 waiting_for_step_queue,
    //                 delayed_step_queue,
    //             )
    //             .await
    //         }
    //         WorkflowStep::Step1(step) => {
    //             Step::enqueue(
    //                 step,
    //                 instance_id,
    //                 settings,
    //                 active_step_queue,
    //                 waiting_for_step_queue,
    //                 delayed_step_queue,
    //             )
    //             .await
    //         }
    //     }
    // }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StepError {
    #[error("Unknown step error")]
    Unknown,
}

pub(crate) struct StepWithSettings {
    pub step: WorkflowStep,
    pub settings: StepSettings,
}

#[derive(Debug)]
pub(crate) struct StepSettings {
    pub max_retry_count: u32,
    pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

pub(crate) struct FullyQualifiedStep {
    pub step: StepWithSettings,
    pub event: Option<WorkflowEvent>,
    pub retry_count: u32,
}
