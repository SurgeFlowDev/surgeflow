use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::senders::{EventSender, NewInstanceSender};

pub struct ControlServerDependencies<P, EventSenderT, NewInstanceSenderT>
where
    P: Project,
    EventSenderT: EventSender<P>,
    NewInstanceSenderT: NewInstanceSender<P>,
{
    pub event_sender: EventSenderT,
    pub new_instance_sender: NewInstanceSenderT,
    _marker: PhantomData<P>,
}
impl<P, EventSenderT, NewInstanceSenderT>
    ControlServerDependencies<P, EventSenderT, NewInstanceSenderT>
where
    P: Project,
    EventSenderT: EventSender<P>,
    NewInstanceSenderT: NewInstanceSender<P>,
{
    pub fn new(event_sender: EventSenderT, new_instance_sender: NewInstanceSenderT) -> Self {
        Self {
            event_sender,
            new_instance_sender,
            _marker: PhantomData,
        }
    }
}
