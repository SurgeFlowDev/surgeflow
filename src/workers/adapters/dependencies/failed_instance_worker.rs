use std::marker::PhantomData;

use crate::{workers::adapters::receivers::FailedInstanceReceiver, workflows::Workflow};

pub struct FailedInstanceWorkerDependencies<W, FailedInstanceReceiverT>
where
    W: Workflow,
    FailedInstanceReceiverT: FailedInstanceReceiver<W>,
{
    pub failed_instance_receiver: FailedInstanceReceiverT,
    marker: PhantomData<W>,
}

impl<W, FailedInstanceReceiverT> FailedInstanceWorkerDependencies<W, FailedInstanceReceiverT>
where
    W: Workflow,
    FailedInstanceReceiverT: FailedInstanceReceiver<W>,
{
    pub fn new(failed_instance_receiver: FailedInstanceReceiverT) -> Self {
        Self {
            failed_instance_receiver,
            marker: PhantomData,
        }
    }
}
