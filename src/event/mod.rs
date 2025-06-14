use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use event_0::Event0;

pub(crate) mod event_0;

#[enum_dispatch(Event)]
#[derive(Debug, Serialize, Deserialize)]
// untagged, because we want the enum and the structs serialization to be interchangeable
#[serde(untagged)]
pub(crate) enum WorkflowEvent {
    Event0(Event0),
}

#[enum_dispatch]
pub(crate) trait Event: Serialize + for<'a> Deserialize<'a> + Debug {}
