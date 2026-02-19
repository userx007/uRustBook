# 62. Lock-Free Data Structures in Rust

1. **Why lock-free** — deadlock, convoying, scalability problems with mutexes
2. **Compare-and-swap** — `compare_exchange`/`compare_exchange_weak`, CAS loops, memory orderings table, and the ABA problem
3. **Epoch-based reclamation** — the 3-epoch scheme explained + a full lock-free stack built with `crossbeam-epoch`
4. **Hazard pointers** — load-protect-validate pattern with the `haphazard` crate
5. **Crossbeam ecosystem** — `SegQueue`, `ArrayQueue`, `deque` work-stealing, and channels with runnable examples
6. **Practical patterns** — atomic snapshots, seqlocks, `OnceLock` lazy init, and RCU-style updates with `arc-swap`
7. **Pitfalls** — memory ordering mistakes, ABA, busy-loop CAS without backoff, panic safety
8. **Summary table** mapping each concept to its Rust tool


Lock-free data structures allow multiple threads to operate on shared data **without holding mutual exclusion locks**. Instead of blocking threads, they rely on atomic hardware primitives to achieve thread safety while preserving progress guarantees. This topic covers the core concepts: compare-and-swap, memory reclamation strategies, and practical patterns using the `crossbeam` ecosystem.

---

## 1. Why Lock-Free?

Traditional mutex-based concurrency has several pitfalls:

- **Deadlock** – two threads waiting on each other's locks forever.
- **Priority inversion** – a low-priority thread holds a lock needed by a high-priority thread.
- **Convoying** – threads pile up waiting for one hot lock.
- **Poor scalability** – contention degrades performance under high parallelism.

Lock-free structures guarantee that **at least one thread always makes progress** (lock-freedom), and the strongest variant, **wait-freedom**, guarantees every thread completes in a bounded number of steps.

---

## 2. Compare-and-Swap (CAS)

The fundamental primitive behind virtually all lock-free algorithms is **Compare-and-Swap (CAS)**. It atomically does:

```
if *addr == expected {
    *addr = new_value;
    return Ok(new_value);
} else {
    return Err(*addr); // current value
}
```

In Rust, this is exposed via `std::sync::atomic`:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    let counter = AtomicUsize::new(0);

    // CAS: only update if value is currently 0
    let result = counter.compare_exchange(
        0,           // expected
        1,           // new value
        Ordering::SeqCst,  // success ordering
        Ordering::SeqCst,  // failure ordering
    );

    match result {
        Ok(prev) => println!("Updated! Previous value: {}", prev),
        Err(actual) => println!("CAS failed. Current value: {}", actual),
    }
}
```

### Lock-Free Counter Using CAS Loop

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..8 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // CAS loop: retry until we succeed
                loop {
                    let current = c.load(Ordering::Relaxed);
                    let new_val = current + 1;
                    if c.compare_exchange_weak(
                        current,
                        new_val,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ).is_ok() {
                        break;
                    }
                }
            }
        }));
    }

    for h in handles { h.join().unwrap(); }
    println!("Final count: {}", counter.load(Ordering::Acquire)); // 8000
}
```

`compare_exchange_weak` may spuriously fail on some architectures (like ARM) but is preferred in loops because it can generate more efficient code.

### Memory Orderings

| Ordering | Guarantees |
|----------|-----------|
| `Relaxed` | No synchronization, just atomicity |
| `Acquire` | Reads see all writes before a matching `Release` |
| `Release` | Writes become visible before a matching `Acquire` |
| `AcqRel` | Both Acquire and Release |
| `SeqCst` | Total sequential consistency (strongest, most expensive) |

---

## 3. The ABA Problem

A subtle hazard in CAS-based algorithms: a value changes from A → B → A between your load and your CAS. Your CAS succeeds (it sees A), but the intermediate state may have corrupted your invariant.

```
Thread 1: reads node* = 0xABCD (value A)
Thread 2: pops 0xABCD, pushes new node, pushes 0xABCD again (from a free list)
Thread 1: CAS sees 0xABCD == expected, succeeds → logic error!
```

Solutions include tagged pointers (storing a version counter in the pointer's low bits) or using epoch-based/hazard pointer reclamation to prevent pointer reuse during operations.

---

## 4. Epoch-Based Reclamation

When we remove a node from a lock-free structure, we cannot `drop` it immediately — another thread may still be reading it. **Epoch-based reclamation** tracks which "epoch" (logical time period) each thread is in, and only frees memory once it's safe.

The three-epoch scheme:
1. Threads **pin** themselves to the current epoch before accessing shared data.
2. Retired objects go into the current epoch's garbage bag.
3. An epoch advances when all active threads have passed through it.
4. Objects from epoch `e` are freed once epoch `e+2` begins.

### Using `crossbeam-epoch`

```toml
[dependencies]
crossbeam-epoch = "0.9"
```

```rust
use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};
use std::sync::atomic::Ordering;

struct Node<T> {
    data: T,
    next: Atomic<Node<T>>,
}

struct Stack<T> {
    head: Atomic<Node<T>>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { head: Atomic::null() }
    }

    fn push(&self, data: T) {
        let node = Owned::new(Node {
            data,
            next: Atomic::null(),
        });

        let guard = epoch::pin(); // pin this thread to the current epoch
        let node = node.into_shared(&guard);

        loop {
            let head = self.head.load(Ordering::Relaxed, &guard);
            unsafe { node.deref() }
                .next
                .store(head, Ordering::Relaxed);

            if self.head
                .compare_exchange(head, node, Ordering::Release, Ordering::Relaxed, &guard)
                .is_ok()
            {
                break;
            }
        }
    }

    fn pop(&self) -> Option<T> {
        let guard = epoch::pin();

        loop {
            let head = self.head.load(Ordering::Acquire, &guard);

            match unsafe { head.as_ref() } {
                None => return None,
                Some(node) => {
                    let next = node.next.load(Ordering::Relaxed, &guard);

                    if self.head
                        .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed, &guard)
                        .is_ok()
                    {
                        unsafe {
                            // Schedule deferred drop — freed when epoch advances
                            guard.defer_destroy(head);
                            let data = std::ptr::read(&node.data);
                            return Some(data);
                        }
                    }
                }
            }
        }
    }
}
```

The key insight: `guard.defer_destroy(ptr)` queues the pointer for destruction. The epoch machinery ensures it's not freed while any thread still has a reference via a pinned guard.

---

## 5. Hazard Pointers

An alternative to epoch reclamation. Each thread maintains a small list of **hazard pointers** — pointers it is currently using. Before freeing a retired node, the reclaimer checks all hazard pointers lists; if any thread is hazard-guarding that pointer, freeing is deferred.

Advantages over epochs:
- Bounded memory overhead (proportional to thread count × hazard pointer slots).
- Works better when threads may be inactive for long periods (epochs can stall).

Disadvantages:
- Per-access overhead to publish/clear hazard pointers.
- More complex implementation.

The `haphazard` crate provides a production-quality implementation:

```toml
[dependencies]
haphazard = "0.1"
```

```rust
use haphazard::{Domain, HazardPointer};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

struct LFList<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

impl<T> LFList<T> {
    fn new() -> Self {
        LFList { head: AtomicPtr::new(ptr::null_mut()) }
    }

    fn push(&self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        }));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next.store(head, Ordering::Relaxed) };
            if self.head
                .compare_exchange(head, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }
    }

    fn peek<'domain>(&self, hp: &mut HazardPointer<'_, 'domain>) -> Option<&T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() { return None; }
            // Protect the pointer with a hazard pointer
            unsafe { hp.protect_raw(head) };
            // Validate that head hasn't changed between our load and protection
            if self.head.load(Ordering::Acquire) == head {
                return unsafe { Some(&(*head).value) };
            }
        }
    }
}
```

The key pattern: load a pointer, publish it as a hazard pointer, then validate (the pointer might have been freed between load and publication). This "load-protect-validate" loop is the core of hazard pointer usage.

---

## 6. The `crossbeam` Ecosystem

`crossbeam` is Rust's premier library for concurrent data structures and utilities. It builds on `crossbeam-epoch` and provides ready-to-use lock-free collections.

```toml
[dependencies]
crossbeam = "0.8"
```

### `crossbeam::deque` — Work-Stealing Deque

Used internally by Rayon's thread pool. A single owner can push/pop from one end, while stealers can steal from the other end.

```rust
use crossbeam::deque::{Worker, Stealer, Steal};

fn work_stealing_example() {
    let worker = Worker::<i32>::new_fifo();
    let stealer = worker.stealer();

    // Producer pushes tasks
    for i in 0..10 {
        worker.push(i);
    }

    // Owner pops from front
    while let Some(task) = worker.pop() {
        println!("Owner processed: {}", task);
        break; // just one for demo
    }

    // Another thread steals from back
    match stealer.steal() {
        Steal::Success(task) => println!("Stolen: {}", task),
        Steal::Empty => println!("Nothing to steal"),
        Steal::Retry => println!("Try again"),
    }
}
```

### `crossbeam::queue::SegQueue` — Unbounded MPMC Queue

A Michael-Scott lock-free queue, segmented for cache efficiency. Safe for multiple producers and multiple consumers.

```rust
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::thread;

fn main() {
    let queue = Arc::new(SegQueue::new());
    let mut handles = vec![];

    // 4 producers
    for i in 0..4 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for j in 0..25 {
                q.push(i * 100 + j);
            }
        }));
    }

    // 2 consumers
    for _ in 0..2 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            loop {
                match q.pop() {
                    Some(val) => {
                        count += 1;
                        // process val...
                        let _ = val;
                    }
                    None => break,
                }
            }
            println!("Consumer processed {} items", count);
        }));
    }

    for h in handles { h.join().unwrap(); }
    println!("Remaining in queue: {}", queue.len());
}
```

### `crossbeam::queue::ArrayQueue` — Bounded SPMC/MPSC Queue

A ring-buffer-based bounded queue. The fixed capacity makes allocation predictable.

```rust
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;

fn main() {
    let queue = Arc::new(ArrayQueue::new(128)); // capacity 128

    // Push returns Err if full
    match queue.push(42) {
        Ok(()) => println!("Pushed!"),
        Err(v) => println!("Queue full, rejected: {}", v),
    }

    if let Some(val) = queue.pop() {
        println!("Popped: {}", val);
    }
}
```

### `crossbeam::channel` — MPMC Channels

Not strictly lock-free internally, but provides high-performance MPMC channels as a safer alternative to `std::sync::mpsc`.

```rust
use crossbeam::channel;
use std::thread;

fn main() {
    let (s, r) = channel::bounded(100);

    // Multiple senders
    for i in 0..4 {
        let s = s.clone();
        thread::spawn(move || {
            s.send(i).unwrap();
        });
    }
    drop(s); // close the sending side

    // Multiple receivers
    for val in r.iter() {
        println!("Received: {}", val);
    }
}
```

---

## 7. Practical Lock-Free Patterns

### Pattern 1: Lock-Free Reference Counting with `Arc`

`Arc<T>` itself uses lock-free atomic reference counting. Cloning is a lock-free increment, dropping is a lock-free decrement.

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Arc + atomic for shared, mutable state
let shared = Arc::new(AtomicUsize::new(0));
let clone = Arc::clone(&shared);
std::thread::spawn(move || {
    clone.fetch_add(1, Ordering::Relaxed);
}).join().unwrap();
println!("{}", shared.load(Ordering::Relaxed)); // 1
```

### Pattern 2: Atomic Snapshot

Capture a consistent view of multiple fields without locking, using a version counter:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct Stats {
    version: AtomicU64,
    requests: AtomicU64,
    errors: AtomicU64,
}

impl Stats {
    fn snapshot(&self) -> (u64, u64) {
        loop {
            let v1 = self.version.load(Ordering::Acquire);
            let req = self.requests.load(Ordering::Relaxed);
            let err = self.errors.load(Ordering::Relaxed);
            let v2 = self.version.load(Ordering::Acquire);
            if v1 == v2 && v1 % 2 == 0 {
                // Even version = no write in progress
                return (req, err);
            }
        }
    }

    fn record_request(&self, is_error: bool) {
        self.version.fetch_add(1, Ordering::Release); // start write (odd)
        self.requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
        self.version.fetch_add(1, Ordering::Release); // end write (even)
    }
}
```

### Pattern 3: Seqlock (Sequence Lock)

Similar to atomic snapshot above but more formalized. Writers increment a sequence counter before and after writing; readers retry if they observe an odd count or if the count changed during reading.

### Pattern 4: Lazy Initialization with `OnceLock` / `OnceCell`

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn get_config() -> &'static str {
    CONFIG.get_or_init(|| {
        // Initialized exactly once, lock-free after first call
        "production".to_string()
    })
}
```

### Pattern 5: RCU (Read-Copy-Update) with `arc-swap`

The `arc-swap` crate allows lock-free swapping of an `Arc` pointer, ideal for configurations that are read frequently but updated rarely.

```toml
[dependencies]
arc-swap = "1"
```

```rust
use arc_swap::ArcSwap;
use std::sync::Arc;

struct Config {
    timeout_ms: u64,
    max_retries: u32,
}

static GLOBAL_CONFIG: ArcSwap<Config> = ArcSwap::const_empty();

fn update_config(new: Config) {
    GLOBAL_CONFIG.store(Arc::new(new));
}

fn get_timeout() -> u64 {
    // Fast, lock-free read
    GLOBAL_CONFIG.load().timeout_ms
}
```

---

## 8. Performance Considerations

**When to use lock-free structures:**
- Read-heavy workloads where contention on a mutex would be significant.
- Low-latency requirements where blocking is unacceptable (real-time, game engines).
- Work-stealing schedulers and task queues in runtimes.

**When to prefer mutexes:**
- Write-heavy workloads — CAS loops under contention can waste CPU (livelock risk).
- Complex, multi-step operations that require true atomicity across many fields.
- Simpler code is more maintainable; reach for `Mutex<T>` first.

**Benchmarking tip:** Lock-free ≠ always faster. Under low contention, a `Mutex` is extremely fast (uncontended acquisition is essentially a single atomic op). Always measure.

---

## 9. Common Pitfalls

**1. Misusing memory orderings.** Using `Relaxed` everywhere for performance is tempting but incorrect. Use `Acquire`/`Release` pairs around data that must be synchronized.

**2. Forgetting the ABA problem.** If you recycle memory (e.g., a free list), naïve CAS can succeed incorrectly. Use tagged pointers or a proper reclamation scheme.

**3. Busy-looping CAS without backoff.** Under contention, tight CAS loops spike CPU. Consider exponential backoff or `std::hint::spin_loop()`:

```rust
use std::hint;
let mut spin_count = 0;
loop {
    if cas_succeeds() { break; }
    spin_count += 1;
    if spin_count > 10 {
        std::thread::yield_now();
    } else {
        hint::spin_loop();
    }
}
```

**4. Leaking memory on panic.** Epoch-based schemes and hazard pointers have their own cleanup logic. If you implement custom lock-free structures, ensure panics don't skip deferred frees.

---

## Summary

| Concept | Purpose | Rust Tool |
|---------|---------|-----------|
| Compare-and-swap | Atomic conditional update | `AtomicT::compare_exchange` |
| Epoch reclamation | Safe deferred memory freeing | `crossbeam-epoch` |
| Hazard pointers | Per-thread pointer protection | `haphazard` crate |
| Lock-free stack/queue | MPMC data structures | `crossbeam::queue` |
| Work stealing | Dynamic load balancing | `crossbeam::deque` |
| RCU pointer swap | Fast config/state updates | `arc-swap` crate |
| Once initialization | Lock-free lazy init | `std::sync::OnceLock` |

Lock-free programming in Rust is made tractable by the type system — the ownership model prevents entire classes of race conditions that plague C/C++ implementations. The `crossbeam` ecosystem provides battle-tested building blocks so you rarely need to implement these primitives from scratch.