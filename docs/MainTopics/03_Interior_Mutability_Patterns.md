# Interior Mutability Patterns in Rust

Interior mutability is a design pattern in Rust that allows you to mutate data even when there are immutable references to that data. This might seem to violate Rust's borrowing rules, but these types use `unsafe` code internally to provide safe abstractions that enforce borrowing rules at runtime instead of compile time.

Let me create a comprehensive guide with examples:I've created a comprehensive guide to Rust's interior mutability patterns. Let me break down the key concepts:

## What is Interior Mutability?

Interior mutability is Rust's way of allowing mutation of data even when there are immutable references to it. This seems to violate Rust's borrowing rules, but these types use `unsafe` code internally to maintain safety guarantees—just checked at runtime rather than compile-time.

## The Four Main Types:

### **Cell\<T>** - Simple & Fast
- **Only for `Copy` types** (integers, bools, chars)
- Zero runtime overhead
- Single-threaded only
- Replaces the entire value with `get()` and `set()`
- **Use when**: Tracking metadata in immutable structs (counters, flags)

[More `Cell<T>` examples here...](../Other/108_When_Cell_is_Needed_in_Rust.md)


### **RefCell\<T>** - Flexible Runtime Checking
- Works with **any type**
- Enforces borrowing rules at runtime (panics if violated)
- Can borrow mutably (`borrow_mut()`) or immutably (`borrow()`)
- Single-threaded only
- **Use when**: Building graphs/trees with cycles, mock objects, or complex shared data structures in single-threaded code

[More `RefCell<T>` examples here...](../Other/109_When_RefCell_is_Needed_in_Rust.md)


### **Mutex\<T>** - Thread-Safe Exclusivity
- Thread-safe via mutual exclusion
- Only one thread can access at a time
- Blocks waiting threads
- Use with `Arc` for shared ownership
- **Use when**: Multiple threads need exclusive access to shared data (caches, counters, queues)

[More `Mutex<T>` examples here...](../Other/110_When_Mutex_is_Needed_in_Rust.md)


### **RwLock\<T>** - Optimized for Reads
- Allows **multiple simultaneous readers** OR one writer
- Better performance when reads vastly outnumber writes
- Writers block all access, readers only block writers
- **Use when**: Configuration systems, read-heavy data structures, rarely-updated shared state

[More `RwLock<T>` examples here...](../Other/111_When_RwLock_is_Needed_in_Rust.md)


## Key Decision Factors:

1. **Thread safety needed?** → Mutex/RwLock (multi-threaded) vs Cell/RefCell (single-threaded)
2. **Copy type?** → Cell is simplest
3. **Read-heavy workload?** → RwLock over Mutex
4. **Need runtime flexibility?** → RefCell for graphs/cycles

The code examples show practical patterns like thread-safe caches, configuration systems, and graph structures—all common use cases where interior mutability shines!

```rust
// ============================================================================
// INTERIOR MUTABILITY PATTERNS IN RUST
// ============================================================================
// Interior mutability allows mutation through immutable references
// Different types provide different guarantees and use cases

use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock, Arc};
use std::thread;
use std::rc::Rc;

// ============================================================================
// 1. CELL<T> - For Copy types only
// ============================================================================
// - No runtime overhead
// - Works only with Copy types (integers, bools, chars, etc.)
// - Replaces the entire value
// - Single-threaded only

fn cell_example() {
    println!("=== Cell Example ===");
    
    // Cell allows mutation of a Copy type inside an immutable struct
    struct Counter {
        count: Cell<i32>,
    }
    
    let counter = Counter {
        count: Cell::new(0),
    };
    
    // counter is immutable, but we can mutate count
    counter.count.set(counter.count.get() + 1);
    counter.count.set(counter.count.get() + 1);
    
    println!("Counter value: {}", counter.count.get());
    
    // Common use case: tracking state in immutable context
    struct Point {
        x: i32,
        y: i32,
        access_count: Cell<u32>,
    }
    
    impl Point {
        fn new(x: i32, y: i32) -> Self {
            Point { x, y, access_count: Cell::new(0) }
        }
        
        // Immutable method that tracks access
        fn distance_from_origin(&self) -> f64 {
            self.access_count.set(self.access_count.get() + 1);
            ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
        }
        
        fn times_accessed(&self) -> u32 {
            self.access_count.get()
        }
    }
    
    let point = Point::new(3, 4);
    println!("Distance: {}", point.distance_from_origin());
    println!("Distance: {}", point.distance_from_origin());
    println!("Point accessed {} times", point.times_accessed());
}

// ============================================================================
// 2. REFCELL<T> - For any type with runtime borrow checking
// ============================================================================
// - Works with any type (not just Copy)
// - Runtime borrow checking (panics on violations)
// - Can borrow mutably or immutably
// - Single-threaded only

fn refcell_example() {
    println!("\n=== RefCell Example ===");
    
    // RefCell allows mutation through immutable reference
    let data = RefCell::new(vec![1, 2, 3]);
    
    // Borrow mutably and modify
    data.borrow_mut().push(4);
    data.borrow_mut().push(5);
    
    // Borrow immutably to read
    println!("Data: {:?}", *data.borrow());
    
    // Common pattern: Graph with cycles using Rc<RefCell<T>>
    #[derive(Debug)]
    struct Node {
        value: i32,
        children: RefCell<Vec<Rc<Node>>>,
    }
    
    impl Node {
        fn new(value: i32) -> Rc<Self> {
            Rc::new(Node {
                value,
                children: RefCell::new(vec![]),
            })
        }
        
        fn add_child(&self, child: Rc<Node>) {
            self.children.borrow_mut().push(child);
        }
    }
    
    let root = Node::new(1);
    let child1 = Node::new(2);
    let child2 = Node::new(3);
    
    root.add_child(child1.clone());
    root.add_child(child2.clone());
    
    println!("Root has {} children", root.children.borrow().len());
    
    // Example: Mock object that tracks calls
    struct MockDatabase {
        queries: RefCell<Vec<String>>,
    }
    
    impl MockDatabase {
        fn new() -> Self {
            MockDatabase {
                queries: RefCell::new(Vec::new()),
            }
        }
        
        // Immutable method that internally mutates
        fn query(&self, sql: &str) -> Vec<String> {
            self.queries.borrow_mut().push(sql.to_string());
            vec!["result1".to_string(), "result2".to_string()]
        }
        
        fn query_count(&self) -> usize {
            self.queries.borrow().len()
        }
    }
    
    let db = MockDatabase::new();
    db.query("SELECT * FROM users");
    db.query("SELECT * FROM posts");
    println!("Database received {} queries", db.query_count());
}

// ============================================================================
// 3. MUTEX<T> - Thread-safe mutual exclusion
// ============================================================================
// - Works across threads
// - Only one thread can access at a time
// - Blocks other threads until lock is released
// - Use with Arc for shared ownership across threads

fn mutex_example() {
    println!("\n=== Mutex Example ===");
    
    // Mutex provides thread-safe interior mutability
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for i in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
            println!("Thread {} incremented counter", i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final counter value: {}", *counter.lock().unwrap());
    
    // Practical example: Thread-safe cache
    use std::collections::HashMap;
    
    struct Cache {
        data: Mutex<HashMap<String, String>>,
    }
    
    impl Cache {
        fn new() -> Self {
            Cache {
                data: Mutex::new(HashMap::new()),
            }
        }
        
        fn get(&self, key: &str) -> Option<String> {
            self.data.lock().unwrap().get(key).cloned()
        }
        
        fn set(&self, key: String, value: String) {
            self.data.lock().unwrap().insert(key, value);
        }
    }
    
    let cache = Arc::new(Cache::new());
    
    let cache1 = Arc::clone(&cache);
    let h1 = thread::spawn(move || {
        cache1.set("key1".to_string(), "value1".to_string());
    });
    
    let cache2 = Arc::clone(&cache);
    let h2 = thread::spawn(move || {
        cache2.set("key2".to_string(), "value2".to_string());
    });
    
    h1.join().unwrap();
    h2.join().unwrap();
    
    println!("Cache key1: {:?}", cache.get("key1"));
    println!("Cache key2: {:?}", cache.get("key2"));
}

// ============================================================================
// 4. RWLOCK<T> - Multiple readers OR one writer
// ============================================================================
// - Allows multiple simultaneous readers
// - Only one writer at a time
// - Readers block writers, writers block everyone
// - Better performance when reads outnumber writes

fn rwlock_example() {
    println!("\n=== RwLock Example ===");
    
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];
    
    // Spawn multiple reader threads
    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let read_guard = data.read().unwrap();
            println!("Reader {} sees: {:?}", i, *read_guard);
            thread::sleep(std::time::Duration::from_millis(100));
        });
        handles.push(handle);
    }
    
    // Spawn a writer thread
    let data_writer = Arc::clone(&data);
    let writer_handle = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(50));
        let mut write_guard = data_writer.write().unwrap();
        write_guard.push(6);
        println!("Writer added value");
    });
    handles.push(writer_handle);
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final data: {:?}", *data.read().unwrap());
    
    // Practical example: Configuration that's read often, written rarely
    struct Config {
        settings: RwLock<HashMap<String, String>>,
    }
    
    impl Config {
        fn new() -> Self {
            Config {
                settings: RwLock::new(HashMap::new()),
            }
        }
        
        fn get(&self, key: &str) -> Option<String> {
            // Many threads can read simultaneously
            self.settings.read().unwrap().get(key).cloned()
        }
        
        fn set(&self, key: String, value: String) {
            // Only one thread can write
            self.settings.write().unwrap().insert(key, value);
        }
    }
}

// ============================================================================
// 5. WHEN TO USE EACH TYPE
// ============================================================================

fn decision_guide() {
    println!("\n=== Decision Guide ===");
    println!("
Cell<T>:
  ✓ Copy types only (integers, bools, etc.)
  ✓ Single-threaded
  ✓ No runtime overhead
  ✓ Example: Counters, flags in immutable structs

RefCell<T>:
  ✓ Any type
  ✓ Single-threaded
  ✓ Need mutable borrows at runtime
  ✓ Example: Mock objects, complex data structures, DAGs

Mutex<T>:
  ✓ Any type
  ✓ Multi-threaded
  ✓ Exclusive access required
  ✓ Example: Shared counters, thread-safe caches

RwLock<T>:
  ✓ Any type
  ✓ Multi-threaded
  ✓ Many readers, few writers
  ✓ Example: Configuration, read-heavy data structures
    ");
}

// ============================================================================
// 6. COMMON PITFALLS
// ============================================================================

fn common_pitfalls() {
    println!("\n=== Common Pitfalls ===");
    
    // Pitfall 1: RefCell panic from borrowing violations
    println!("Pitfall 1: RefCell borrow violations");
    let data = RefCell::new(5);
    let _borrow1 = data.borrow_mut();
    // This would panic: let _borrow2 = data.borrow_mut();
    println!("Remember: Only one mutable borrow at a time!");
    
    // Pitfall 2: Mutex deadlock
    println!("\nPitfall 2: Potential deadlock with Mutex");
    let m1 = Mutex::new(1);
    let m2 = Mutex::new(2);
    
    // Deadlock can occur if two threads lock in different orders
    // Thread 1: lock m1, then m2
    // Thread 2: lock m2, then m1
    println!("Always acquire locks in the same order to prevent deadlock!");
    
    // Pitfall 3: Holding locks too long
    println!("\nPitfall 3: Holding locks too long");
    let data = Mutex::new(vec![1, 2, 3]);
    {
        let mut guard = data.lock().unwrap();
        guard.push(4);
        // guard is automatically dropped here
    }
    // Lock is released, other threads can proceed
    println!("Release locks as soon as possible!");
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    cell_example();
    refcell_example();
    mutex_example();
    rwlock_example();
    decision_guide();
    common_pitfalls();
    
    println!("\n=== Summary ===");
    println!("Interior mutability lets you mutate data through shared references.");
    println!("Choose based on: thread safety needs, data type, and access patterns.");
}
```

### Summary

| Type           | Key Characteristics                                                                                                                | Thread Safety        | Access Rules                                      | Typical Use Cases                                                      |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------- | -------------------- | ------------------------------------------------- | ---------------------------------------------------------------------- |
| **Cell<T>**    | - Only for `Copy` types (integers, bools, chars)<br>- Zero runtime overhead<br>- Replaces entire value with `get()` and `set()`    | Single-threaded only | Direct get/set, no borrowing                      | Tracking metadata in immutable structs (counters, flags)               |
| **RefCell<T>** | - Works with any type<br>- Runtime borrow checking (panics if violated)<br>- `borrow()` for immutable, `borrow_mut()` for mutable  | Single-threaded only | Mutable or immutable borrowing checked at runtime | Graphs/trees with cycles, mock objects, complex shared data structures |
| **Mutex<T>**   | - Thread-safe via mutual exclusion<br>- Only one thread accesses at a time<br>- Blocks waiting threads<br>- Often used with `Arc`  | Multi-threaded       | Exclusive mutable access                          | Multi-threaded caches, counters, queues                                |
| **RwLock<T>**  | - Allows multiple readers OR one writer<br>- Readers block only writers; writers block all<br>- Optimized for read-heavy workloads | Multi-threaded       | Multiple immutable or single mutable access       | Configuration systems, read-heavy shared state, rarely updated data    |


### Common Pitfalls

| Type           | Common Pitfalls                                   | Explanation / Example                                                                                                     |
| -------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| **Cell<T>**    | Misuse with non-`Copy` types                      | `Cell` only works with `Copy` types; trying to store a non-Copy type will not compile.                                    |
| **RefCell<T>** | Borrowing violations                              | Panics occur if you try to mutably borrow while already borrowed, e.g., `borrow_mut()` while `borrow()` is active.        |
| **Mutex<T>**   | Deadlock                                          | Can happen if multiple threads try to acquire locks in inconsistent order or hold locks while waiting on other resources. |
| **Mutex<T>**   | Holding lock too long                             | Holding a mutex across slow operations blocks other threads, reducing concurrency.                                        |
| **RwLock<T>**  | Writer starvation                                 | Continuous readers can block writers indefinitely if not carefully managed.                                               |
| **RwLock<T>**  | Deadlock                                          | Like mutexes, acquiring multiple locks in inconsistent order can deadlock.                                                |
| **All**        | Panic in single-threaded vs multi-threaded misuse | Using single-threaded types (`Cell` / `RefCell`) in multi-threaded code leads to unsafe behavior or compile errors.       |



