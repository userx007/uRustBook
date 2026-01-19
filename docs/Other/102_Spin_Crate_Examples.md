# Set of examples using the `spin` crate 

Here's what each example demonstrates:

1. **Basic SpinMutex** - Simple counter increment showing RAII-style locking
2. **Complex Data Structures** - Producer-consumer pattern with a shared task queue
3. **SpinRwLock** - Multiple readers and single writer scenario with configuration data
4. **try_lock()** - Non-blocking lock acquisition for scenarios where you can do other work
5. **Lock Contention Comparison** - Performance comparison showing when spin locks shine (low contention) vs struggle (high contention)

Key advantages of the `spin` crate over the manual implementation:
- **RAII guards** automatically unlock when they go out of scope (prevents forgetting to unlock)
- **RwLock support** for reader-writer scenarios
- **try_lock()** for non-blocking attempts
- **Battle-tested** and optimized implementation
- **Clean API** that feels like std::sync::Mutex

To run this, add to your `Cargo.toml`:
```toml
[dependencies]
spin = "0.9"
```

The examples show practical use cases and best practices for when spin locks are appropriate (short critical sections with low contention) versus when they might cause performance issues (long critical sections with high contention).

```rust
use std::sync::Arc;
use std::thread;
use spin::{Mutex as SpinMutex, RwLock as SpinRwLock};

/// Example 1: Basic SpinMutex usage with a counter
/// Demonstrates RAII-style locking where the lock is automatically released
fn basic_spin_mutex_example() {
    println!("=== Basic SpinMutex Example ===");
    
    // Create a counter protected by a spin lock
    let counter = Arc::new(SpinMutex::new(0));
    let mut handles = vec![];
    
    // Spawn 4 threads that increment the counter
    for thread_id in 0..4 {
        let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            for _ in 0..10_000 {
                // lock() returns a MutexGuard that derefs to the inner value
                // The guard automatically unlocks when it goes out of scope
                let mut num = counter_clone.lock();
                *num += 1;
                // Lock is automatically released here when 'num' goes out of scope
            }
            println!("Thread {} finished", thread_id);
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Should print 40,000 (4 threads × 10,000 increments)
    println!("Final counter: {}\n", *counter.lock());
}

/// Example 2: Using SpinMutex with complex data structures
/// Shows how to protect shared state beyond simple counters
fn complex_data_example() {
    println!("=== Complex Data Example ===");
    
    // A shared task queue protected by a spin lock
    #[derive(Debug, Clone)]
    struct Task {
        id: u32,
        description: String,
    }
    
    let task_queue = Arc::new(SpinMutex::new(Vec::<Task>::new()));
    let mut handles = vec![];
    
    // Producer threads: add tasks to the queue
    for producer_id in 0..2 {
        let queue_clone = Arc::clone(&task_queue);
        
        let handle = thread::spawn(move || {
            for i in 0..5 {
                let task = Task {
                    id: producer_id * 100 + i,
                    description: format!("Task from producer {}", producer_id),
                };
                
                // Acquire lock, push task, lock automatically released
                queue_clone.lock().push(task);
                
                // Small delay to make the output more interesting
                thread::sleep(std::time::Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }
    
    // Consumer thread: process tasks from the queue
    let queue_consumer = Arc::clone(&task_queue);
    let consumer_handle = thread::spawn(move || {
        let mut processed = 0;
        while processed < 10 {
            // Try to pop a task from the queue
            let task = queue_consumer.lock().pop();
            
            if let Some(task) = task {
                println!("Processed: {:?}", task);
                processed += 1;
            } else {
                // Queue is empty, yield CPU briefly
                thread::sleep(std::time::Duration::from_micros(50));
            }
        }
    });
    handles.push(consumer_handle);
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Remaining tasks in queue: {}\n", task_queue.lock().len());
}

/// Example 3: SpinRwLock for multiple readers, single writer
/// RwLocks allow multiple concurrent readers or one exclusive writer
fn read_write_lock_example() {
    println!("=== SpinRwLock Example ===");
    
    // Shared configuration that's read often but written rarely
    #[derive(Debug, Clone)]
    struct Config {
        timeout_ms: u64,
        max_connections: usize,
        debug_mode: bool,
    }
    
    let config = Arc::new(SpinRwLock::new(Config {
        timeout_ms: 1000,
        max_connections: 100,
        debug_mode: false,
    }));
    
    let mut handles = vec![];
    
    // Spawn multiple reader threads
    for reader_id in 0..3 {
        let config_clone = Arc::clone(&config);
        
        let handle = thread::spawn(move || {
            for _ in 0..5 {
                // read() acquires a shared read lock
                // Multiple threads can hold read locks simultaneously
                let cfg = config_clone.read();
                println!(
                    "Reader {} sees: timeout={}ms, max_conn={}, debug={}",
                    reader_id, cfg.timeout_ms, cfg.max_connections, cfg.debug_mode
                );
                // Read lock automatically released here
                
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    // Spawn a writer thread that updates configuration
    let config_writer = Arc::clone(&config);
    let writer_handle = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(20));
        
        // write() acquires an exclusive write lock
        // This blocks until all readers have released their locks
        let mut cfg = config_writer.write();
        println!("Writer: Updating configuration...");
        cfg.timeout_ms = 2000;
        cfg.max_connections = 200;
        cfg.debug_mode = true;
        // Write lock automatically released here
        
        println!("Writer: Configuration updated!");
    });
    handles.push(writer_handle);
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final config: {:?}\n", *config.read());
}

/// Example 4: try_lock() for non-blocking lock acquisition
/// Useful when you want to attempt to acquire a lock without spinning
fn try_lock_example() {
    println!("=== try_lock() Example ===");
    
    let counter = Arc::new(SpinMutex::new(0));
    let mut handles = vec![];
    
    for thread_id in 0..3 {
        let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            let mut successful_locks = 0;
            let mut failed_attempts = 0;
            
            for _ in 0..1000 {
                // try_lock() returns Some(guard) if lock is acquired,
                // None if the lock is currently held by another thread
                match counter_clone.try_lock() {
                    Some(mut num) => {
                        *num += 1;
                        successful_locks += 1;
                        // Lock automatically released when guard drops
                    }
                    None => {
                        // Lock was busy, do something else instead of spinning
                        failed_attempts += 1;
                    }
                }
            }
            
            println!(
                "Thread {}: {} successful locks, {} failed attempts",
                thread_id, successful_locks, failed_attempts
            );
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final counter: {}\n", *counter.lock());
}

/// Example 5: Comparison of lock contention
/// Shows the difference between high and low contention scenarios
fn lock_contention_comparison() {
    println!("=== Lock Contention Comparison ===");
    
    // High contention: very short critical section
    let start = std::time::Instant::now();
    let counter_high = Arc::new(SpinMutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..4 {
        let counter_clone = Arc::clone(&counter_high);
        let handle = thread::spawn(move || {
            for _ in 0..50_000 {
                *counter_clone.lock() += 1;
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!(
        "High contention (spin lock): {}ms for {} operations",
        start.elapsed().as_millis(),
        *counter_high.lock()
    );
    
    // Low contention: work done outside the critical section
    let start = std::time::Instant::now();
    let counter_low = Arc::new(SpinMutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..4 {
        let counter_clone = Arc::clone(&counter_low);
        let handle = thread::spawn(move || {
            let mut local_sum = 0;
            for _ in 0..50_000 {
                // Do work outside the lock
                local_sum += 1;
            }
            // Only acquire lock once to update shared state
            *counter_clone.lock() += local_sum;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!(
        "Low contention (batch updates): {}ms for {} operations",
        start.elapsed().as_millis(),
        *counter_low.lock()
    );
}

fn main() {
    basic_spin_mutex_example();
    complex_data_example();
    read_write_lock_example();
    try_lock_example();
    lock_contention_comparison();
    
    println!("=== Summary ===");
    println!("SpinMutex: Best for very short critical sections with low contention");
    println!("SpinRwLock: Best when reads greatly outnumber writes");
    println!("try_lock(): Best when you can do useful work if lock is unavailable");
}
```