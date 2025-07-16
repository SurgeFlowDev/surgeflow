use crate::{workers::adapters::senders::EventSender, workflows::Workflow};

pub struct ControlServerDependencies<W: Workflow, C: ControlServerContext<W>> {
    pub event_sender: C::EventSender,
    #[expect(dead_code)]
    context: C,
}

impl<W: Workflow, C: ControlServerContext<W>> ControlServerDependencies<W, C> {
    pub fn new(event_sender: C::EventSender, context: C) -> Self {
        Self {
            event_sender,
            context,
        }
    }
}

pub trait ControlServerContext<W: Workflow>: Sized {
    type EventSender: EventSender<W>;
    //
    fn dependencies()
    -> impl Future<Output = anyhow::Result<ControlServerDependencies<W, Self>>> + Send;
}
