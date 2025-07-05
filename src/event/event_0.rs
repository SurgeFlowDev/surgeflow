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

impl From<Event0> for Option<<Workflow0 as crate::Workflow>::Event> {
    fn from(val: Event0) -> Self {
        Some(Workflow0Event::Event0(val))
    }
}
