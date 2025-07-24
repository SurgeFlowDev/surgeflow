use std::{marker::PhantomData, sync::Arc};

use crate::{
    workers::adapters::{
        dependencies::control_server::ControlServerDependencies,
        senders::{EventSender, NewInstanceSender},
    },
    workflows::Project,
};

pub mod event;
pub mod step;
pub mod workers;
pub mod workflows;

pub struct AppState<P: Project, E: EventSender<P>, I: NewInstanceSender<P>> {
    pub dependencies: ControlServerDependencies<P, E, I>,

    _marker: PhantomData<P>,
}

pub struct ArcAppState<P: Project, E: EventSender<P>, I: NewInstanceSender<P>>(
    pub Arc<AppState<P, E, I>>,
);

impl<P: Project, E: EventSender<P>, I: NewInstanceSender<P>> Clone for ArcAppState<P, E, I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
