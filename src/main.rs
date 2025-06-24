use std::sync::Arc;

use rust_workflow_2::{
    ActiveStepQueue, InstanceId, WaitingForEventStepQueue,
    event::event_0::Event0,
    runner::{handle_event, handle_step},
};
use tokio::{join, sync::Mutex, try_join};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut active_step_queue = Arc::new(Mutex::new(ActiveStepQueue {
        queues: std::collections::HashMap::new(),
    }));
    let mut waiting_for_step_queue = Arc::new(Mutex::new(WaitingForEventStepQueue {
        queues: std::collections::HashMap::new(),
    }));
    let completed_step_queue = rust_workflow_2::CompletedStepQueue {
        queues: std::collections::HashMap::new(),
    };
    let instance_id = InstanceId::from(0);
    let step_0 = rust_workflow_2::step::step_0::Step0 {};

    let ac = active_step_queue.clone();
    let wt = waiting_for_step_queue.clone();
    let handle = tokio::spawn(async move {
        loop {
            let mut active_queue = ac.lock().await;
            let mut waiting_queue = wt.lock().await;
            let step = active_queue.wait_until_dequeue(instance_id).await.unwrap();
            // drop(queue); // Release the lock before handling the step
            let is_completed = handle_step(step, &mut waiting_queue, &mut active_queue)
                .await
                .unwrap();
            
            if is_completed {
                break;
            }
        }
    });

    let mut waiting_queue = waiting_for_step_queue.lock().await;
    waiting_queue
        .enqueue(rust_workflow_2::step::FullyQualifiedStep {
            instance_id,
            step: rust_workflow_2::step::StepWithSettings {
                step: rust_workflow_2::step::WorkflowStep::Step0(step_0),
                settings: rust_workflow_2::step::StepSettings {
                    max_retry_count: 1,
                    // delay: None,
                },
            },
            event: None,
            retry_count: 0,
        })
        .await?;

    let mut queue = active_step_queue.lock().await;
    handle_event(
        instance_id,
        Event0 {}.into(),
        &mut waiting_queue,
        &mut queue,
    )
    .await?;

    // let next_step = active_step_queue.dequeue(instance_id).await?;

    // handle_step(
    //     next_step,
    //     &mut waiting_for_step_queue,
    //     &mut active_step_queue,
    // )
    // .await?;

    // let next_step = active_step_queue.dequeue(instance_id).await?;

    // handle_step(
    //     next_step,
    //     &mut waiting_for_step_queue,
    //     &mut active_step_queue,
    // )
    // .await?;

    try_join!(handle)?;

    Ok(())
}
