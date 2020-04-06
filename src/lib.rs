//! ## Add a `split` method to tokio's `DelayQueue` using an extension trait.
//!
//! The `split` method returns an `Insert` and a `Read` half. Those two can
//! be used in separate tasks.

use tokio::time::delay_queue::Expired;
use tokio::time::DelayQueue;
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{channel, Sender, Receiver, error::SendError};

use std::time::Duration;

/// Extension trait for DelayQueue that adds `split()`.
pub trait DelayQueueExt<T> {
    fn split(self) -> (DelayQueueInsert<T>, DelayQueueRead<T>)
        where T: Send + 'static;
}

impl<T> DelayQueueExt<T> for DelayQueue<T> {

    /// Split a DelayQueue into separate "Insert" and "Read" parts.
    fn split(self) -> (DelayQueueInsert<T>, DelayQueueRead<T>)
    where
        T: Send + 'static,
    {

        // create a set of channels.
        let (queue_add, mut queue_add_rx) = channel::<(T, Duration)>(1);
        let (mut queue_read_tx, queue_read) = channel::<Option<Result<Expired<T>, tokio::time::Error>>>(1);
        
        // run the DelayQueue in a separate task.
        tokio::spawn(async move {
            let mut delay_queue = self;
            let mut queue_add_eof = false;
            loop {
                tokio::select! {
                    // delayed item from the DelayQueue becomes available.
                    delayed_item = delay_queue.next() => {
                        match delayed_item {
                            Some(item) => {
                                // forward it to the queue_read channel.
                                if queue_read_tx.send(Some(item)).await.is_err() {
                                    // queue_read channel receiver side was dropped.
                                    break;
                                }
                            },
                            None => {
                                // if the queue_add channel is gone, we're done.
                                if queue_add_eof {
                                    break;
                                }
                                // otherwise, send a 'None' to indicate the queue is empty.
                                if queue_read_tx.send(None).await.is_err() {
                                    // queue_read channel receiver side was dropped.
                                    break;
                                }
                            },
                        }
                    }
                    // an item is received from the queue_add channel.
                    new_item = queue_add_rx.next(), if !queue_add_eof => {
                        match new_item {
                            Some(item) => {
                                // insert it into the DelayQueue.
                                delay_queue.insert(item.0, item.1);
                            },
                            None => {
                                // the queue_add channel was dropped.
                                queue_add_eof = false;
                            },
                        }
                    }
                }
            }
        });
        
        (DelayQueueInsert(queue_add), DelayQueueRead(queue_read))
    }
}

/// "insert" half of a DelayQueue.
pub struct DelayQueueInsert<T>(Sender<(T, Duration)>);

impl<T> DelayQueueInsert<T> {
    /// Insert an item into the DelayQueue.
    ///
    /// Other than `DelayQueue`'s normal `insert`, this function can return
    /// an error if the `Read` half was dropped.
    pub async fn insert(&mut self, item: T, d: Duration) -> Result<(), SendError<(T, Duration)>> {
        self.0.send((item, d)).await
    }
}


/// "read" half of a DelayQueue.
pub struct DelayQueueRead<T>(Receiver<Option<Result<Expired<T>, tokio::time::Error>>>);

impl<T> DelayQueueRead<T> {
    /// Read the next item from the DelayQueue.
    pub async fn next(&mut self) -> Option<Result<Expired<T>, tokio::time::Error>> {
        match self.0.next().await {
            Some(Some(item)) => Some(item),
            Some(None) => None,
            None => None,
        }
    }
}

