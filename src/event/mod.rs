use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use event_0::Event0;

pub(crate) mod event_0;

#[derive(Debug, Serialize, Deserialize, From, TryInto)]
// untagged, because we want the enum and the structs serialization to be interchangeable
// TODO: do we?
#[serde(untagged)]
pub(crate) enum WorkflowEvent {
    Event0(Event0),
}

impl Event for WorkflowEvent {}

pub(crate) trait Event: Serialize + for<'a> Deserialize<'a> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Immediate;

impl Event for Immediate {}
