mod step_0;
mod step_1;

use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug, time::Duration};
use step_0::Step0;
use step_1::Step1;

use crate::{
    ActiveStepQueue, DelayedStepQueue, InstanceId, WaitingForEventStepQueue,
    event::{Event, Immediate, WorkflowEvent},
};

pub trait Step: Serialize + for<'a> Deserialize<'a> + Debug
where
    WorkflowStep: From<Self>,
{
    type Event: Event;
    async fn run_raw(
        &self,
        event: Option<WorkflowEvent>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<StepWithSettings<WorkflowStep>>, StepError>;
    async fn enqueue(
        step: FullyQualifiedStep<Self>,
        active_step_queue: &ActiveStepQueue,
        waiting_for_step_queue: &WaitingForEventStepQueue,
        delayed_step_queue: &DelayedStepQueue,
    ) -> anyhow::Result<()>
    where
        Self::Event: 'static,
    {
        if TypeId::of::<Self::Event>() != TypeId::of::<Immediate>() && step.event.is_none() {
            // If the next step requires an event, enqueue it in the waiting for event queue
            waiting_for_step_queue
                .enqueue(FullyQualifiedStep::<WorkflowStep> {
                    event: step.event,
                    instance_id: step.instance_id,
                    step: StepWithSettings::<WorkflowStep> {
                        step: step.step.step.into(),
                        settings: step.step.settings,
                    },
                    retry_count: step.retry_count,
                })
                .await?;
            return Ok(());
        } else {
            active_step_queue
                .enqueue(FullyQualifiedStep::<WorkflowStep> {
                    event: step.event,
                    instance_id: step.instance_id,
                    step: StepWithSettings::<WorkflowStep> {
                        step: step.step.step.into(),
                        settings: step.step.settings,
                    },
                    retry_count: step.retry_count,
                })
                .await?;
        }
        Ok(())
    }
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
    ) -> Result<Option<StepWithSettings<Self>>, StepError> {
        match self {
            WorkflowStep::Step0(step) => Step::run_raw(step, event).await,
            WorkflowStep::Step1(step) => Step::run_raw(step, event).await,
        }
    }

    // async fn enqueue(
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StepError {
    #[error("Unknown step error")]
    Unknown,
}

pub(crate) struct StepWithSettings<S: Step>
where
    WorkflowStep: From<S>,
{
    pub step: S,
    pub settings: StepSettings,
}

#[derive(Debug)]
pub(crate) struct StepSettings {
    pub max_retry_count: u32,
    pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

pub(crate) struct FullyQualifiedStep<S: Step>
where
    WorkflowStep: From<S>,
{
    pub instance_id: InstanceId,
    pub step: StepWithSettings<S>,
    pub event: Option<WorkflowEvent>,
    pub retry_count: u32,
}
