use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

use crate::workflows::{Workflow, WorkflowInstanceId};

pub trait Event: Serialize + for<'a> Deserialize<'a> + Clone {
    type Workflow: Workflow;
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Immediate<W: Workflow>(PhantomData<W>);

impl<W: Workflow> Event for Immediate<W> {
    type Workflow = W;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InstanceEvent<W: Workflow> {
    #[serde(bound = "")]
    pub event: W::Event,
    pub instance_id: WorkflowInstanceId,
}
