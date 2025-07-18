use std::{marker::PhantomData, sync::Arc};

use crate::{
    workers::adapters::{managers::WorkflowInstanceManager, senders::EventSender},
    workflows::{TxState, Workflow},
};

pub mod event;
pub mod step;
pub mod workers;
pub mod workflows;

pub struct AppState<W: Workflow, E: EventSender<W>, M: WorkflowInstanceManager<W>> {
    pub event_sender: E,
    pub workflow_instance_manager: M,
    pub sqlx_tx_state: TxState,
    _marker: PhantomData<W>,
}

pub struct ArcAppState<W: Workflow, E: EventSender<W>, M: WorkflowInstanceManager<W>>(
    pub Arc<AppState<W, E, M>>,
);

impl<W: Workflow, E: EventSender<W>, M: WorkflowInstanceManager<W>> Clone for ArcAppState<W, E, M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
