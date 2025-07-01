use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{event::{Event, WorkflowEvent}, Workflow0};

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {}

impl Event for Event0 {
    type Workflow = Workflow0;
}

impl Into<Option<<Workflow0 as crate::Workflow>::Event>> for Event0 {
    fn into(self) -> Option<<Workflow0 as crate::Workflow>::Event> {
        Some(WorkflowEvent::Event0(self))
    }
}
