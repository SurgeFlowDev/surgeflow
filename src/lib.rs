use std::{marker::PhantomData, sync::Arc};

use crate::{
    workers::adapters::{managers::WorkflowInstanceManager, senders::EventSender},
    workflows::{Project, TxState, Workflow},
};

pub mod event;
pub mod step;
pub mod workers;
pub mod workflows;

pub struct AppState<P: Project, E: EventSender<P>, M: WorkflowInstanceManager<P>> {
    pub event_sender: E,
    pub workflow_instance_manager: M,
    pub sqlx_tx_state: TxState,
    _marker: PhantomData<P>,
}

pub struct ArcAppState<P: Project, E: EventSender<P>, M: WorkflowInstanceManager<P>>(
    pub Arc<AppState<P, E, M>>,
);

impl<P: Project, E: EventSender<P>, M: WorkflowInstanceManager<P>> Clone for ArcAppState<P, E, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
