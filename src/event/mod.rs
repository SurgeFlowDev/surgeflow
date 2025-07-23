use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::workflows::{Workflow, WorkflowInstanceId};

pub trait Event: Serialize + for<'a> Deserialize<'a> + Clone {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl Event for Immediate {}

// impl AsWorkflowEventType for Immediate {
//     fn as_event_type(&self) -> EventType {
//         EventType::Immediate
//     }
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<W: Workflow> {
    #[serde(bound = "")]
    pub event: W::Event,
    pub instance_id: WorkflowInstanceId,
}
