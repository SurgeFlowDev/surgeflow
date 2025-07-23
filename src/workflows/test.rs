// existing traits:

trait WorkflowEventType: Copy + Eq + PartialEq { }
trait WorkflowEvent {
    type EventType: WorkflowEventType;
    fn event_type(&self) -> Self::EventType;
}

////////////////////////////////////

struct Workflow1 {}

#[workflow("workflow_1")]
impl Workflow for Workflow1 {
    workflow_event! {
        Event0,
        Event1,
    }
    workflow_step! {
        Step0,
        Step1,
    }
    fn entrypoint() -> StepWithSettings<Self::Step> {
        StepWithSettings {
            step: Step0 {}.into(),
            settings: StepSettings { max_retries: 0 },
        }
    }
}
// The macro would generate:
enum Workflow1Event {
    Event0(Event0),
    Event1(Event1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Workflow1EventType {
    Event0,
    Event1,
}
impl WorkflowEventType for Workflow1EventType {}

impl WorkflowEvent for Workflow1Event {
    fn event_type(&self) -> Workflow1EventType {
        match self {
            Workflow1Event::Event0(_) => Workflow1EventType::Event0,
            Workflow1Event::Event1(_) => Workflow1EventType::Event1,
        }
    }
}
