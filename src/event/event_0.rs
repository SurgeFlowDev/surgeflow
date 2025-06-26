use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::event::Event;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Event0 {}

impl Event for Event0 {}
