use std::marker::PhantomData;

use surgeflow_types::Project;

use crate::receivers::FailedInstanceReceiver;

pub struct FailedInstanceWorkerDependencies<P, FailedInstanceReceiverT>
where
    P: Project,
    FailedInstanceReceiverT: FailedInstanceReceiver<P>,
{
    pub failed_instance_receiver: FailedInstanceReceiverT,
    marker: PhantomData<P>,
}

impl<P, FailedInstanceReceiverT> FailedInstanceWorkerDependencies<P, FailedInstanceReceiverT>
where
    P: Project,
    FailedInstanceReceiverT: FailedInstanceReceiver<P>,
{
    pub fn new(failed_instance_receiver: FailedInstanceReceiverT) -> Self {
        Self {
            failed_instance_receiver,
            marker: PhantomData,
        }
    }
}
