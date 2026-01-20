# When `Arc<T>` is Needed in Rust

## Overview

`Arc<T>` (Atomically Reference Counted) provides **shared ownership** of heap-allocated data with **thread-safe** reference counting. It's the multi-threaded version of `Rc<T>`.

## 1. **Sharing Data Across Threads**

The fundamental use case: when multiple threads need to own the same data.

```rust
use std::sync::Arc;
use std::thread;

struct SharedData {
    config: String,
    values: Vec<i32>,
}

fn main() {
    let data = Arc::new(SharedData {
        config: "production".to_string(),
        values: vec![1, 2, 3, 4, 5],
    });

    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: config = {}", i, data_clone.config);
            println!("Thread {}: sum = {}", i, data_clone.values.iter().sum::<i32>());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 2. **Thread Pools and Worker Threads**

When workers need shared access to configuration, state, or work queues.

```rust
use std::sync::Arc;
use std::thread;

struct ThreadPoolConfig {
    max_workers: usize,
    timeout_ms: u64,
}

struct Worker {
    id: usize,
    config: Arc<ThreadPoolConfig>,
}

impl Worker {
    fn new(id: usize, config: Arc<ThreadPoolConfig>) -> Self {
        Worker { id, config }
    }
    
    fn start(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            println!(
                "Worker {} started with max_workers: {}",
                self.id, self.config.max_workers
            );
            // Do work...
        })
    }
}

fn main() {
    let config = Arc::new(ThreadPoolConfig {
        max_workers: 4,
        timeout_ms: 5000,
    });

    let mut handles = vec![];
    for i in 0..4 {
        let worker = Worker::new(i, Arc::clone(&config));
        handles.push(worker.start());
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 3. **Shared State with Mutex or RwLock**

When multiple threads need to read **and write** shared data.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct Counter {
    value: i32,
}

fn main() {
    let counter = Arc::new(Mutex::new(Counter { value: 0 }));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut counter = counter_clone.lock().unwrap();
            counter.value += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.lock().unwrap().value);
}
```

## 4. **Async/Await Contexts**

When sharing data across async tasks (which may run on different threads).

```rust
use std::sync::Arc;
use tokio::task;

struct Database {
    connection_string: String,
}

impl Database {
    async fn query(&self, sql: &str) -> String {
        format!("Querying '{}' on {}", sql, self.connection_string)
    }
}

#[tokio::main]
async fn main() {
    let db = Arc::new(Database {
        connection_string: "localhost:5432".to_string(),
    });

    let mut handles = vec![];

    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = task::spawn(async move {
            let result = db_clone.query(&format!("SELECT * FROM table_{}", i)).await;
            println!("{}", result);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

## 5. **Caching Across Threads**

Thread-safe caches where multiple threads share cached data.

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;

struct Cache {
    data: Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn get(&self, key: &str) -> Option<Arc<Vec<u8>>> {
        self.data.read().unwrap().get(key).cloned()
    }
    
    fn insert(&self, key: String, value: Vec<u8>) {
        self.data.write().unwrap().insert(key, Arc::new(value));
    }
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Cache {
            data: Arc::clone(&self.data),
        }
    }
}

fn main() {
    let cache = Cache::new();
    cache.insert("key1".to_string(), vec![1, 2, 3]);

    let mut handles = vec![];
    
    for i in 0..3 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            if let Some(data) = cache_clone.get("key1") {
                println!("Thread {}: {:?}", i, data);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 6. **Connection Pools**

Database or network connection pools shared across threads.

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct Connection {
    id: usize,
}

struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<Connection>>>,
    max_size: usize,
}

impl ConnectionPool {
    fn new(max_size: usize) -> Self {
        let mut connections = VecDeque::new();
        for i in 0..max_size {
            connections.push_back(Connection { id: i });
        }
        
        ConnectionPool {
            connections: Arc::new(Mutex::new(connections)),
            max_size,
        }
    }
    
    fn acquire(&self) -> Option<Connection> {
        self.connections.lock().unwrap().pop_front()
    }
    
    fn release(&self, conn: Connection) {
        self.connections.lock().unwrap().push_back(conn);
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool {
            connections: Arc::clone(&self.connections),
            max_size: self.max_size,
        }
    }
}
```

## 7. **Event Handlers / Callbacks in Multi-threaded Contexts**

When event handlers need to be called from multiple threads.

```rust
use std::sync::Arc;
use std::thread;

trait EventHandler: Send + Sync {
    fn handle(&self, event: &str);
}

struct Logger;

impl EventHandler for Logger {
    fn handle(&self, event: &str) {
        println!("Logged event: {}", event);
    }
}

struct EventBus {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus { handlers: vec![] }
    }
    
    fn register(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }
    
    fn emit(&self, event: &str) {
        for handler in &self.handlers {
            handler.handle(event);
        }
    }
}
```

## 8. **Actor Model / Message Passing**

Actors that need to share references to other actors.

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

struct Actor {
    id: String,
    receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<String>>>,
}

impl Actor {
    fn new(id: String) -> (Self, mpsc::Sender<String>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Actor {
                id,
                receiver: Arc::new(tokio::sync::Mutex::new(rx)),
            },
            tx,
        )
    }
    
    async fn run(self: Arc<Self>) {
        loop {
            let mut receiver = self.receiver.lock().await;
            if let Some(msg) = receiver.recv().await {
                println!("Actor {} received: {}", self.id, msg);
            } else {
                break;
            }
        }
    }
}
```

## 9. **Lazy Initialization Across Threads**

Using `Arc` with `Once` or `OnceLock` for thread-safe lazy initialization.

```rust
use std::sync::{Arc, OnceLock};

struct ExpensiveResource {
    data: Vec<u8>,
}

impl ExpensiveResource {
    fn new() -> Self {
        println!("Creating expensive resource...");
        ExpensiveResource {
            data: vec![0; 1_000_000],
        }
    }
}

static RESOURCE: OnceLock<Arc<ExpensiveResource>> = OnceLock::new();

fn get_resource() -> Arc<ExpensiveResource> {
    Arc::clone(RESOURCE.get_or_init(|| {
        Arc::new(ExpensiveResource::new())
    }))
}

fn main() {
    let handles: Vec<_> = (0..5)
        .map(|i| {
            std::thread::spawn(move || {
                let resource = get_resource();
                println!("Thread {} got resource with {} bytes", i, resource.data.len());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 10. **Sharing Immutable Application State**

Application-wide immutable configuration or state accessible from all threads.

```rust
use std::sync::Arc;
use std::thread;

struct AppConfig {
    app_name: String,
    version: String,
    max_connections: usize,
    features: Vec<String>,
}

struct Application {
    config: Arc<AppConfig>,
}

impl Application {
    fn new(config: AppConfig) -> Self {
        Application {
            config: Arc::new(config),
        }
    }
    
    fn spawn_worker(&self, id: usize) -> thread::JoinHandle<()> {
        let config = Arc::clone(&self.config);
        thread::spawn(move || {
            println!(
                "Worker {}: Running {} v{}",
                id, config.app_name, config.version
            );
        })
    }
}
```

## 11. **Rayon Parallel Iterators**

Sharing data across parallel iterations.

```rust
use std::sync::Arc;
use rayon::prelude::*;

struct LargeDataset {
    values: Vec<i32>,
}

fn main() {
    let dataset = Arc::new(LargeDataset {
        values: (0..1000).collect(),
    });

    let results: Vec<_> = (0..10)
        .into_par_iter()
        .map(|i| {
            let dataset = Arc::clone(&dataset);
            // Each parallel task gets its own Arc clone
            dataset.values[i * 100..(i + 1) * 100].iter().sum::<i32>()
        })
        .collect();

    println!("Results: {:?}", results);
}
```

## 12. **Web Server Request Handlers**

Sharing application state across request handlers in web frameworks.

```rust
use std::sync::Arc;
use axum::{Router, extract::State, routing::get};

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    config: Arc<Config>,
}

struct Database {
    url: String,
}

struct Config {
    env: String,
}

async fn handler(State(state): State<AppState>) -> String {
    format!("DB: {}, Env: {}", state.db.url, state.config.env)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: Arc::new(Database {
            url: "postgres://localhost".to_string(),
        }),
        config: Arc::new(Config {
            env: "production".to_string(),
        }),
    };

    let app = Router::new()
        .route("/", get(handler))
        .with_state(state);

    // Run server...
}
```

## 13. **Weak References for Avoiding Cycles**

Breaking reference cycles in multi-threaded contexts.

```rust
use std::sync::{Arc, Weak, Mutex};

struct Parent {
    children: Mutex<Vec<Arc<Child>>>,
}

struct Child {
    parent: Weak<Parent>,  // Weak to avoid cycle
    value: i32,
}

fn main() {
    let parent = Arc::new(Parent {
        children: Mutex::new(vec![]),
    });

    let child = Arc::new(Child {
        parent: Arc::downgrade(&parent),
        value: 42,
    });

    parent.children.lock().unwrap().push(Arc::clone(&child));

    // child holds Weak<Parent>, so parent can be dropped
}
```

## When **NOT** to Use `Arc<T>`

❌ **Single-threaded code** - Use `Rc<T>` instead (cheaper, no atomic operations)  
❌ **Single ownership is sufficient** - Use `Box<T>` or direct ownership  
❌ **Borrowing works** - Use `&T` or `&mut T`  
❌ **Data is `Copy` and small** - Just copy it  
❌ **Only one thread needs ownership** - Send the data with `move`  
❌ **Mutable access without locks is needed** - Redesign to avoid shared mutable state  

## Common Patterns

### Arc with Mutex (Read-Write Access)

```rust
use std::sync::{Arc, Mutex};

// For data that needs frequent mutation
let shared = Arc::new(Mutex::new(Vec::<i32>::new()));
```

### Arc with RwLock (Many Readers, Few Writers)

```rust
use std::sync::{Arc, RwLock};

// For data with many readers, occasional writers
let shared = Arc::new(RwLock::new(HashMap::<String, String>::new()));

// Multiple readers can access simultaneously
let read_guard = shared.read().unwrap();

// Writers get exclusive access
let write_guard = shared.write().unwrap();
```

### Arc Alone (Immutable Shared Data)

```rust
use std::sync::Arc;

// For immutable data shared across threads
let shared = Arc::new(Config { /* ... */ });
```

### Arc with Atomic Types (Lock-Free Mutations)

```rust
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

// For simple counters/flags without locks
let counter = Arc::new(AtomicUsize::new(0));
counter.fetch_add(1, Ordering::SeqCst);
```

## Important Caveats

### Memory Overhead

```rust
use std::mem::size_of;

// Arc has overhead for atomic reference counting
assert_eq!(size_of::<Arc<i32>>(), size_of::<usize>());  // Just a pointer
// But the allocation includes: data + strong_count + weak_count
```

### Performance Cost

```rust
use std::sync::Arc;

// Cloning Arc requires atomic operations (slower than Rc)
let arc1 = Arc::new(42);
let arc2 = Arc::clone(&arc1);  // Atomic increment
drop(arc2);                     // Atomic decrement
```

### Deadlocks with Multiple Locks

```rust
use std::sync::{Arc, Mutex};

let lock1 = Arc::new(Mutex::new(1));
let lock2 = Arc::new(Mutex::new(2));

// ❌ Can deadlock if another thread locks in opposite order
let _guard1 = lock1.lock().unwrap();
let _guard2 = lock2.lock().unwrap();

// ✅ Always acquire locks in consistent order
```

### Memory Leaks via Reference Cycles

```rust
use std::sync::{Arc, Mutex};

struct Node {
    next: Option<Arc<Mutex<Node>>>,
}

// ❌ Creates cycle, memory leak
let a = Arc::new(Mutex::new(Node { next: None }));
let b = Arc::new(Mutex::new(Node { next: Some(Arc::clone(&a)) }));
a.lock().unwrap().next = Some(Arc::clone(&b));

// ✅ Use Weak to break cycles
use std::sync::Weak;

struct SafeNode {
    next: Option<Arc<Mutex<SafeNode>>>,
    prev: Option<Weak<Mutex<SafeNode>>>,
}
```

## Quick Decision Guide

```
Do you need multiple owners? → NO: Use Box<T> or direct ownership
                              ↓ YES
Is this multi-threaded? → NO: Use Rc<T>
                          ↓ YES
Do you need mutability? → NO: Use Arc<T>
                          ↓ YES
Many readers, few writers? → YES: Use Arc<RwLock<T>>
                            ↓ NO
Simple counter/flag? → YES: Use Arc<AtomicXxx>
                      ↓ NO
                      Use Arc<Mutex<T>>
```

## Performance Characteristics

- **Clone cost**: O(1) - atomic increment (slower than `Rc`)
- **Drop cost**: O(1) - atomic decrement (deallocates if count reaches 0)
- **Memory overhead**: Two atomic `usize` values (strong + weak counts)
- **Thread-safe**: Uses atomic operations, safe across threads
- **Cost vs Rc**: ~2-3x slower due to atomic operations, but still very fast

## Trait Bounds

```rust
use std::sync::Arc;

// Arc requires T: Send + Sync for thread safety
fn share_across_threads<T: Send + Sync>(data: T) -> Arc<T> {
    Arc::new(data)
}

// Send: Can be transferred across thread boundaries
// Sync: Can be referenced from multiple threads
```

**Key insight**: `Arc<T>` is for **shared ownership** in **multi-threaded** contexts. It's the thread-safe version of `Rc<T>`, using atomic reference counting to safely share data across threads without data races.