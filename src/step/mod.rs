use fe2o3_amqp::{Receiver, Sender, session::SessionHandle};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug, marker::PhantomData};
use uuid::Uuid;

use crate::{
    event::Event,
    workflows::{StepId, Workflow, WorkflowInstanceId},
};

pub type StepResult<W> = Result<Option<StepWithSettings<<W as Workflow>::Step>>, StepError>;

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

    fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Option<<Self::Workflow as Workflow>::Event>,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> impl std::future::Future<
        Output = Result<Option<StepWithSettings<<Self::Workflow as Workflow>::Step>>, StepError>,
    > + Send;
}

pub trait WorkflowStep: Step {
    /// Returns the TypeId of the event type associated with this step.
    fn variant_event_type_id(&self) -> TypeId;
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("Unknown step error")]
    Unknown,
    #[error("couldn't convert event")]
    WrongEventType,
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
    pub step_id: StepId,
    #[serde(bound = "")]
    pub step: StepWithSettings<S>,
    pub event: Option<<S::Workflow as Workflow>::Event>,
    pub retry_count: u32,
}

#[derive(Debug)]
pub struct ActiveStepReceiver<W: Workflow>(Receiver, PhantomData<W>);

impl<W: Workflow> ActiveStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
    pub async fn recv(&mut self) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        self.0.accept(msg).await?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct ActiveStepSender<W: Workflow>(Sender, PhantomData<W>);

impl<W: Workflow> ActiveStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-active-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
    pub async fn send(&mut self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FailedStepReceiver<W: Workflow>(Receiver, PhantomData<W>);

impl<W: Workflow> FailedStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-failed-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
    pub async fn recv(&mut self) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        self.0.accept(msg).await?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct FailedStepSender<W: Workflow>(Sender, PhantomData<W>);

impl<W: Workflow> FailedStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-failed-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
    pub async fn send(&mut self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SucceededStepReceiver<W: Workflow>(Receiver, PhantomData<W>);

impl<W: Workflow> SucceededStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-succeeded-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
    pub async fn recv(&mut self) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        self.0.accept(msg).await?;

        Ok(event)
    }
}

#[derive(Debug)]
pub struct SucceededStepSender<W: Workflow>(Sender, PhantomData<W>);

impl<W: Workflow> SucceededStepSender<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-succeeded-steps", W::NAME);
        let link_name = format!("{addr}-sender-{}", Uuid::new_v4().as_hyphenated());
        let sender = Sender::attach(session, link_name, addr).await?;
        Ok(Self(sender, PhantomData))
    }
    pub async fn send(&mut self, step: FullyQualifiedStep<W::Step>) -> anyhow::Result<()> {
        // TODO: using string while developing, change to Vec<u8> in production
        let event = serde_json::to_string(&step)?;
        self.0.send(event).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NextStepReceiver<W: Workflow>(Receiver, PhantomData<W>);

impl<W: Workflow> NextStepReceiver<W> {
    pub async fn new<T>(session: &mut SessionHandle<T>) -> anyhow::Result<Self> {
        let addr = format!("{}-next-steps", W::NAME);
        let link_name = format!("{addr}-receiver-{}", Uuid::new_v4().as_hyphenated());
        let receiver = Receiver::attach(session, link_name, addr).await?;
        Ok(Self(receiver, PhantomData))
    }
    pub async fn recv(&mut self) -> anyhow::Result<FullyQualifiedStep<W::Step>> {
        // TODO: using string while developing, change to Vec<u8> in production
        let msg = self.0.recv::<String>().await?;

        let event = match serde_json::from_str(msg.body()) {
            Ok(event) => event,
            Err(e) => {
                let err = anyhow::anyhow!("Failed to deserialize step: {}", e);
                tracing::error!("{}", err);
                self.0.reject(msg, None).await?;
                return Err(err);
            }
        };
        self.0.accept(msg).await?;

        Ok(event)
    }
}
