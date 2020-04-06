use std::time::Duration;
use tokio::time::DelayQueue;

use delay_queue_split::DelayQueueExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let delay_queue = DelayQueue::new();

    let (mut delay_queue_insert, mut delay_queue_read) = delay_queue.split();

    delay_queue_insert.insert("hello", Duration::from_secs(1)).await?;
    delay_queue_insert.insert("world", Duration::from_secs(2)).await?;

    while let Some(value) = delay_queue_read.next().await {
            println!("{:?}", value);
    }

    Ok(())
}

