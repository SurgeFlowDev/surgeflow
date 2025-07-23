use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{any::TypeId, fmt::Debug};

use crate::workflows::{Workflow, WorkflowInstanceId};

pub trait Event: Serialize + for<'a> Deserialize<'a> + Clone
where
    Self: 'static,
{
    // move to extension trait so it can't be overridden
    fn matches_type<T: Event + 'static>() -> bool {
        TypeId::of::<Self>() == TypeId::of::<T>()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, JsonSchema)]
pub struct Immediate;

impl Event for Immediate {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceEvent<W: Workflow> {
    #[serde(bound = "")]
    pub event: W::Event,
    pub instance_id: WorkflowInstanceId,
}
