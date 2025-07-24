use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug};

use crate::workflows::{Event, Project, Workflow, WorkflowInstanceId};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl Event for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<P: Project> {
    #[serde(bound = "")]
    pub event: P::Event,
    pub instance_id: WorkflowInstanceId,
}
