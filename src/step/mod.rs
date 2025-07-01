pub mod step_0;
pub mod step_1;

use anyhow::Context;
use derive_more::{From, TryInto};
use fe2o3_amqp::{Receiver, Sender, Session, session::SessionHandle};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug, marker::PhantomData};
use step_0::Step0;
use step_1::Step1;
use tikv_client::RawClient;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    ActiveStepQueue, WaitingForEventStepQueue, Workflow, Workflow0, WorkflowInstanceId,
    event::{Event, Immediate, WorkflowEvent},
};

pub trait Step:
    Serialize
    + for<'a> Deserialize<'a>
    + Debug
    + Into<<Self::Workflow as Workflow>::Step>
    + Send
    + Clone
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
        <Self as Step>::Workflow: 'static,
    {
        if TypeId::of::<Self::Event>() != TypeId::of::<Immediate<Self::Workflow>>()
            && step.event.is_none()
        {
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

#[derive(Debug, Serialize, Deserialize, From, TryInto, Clone)]
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
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("Unknown step error")]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepWithSettings<S: Debug + Step + for<'a> Deserialize<'a>> {
    #[serde(bound = "")]
    pub step: S,
    pub settings: StepSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct StepSettings {
    pub max_retries: u32,
    // pub delay: Option<Duration>,
    // TODO
    // backoff: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FullyQualifiedStep<S: Debug + Step + for<'a> Deserialize<'a>> {
    pub instance_id: WorkflowInstanceId,
    #[serde(bound = "")]
    pub step: StepWithSettings<S>,
    pub event: Option<WorkflowEvent>,
    pub retry_count: u32,
}

#[derive(Clone)]
pub struct StepsAwaitingEventManager<W: Workflow> {
    tikv_client: RawClient,
    _phantom: PhantomData<W>,
}
impl<W: Workflow> StepsAwaitingEventManager<W> {
    pub fn new(tikv_client: RawClient) -> Self {
        Self {
            tikv_client,
            _phantom: PhantomData,
        }
    }
    pub async fn put_step(&self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        let instance_id = step.instance_id;
        let payload = serde_json::to_vec(&step)?;
        self.tikv_client
            .put(format!("instance_{}", instance_id.0), payload)
            .await?;
        Ok(())
    }

    pub async fn get_step(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        let value = self
            .tikv_client
            .get(format!("instance_{}", instance_id.0))
            .await?
            .context("no event")?;
        let data = serde_json::from_slice(&value)?;
        Ok(data)
    }
}

#[derive(Debug)]
pub struct ActiveStepReceiver<W: Workflow>(Mutex<Receiver>, PhantomData<W>);

impl<W: Workflow> ActiveStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(Mutex::new(receiver), PhantomData::default()))
    }
    pub async fn recv(&self) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        let mut receiver = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let msg = receiver.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                receiver.reject(msg, None).await?;
                return Err(err);
            }
        };
        receiver.accept(msg).await?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct ActiveStepSender<W: Workflow>(Mutex<Sender>, PhantomData<W>);

impl<W: Workflow> ActiveStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(Mutex::new(sender), PhantomData::default()))
    }
    pub async fn send(&self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        let mut sender = self.0.lock().await;

        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        sender.send(event).await?;
        Ok(())
    }
}
