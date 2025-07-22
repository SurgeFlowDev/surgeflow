use std::marker::PhantomData;

use crate::{workers::adapters::receivers::CompletedInstanceReceiver, workflows::Workflow};

pub struct CompletedInstanceWorkerDependencies<W, CompletedInstanceReceiverT>
where
    W: Workflow,
    CompletedInstanceReceiverT: CompletedInstanceReceiver<W>,
{
    pub completed_instance_receiver: CompletedInstanceReceiverT,
    marker: PhantomData<W>,
}

impl<W, CompletedInstanceReceiverT>
    CompletedInstanceWorkerDependencies<W, CompletedInstanceReceiverT>
where
    W: Workflow,
    CompletedInstanceReceiverT: CompletedInstanceReceiver<W>,
{
    pub fn new(completed_instance_receiver: CompletedInstanceReceiverT) -> Self {
        Self {
            completed_instance_receiver,
            marker: PhantomData,
        }
    }
}
