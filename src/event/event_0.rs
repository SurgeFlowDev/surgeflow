use serde::{Deserialize, Serialize};

use crate::event::Event;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Event0 {}

impl Event for Event0 {}
