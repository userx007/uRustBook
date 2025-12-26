# Streams and Async Iteration in Rust

## What is a Stream?

A **Stream** is the asynchronous counterpart to Rust's synchronous `Iterator`. While an iterator produces values synchronously (blocking until the next value is ready), a stream produces values asynchronously, allowing other tasks to run while waiting for the next item.

The `Stream` trait (from the `futures` crate) is defined similarly to `Iterator`:

```rust
pub trait Stream {
    type Item;
    
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>>;
}
```

Compare this to the synchronous `Iterator`:

```rust
pub trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
}
```

## Key Differences from Sync Iterators

**Asynchronous Execution**: Streams don't block while waiting for the next item. Instead, they return `Poll::Pending` when no item is ready, allowing the async runtime to schedule other work.

**Polling-based**: Streams use the same polling mechanism as `Future`s. You need to poll them within an async context, and they'll notify when ready via the waker system.

**No built-in iteration syntax**: Unlike iterators which can use `for` loops, streams require explicit iteration patterns (though `async for` is being considered for future Rust versions).

## Basic Stream Example


```rust
use futures::stream::{self, Stream, StreamExt};
use std::time::Duration;
use tokio::time::sleep;

// Example 1: Creating a simple stream from values
async fn basic_stream_example() {
    let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    
    // Collect all items
    let items: Vec<i32> = stream.collect().await;
    println!("Collected items: {:?}", items);
}

// Example 2: Creating an async stream that yields values over time
async fn timed_stream_example() {
    let mut stream = stream::iter(1..=5)
        .then(|n| async move {
            sleep(Duration::from_millis(100)).await;
            n * 2
        });
    
    // Iterate over stream items
    while let Some(item) = stream.next().await {
        println!("Received: {}", item);
    }
}

// Example 3: Custom stream implementation
use std::pin::Pin;
use std::task::{Context, Poll};

struct CounterStream {
    count: u32,
    max: u32,
}

impl CounterStream {
    fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Stream for CounterStream {
    type Item = u32;
    
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        if self.count < self.max {
            let current = self.count;
            self.count += 1;
            Poll::Ready(Some(current))
        } else {
            Poll::Ready(None)
        }
    }
}

async fn custom_stream_example() {
    let mut stream = CounterStream::new(5);
    
    while let Some(n) = stream.next().await {
        println!("Count: {}", n);
    }
}

#[tokio::main]
async fn main() {
    println!("=== Basic Stream ===");
    basic_stream_example().await;
    
    println!("\n=== Timed Stream ===");
    timed_stream_example().await;
    
    println!("\n=== Custom Stream ===");
    custom_stream_example().await;
}

```
## Stream Combinators and Transformation

Streams provide many combinators similar to iterators, such as `map`, `filter`, `fold`, `take`, etc. These allow you to transform and process streams declaratively:

```rust
use futures::stream::{self, StreamExt};
use tokio::time::{sleep, Duration};

// Example 1: Chaining stream operations
async fn stream_combinators() {
    let result = stream::iter(1..=10)
        .filter(|x| async move { x % 2 == 0 })
        .map(|x| x * x)
        .take(3)
        .collect::<Vec<_>>()
        .await;
    
    println!("Filtered, mapped, and limited: {:?}", result);
}

// Example 2: Folding a stream
async fn stream_fold() {
    let sum = stream::iter(1..=5)
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum: {}", sum);
}

// Example 3: Concurrent stream processing with buffer_unordered
async fn concurrent_processing() {
    let results = stream::iter(1..=5)
        .map(|n| async move {
            // Simulate async work with varying delays
            sleep(Duration::from_millis(100 / n as u64)).await;
            println!("Processing {}", n);
            n * 2
        })
        .buffer_unordered(3) // Process up to 3 items concurrently
        .collect::<Vec<_>>()
        .await;
    
    println!("Results (may be out of order): {:?}", results);
}

// Example 4: Merging multiple streams
async fn merge_streams() {
    let stream1 = stream::iter(vec!["a", "b", "c"]);
    let stream2 = stream::iter(vec!["x", "y", "z"]);
    
    use futures::stream::select;
    let mut merged = select(stream1, stream2);
    
    while let Some(item) = merged.next().await {
        println!("Merged item: {}", item);
    }
}

// Example 5: Stream with error handling
use std::error::Error;

async fn stream_with_errors() -> Result<(), Box<dyn Error>> {
    let stream = stream::iter(vec![
        Ok(1),
        Ok(2),
        Err("Error at 3"),
        Ok(4),
    ]);
    
    // Collect successful items until first error
    match stream.try_collect::<Vec<_>>().await {
        Ok(values) => println!("All values: {:?}", values),
        Err(e) => println!("Error occurred: {}", e),
    }
    
    // Or handle errors individually
    let mut stream = stream::iter(vec![
        Ok(1),
        Ok(2),
        Err("Error at 3"),
        Ok(4),
    ]);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => println!("Value: {}", value),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}

// Example 6: Chunks - processing items in batches
async fn stream_chunks() {
    use futures::stream::StreamExt;
    
    let mut chunks = stream::iter(1..=10)
        .chunks(3);
    
    while let Some(chunk) = chunks.next().await {
        println!("Chunk: {:?}", chunk);
    }
}

#[tokio::main]
async fn main() {
    println!("=== Stream Combinators ===");
    stream_combinators().await;
    
    println!("\n=== Stream Fold ===");
    stream_fold().await;
    
    println!("\n=== Concurrent Processing ===");
    concurrent_processing().await;
    
    println!("\n=== Merge Streams ===");
    merge_streams().await;
    
    println!("\n=== Error Handling ===");
    let _ = stream_with_errors().await;
    
    println!("\n=== Chunks ===");
    stream_chunks().await;
}
```


## Practical Real-World Examples

```rust
use futures::stream::{self, StreamExt};
use tokio::sync::mpsc;
use std::time::Duration;
use tokio::time::sleep;

// Example 1: Converting a channel receiver to a stream
async fn channel_stream() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);
    
    // Spawn a task that sends values
    tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(100)).await;
        }
    });
    
    // Convert receiver to stream using StreamExt
    use tokio_stream::wrappers::ReceiverStream;
    let mut stream = ReceiverStream::new(rx);
    
    println!("Receiving from channel stream:");
    while let Some(value) = stream.next().await {
        println!("  Received: {}", value);
    }
}

// Example 2: Interval stream - periodic events
async fn interval_stream() {
    use tokio::time::interval;
    use tokio_stream::wrappers::IntervalStream;
    
    let interval = interval(Duration::from_millis(200));
    let mut stream = IntervalStream::new(interval)
        .take(5)
        .enumerate();
    
    println!("Interval stream ticks:");
    while let Some((i, _instant)) = stream.next().await {
        println!("  Tick {}", i);
    }
}

// Example 3: Processing a data pipeline
#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    value: i32,
}

async fn data_pipeline() {
    // Simulate incoming data
    let data_stream = stream::iter(vec![
        DataRecord { id: 1, value: 10 },
        DataRecord { id: 2, value: -5 },
        DataRecord { id: 3, value: 20 },
        DataRecord { id: 4, value: 15 },
        DataRecord { id: 5, value: -3 },
    ]);
    
    // Build a processing pipeline
    let processed = data_stream
        // Filter out negative values
        .filter(|record| async move { record.value > 0 })
        // Transform the data
        .map(|record| async move {
            sleep(Duration::from_millis(50)).await; // Simulate processing
            DataRecord {
                id: record.id,
                value: record.value * 2,
            }
        })
        // Process concurrently
        .buffered(2)
        .collect::<Vec<_>>()
        .await;
    
    println!("Processed data pipeline:");
    for record in processed {
        println!("  {:?}", record);
    }
}

// Example 4: Fan-out pattern - broadcasting to multiple consumers
async fn fan_out_pattern() {
    let (tx, _rx) = tokio::sync::broadcast::channel::<i32>(10);
    
    // Create multiple subscribers
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    
    // Spawn producer
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        for i in 1..=5 {
            let _ = tx_clone.send(i);
            sleep(Duration::from_millis(100)).await;
        }
    });
    
    // Consumer 1
    let consumer1 = tokio::spawn(async move {
        println!("Consumer 1 starting");
        while let Ok(value) = rx1.recv().await {
            println!("  Consumer 1 got: {}", value);
        }
    });
    
    // Consumer 2
    let consumer2 = tokio::spawn(async move {
        println!("Consumer 2 starting");
        while let Ok(value) = rx2.recv().await {
            println!("  Consumer 2 got: {}", value);
        }
    });
    
    // Wait for consumers
    let _ = tokio::join!(consumer1, consumer2);
}

// Example 5: Backpressure handling
async fn backpressure_example() {
    let (tx, mut rx) = mpsc::channel::<i32>(2); // Small buffer
    
    // Fast producer
    tokio::spawn(async move {
        for i in 1..=10 {
            println!("Sending {}", i);
            // send() will wait when buffer is full (backpressure)
            tx.send(i).await.unwrap();
            println!("  Sent {}", i);
        }
    });
    
    // Slow consumer
    println!("Slow consumer processing:");
    while let Some(value) = rx.recv().await {
        println!("  Processing {}", value);
        sleep(Duration::from_millis(300)).await; // Slow processing
    }
}

// Example 6: Stream select - race between multiple streams
async fn stream_race() {
    use futures::stream::select;
    
    let stream1 = stream::iter(vec![1, 2, 3])
        .then(|n| async move {
            sleep(Duration::from_millis(100)).await;
            format!("Stream1: {}", n)
        });
    
    let stream2 = stream::iter(vec![10, 20, 30])
        .then(|n| async move {
            sleep(Duration::from_millis(150)).await;
            format!("Stream2: {}", n)
        });
    
    let mut combined = select(stream1, stream2);
    
    println!("Racing streams:");
    while let Some(item) = combined.next().await {
        println!("  {}", item);
    }
}

#[tokio::main]
async fn main() {
    println!("=== Channel Stream ===");
    channel_stream().await;
    
    println!("\n=== Interval Stream ===");
    interval_stream().await;
    
    println!("\n=== Data Pipeline ===");
    data_pipeline().await;
    
    println!("\n=== Fan-out Pattern ===");
    fan_out_pattern().await;
    
    println!("\n=== Backpressure ===");
    backpressure_example().await;
    
    println!("\n=== Stream Race ===");
    stream_race().await;
}
```

## Key Concepts and Patterns

### Iteration Patterns

Since Rust doesn't yet have native `async for` syntax, you use these patterns to iterate over streams:

**Pattern 1: While-let loop**
```rust
while let Some(item) = stream.next().await {
    // process item
}
```

**Pattern 2: For-each combinator**
```rust
stream.for_each(|item| async move {
    // process item
}).await;
```

**Pattern 3: Collect into a collection**
```rust
let items: Vec<_> = stream.collect().await;
```

### Important Stream Combinators

- **`map`** / **`then`**: Transform items (use `then` when transformation is async)
- **`filter`**: Keep only items matching a predicate
- **`take`** / **`skip`**: Limit or skip items
- **`fold`** / **`scan`**: Accumulate values
- **`buffer_unordered`** / **`buffered`**: Process multiple items concurrently
- **`chunks`**: Group items into batches
- **`merge`** / **`select`**: Combine multiple streams
- **`throttle`** / **`timeout`**: Control timing

### Sync vs Async Iteration - Key Differences

| Aspect | Iterator (Sync) | Stream (Async) |
|--------|----------------|----------------|
| Blocking | Blocks until next item | Returns Poll::Pending when not ready |
| Syntax | `for item in iter { }` | `while let Some(item) = stream.next().await { }` |
| Performance | No overhead | Small runtime overhead for polling |
| Use case | CPU-bound, immediate data | I/O-bound, delayed data |
| Concurrency | Sequential only | Can process items concurrently |
| Backpressure | Pull-based (implicit) | Pull-based with async awareness |

### When to Use Streams

Streams are ideal for handling asynchronous data sources such as network I/O (WebSocket connections, HTTP/2 server-sent events), file I/O (reading large files asynchronously), timers and intervals, channels and message passing, database query results, and event-driven systems. You should use regular iterators when data is immediately available and processing is CPU-bound, while streams shine when you need to handle data that arrives over time or when you want to process multiple items concurrently.

### Common Pitfalls

**Forgetting to await**: Streams are lazy—you must call `.await` to drive them.

**Not pinning**: Custom stream implementations often need `Pin` for safety with self-referential types.

**Buffering too much**: `buffer_unordered(n)` with large `n` can consume lots of memory.

**Ignoring backpressure**: Fast producers with slow consumers can overwhelm the system without proper channel sizing.

### Dependencies

To work with streams, you'll typically need:

```toml
[dependencies]
futures = "0.3"
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
```

Streams are a powerful abstraction for handling asynchronous data flows in Rust, providing a composable and efficient way to work with data that arrives over time. They integrate seamlessly with Rust's async/await ecosystem and provide essential tools for building responsive, concurrent applications.