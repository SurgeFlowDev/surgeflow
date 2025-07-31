use std::{marker::PhantomData, sync::Arc};

use adapter_types::{
    dependencies::control_server::ControlServerDependencies,
    senders::{EventSender, NewInstanceSender},
};
use aide::{OperationIo, axum::ApiRouter};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::routing::TypedPath;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use surgeflow_types::{InstanceEvent, Project, Workflow, WorkflowInstance, WorkflowInstanceId};

pub mod workers;
pub mod workflows;

