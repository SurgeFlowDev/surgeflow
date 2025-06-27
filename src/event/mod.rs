use derive_more::{From, TryInto};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use event_0::Event0;

use crate::{Workflow, Workflow0, step::WorkflowStep};

pub mod event_0;

#[derive(Debug, Serialize, Deserialize, From, TryInto, JsonSchema)]
// untagged, because we want the enum and the structs serialization to be interchangeable
// TODO: do we?
#[serde(untagged)]
pub enum WorkflowEvent {
    Event0(Event0),
}

impl Event for WorkflowEvent {
    type Workflow = Workflow0;
}

pub trait Event: Serialize + for<'a> Deserialize<'a> {
    type Workflow: Workflow;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Immediate;

impl Event for Immediate {
    type Workflow = Workflow0;
}
