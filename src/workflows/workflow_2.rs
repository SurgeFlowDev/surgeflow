use aide::axum::ApiRouter;
use derive_more::{From, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    event::Immediate,
    step::{StepSettings, StepWithSettings},
    workflows::{
        Event, MyProject, MyProjectEvent, MyProjectStep, Step, TryFromRef, Workflow, WorkflowEvent,
        WorkflowStep,
    },
};

///////////////

#[derive(Clone, Debug)]
pub struct Workflow2;

impl Workflow for Workflow2 {
    type Project = MyProject;

    type Event = Workflow2Event;

    type Step = Workflow2Step;

    const NAME: &'static str = "workflow_2";

    fn entrypoint() -> crate::step::StepWithSettings<Self::Project> {
        crate::step::StepWithSettings {
            step: MyProjectStep::Workflow2(Workflow2Step::Step0(Step0)),
            settings: crate::step::StepSettings { max_retries: 3 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TryInto, From)]
pub enum Workflow2Step {
    Step0(Step0),
    Step1(Step1),
}

impl WorkflowStep for Workflow2Step {
    type Workflow = Workflow2;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: <Self::Workflow as Workflow>::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<
        Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
        crate::step::StepError,
    > {
        match self {
            Workflow2Step::Step0(step) => step.run_raw(wf, event.try_into().unwrap()).await,
            Workflow2Step::Step1(step) => step.run_raw(wf, event.try_into().unwrap()).await,
        }
    }

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow2Step::Step0(_) => <Step0 as Step>::Event::is::<T>(),
            Workflow2Step::Step1(_) => <Step1 as Step>::Event::is::<T>(),
        }
    }

    fn is_workflow_event(&self, event: &<Self::Workflow as Workflow>::Event) -> bool {
        match self {
            Workflow2Step::Step0(_) => event.is_event::<<Step0 as Step>::Event>(),
            Workflow2Step::Step1(_) => event.is_event::<<Step1 as Step>::Event>(),
        }
    }
}

///// Steps

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step0;

impl Step for Step0 {
    type Event = Event0;

    type Workflow = Workflow2;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
        // TODO: WorkflowStep should not be hardcoded here, but rather there should be a "Workflow" associated type,
        // where we can get the WorkflowStep type from
    ) -> Result<
        Option<StepWithSettings<<Self::Workflow as Workflow>::Project>>,
        crate::step::StepError,
    > {
        tracing::info!("Running Step0 in Workflow2");
        Ok(Some(StepWithSettings {
            // TODO: this should just return Workflow2Step, right? It shouldn't be able to return steps from other workflows
            step: MyProjectStep::Workflow2(Workflow2Step::Step1(Step1)),
            settings: StepSettings { max_retries: 3 },
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Step1;

impl Step for Step1 {
    type Event = Immediate;

    type Workflow = Workflow2;

    async fn run_raw(
        &self,
        wf: Self::Workflow,
        event: Self::Event,
    ) -> Result<
        Option<crate::step::StepWithSettings<<Self::Workflow as Workflow>::Project>>,
        crate::step::StepError,
    > {
        tracing::info!("Running Step1 in Workflow2");
        Ok(None)
    }
}

// events

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Workflow2Event {
    Event0(Event0),
    #[serde(skip)]
    Immediate(Immediate),
}

impl WorkflowEvent for Workflow2Event {
    type Workflow = Workflow2;

    fn is_event<T: Event + 'static>(&self) -> bool {
        match self {
            Workflow2Event::Event0(_) => Event0::is::<T>(),
            Workflow2Event::Immediate(_) => Immediate::is::<T>(),
        }
    }
}

impl TryFrom<Workflow2Event> for Immediate {
    type Error = ();

    fn try_from(event: Workflow2Event) -> Result<Self, Self::Error> {
        match event {
            Workflow2Event::Immediate(immediate) => Ok(immediate),
            _ => Err(()),
        }
    }
}

impl From<Immediate> for Workflow2Event {
    fn from(immediate: Immediate) -> Self {
        Workflow2Event::Immediate(immediate)
    }
}

impl TryFrom<Workflow2Event> for Event0 {
    type Error = ();

    fn try_from(event: Workflow2Event) -> Result<Self, Self::Error> {
        if let Workflow2Event::Event0(event0) = event {
            Ok(event0)
        } else {
            Err(())
        }
    }
}

impl TryFrom<MyProjectEvent> for Workflow2Event {
    type Error = ();

    fn try_from(event: MyProjectEvent) -> Result<Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow2(workflow_event) => Ok(workflow_event),
            MyProjectEvent::Immediate(immediate) => Ok(Workflow2Event::Immediate(immediate)),
            _ => Err(()),
        }
    }
}

impl TryFromRef<MyProjectEvent> for Workflow2Event {
    type Error = ();

    fn try_from_ref(event: &MyProjectEvent) -> Result<&Self, Self::Error> {
        match event {
            MyProjectEvent::Workflow2(workflow_event) => Ok(workflow_event),
            MyProjectEvent::Immediate(_) => Err(()),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event0 {}

impl From<Event0> for Workflow2Event {
    fn from(event: Event0) -> Self {
        Workflow2Event::Event0(event)
    }
}

impl Event for Event0 {}
