use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Workflow0, event::Event};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Event0 {}

impl Event for Event0 {
    type Workflow = Workflow0;
}
