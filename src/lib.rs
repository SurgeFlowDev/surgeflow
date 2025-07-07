use schemars::JsonSchema;
use std::hash::Hash;

use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

pub mod event;
pub mod step;
pub mod workflows;

use step::StepError;

use crate::{
    event::Event,
    step::{Step, StepWithSettings, WorkflowStep},
};



// step
