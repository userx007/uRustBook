# Guide to Rust's thread safety and concurrency primitives 

## **Core Concepts:**

**Send and Sync Traits:**
- `Send`: Types that can be transferred across thread boundaries (ownership can move)
- `Sync`: Types that can be safely shared between threads via immutable references (`&T`)
- These are **marker traits** that Rust uses to enforce thread safety at compile time
- Most types are `Send`, but `Rc` and raw pointers are not
- `Arc` (atomic reference counting) is the thread-safe alternative to `Rc`

**Data Race Prevention:**
Rust eliminates data races at compile time through:
- Ownership rules (only one mutable reference OR multiple immutable references)
- Type system enforcement via `Send`/`Sync`
- Explicit synchronization primitives

## **Synchronization Primitives:**

**Mutex<T>** (Mutual Exclusion):
- Allows only one thread to access data at a time
- `lock()` blocks until lock is acquired
- Automatically releases lock when guard drops (RAII)
- Use when: You need exclusive access to shared data

**RwLock<T>** (Reader-Writer Lock):
- Multiple readers OR one writer
- Better performance for read-heavy workloads
- Can lead to writer starvation in extreme cases
- Use when: Many reads, few writes

**Atomic Types**:
- Lock-free, single-value primitives (`AtomicBool`, `AtomicUsize`, etc.)
- No blocking, no deadlocks possible
- Memory ordering control for performance optimization
- Use when: Simple counters, flags, or lock-free algorithms

**Lock-Free Structures**:
- Built using atomic operations and CAS (compare-and-swap)
- Complex but highest performance under contention
- Require careful attention to memory ordering
- Use when: Maximum performance needed and complexity is justified

## **When to Use What:**

- **Simple counter/flag**: Atomics
- **Protecting complex data**: Mutex
- **Read-heavy workloads**: RwLock
- **Maximum performance**: Lock-free structures (if complexity justified)
- **Sharing ownership**: Wrap in `Arc<>`

The code includes working examples of all these concepts, including a practical concurrent hash map implementation using sharding!

```rust
// ============================================================================
// THREAD SAFETY AND CONCURRENCY PRIMITIVES IN RUST
// ============================================================================

// ============================================================================
// 1. SEND AND SYNC TRAITS - THE FOUNDATION OF THREAD SAFETY
// ============================================================================

/*
Send: A type is Send if it can be transferred across thread boundaries.
      Most types are Send, but raw pointers and Rc are not.

Sync: A type is Sync if it can be safely shared between threads via 
      immutable references (&T). T is Sync if &T is Send.

Key Rule: &T is Send if T is Sync
*/

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// Example: Send but not Sync
struct MySendType {
    data: Vec<i32>,
}
// Vec<i32> is Send, so MySendType is automatically Send
// But if we add interior mutability without synchronization, it's not Sync

// Example: Neither Send nor Sync
struct NotSendNotSync {
    rc: Rc<i32>, // Rc is neither Send nor Sync
}

// Example: Both Send and Sync
struct SendAndSync {
    data: i32, // i32 is both Send and Sync
}

fn demonstrate_send_sync() {
    println!("=== SEND AND SYNC DEMONSTRATION ===\n");
    
    // This works - Vec is Send
    let data = vec![1, 2, 3, 4, 5];
    let handle = thread::spawn(move || {
        println!("Thread processing data: {:?}", data);
    });
    handle.join().unwrap();
    
    // This won't compile - Rc is not Send
    // let rc = Rc::new(5);
    // let handle = thread::spawn(move || {
    //     println!("Value: {}", *rc); // ERROR!
    // });
    
    // Use Arc instead - it's both Send and Sync
    let arc = Arc::new(5);
    let arc_clone = Arc::clone(&arc);
    let handle = thread::spawn(move || {
        println!("Arc value in thread: {}", *arc_clone);
    });
    handle.join().unwrap();
    println!();
}

// ============================================================================
// 2. DATA RACES AND HOW RUST PREVENTS THEM
// ============================================================================

/*
Data Race: Occurs when:
1. Two or more threads access the same memory location
2. At least one access is a write
3. Accesses are not synchronized

Rust prevents data races at compile time through:
- Ownership system
- Borrowing rules
- Send/Sync traits
- Type system enforcement
*/

fn demonstrate_data_race_prevention() {
    println!("=== DATA RACE PREVENTION ===\n");
    
    // This won't compile - can't share mutable reference
    // let mut counter = 0;
    // let handle = thread::spawn(|| {
    //     counter += 1; // ERROR: can't capture mutable reference
    // });
    
    // Correct way: Use Mutex or atomic
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final counter (Mutex): {}", *counter.lock().unwrap());
    println!();
}

// ============================================================================
// 3. MUTEX - MUTUAL EXCLUSION
// ============================================================================

/*
Mutex<T>: Provides mutual exclusion for data of type T
- Only one thread can access the data at a time
- lock() returns a MutexGuard that provides access
- Automatically unlocks when guard goes out of scope
*/

fn demonstrate_mutex() {
    println!("=== MUTEX DEMONSTRATION ===\n");
    
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mut handles = vec![];
    
    // Multiple threads modifying shared data
    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let mut vec = data.lock().unwrap();
            vec.push(i);
            println!("Thread {} added {}, current vec: {:?}", i, i, *vec);
            // Lock is automatically released when vec (MutexGuard) goes out of scope
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final vector: {:?}", *data.lock().unwrap());
    
    // Demonstrating lock contention
    println!("\n--- Lock Contention Example ---");
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for i in 0..3 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            println!("Thread {} acquired lock", i);
            *num += 1;
            thread::sleep(Duration::from_millis(100)); // Simulate work
            println!("Thread {} releasing lock", i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    println!();
}

// ============================================================================
// 4. RWLOCK - READER-WRITER LOCK
// ============================================================================

/*
RwLock<T>: Allows multiple readers OR one writer
- read() returns RwLockReadGuard - multiple concurrent readers allowed
- write() returns RwLockWriteGuard - exclusive access
- Better performance when reads are more frequent than writes
*/

fn demonstrate_rwlock() {
    println!("=== RWLOCK DEMONSTRATION ===\n");
    
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];
    
    // Multiple readers
    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let vec = data.read().unwrap();
            println!("Reader {} sees: {:?}", i, *vec);
            thread::sleep(Duration::from_millis(100));
        });
        handles.push(handle);
    }
    
    // One writer
    let data_writer = Arc::clone(&data);
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let mut vec = data_writer.write().unwrap();
        println!("Writer acquired lock");
        vec.push(6);
        println!("Writer modified data: {:?}", *vec);
    });
    handles.push(handle);
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final data: {:?}", *data.read().unwrap());
    
    // Performance comparison scenario
    println!("\n--- Read-Heavy Workload ---");
    let shared_data = Arc::new(RwLock::new(42));
    let mut handles = vec![];
    
    // 10 readers
    for i in 0..10 {
        let data = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            let value = data.read().unwrap();
            println!("Reader {} read: {}", i, *value);
        });
        handles.push(handle);
    }
    
    // 1 writer
    let data = Arc::clone(&shared_data);
    let handle = thread::spawn(move || {
        let mut value = data.write().unwrap();
        *value += 1;
        println!("Writer updated value to: {}", *value);
    });
    handles.push(handle);
    
    for handle in handles {
        handle.join().unwrap();
    }
    println!();
}

// ============================================================================
// 5. ATOMIC OPERATIONS
// ============================================================================

/*
Atomic types: Lock-free thread-safe primitives
- AtomicBool, AtomicI32, AtomicUsize, etc.
- Operations are guaranteed to be atomic (indivisible)
- No locks, so no risk of deadlock
- Better performance for simple operations

Memory Ordering:
- Relaxed: No ordering guarantees (fastest)
- Acquire: Prevents reordering of subsequent reads/writes
- Release: Prevents reordering of previous reads/writes
- AcqRel: Both Acquire and Release
- SeqCst: Sequential consistency (strongest, slowest)
*/

fn demonstrate_atomics() {
    println!("=== ATOMIC OPERATIONS ===\n");
    
    // Simple counter
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Atomic counter final value: {}", counter.load(Ordering::Relaxed));
    
    // Compare-and-swap (CAS) operation
    println!("\n--- Compare-and-Swap Example ---");
    let value = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for i in 0..5 {
        let value = Arc::clone(&value);
        let handle = thread::spawn(move || {
            loop {
                let current = value.load(Ordering::Acquire);
                let new = current + i;
                
                // Try to update only if value hasn't changed
                match value.compare_exchange(
                    current,
                    new,
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        println!("Thread {} successfully updated {} to {}", i, current, new);
                        break;
                    }
                    Err(actual) => {
                        println!("Thread {} CAS failed, expected {} but was {}", i, current, actual);
                    }
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final value: {}", value.load(Ordering::Acquire));
    println!();
}

// ============================================================================
// 6. LOCK-FREE STRUCTURES
// ============================================================================

/*
Lock-free data structures avoid locks entirely, using atomic operations
Benefits:
- No deadlocks
- Better performance under high contention
- Progress guarantee: at least one thread makes progress

Challenges:
- More complex to implement
- Harder to reason about
- Memory ordering considerations
*/

// Simple lock-free stack using atomics
use std::ptr;
use std::sync::atomic::AtomicPtr;

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    fn new() -> Self {
        LockFreeStack {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }
    
    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));
        
        loop {
            let old_head = self.head.load(Ordering::Acquire);
            unsafe { (*new_node).next = old_head; }
            
            // Try to swing the head pointer
            if self.head.compare_exchange(
                old_head,
                new_node,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                break;
            }
        }
    }
    
    fn pop(&self) -> Option<T> {
        loop {
            let old_head = self.head.load(Ordering::Acquire);
            
            if old_head.is_null() {
                return None;
            }
            
            let next = unsafe { (*old_head).next };
            
            if self.head.compare_exchange(
                old_head,
                next,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                unsafe {
                    let data = ptr::read(&(*old_head).data);
                    drop(Box::from_raw(old_head));
                    return Some(data);
                }
            }
        }
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

unsafe impl<T: Send> Send for LockFreeStack<T> {}
unsafe impl<T: Send> Sync for LockFreeStack<T> {}

fn demonstrate_lock_free() {
    println!("=== LOCK-FREE STACK ===\n");
    
    let stack = Arc::new(LockFreeStack::new());
    let mut handles = vec![];
    
    // Push from multiple threads
    for i in 0..5 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            stack.push(i);
            println!("Thread {} pushed {}", i, i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Pop from multiple threads
    let mut handles = vec![];
    for i in 0..5 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            if let Some(value) = stack.pop() {
                println!("Thread {} popped {}", i, value);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    println!();
}

// ============================================================================
// 7. PRACTICAL EXAMPLE: CONCURRENT HASHMAP-LIKE STRUCTURE
// ============================================================================

use std::collections::HashMap;
use std::hash::Hash;

struct ConcurrentMap<K, V> {
    shards: Vec<RwLock<HashMap<K, V>>>,
}

impl<K: Hash + Eq, V> ConcurrentMap<K, V> {
    fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(RwLock::new(HashMap::new()));
        }
        ConcurrentMap { shards }
    }
    
    fn get_shard(&self, key: &K) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shards.len()
    }
    
    fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_idx = self.get_shard(&key);
        let mut shard = self.shards[shard_idx].write().unwrap();
        shard.insert(key, value)
    }
    
    fn get(&self, key: &K) -> Option<V> where V: Clone {
        let shard_idx = self.get_shard(key);
        let shard = self.shards[shard_idx].read().unwrap();
        shard.get(key).cloned()
    }
}

fn demonstrate_concurrent_map() {
    println!("=== CONCURRENT MAP (SHARDED) ===\n");
    
    let map = Arc::new(ConcurrentMap::new(4));
    let mut handles = vec![];
    
    // Multiple threads inserting
    for i in 0..10 {
        let map = Arc::clone(&map);
        let handle = thread::spawn(move || {
            map.insert(format!("key_{}", i), i);
            println!("Inserted key_{} = {}", i, i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Multiple threads reading
    let mut handles = vec![];
    for i in 0..10 {
        let map = Arc::clone(&map);
        let handle = thread::spawn(move || {
            if let Some(value) = map.get(&format!("key_{}", i)) {
                println!("Read key_{} = {}", i, value);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    println!();
}

// ============================================================================
// MAIN FUNCTION - RUN ALL DEMONSTRATIONS
// ============================================================================

fn main() {
    println!("RUST THREAD SAFETY AND CONCURRENCY PRIMITIVES\n");
    println!("==============================================\n");
    
    demonstrate_send_sync();
    demonstrate_data_race_prevention();
    demonstrate_mutex();
    demonstrate_rwlock();
    demonstrate_atomics();
    demonstrate_lock_free();
    demonstrate_concurrent_map();
    
    println!("==============================================");
    println!("\nKEY TAKEAWAYS:");
    println!("1. Send/Sync traits enforce thread safety at compile time");
    println!("2. Mutex provides exclusive access (one thread at a time)");
    println!("3. RwLock allows multiple readers or one writer");
    println!("4. Atomics provide lock-free operations for simple types");
    println!("5. Lock-free structures avoid locks but are complex");
    println!("6. Choose the right primitive based on your use case");
}
```