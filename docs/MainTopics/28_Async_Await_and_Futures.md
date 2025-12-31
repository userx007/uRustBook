# Async/Await and Futures in Rust

## Overview

Rust's async/await system enables writing asynchronous code that looks like synchronous code, making it easier to handle concurrent operations efficiently. Unlike threads, async tasks are lightweight and can handle thousands of concurrent operations with minimal overhead.

## The Future Trait

At the core of Rust's async system is the `Future` trait:

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

A `Future` represents a value that may not be ready yet. When polled:
- **`Poll::Ready(value)`** - The future has completed with a value
- **`Poll::Pending`** - The future isn't ready; it will notify the executor when it can make progress

## Async Syntax

### Basic Async Function

```rust
// Async function
async fn fetch_data() -> String {
    // Simulated async operation
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    "Data fetched!".to_string()
}

// Equivalent to:
fn fetch_data() -> impl Future<Output = String> {
    async {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        "Data fetched!".to_string()
    }
}
```

### Async Blocks

```rust
use tokio::time::{sleep, Duration};

async fn example() {
    // Async block
    let future = async {
        sleep(Duration::from_millis(100)).await;
        println!("Async block executed!");
        42
    };
    
    let result = future.await;
    println!("Result: {}", result);
}
```

## Async Function Transformation

When you write an async function, the compiler transforms it into a state machine. Here's a conceptual example:

```rust
// What you write:
async fn compute(x: i32) -> i32 {
    let y = async_operation_1(x).await;
    let z = async_operation_2(y).await;
    z + 1
}

// Roughly transforms to:
enum ComputeStateMachine {
    Start { x: i32 },
    WaitingForOp1 { x: i32, fut1: impl Future<Output = i32> },
    WaitingForOp2 { fut2: impl Future<Output = i32> },
    Done,
}

impl Future for ComputeStateMachine {
    type Output = i32;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        // State machine logic that advances through states
        // on each poll until completion
        todo!()
    }
}
```

## Poll-Based Execution

Futures are **lazy** - they don't do anything until polled. An executor (like Tokio) drives futures to completion:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Custom simple future
struct CountdownFuture {
    count: u32,
}

impl Future for CountdownFuture {
    type Output = String;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.count == 0 {
            Poll::Ready("Liftoff!".to_string())
        } else {
            println!("Counting: {}", self.count);
            self.count -= 1;
            
            // Wake the executor to poll again
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    let result = CountdownFuture { count: 3 }.await;
    println!("{}", result);
}
```

## Practical Examples

### Example 1: Concurrent HTTP Requests

```rust
use tokio;

async fn fetch_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simulated fetch
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(format!("Content from {}", url))
}

#[tokio::main]
async fn main() {
    // Sequential execution
    let result1 = fetch_url("https://example.com").await.unwrap();
    let result2 = fetch_url("https://example.org").await.unwrap();
    
    println!("Sequential: {} and {}", result1, result2);
    
    // Concurrent execution with join!
    let (result1, result2) = tokio::join!(
        fetch_url("https://example.com"),
        fetch_url("https://example.org")
    );
    
    println!("Concurrent: {:?} and {:?}", result1, result2);
}
```

### Example 2: Async with Error Handling

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

async fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

async fn write_file(path: &str, data: &str) -> io::Result<()> {
    let mut file = File::create(path).await?;
    file.write_all(data.as_bytes()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    write_file("test.txt", "Hello, async!").await?;
    let contents = read_file("test.txt").await?;
    println!("File contents: {}", contents);
    Ok(())
}
```

### Example 3: Spawning Tasks

```rust
use tokio;
use std::time::Duration;

async fn task(id: u32, duration: u64) -> u32 {
    println!("Task {} starting", id);
    tokio::time::sleep(Duration::from_millis(duration)).await;
    println!("Task {} completed", id);
    id * 2
}

#[tokio::main]
async fn main() {
    // Spawn concurrent tasks
    let handle1 = tokio::spawn(task(1, 100));
    let handle2 = tokio::spawn(task(2, 50));
    let handle3 = tokio::spawn(task(3, 150));
    
    // Wait for all tasks
    let results = tokio::try_join!(handle1, handle2, handle3).unwrap();
    println!("Results: {:?}", results);
}
```

### Example 4: Select! Macro (Racing Futures)

```rust
use tokio::time::{sleep, Duration};

async fn operation_a() -> &'static str {
    sleep(Duration::from_millis(100)).await;
    "Operation A completed"
}

async fn operation_b() -> &'static str {
    sleep(Duration::from_millis(200)).await;
    "Operation B completed"
}

#[tokio::main]
async fn main() {
    tokio::select! {
        result = operation_a() => {
            println!("First to complete: {}", result);
        }
        result = operation_b() => {
            println!("First to complete: {}", result);
        }
    }
    // Operation A will win since it's faster
}
```

## Key Concepts

**Zero-cost abstraction**: Async/await has minimal runtime overhead compared to hand-written state machines.

**Lazy evaluation**: Futures do nothing until polled by an executor.

**Pinning**: Futures often need to be pinned in memory because they contain self-referential data in their state machines.

**Waker pattern**: The `Context` contains a `Waker` that futures use to notify the executor when they can make progress.

**Executors**: You need a runtime (like Tokio, async-std, or smol) to actually run async code. The `#[tokio::main]` macro sets up the runtime.

## Common Pitfalls

1. **Forgetting to `.await`**: A future without `.await` won't execute
2. **Blocking in async**: Don't use blocking operations in async functions
3. **No executor**: Async functions need a runtime to execute
4. **Send bounds**: Futures spawned on multi-threaded executors must be `Send`

This poll-based model allows Rust to provide efficient async I/O without garbage collection or heavyweight threads, making it ideal for high-performance networked applications.