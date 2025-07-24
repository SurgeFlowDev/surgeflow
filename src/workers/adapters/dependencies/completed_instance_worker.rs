use std::marker::PhantomData;

use crate::{workers::adapters::receivers::CompletedInstanceReceiver, workflows::Project};

pub struct CompletedInstanceWorkerDependencies<P, CompletedInstanceReceiverT>
where
    P: Project,
    CompletedInstanceReceiverT: CompletedInstanceReceiver<P>,
{
    pub completed_instance_receiver: CompletedInstanceReceiverT,
    marker: PhantomData<P>,
}

impl<P, CompletedInstanceReceiverT>
    CompletedInstanceWorkerDependencies<P, CompletedInstanceReceiverT>
where
    P: Project,
    CompletedInstanceReceiverT: CompletedInstanceReceiver<P>,
{
    pub fn new(completed_instance_receiver: CompletedInstanceReceiverT) -> Self {
        Self {
            completed_instance_receiver,
            marker: PhantomData,
        }
    }
}
