use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::workflows::{Event, Project, WorkflowInstanceId};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl Event for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: P::Event,
    pub instance_id: WorkflowInstanceId,
}
