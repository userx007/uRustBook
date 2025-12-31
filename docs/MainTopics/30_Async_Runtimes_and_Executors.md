# Async Runtimes and Executors in Rust

## **What Are Async Runtimes?**

Rust's `async`/`await` is just syntax - you need a runtime to actually execute async code. Runtimes poll futures and drive them to completion. They're not part of the standard library, so you choose one.

## **Key Concepts Covered:**

### **1. Tokio**
- The most popular runtime with a mature ecosystem
- Multi-threaded by default with work-stealing scheduler
- Rich feature set: timers, I/O, synchronization primitives
- Use `#[tokio::main]` macro for easy setup

### **2. async-std**
- Designed to mirror the standard library's API
- More intuitive for those familiar with std
- Smaller ecosystem but simpler conceptual model

### **3. Single vs Multi-threaded**
- **Single-threaded**: One thread runs all tasks concurrently. Lower overhead, no `Send` requirement, good for I/O-bound work
- **Multi-threaded**: Tasks distributed across worker threads. Can utilize multiple cores, requires `Send`, better for mixed workloads

### **4. Work Stealing**
A key optimization in multi-threaded runtimes:
- Each worker thread has its own task queue
- Idle workers "steal" tasks from busy workers
- Provides automatic load balancing
- Reduces contention while maintaining cache locality

### **5. Custom Executors**
The code shows how to build a basic executor from scratch, demonstrating:
- Task queuing
- Future polling with `Context` and `Waker`
- The complexity that production runtimes handle for you

## **When to Use What:**

- **Tokio multi-threaded**: Network services, high-performance servers, mixed workloads
- **Tokio current_thread**: Simple apps, embedded systems, pure I/O
- **async-std**: Learning async Rust, projects wanting std-like APIs
- **Custom**: Educational purposes or very specific requirements

The examples include practical demonstrations of spawning tasks, runtime configuration, blocking operations, and performance considerations!

```rust
// ========================================
// ASYNC RUNTIMES AND EXECUTORS IN RUST
// ========================================

// Rust's async/await syntax is just syntax sugar. The actual execution
// of async code requires a runtime/executor to poll futures and drive them to completion.

// ========================================
// 1. TOKIO - The Most Popular Runtime
// ========================================

// Tokio is a multi-threaded, work-stealing runtime for async Rust.
// Add to Cargo.toml: tokio = { version = "1", features = ["full"] }

use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn tokio_basic_example() {
    println!("Starting Tokio example...");
    
    // Spawn multiple tasks that run concurrently
    let task1 = tokio::spawn(async {
        sleep(Duration::from_millis(100)).await;
        println!("Task 1 completed");
        42
    });
    
    let task2 = tokio::spawn(async {
        sleep(Duration::from_millis(50)).await;
        println!("Task 2 completed");
        100
    });
    
    // Wait for both tasks
    let (result1, result2) = tokio::join!(task1, task2);
    println!("Results: {:?}, {:?}", result1, result2);
}

// Tokio runtime configurations
async fn tokio_runtime_configurations() {
    // Multi-threaded runtime (default)
    let multi_threaded = tokio::runtime::Runtime::new().unwrap();
    
    // Single-threaded runtime (current_thread)
    let single_threaded = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    // Custom multi-threaded runtime with specific worker threads
    let custom = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-custom-worker")
        .enable_all()
        .build()
        .unwrap();
    
    custom.block_on(async {
        println!("Running on custom runtime with 4 workers");
        sleep(Duration::from_millis(10)).await;
    });
}

// ========================================
// 2. ASYNC-STD - Alternative Runtime
// ========================================

// async-std aims to be a "async version" of the standard library
// Add to Cargo.toml: async-std = { version = "1", features = ["attributes"] }

use async_std::task;
use async_std::io;

#[async_std::main]
async fn async_std_example() {
    println!("Starting async-std example...");
    
    let task1 = task::spawn(async {
        task::sleep(Duration::from_millis(100)).await;
        println!("async-std task 1");
        42
    });
    
    let task2 = task::spawn(async {
        task::sleep(Duration::from_millis(50)).await;
        println!("async-std task 2");
        100
    });
    
    let results = futures::join!(task1, task2);
    println!("async-std results: {:?}", results);
}

// ========================================
// 3. CUSTOM EXECUTOR - Building from Scratch
// ========================================

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Wake, Waker};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// Simple task that can be queued
type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

// Basic single-threaded executor
struct SimpleExecutor {
    queue: VecDeque<Task>,
}

impl SimpleExecutor {
    fn new() -> Self {
        SimpleExecutor {
            queue: VecDeque::new(),
        }
    }
    
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static + Send) {
        self.queue.push_back(Box::pin(future));
    }
    
    fn run(&mut self) {
        while let Some(mut task) = self.queue.pop_front() {
            // Create a waker that does nothing (for simplicity)
            let waker = Arc::new(DummyWaker).into();
            let mut context = Context::from_waker(&waker);
            
            // Poll the future
            match task.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    // Task completed
                }
                Poll::Pending => {
                    // Task not ready, put it back in queue
                    self.queue.push_back(task);
                }
            }
        }
    }
}

struct DummyWaker;

impl Wake for DummyWaker {
    fn wake(self: Arc<Self>) {}
}

// More sophisticated executor with proper waking
struct BetterExecutor {
    tasks: Arc<Mutex<VecDeque<Task>>>,
}

impl BetterExecutor {
    fn new() -> Self {
        BetterExecutor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        self.tasks.lock().unwrap().push_back(Box::pin(future));
    }
    
    fn run(&self) {
        loop {
            let task = {
                let mut tasks = self.tasks.lock().unwrap();
                tasks.pop_front()
            };
            
            let Some(mut task) = task else {
                break;
            };
            
            let tasks_clone = self.tasks.clone();
            let waker = Arc::new(TaskWaker { 
                task: Mutex::new(Some(task)),
                tasks: tasks_clone,
            }).into();
            
            let mut context = Context::from_waker(&waker);
            let task_mut = Arc::get_mut(&mut Arc::try_unwrap(
                waker.clone().into()
            ).unwrap().downcast::<TaskWaker>().unwrap())
                .unwrap()
                .task
                .get_mut()
                .unwrap()
                .as_mut()
                .unwrap();
                
            // Simplified polling - real implementation would be more complex
        }
    }
}

struct TaskWaker {
    task: Mutex<Option<Task>>,
    tasks: Arc<Mutex<VecDeque<Task>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        if let Some(task) = self.task.lock().unwrap().take() {
            self.tasks.lock().unwrap().push_back(task);
        }
    }
}

// ========================================
// 4. SINGLE VS MULTI-THREADED RUNTIMES
// ========================================

async fn demonstrate_runtime_types() {
    // SINGLE-THREADED RUNTIME
    // - All tasks run on one thread
    // - No Send requirement for futures
    // - Lower overhead, simpler
    // - Good for I/O-bound workloads with no parallelism
    
    let single = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    single.block_on(async {
        let start = Instant::now();
        
        // These run concurrently but on same thread
        let t1 = tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            println!("Single-threaded task 1: {:?}", Instant::now().duration_since(start));
        });
        
        let t2 = tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            println!("Single-threaded task 2: {:?}", Instant::now().duration_since(start));
        });
        
        let _ = tokio::join!(t1, t2);
    });
    
    // MULTI-THREADED RUNTIME
    // - Tasks distributed across worker threads
    // - Futures must implement Send
    // - Can utilize multiple CPU cores
    // - Work stealing for load balancing
    // - Good for CPU-bound or mixed workloads
    
    let multi = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    
    multi.block_on(async {
        let start = Instant::now();
        
        // These can run on different threads
        let t1 = tokio::spawn(async {
            // Simulate CPU work
            let mut sum = 0u64;
            for i in 0..10_000_000 {
                sum = sum.wrapping_add(i);
            }
            println!("Multi-threaded task 1: {:?}, sum={}", 
                     Instant::now().duration_since(start), sum);
        });
        
        let t2 = tokio::spawn(async {
            let mut sum = 0u64;
            for i in 0..10_000_000 {
                sum = sum.wrapping_add(i);
            }
            println!("Multi-threaded task 2: {:?}, sum={}", 
                     Instant::now().duration_since(start), sum);
        });
        
        let _ = tokio::join!(t1, t2);
    });
}

// ========================================
// 5. WORK STEALING
// ========================================

// Work stealing is a scheduling algorithm used by multi-threaded runtimes
// Each worker thread has its own task queue (deque)
// When idle, workers "steal" tasks from other busy workers' queues

async fn demonstrate_work_stealing() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    
    runtime.block_on(async {
        println!("\n=== Work Stealing Demo ===");
        
        // Spawn many tasks - they'll be distributed across workers
        let mut handles = vec![];
        
        for i in 0..20 {
            let handle = tokio::spawn(async move {
                // Get current thread info
                let thread_id = std::thread::current().id();
                
                // Mix of I/O and CPU work
                if i % 2 == 0 {
                    sleep(Duration::from_millis(10)).await;
                } else {
                    let mut sum = 0u64;
                    for j in 0..1_000_000 {
                        sum = sum.wrapping_add(j);
                    }
                }
                
                println!("Task {} completed on thread {:?}", i, thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            let _ = handle.await;
        }
    });
    
    // Key Work Stealing Benefits:
    // 1. Load balancing - busy threads share work with idle ones
    // 2. Cache locality - threads work on their own queue first
    // 3. Reduced contention - stealing happens less frequently than pushing
    // 4. Better CPU utilization - no thread stays idle while others are busy
}

// ========================================
// 6. PRACTICAL COMPARISON
// ========================================

async fn practical_comparison() {
    println!("\n=== Tokio vs async-std Comparison ===\n");
    
    // Tokio approach
    println!("Tokio style:");
    let handle = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Tokio task done");
    });
    handle.await.unwrap();
    
    // async-std approach (more similar to sync std lib)
    println!("\nasync-std style:");
    let handle = async_std::task::spawn(async {
        async_std::task::sleep(Duration::from_millis(100)).await;
        println!("async-std task done");
    });
    handle.await;
}

// ========================================
// 7. CHOOSING THE RIGHT RUNTIME
// ========================================

// Use SINGLE-THREADED when:
// - Simple I/O-bound applications
// - Embedded systems or resource-constrained environments
// - Don't need parallelism
// - Want lowest overhead

// Use MULTI-THREADED when:
// - Need to utilize multiple CPU cores
// - Mix of CPU and I/O bound work
// - High concurrency requirements
// - Building servers or high-performance applications

// Use TOKIO when:
// - Need mature ecosystem (tracing, metrics, etc.)
// - Building network services
// - Want fine-grained control over runtime
// - Large community and extensive documentation

// Use ASYNC-STD when:
// - Want API similar to standard library
// - Simpler conceptual model
// - Cross-platform consistency
// - Educational purposes

fn main() {
    println!("=== Async Runtimes and Executors Examples ===\n");
    
    // Run examples
    // Note: In real code, you'd only have one runtime active at a time
    
    println!("1. Tokio Basic Example:");
    tokio::runtime::Runtime::new().unwrap().block_on(tokio_basic_example());
    
    println!("\n2. Runtime Types Demonstration:");
    tokio::runtime::Runtime::new().unwrap().block_on(demonstrate_runtime_types());
    
    println!("\n3. Work Stealing Demonstration:");
    tokio::runtime::Runtime::new().unwrap().block_on(demonstrate_work_stealing());
    
    println!("\n=== Summary ===");
    println!("- Executors poll futures to drive them to completion");
    println!("- Tokio: Production-ready, feature-rich, work-stealing multi-threaded");
    println!("- async-std: Ergonomic, std-lib-like API");
    println!("- Single-threaded: Lower overhead, simpler, I/O focused");
    println!("- Multi-threaded: Parallel execution, work stealing, CPU utilization");
    println!("- Work stealing: Automatic load balancing across worker threads");
}

// ========================================
// ADDITIONAL CONCEPTS
// ========================================

// Blocking Operations
async fn handling_blocking_operations() {
    // WRONG: This blocks the executor thread!
    // std::thread::sleep(Duration::from_secs(1));
    
    // RIGHT: Use spawn_blocking for CPU-intensive or blocking operations
    tokio::task::spawn_blocking(|| {
        // This runs on a separate thread pool
        std::thread::sleep(Duration::from_secs(1));
        // Heavy computation here
    }).await.unwrap();
}

// Runtime Handles
fn runtime_handle_example() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    
    // Can spawn tasks from sync context using handle
    std::thread::spawn(move || {
        handle.spawn(async {
            println!("Spawned from different thread!");
        });
    });
}

// Local Task Sets (for !Send futures in multi-threaded runtime)
async fn local_task_set_example() {
    let local = tokio::task::LocalSet::new();
    
    // Can run !Send futures
    local.run_until(async {
        tokio::task::spawn_local(async {
            // This doesn't need to be Send
            let rc = std::rc::Rc::new(42);
            println!("Value: {}", rc);
        }).await.unwrap();
    }).await;
}
```