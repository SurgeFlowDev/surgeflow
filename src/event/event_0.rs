use serde::{Deserialize, Serialize};

use crate::event::Event;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event0 {}

impl Event for Event0 {}
