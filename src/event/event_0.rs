use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Workflow0,
    event::{Event, Workflow0Event},
};

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Event0 {}

impl Event for Event0 {
    type Workflow = Workflow0;
}

impl Into<Option<<Workflow0 as crate::Workflow>::Event>> for Event0 {
    fn into(self) -> Option<<Workflow0 as crate::Workflow>::Event> {
        Some(Workflow0Event::Event0(self))
    }
}
