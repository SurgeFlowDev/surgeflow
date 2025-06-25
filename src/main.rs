use std::{any::TypeId, sync::Arc, time::Duration};

use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    publisher_confirm::Confirmation,
    types::FieldTable,
};
use rust_workflow_2::{
    ActiveStepQueue, InstanceId, WaitingForEventStepQueue,
    event::event_0::Event0,
    runner::{handle_event, handle_step},
    step::{Step, step_0::Step0},
};
use tokio::{join, sync::Mutex, try_join};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

    //  let channel_a = conn.create_channel().await?;

    // let queue = channel_a
    //     .queue_declare(
    //         "workflow-name",
    //         QueueDeclareOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;

    // let payload = b"Hello world!";

    // let confirm = channel_a
    //     .basic_publish(
    //         "",
    //         "hello",
    //         BasicPublishOptions::default(),
    //         payload,
    //         BasicProperties::default(),
    //     )
    //     .await?
    //     .await?;

    // assert_eq!(confirm, Confirmation::NotRequested);
    // info!(?queue, "Declared queue");

    // tokio::time::sleep(Duration::from_secs(20)).await;

    let sub_channel = conn.create_channel().await?;
    let consumer = sub_channel
        .basic_consume(
            "workflow-name",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let active_step_queue = Arc::new(Mutex::new(ActiveStepQueue {
        pub_channel: conn.create_channel().await?,
        sub_channel,
        consumer,
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
    // let handle = tokio::spawn(async move {
    //     loop {
    //         let mut active_queue = ac.lock().await;
    //         let mut waiting_queue = wt.lock().await;
    //         let step = active_queue.dequeue(instance_id).await.unwrap();
    //         // drop(queue); // Release the lock before handling the step
    //         let is_completed = handle_step(step, &mut waiting_queue, &mut active_queue)
    //             .await
    //             .unwrap();

    //         if is_completed {
    //             break;
    //         }
    //     }
    // });

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
    tracing::info!("handle event");
    handle_event(
        instance_id,
        Event0 {}.into(),
        &mut waiting_queue,
        &mut queue,
    )
    .await?;

    tracing::info!("next step 1");
    let next_step = queue.dequeue(instance_id).await?;

    tracing::info!("handle step 1");
    handle_step(next_step, &mut waiting_queue, &mut queue).await?;

    tracing::info!("next step 2");
    let next_step = queue.dequeue(instance_id).await?;

    tracing::info!("handle step 1");
    handle_step(next_step, &mut waiting_queue, &mut queue).await?;

    // try_join!(handle)?;

    Ok(())
}
