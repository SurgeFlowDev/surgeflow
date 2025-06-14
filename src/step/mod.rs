mod step_0;
mod step_1;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use step_0::Step0;
use step_1::Step1;

use crate::{event::WorkflowEvent, ActiveStepQueue, InstanceId, WaitingForEventStepQueue};

#[enum_dispatch]
pub trait Step: Serialize + for<'a> Deserialize<'a> + Debug {
    async fn run_raw(&self, event: Option<WorkflowEvent>) -> Result<Option<StepWithSettings>, StepError>;
    async fn enqueue(
        self,
        instance_id: InstanceId,
        settings: StepSettings,
        active_step_queue: &ActiveStepQueue,
        waiting_for_step_queue: &WaitingForEventStepQueue,
    ) -> anyhow::Result<()>;
}

#[enum_dispatch(Step)]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum WorkflowStep {
    Step0(Step0),
    Step1(Step1),
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
