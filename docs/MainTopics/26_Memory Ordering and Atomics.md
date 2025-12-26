# Memory Ordering and Atomics in Rust

Memory ordering and atomics are fundamental building blocks for writing safe concurrent code in Rust. They provide low-level primitives for thread synchronization without the overhead of locks, enabling highly efficient multi-threaded programs.

## Understanding Atomics

Atomic types in Rust are special types that guarantee operations complete indivisibly—they cannot be interrupted mid-operation by other threads. Rust provides atomic variants for common primitive types through the `std::sync::atomic` module, including `AtomicBool`, `AtomicI32`, `AtomicU64`, `AtomicPtr`, and others.

Without atomics, even simple operations like incrementing a counter can cause data races in concurrent contexts. Consider a regular integer being incremented by multiple threads: the read-modify-write cycle can interleave, causing lost updates. Atomics solve this by ensuring operations happen as single, indivisible units.

## Memory Ordering Semantics

Memory ordering defines how atomic operations synchronize with other operations across threads. Rust provides four ordering levels through the `Ordering` enum, each offering different guarantees about visibility and ordering of memory operations.

**Relaxed ordering** (`Ordering::Relaxed`) provides the weakest guarantees. It ensures only that the atomic operation itself is indivisible, but makes no promises about the ordering of other memory operations relative to it. Different threads might observe operations in different orders. This ordering is useful when you only care about the atomic variable itself, not surrounding memory.

**Acquire ordering** (`Ordering::Acquire`) is used on load operations. It ensures that all memory operations that follow the acquire load in the current thread cannot be reordered to occur before it. More importantly, if another thread performs a release store, the acquire load will see all memory writes that happened before that release store. This creates a happens-before relationship between threads.

**Release ordering** (`Ordering::Release`) is used on store operations. It ensures that all memory operations that precede the release store in the current thread cannot be reordered to occur after it. When paired with an acquire load, it establishes synchronization, making all previous writes visible to the thread that performs the acquire.

**Sequentially consistent ordering** (`Ordering::SeqCst`) provides the strongest guarantees. It includes all the properties of acquire and release ordering, plus a global total order on all sequentially consistent operations across all threads. This means all threads agree on a single ordering of these operations, making reasoning about concurrent code easier at the cost of some performance.

## Practical Examples

Here's an example demonstrating different memory orderings in action:

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Example 1: Simple atomic counter with Relaxed ordering
fn relaxed_counter() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.load(Ordering::Relaxed));
    // Prints: Final count: 10000
}

// Example 2: Acquire-Release synchronization pattern
fn acquire_release_sync() {
    let data = Arc::new(AtomicU64::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let data_clone = Arc::clone(&data);
    let ready_clone = Arc::clone(&ready);

    // Producer thread
    let producer = thread::spawn(move || {
        // Perform some computation
        data_clone.store(42, Ordering::Relaxed);
        
        // Signal that data is ready with Release ordering
        // This ensures all previous writes (including to data) 
        // are visible to threads that acquire this flag
        ready_clone.store(true, Ordering::Release);
    });

    // Consumer thread
    let consumer = thread::spawn(move || {
        // Wait until data is ready with Acquire ordering
        // This ensures we see all writes that happened before 
        // the Release store in the producer
        while !ready.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(1));
        }
        
        // Now we're guaranteed to see data == 42
        let value = data.load(Ordering::Relaxed);
        println!("Received data: {}", value);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

Here's a more sophisticated example implementing a simple spinlock:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        // Try to acquire the lock using compare_exchange
        while self
            .locked
            .compare_exchange(
                false,                    // expected value
                true,                     // new value
                Ordering::Acquire,        // success ordering
                Ordering::Relaxed,        // failure ordering
            )
            .is_err()
        {
            // Spin while waiting for the lock
            std::hint::spin_loop();
        }

        SpinLockGuard { lock: self }
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        // Release the lock
        self.lock.locked.store(false, Ordering::Release);
    }
}
```

Another common pattern is implementing a simple message passing mechanism:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

struct Message {
    value: i32,
}

fn message_passing_example() {
    let shared = Arc::new(AtomicPtr::new(ptr::null_mut()));
    
    let shared_writer = Arc::clone(&shared);
    let writer = thread::spawn(move || {
        let message = Box::new(Message { value: 100 });
        let ptr = Box::into_raw(message);
        
        // Store pointer with Release ordering
        shared_writer.store(ptr, Ordering::Release);
    });
    
    let shared_reader = Arc::clone(&shared);
    let reader = thread::spawn(move || {
        let mut ptr;
        loop {
            // Load pointer with Acquire ordering
            ptr = shared_reader.load(Ordering::Acquire);
            if !ptr.is_null() {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        
        // Safe to read the message now
        let message = unsafe { Box::from_raw(ptr) };
        println!("Received message: {}", message.value);
    });
    
    writer.join().unwrap();
    reader.join().unwrap();
}
```

## Choosing the Right Ordering

Selecting the appropriate memory ordering requires understanding your synchronization needs. Use `SeqCst` when you need the simplest reasoning and performance isn't critical. Use `Acquire` and `Release` pairs when you need to synchronize data between threads—this is the most common pattern. Use `Relaxed` only when you're working with a single atomic variable in isolation and don't need to synchronize other memory. When in doubt, `SeqCst` is the safest choice, though it may introduce unnecessary performance overhead.

The key insight is that memory ordering isn't just about the atomic variable itself—it's about establishing visibility guarantees for all surrounding memory operations. Acquire-Release pairs create synchronization boundaries that ensure proper happens-before relationships, making concurrent code both safe and efficient.