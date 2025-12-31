# Async Patterns and Best Practices in Rust

## **1. Select Operations**
The `select!` macro allows you to wait on multiple async operations simultaneously, proceeding with whichever completes first. This is crucial for:
- Handling multiple event sources (channels, timers, signals)
- Implementing timeouts in combination with other operations
- Building responsive applications that react to multiple inputs

Key points: branches are cancel-safe, use `biased` for priority ordering, and all incomplete branches are cancelled when one completes.

## **2. Join Operations**
`join!` and `try_join!` run futures concurrently and wait for all to complete:
- **`join!`**: Waits for all futures, collecting results
- **`try_join!`**: Short-circuits on first error

This is more efficient than sequential `.await` calls and maintains structured concurrency without spawning tasks.

## **3. Cancellation**
Rust futures are cancelled by dropping them. Best practices include:
- Using `select!` with cancellation tokens for graceful shutdown
- Implementing cleanup logic that runs even on cancellation
- Understanding cancel-safety: some operations (like `Mutex::lock`) can't be safely cancelled mid-execution
- Using `JoinHandle::abort()` for spawned tasks

## **4. Timeouts**
Essential for preventing operations from hanging indefinitely:
- Wrap operations with `tokio::time::timeout`
- Combine with retry logic and exponential backoff
- Always have a fallback strategy for timeout scenarios
- Consider different timeout values for different operation types

## **5. Backpressure**
Controls flow between fast producers and slow consumers:
- **Bounded channels**: Natural backpressure when channel fills
- **Semaphores**: Rate limiting and resource quotas
- **Stream combinators**: `buffer()`, `throttle()`
- Monitor and tune buffer sizes based on actual performance

## **6. Structured Concurrency**
Ensures spawned tasks don't outlive their logical scope:
- Group related tasks together
- Always await or explicitly detach spawned tasks
- Use task groups or scopes to manage lifetimes
- Prevents resource leaks and unexpected behavior

## **Common Pitfalls to Avoid**

1. **Blocking in async code**: Never use `std::thread::sleep` or blocking I/O
2. **Unbounded spawning**: Limit concurrent tasks with Semaphore
3. **Ignored errors**: Always handle `Result` types from async operations
4. **Large futures**: Keep async functions small to reduce memory usage
5. **Missing timeouts**: External operations should always have timeouts

To run this code, add to your `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
// Rust Async Patterns and Best Practices
// This example demonstrates key async patterns using Tokio

use tokio::time::{sleep, timeout, Duration};
use tokio::sync::{mpsc, Semaphore};
use tokio::select;
use std::sync::Arc;

// =============================================================================
// 1. SELECT OPERATIONS - Running multiple futures concurrently
// =============================================================================

async fn select_example() {
    println!("\n=== SELECT OPERATIONS ===");
    
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let (tx, mut rx) = mpsc::channel::<String>(10);
    
    // Spawn a task that sends messages
    tokio::spawn(async move {
        sleep(Duration::from_millis(1500)).await;
        tx.send("Hello from channel!".to_string()).await.unwrap();
    });
    
    let mut count = 0;
    
    // Select waits on multiple futures, executing whichever completes first
    loop {
        select! {
            _ = interval.tick() => {
                count += 1;
                println!("Tick {}", count);
                if count >= 3 {
                    break;
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(m) => println!("Received: {}", m),
                    None => {
                        println!("Channel closed");
                        break;
                    }
                }
            }
        }
    }
}

// =============================================================================
// 2. JOIN OPERATIONS - Waiting for multiple futures to complete
// =============================================================================

async fn fetch_user(id: u32) -> Result<String, &'static str> {
    sleep(Duration::from_millis(100 * id as u64)).await;
    Ok(format!("User {}", id))
}

async fn fetch_posts(user_id: u32) -> Result<Vec<String>, &'static str> {
    sleep(Duration::from_millis(150)).await;
    Ok(vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ])
}

async fn join_example() {
    println!("\n=== JOIN OPERATIONS ===");
    
    // tokio::join! waits for all futures to complete
    let (user_result, posts_result) = tokio::join!(
        fetch_user(1),
        fetch_posts(1)
    );
    
    println!("User: {:?}", user_result);
    println!("Posts: {:?}", posts_result);
    
    // try_join! short-circuits on first error
    let results = tokio::try_join!(
        fetch_user(2),
        fetch_posts(2)
    );
    
    match results {
        Ok((user, posts)) => {
            println!("Success - User: {}, Posts: {:?}", user, posts);
        }
        Err(e) => println!("Error: {}", e),
    }
}

// =============================================================================
// 3. CANCELLATION - Properly handling task cancellation
// =============================================================================

async fn cancellable_work(id: u32) {
    println!("Task {} starting", id);
    
    // This demonstrates graceful cancellation
    let result = tokio::select! {
        _ = sleep(Duration::from_secs(10)) => {
            println!("Task {} completed normally", id);
            "completed"
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Task {} cancelled by signal", id);
            "cancelled"
        }
    };
    
    // Cleanup code here
    println!("Task {} cleaning up: {}", id, result);
}

async fn cancellation_example() {
    println!("\n=== CANCELLATION ===");
    
    let handle = tokio::spawn(async {
        for i in 0..5 {
            println!("Working... step {}", i);
            sleep(Duration::from_millis(500)).await;
        }
        "Task completed"
    });
    
    // Cancel after 1.2 seconds
    sleep(Duration::from_millis(1200)).await;
    handle.abort();
    
    match handle.await {
        Ok(result) => println!("Result: {}", result),
        Err(e) if e.is_cancelled() => println!("Task was cancelled"),
        Err(e) => println!("Task panicked: {:?}", e),
    }
}

// =============================================================================
// 4. TIMEOUTS - Setting time limits on async operations
// =============================================================================

async fn slow_operation(delay_ms: u64) -> Result<String, &'static str> {
    sleep(Duration::from_millis(delay_ms)).await;
    Ok("Operation completed".to_string())
}

async fn timeout_example() {
    println!("\n=== TIMEOUTS ===");
    
    // Success case - completes before timeout
    match timeout(Duration::from_secs(2), slow_operation(500)).await {
        Ok(Ok(result)) => println!("Success: {}", result),
        Ok(Err(e)) => println!("Operation error: {}", e),
        Err(_) => println!("Timeout!"),
    }
    
    // Timeout case - takes too long
    match timeout(Duration::from_millis(200), slow_operation(1000)).await {
        Ok(Ok(result)) => println!("Success: {}", result),
        Ok(Err(e)) => println!("Operation error: {}", e),
        Err(_) => println!("Timeout occurred!"),
    }
}

// =============================================================================
// 5. BACKPRESSURE - Managing flow control
// =============================================================================

async fn producer(tx: mpsc::Sender<u32>, rate_limiter: Arc<Semaphore>) {
    for i in 0..10 {
        // Acquire permit before sending (backpressure mechanism)
        let _permit = rate_limiter.acquire().await.unwrap();
        
        match tx.send(i).await {
            Ok(_) => println!("Produced: {}", i),
            Err(_) => {
                println!("Consumer dropped, stopping producer");
                break;
            }
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}

async fn consumer(mut rx: mpsc::Receiver<u32>, rate_limiter: Arc<Semaphore>) {
    while let Some(item) = rx.recv().await {
        println!("  Consuming: {}", item);
        
        // Simulate slow processing
        sleep(Duration::from_millis(300)).await;
        
        println!("  Processed: {}", item);
        
        // Release permit after processing (allow producer to continue)
        rate_limiter.add_permits(1);
    }
}

async fn backpressure_example() {
    println!("\n=== BACKPRESSURE ===");
    
    // Bounded channel provides natural backpressure
    let (tx, rx) = mpsc::channel::<u32>(3);
    
    // Semaphore for additional rate limiting
    let rate_limiter = Arc::new(Semaphore::new(3));
    
    let producer_limiter = Arc::clone(&rate_limiter);
    let consumer_limiter = Arc::clone(&rate_limiter);
    
    let producer_handle = tokio::spawn(producer(tx, producer_limiter));
    let consumer_handle = tokio::spawn(consumer(rx, consumer_limiter));
    
    // Wait for both to complete
    let _ = tokio::join!(producer_handle, consumer_handle);
}

// =============================================================================
// 6. STRUCTURED CONCURRENCY - Managing task lifetimes
// =============================================================================

struct TaskGroup {
    handles: Vec<tokio::task::JoinHandle<Result<String, String>>>,
}

impl TaskGroup {
    fn new() -> Self {
        Self { handles: Vec::new() }
    }
    
    fn spawn<F>(&mut self, task: F)
    where
        F: std::future::Future<Output = Result<String, String>> + Send + 'static,
    {
        self.handles.push(tokio::spawn(task));
    }
    
    async fn join_all(self) -> Vec<Result<String, String>> {
        let mut results = Vec::new();
        
        for handle in self.handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(format!("Task panicked: {:?}", e))),
            }
        }
        
        results
    }
}

async fn structured_concurrency_example() {
    println!("\n=== STRUCTURED CONCURRENCY ===");
    
    let mut group = TaskGroup::new();
    
    // Spawn related tasks in a group
    group.spawn(async {
        sleep(Duration::from_millis(100)).await;
        Ok("Task 1 completed".to_string())
    });
    
    group.spawn(async {
        sleep(Duration::from_millis(200)).await;
        Ok("Task 2 completed".to_string())
    });
    
    group.spawn(async {
        sleep(Duration::from_millis(150)).await;
        Err("Task 3 failed".to_string())
    });
    
    // All tasks complete before moving forward
    let results = group.join_all().await;
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(msg) => println!("Task {} result: {}", i, msg),
            Err(e) => println!("Task {} error: {}", i, e),
        }
    }
}

// =============================================================================
// 7. COMBINING PATTERNS - Real-world example
// =============================================================================

async fn fetch_with_retry_and_timeout(
    url: &str,
    max_retries: u32,
    timeout_duration: Duration,
) -> Result<String, String> {
    for attempt in 0..max_retries {
        println!("Attempt {} to fetch {}", attempt + 1, url);
        
        let fetch_future = async {
            // Simulate network request
            sleep(Duration::from_millis(500)).await;
            
            if attempt < 2 {
                Err("Network error")
            } else {
                Ok(format!("Data from {}", url))
            }
        };
        
        match timeout(timeout_duration, fetch_future).await {
            Ok(Ok(data)) => return Ok(data),
            Ok(Err(e)) => {
                println!("  Fetch failed: {}", e);
                if attempt < max_retries - 1 {
                    sleep(Duration::from_millis(200)).await;
                }
            }
            Err(_) => {
                println!("  Timeout!");
                if attempt < max_retries - 1 {
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }
    
    Err(format!("Failed after {} attempts", max_retries))
}

async fn combined_patterns_example() {
    println!("\n=== COMBINED PATTERNS ===");
    
    let result = fetch_with_retry_and_timeout(
        "https://api.example.com/data",
        3,
        Duration::from_secs(1),
    )
    .await;
    
    match result {
        Ok(data) => println!("Successfully fetched: {}", data),
        Err(e) => println!("Failed to fetch: {}", e),
    }
}

// =============================================================================
// MAIN - Run all examples
// =============================================================================

#[tokio::main]
async fn main() {
    println!("Rust Async Patterns and Best Practices\n");
    println!("=========================================");
    
    select_example().await;
    join_example().await;
    cancellation_example().await;
    timeout_example().await;
    backpressure_example().await;
    structured_concurrency_example().await;
    combined_patterns_example().await;
    
    println!("\n=========================================");
    println!("All examples completed!");
}

// =============================================================================
// KEY TAKEAWAYS AND BEST PRACTICES
// =============================================================================

/*
BEST PRACTICES SUMMARY:

1. SELECT OPERATIONS:
   - Use select! when you need to wait on multiple operations
   - The first branch to complete wins
   - Use biased select! for priority ordering
   - Remember to handle all branches, including cleanup

2. JOIN OPERATIONS:
   - Use join! to wait for multiple futures concurrently
   - Use try_join! when you want early termination on error
   - Prefer join! over spawning tasks for structured concurrency
   - Join operations are cancel-safe

3. CANCELLATION:
   - Dropping a future cancels it immediately
   - Use select! with cancellation tokens for graceful shutdown
   - Always implement cleanup logic
   - Be aware of cancel-safety (some futures can't be safely cancelled mid-execution)

4. TIMEOUTS:
   - Always set timeouts for external operations
   - Use timeout() wrapper from tokio::time
   - Consider implementing retry logic with exponential backoff
   - Handle timeout errors gracefully

5. BACKPRESSURE:
   - Use bounded channels (mpsc::channel with capacity)
   - Implement rate limiting with Semaphore
   - Monitor queue depths
   - Apply backpressure early in the pipeline

6. STRUCTURED CONCURRENCY:
   - Group related tasks together
   - Ensure all spawned tasks are awaited or explicitly detached
   - Use scoped tasks when possible
   - Avoid leaked tasks that outlive their parent

7. GENERAL TIPS:
   - Avoid blocking operations in async code
   - Use spawn_blocking for CPU-intensive work
   - Keep async functions small and focused
   - Profile and monitor your async code
   - Use tracing for observability
*/
```

