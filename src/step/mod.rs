pub mod step_0;
pub mod step_1;

use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug};
use step_0::Step0;
use step_1::Step1;

use crate::{
    ActiveStepQueue, WaitingForEventStepQueue, Workflow, Workflow0, WorkflowInstanceId,
    event::{Event, Immediate, WorkflowEvent},
};

pub trait Step:
    Serialize + for<'a> Deserialize<'a> + Debug + Into<<Self::Workflow as Workflow>::Step>
{
    type Event: Event<Workflow = Self::Workflow>;
    type Workflow: Workflow;
    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<WorkflowEvent>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Step>>, StepError>;
    async fn enqueue(
        step: FullyQualifiedStep<Self>,
        active_step_queue: &mut ActiveStepQueue,
        waiting_for_step_queue: &mut WaitingForEventStepQueue,
        // delayed_step_queue: &DelayedStepQueue,
    ) -> anyhow::Result<()>
    where
        Self::Event: 'static,
    {
        if TypeId::of::<Self::Event>() != TypeId::of::<Immediate>() && step.event.is_none() {
            // If the next step requires an event, enqueue it in the waiting for event queue
            waiting_for_step_queue
                .enqueue(FullyQualifiedStep::<<Self::Workflow as Workflow>::Step> {
                    event: step.event,
                    instance_id: step.instance_id,
                    step: StepWithSettings::<<Self::Workflow as Workflow>::Step> {
                        step: step.step.step.into(),
                        settings: step.step.settings,
                    },
                    retry_count: step.retry_count,
                })
                .await?;
            return Ok(());
        } else {
            active_step_queue
                .enqueue(FullyQualifiedStep::<<Self::Workflow as Workflow>::Step> {
                    event: step.event,
                    instance_id: step.instance_id,
                    step: StepWithSettings::<<Self::Workflow as Workflow>::Step> {
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
pub enum WorkflowStep {
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
    type Workflow = Workflow0;
    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<WorkflowEvent>,
    ) -> Result<Option<StepWithSettings<Self>>, StepError> {
        match self {
            WorkflowStep::Step0(step) => Step::run_raw(step, wf, event).await,
            WorkflowStep::Step1(step) => Step::run_raw(step, wf, event).await,
        }
    }

    // async fn enqueue(
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("Unknown step error")]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StepWithSettings<S: Debug + Step + for<'a> Deserialize<'a>> {
    #[serde(bound = "")]
    pub step: S,
    pub settings: StepSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StepSettings {
    pub max_retry_count: u32,
    // pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullyQualifiedStep<S: Debug + Step + for<'a> Deserialize<'a>> {
    pub instance_id: WorkflowInstanceId,
    #[serde(bound = "")]
    pub step: StepWithSettings<S>,
    pub event: Option<WorkflowEvent>,
    pub retry_count: u32,
}
