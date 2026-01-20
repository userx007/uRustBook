# Comprehensive Guide to `Mutex<T>` for Interior Mutability

`Mutex<T>` provides interior mutability with thread-safe synchronization, allowing multiple threads to safely share and mutate data. It's the thread-safe equivalent of `RefCell<T>`.

## What is `Mutex<T>`?

```rust
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}
```

`Mutex<T>` (mutual exclusion) provides exclusive access to data across threads, enforcing that only one thread can access the data at a time.

## Core Characteristics

- **Thread-safe** - implements `Send` and `Sync` (when `T: Send`)
- **Blocking** - threads wait when lock is held
- **Poisoning** - lock becomes poisoned if a thread panics while holding it
- **Runtime overhead** - OS-level synchronization primitives
- **Works with any type** - no `Copy` requirement

## Borrowing Rules (Enforced via Locking)

1. **Only one thread can hold the lock at a time**
2. **Lock must be released before another thread can acquire it**
3. **Deadlock possible** if not careful with multiple locks

## Core Use Cases

### 1. **Shared Mutable State Across Threads**

The most fundamental use case:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```

**Why Mutex?** Multiple threads need to mutate shared data safely.

### 2. **Thread Pool with Shared Work Queue**

Distributing work across threads:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

struct ThreadPool {
    work_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let work_queue = Arc::new(Mutex::new(VecDeque::new()));
        
        for _ in 0..size {
            let queue = Arc::clone(&work_queue);
            thread::spawn(move || {
                loop {
                    let work = {
                        let mut queue = queue.lock().unwrap();
                        queue.pop_front()
                    }; // Lock released here
                    
                    if let Some(job) = work {
                        job();
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        }
        
        Self { work_queue }
    }
    
    fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.work_queue.lock().unwrap().push_back(Box::new(job));
    }
}
```

### 3. **Shared Cache Across Threads**

Thread-safe caching:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn get_or_insert_with<F>(&self, key: K, f: F) -> V
    where
        F: FnOnce() -> V,
    {
        let mut cache = self.data.lock().unwrap();
        
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        
        let value = f();
        cache.insert(key.clone(), value.clone());
        value
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
```

### 4. **Concurrent Logging**

Thread-safe logger implementation:

```rust
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

struct Logger {
    file: Arc<Mutex<File>>,
}

impl Logger {
    fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            file: Arc::new(Mutex::new(File::create(path)?)),
        })
    }
    
    fn log(&self, message: &str) {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "[{}] {}", 
                 std::time::SystemTime::now()
                     .duration_since(std::time::UNIX_EPOCH)
                     .unwrap()
                     .as_secs(),
                 message).ok();
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            file: Arc::clone(&self.file),
        }
    }
}
```

### 5. **Metrics Collection**

Gathering statistics from multiple threads:

```rust
use std::sync::{Arc, Mutex};

struct Metrics {
    total_requests: Arc<Mutex<u64>>,
    total_errors: Arc<Mutex<u64>>,
    response_times: Arc<Mutex<Vec<u64>>>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            total_requests: Arc::new(Mutex::new(0)),
            total_errors: Arc::new(Mutex::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn record_request(&self, duration_ms: u64, is_error: bool) {
        *self.total_requests.lock().unwrap() += 1;
        
        if is_error {
            *self.total_errors.lock().unwrap() += 1;
        }
        
        self.response_times.lock().unwrap().push(duration_ms);
    }
    
    fn get_stats(&self) -> (u64, u64, f64) {
        let requests = *self.total_requests.lock().unwrap();
        let errors = *self.total_errors.lock().unwrap();
        
        let times = self.response_times.lock().unwrap();
        let avg = if times.is_empty() {
            0.0
        } else {
            times.iter().sum::<u64>() as f64 / times.len() as f64
        };
        
        (requests, errors, avg)
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            total_requests: Arc::clone(&self.total_requests),
            total_errors: Arc::clone(&self.total_errors),
            response_times: Arc::clone(&self.response_times),
        }
    }
}
```

### 6. **Connection Pool**

Managing a pool of reusable connections:

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct ConnectionPool {
    available: Arc<Mutex<VecDeque<Connection>>>,
    max_size: usize,
}

struct Connection {
    id: u32,
}

impl ConnectionPool {
    fn new(max_size: usize) -> Self {
        let mut pool = VecDeque::new();
        for id in 0..max_size {
            pool.push_back(Connection { id: id as u32 });
        }
        
        Self {
            available: Arc::new(Mutex::new(pool)),
            max_size,
        }
    }
    
    fn acquire(&self) -> Option<Connection> {
        self.available.lock().unwrap().pop_front()
    }
    
    fn release(&self, conn: Connection) {
        let mut pool = self.available.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(conn);
        }
    }
}
```

### 7. **Event Aggregator**

Collecting events from multiple sources:

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct Event {
    source: String,
    message: String,
    timestamp: u64,
}

struct EventAggregator {
    events: Arc<Mutex<Vec<Event>>>,
}

impl EventAggregator {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn record(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }
    
    fn get_events_from(&self, source: &str) -> Vec<Event> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.source == source)
            .cloned()
            .collect()
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            events: Arc::clone(&self.events),
        }
    }
}
```

### 8. **Rate Limiter**

Tracking request rates across threads:

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    max_requests: usize,
    window: Duration,
}

struct RateLimiterState {
    requests: Vec<Instant>,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                requests: Vec::new(),
            })),
            max_requests,
            window,
        }
    }
    
    fn try_acquire(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        
        // Remove old requests
        state.requests.retain(|&t| now.duration_since(t) < self.window);
        
        if state.requests.len() < self.max_requests {
            state.requests.push(now);
            true
        } else {
            false
        }
    }
}
```

### 9. **Shared Configuration**

Runtime-mutable configuration:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct Config {
    settings: Arc<Mutex<HashMap<String, String>>>,
}

impl Config {
    fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn set(&self, key: String, value: String) {
        self.settings.lock().unwrap().insert(key, value);
    }
    
    fn get(&self, key: &str) -> Option<String> {
        self.settings.lock().unwrap().get(key).cloned()
    }
    
    fn update_batch(&self, updates: HashMap<String, String>) {
        let mut settings = self.settings.lock().unwrap();
        for (k, v) in updates {
            settings.insert(k, v);
        }
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            settings: Arc::clone(&self.settings),
        }
    }
}
```

### 10. **Job Scheduler**

Scheduling and executing jobs:

```rust
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

struct ScheduledJob {
    run_at: Instant,
    job: Box<dyn FnOnce() + Send>,
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.run_at.cmp(&self.run_at)
    }
}

impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ScheduledJob {}
impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.run_at == other.run_at
    }
}

struct Scheduler {
    jobs: Arc<Mutex<BinaryHeap<ScheduledJob>>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }
    
    fn schedule<F>(&self, delay: Duration, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let run_at = Instant::now() + delay;
        self.jobs.lock().unwrap().push(ScheduledJob {
            run_at,
            job: Box::new(job),
        });
    }
}
```

### 11. **Database Transaction Manager**

Managing active transactions:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct TransactionManager {
    active_transactions: Arc<Mutex<HashMap<u64, Transaction>>>,
    next_id: Arc<Mutex<u64>>,
}

struct Transaction {
    id: u64,
    operations: Vec<String>,
}

impl TransactionManager {
    fn new() -> Self {
        Self {
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    fn begin(&self) -> u64 {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        
        self.active_transactions.lock().unwrap().insert(
            id,
            Transaction {
                id,
                operations: Vec::new(),
            },
        );
        
        id
    }
    
    fn add_operation(&self, tx_id: u64, operation: String) {
        let mut transactions = self.active_transactions.lock().unwrap();
        if let Some(tx) = transactions.get_mut(&tx_id) {
            tx.operations.push(operation);
        }
    }
    
    fn commit(&self, tx_id: u64) -> Option<Vec<String>> {
        self.active_transactions
            .lock()
            .unwrap()
            .remove(&tx_id)
            .map(|tx| tx.operations)
    }
}
```

### 12. **Buffered Writer**

Accumulating writes before flushing:

```rust
use std::sync::{Arc, Mutex};
use std::io::Write;

struct BufferedWriter<W: Write> {
    writer: Arc<Mutex<W>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    buffer_size: usize,
}

impl<W: Write> BufferedWriter<W> {
    fn new(writer: W, buffer_size: usize) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_size,
        }
    }
    
    fn write(&self, data: &[u8]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(data);
        
        if buffer.len() >= self.buffer_size {
            self.flush_internal(&mut buffer);
        }
    }
    
    fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        self.flush_internal(&mut buffer);
    }
    
    fn flush_internal(&self, buffer: &mut Vec<u8>) {
        if !buffer.is_empty() {
            let mut writer = self.writer.lock().unwrap();
            writer.write_all(buffer).ok();
            buffer.clear();
        }
    }
}
```

### 13. **Publish-Subscribe System**

Multi-threaded event distribution:

```rust
use std::sync::{Arc, Mutex};

type Subscriber = Box<dyn Fn(&str) + Send + 'static>;

struct PubSub {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

impl PubSub {
    fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        self.subscribers.lock().unwrap().push(Box::new(callback));
    }
    
    fn publish(&self, message: &str) {
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            subscriber(message);
        }
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}
```

### 14. **Progress Tracker**

Monitoring progress across multiple workers:

```rust
use std::sync::{Arc, Mutex};

struct ProgressTracker {
    completed: Arc<Mutex<usize>>,
    total: usize,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        Self {
            completed: Arc::new(Mutex::new(0)),
            total,
        }
    }
    
    fn increment(&self) {
        *self.completed.lock().unwrap() += 1;
    }
    
    fn get_progress(&self) -> (usize, usize) {
        let completed = *self.completed.lock().unwrap();
        (completed, self.total)
    }
    
    fn percentage(&self) -> f64 {
        let completed = *self.completed.lock().unwrap();
        (completed as f64 / self.total as f64) * 100.0
    }
    
    fn clone_handle(&self) -> Self {
        Self {
            completed: Arc::clone(&self.completed),
            total: self.total,
        }
    }
}
```

### 15. **Lazy Initialization (thread-safe)**

Initialize once across threads:

```rust
use std::sync::{Arc, Mutex};

struct LazyInit<T> {
    data: Arc<Mutex<Option<T>>>,
}

impl<T> LazyInit<T> {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
    }
    
    fn get_or_init<F>(&self, init: F) -> T
    where
        F: FnOnce() -> T,
        T: Clone,
    {
        let mut data = self.data.lock().unwrap();
        
        if let Some(ref value) = *data {
            return value.clone();
        }
        
        let value = init();
        *data = Some(value.clone());
        value
    }
}
```

## Key Methods

### `lock()` - Acquire the lock
```rust
let mutex = Mutex::new(5);
let mut guard = mutex.lock().unwrap();
*guard += 1;
// Guard dropped here, lock released
```

### `try_lock()` - Non-blocking lock attempt
```rust
let mutex = Mutex::new(5);

match mutex.try_lock() {
    Ok(mut guard) => {
        *guard += 1;
    }
    Err(TryLockError::WouldBlock) => {
        println!("Lock is held by another thread");
    }
    Err(TryLockError::Poisoned(_)) => {
        println!("Lock is poisoned");
    }
}
```

### `into_inner()` - Consume mutex and return value
```rust
let mutex = Mutex::new(10);
let value = mutex.into_inner().unwrap(); // value = 10
```

### `get_mut()` - Mutable reference (exclusive access)
```rust
let mut mutex = Mutex::new(5);
*mutex.get_mut().unwrap() = 10; // No locking needed
```

### `is_poisoned()` - Check if poisoned
```rust
let mutex = Mutex::new(5);
if mutex.is_poisoned() {
    println!("Mutex is poisoned");
}
```

## Handling Lock Poisoning

### What is poisoning?
When a thread panics while holding a lock, the mutex becomes "poisoned" to signal potential data corruption.

### Recovering from poisoning
```rust
let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
let mutex_clone = Arc::clone(&mutex);

let handle = thread::spawn(move || {
    let mut guard = mutex_clone.lock().unwrap();
    guard.push(4);
    panic!("Thread panicked!");
});

handle.join().ok();

// Mutex is now poisoned
match mutex.lock() {
    Ok(guard) => println!("Lock acquired: {:?}", *guard),
    Err(poisoned) => {
        println!("Lock is poisoned, recovering...");
        let guard = poisoned.into_inner();
        println!("Data: {:?}", *guard);
    }
}
```

### Ignoring poisoning
```rust
let mutex = Mutex::new(5);
let guard = mutex.lock().unwrap_or_else(|poisoned| {
    poisoned.into_inner()
});
```

## Common Deadlock Scenarios

### ❌ Deadlock: Lock ordering
```rust
let mutex1 = Arc::new(Mutex::new(1));
let mutex2 = Arc::new(Mutex::new(2));

// Thread 1: locks mutex1, then mutex2
// Thread 2: locks mutex2, then mutex1
// DEADLOCK!
```

### ✅ Solution: Consistent lock ordering
```rust
// Always lock mutex1 before mutex2
let _guard1 = mutex1.lock().unwrap();
let _guard2 = mutex2.lock().unwrap();
```

### ❌ Deadlock: Recursive locking
```rust
let mutex = Arc::new(Mutex::new(5));
let _guard1 = mutex.lock().unwrap();
let _guard2 = mutex.lock().unwrap(); // DEADLOCK! Same thread
```

### ✅ Solution: Drop first guard or use RwLock
```rust
{
    let _guard1 = mutex.lock().unwrap();
    // Use guard1
} // Dropped

let _guard2 = mutex.lock().unwrap(); // OK
```

## Performance Considerations

### Minimize critical sections
```rust
// ❌ Bad: Long critical section
let mut data = mutex.lock().unwrap();
expensive_computation();
data.push(result);

// ✅ Good: Short critical section
let result = expensive_computation();
let mut data = mutex.lock().unwrap();
data.push(result);
```

### Avoid holding locks across await points
```rust
// ❌ Bad: Holding lock across await
let data = mutex.lock().unwrap();
async_operation().await; // Lock held during await!

// ✅ Good: Release before await
let value = {
    let data = mutex.lock().unwrap();
    data.clone()
};
async_operation().await;
```

### Clone data out when possible
```rust
// Instead of holding the lock:
let value = {
    let data = mutex.lock().unwrap();
    data.clone()
}; // Lock released

// Use value...
```

## When NOT to Use `Mutex<T>`

### 1. **Single-threaded Context**
Use `RefCell<T>` instead:
```rust
// Single thread:
let data = RefCell::new(vec![1, 2, 3]);
data.borrow_mut().push(4);
```

### 2. **Read-Heavy Workloads**
Use `RwLock<T>` for better read concurrency:
```rust
use std::sync::RwLock;

let lock = RwLock::new(5);
let r1 = lock.read().unwrap(); // Multiple readers OK
let r2 = lock.read().unwrap();
```

### 3. **Atomic Operations on Primitives**
Use `AtomicT` types:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

let counter = AtomicU64::new(0);
counter.fetch_add(1, Ordering::SeqCst); // No lock needed
```

### 4. **Message Passing is Better**
Use channels instead:
```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();
// Send messages instead of shared state
```

### 5. **Lock-Free Algorithms Available**
Consider lock-free data structures when applicable.

## Mutex vs RefCell vs RwLock vs Atomic

| Feature | `Mutex<T>` | `RefCell<T>` | `RwLock<T>` | `AtomicT` |
|---------|------------|--------------|-------------|-----------|
| Thread-safe | Yes | No | Yes | Yes |
| Multiple readers | No | Yes | Yes | N/A |
| Type requirement | Any | Any | Any | Specific types |
| Runtime cost | High (OS lock) | Low | Medium-High | Low |
| Can deadlock | Yes | No | Yes | No |
| Poisoning | Yes | No | Yes | No |

## Best Practices

### 1. **Always use Arc with Mutex for sharing**
```rust
let data = Arc::new(Mutex::new(vec![]));
let data_clone = Arc::clone(&data);
```

### 2. **Keep critical sections short**
```rust
let result = {
    let guard = mutex.lock().unwrap();
    guard.clone()
}; // Lock released immediately
```

### 3. **Use try_lock when appropriate**
```rust
if let Ok(mut guard) = mutex.try_lock() {
    // Got the lock
} else {
    // Do something else
}
```

### 4. **Document lock ordering to prevent deadlocks**
```rust
// Always lock in this order: user_lock -> account_lock
```

### 5. **Consider alternatives**
- Message passing for communication
- `RwLock` for read-heavy workloads
- `Atomic` types for simple counters/flags

## Common Patterns

### Scoped locking
```rust
{
    let mut data = mutex.lock().unwrap();
    data.push(value);
} // Lock automatically released
```

### Lock and clone pattern
```rust
let snapshot = mutex.lock().unwrap().clone();
// Work with snapshot, no lock held
```

### Conditional mutation
```rust
let mut guard = mutex.lock().unwrap();
if guard.len() < MAX_SIZE {
    guard.push(item);
}
```

### Swap pattern
```rust
let new_data = compute_new_data();
let old_data = std::mem::replace(&mut *mutex.lock().unwrap(), new_data);
```

`Mutex<T>` is essential for thread-safe interior mutability, enabling safe concurrent access to shared mutable state. Use it when multiple threads need to mutate shared data, but always consider the performance implications and whether simpler alternatives like message passing or atomic operations might be more appropriate.