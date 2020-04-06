
## DELAY_QUEUE_SPLIT

This is an example I wrote when answering a question in `/r/rust` and got carried away.

It implements an extension trait for tokio's
[DelayQueue](https://docs.rs/tokio/0.2.16/tokio/time/struct.DelayQueue.html)
that adds a `split()` method to split the DelayQueue into `Insert` and `Read` parts,
that can be used in separate tasks (or threads).

See [this example](blob/master/examples/example.rs) to see how it's used.

