# 61. The Allocator API and Custom Allocators

- **`GlobalAlloc`** — the stable trait, `Layout`, manual `alloc`/`dealloc` calls, and a step-by-step logging allocator example. Also shows drop-in replacements with `mimalloc` and `jemalloc`.
- **The `Allocator` trait** — the nightly API with its richer interface (`grow`, `shrink`, fat-pointer returns) and how it differs from `GlobalAlloc`.
- **Bump allocators** — a complete from-scratch implementation with alignment handling and a no-op `deallocate`, plus usage with `Vec` and `Box` via `new_in`.
- **Arena allocators** — safe usage with `typed-arena`, a lifetime-based tree structure example that avoids `Rc`/`RefCell`, and a multi-chunk arena built from scratch.
- **Pool allocators** — a `MaybeUninit`-based fixed-size slot pool with a free list for O(1) alloc/dealloc.
- **Allocator-aware collections** — `Vec`, `Box`, `String`, and `HashMap` with custom allocators on nightly.
- **Tracking allocator** — counting live allocated bytes for debugging.
- **Decision table** — matching workload patterns to the right allocator strategy.

---

## Table of Contents

1. [Introduction](#introduction)
2. [How Rust Memory Allocation Works](#how-rust-memory-allocation-works)
3. [The `GlobalAlloc` Trait](#the-globalalloc-trait)
4. [Writing a Custom Global Allocator](#writing-a-custom-global-allocator)
5. [The `Allocator` Trait (Nightly)](#the-allocator-trait-nightly)
6. [Bump Allocators](#bump-allocators)
7. [Arena Allocators](#arena-allocators)
8. [Allocator-Aware Collections](#allocator-aware-collections)
9. [Pool Allocators](#pool-allocators)
10. [Tracking and Debugging Allocations](#tracking-and-debugging-allocations)
11. [When to Use Custom Allocators](#when-to-use-custom-allocators)
12. [Summary](#summary)

---

## Introduction

Rust's memory model gives you full control over how heap memory is allocated and freed. By default, Rust uses the system allocator (typically `malloc`/`free` on Unix or `HeapAlloc`/`HeapFree` on Windows), but this is a pluggable abstraction. The **Allocator API** lets you:

- **Replace the global allocator** with a custom implementation (e.g., `jemalloc`, `mimalloc`, or a fully custom one).
- **Pass allocators to individual collections** so that `Vec`, `Box`, `HashMap`, etc. can use different memory strategies for different use cases.
- **Build specialized allocators** like bump allocators or arenas that are extremely fast for specific patterns.

Understanding the Allocator API is important for systems programming, game development, embedded systems, high-performance computing, and any domain where allocation latency or memory layout matters.

---

## How Rust Memory Allocation Works

When you write:

```rust
let v: Vec<i32> = Vec::new();
```

…Rust ultimately calls into an **allocator** to obtain heap memory. The chain looks like this:

```
Vec::push()  →  alloc::alloc::alloc()  →  #[global_allocator]  →  OS
```

There are two levels of the abstraction:

| Level | Trait | Scope |
|---|---|---|
| Global / process-wide | `GlobalAlloc` | One allocator for the entire binary |
| Per-collection | `Allocator` (nightly) | Each collection instance can have its own |

---

## The `GlobalAlloc` Trait

`GlobalAlloc` is a **stable** trait in `std::alloc` that defines the interface every global allocator must satisfy.

```rust
pub unsafe trait GlobalAlloc {
    // Required
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    // Optional (have default implementations)
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { ... }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 { ... }
}
```

### `Layout`

`Layout` describes the **size** and **alignment** requirements of an allocation:

```rust
use std::alloc::Layout;

// Layout for a single u64
let layout = Layout::new::<u64>();
println!("size: {}, align: {}", layout.size(), layout.align());

// Layout for an array of 100 u32s
let array_layout = Layout::array::<u32>(100).unwrap();

// Custom layout
let custom = Layout::from_size_align(256, 16).unwrap();
```

### Using the Global Allocator Directly

You can call the global allocator manually with the `alloc` and `dealloc` free functions:

```rust
use std::alloc::{alloc, dealloc, Layout};

fn main() {
    let layout = Layout::array::<u8>(1024).unwrap();

    unsafe {
        let ptr = alloc(layout);
        if ptr.is_null() {
            eprintln!("Allocation failed!");
            return;
        }

        // Write some data
        ptr.write(42u8);
        println!("First byte: {}", ptr.read());

        // Must always dealloc with the same layout
        dealloc(ptr, layout);
    }
}
```

> ⚠️ **Safety:** You are responsible for ensuring that the `Layout` passed to `dealloc` exactly matches the one used for `alloc`, and that the pointer is valid and not yet freed.

---

## Writing a Custom Global Allocator

To replace the global allocator, implement `GlobalAlloc` and annotate it with `#[global_allocator]`.

### Example: A Logging Allocator

This wraps the system allocator and logs every allocation:

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct LoggingAllocator;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for LoggingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            // Note: avoid println! here in production — it may allocate!
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static A: LoggingAllocator = LoggingAllocator;

fn main() {
    let _v: Vec<i32> = vec![1, 2, 3, 4, 5];
    let _s = String::from("hello");

    println!(
        "Allocations: {}, Deallocations: {}",
        ALLOC_COUNT.load(Ordering::Relaxed),
        DEALLOC_COUNT.load(Ordering::Relaxed),
    );
}
```

### Example: Using `mimalloc` as the Global Allocator

The `mimalloc` crate provides a drop-in high-performance allocator:

```toml
# Cargo.toml
[dependencies]
mimalloc = { version = "0.1", default-features = false }
```

```rust
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    // All heap allocations now use mimalloc
    let v: Vec<u8> = vec![0u8; 1_000_000];
    println!("Allocated {} bytes via mimalloc", v.len());
}
```

### Example: Using `jemalloc`

```toml
[dependencies]
tikv-jemallocator = "0.5"
```

```rust
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

---

## The `Allocator` Trait (Nightly)

The `Allocator` trait (available on nightly via `#![feature(allocator_api)]`) is a more powerful, **per-instance** abstraction. Unlike `GlobalAlloc`, it:

- Is **object-safe**.
- Allows collections to accept a **type-parameterized allocator**.
- Supports **growing and shrinking** allocations.
- Returns a `NonNull<[u8]>` (a fat pointer with size) rather than a raw thin pointer.

```rust
#![feature(allocator_api)]

pub unsafe trait Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

    // Provided methods
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { ... }
    unsafe fn grow(&self, ptr, old_layout, new_layout) -> Result<NonNull<[u8]>, AllocError> { ... }
    unsafe fn shrink(&self, ptr, old_layout, new_layout) -> Result<NonNull<[u8]>, AllocError> { ... }
}
```

The `System` struct implements both `GlobalAlloc` and `Allocator`.

---

## Bump Allocators

A **bump allocator** (also called a *linear allocator*) is the simplest and fastest possible allocator. It works by:

1. Owning a large contiguous block of memory.
2. Maintaining a single pointer (the "bump pointer") that marks the next free byte.
3. Allocating by simply advancing (bumping) the pointer.
4. **Never freeing individual allocations** — memory is only reclaimed when the entire allocator is reset or dropped.

```
Before alloc:  [################..........]
                                ^-- bump ptr

After alloc(N): [################NNNNNN....]
                                       ^-- bump ptr
```

### Implementing a Simple Bump Allocator

```rust
#![feature(allocator_api)]

use std::alloc::{AllocError, Allocator, Layout};
use std::cell::Cell;
use std::ptr::NonNull;

pub struct BumpAllocator {
    buffer: Box<[u8]>,
    offset: Cell<usize>,
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        BumpAllocator {
            buffer: vec![0u8; capacity].into_boxed_slice(),
            offset: Cell::new(0),
        }
    }

    pub fn reset(&self) {
        self.offset.set(0);
    }

    pub fn bytes_used(&self) -> usize {
        self.offset.get()
    }

    pub fn bytes_remaining(&self) -> usize {
        self.buffer.len() - self.offset.get()
    }
}

unsafe impl Allocator for BumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let start = self.offset.get();

        // Align the current offset
        let align = layout.align();
        let aligned_start = (start + align - 1) & !(align - 1);
        let end = aligned_start + layout.size();

        if end > self.buffer.len() {
            return Err(AllocError);
        }

        self.offset.set(end);

        let ptr = unsafe {
            NonNull::new_unchecked(self.buffer.as_ptr().add(aligned_start) as *mut u8)
        };

        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: individual frees are ignored in a bump allocator
    }
}
```

### Using the Bump Allocator with Collections

```rust
#![feature(allocator_api)]

fn main() {
    let bump = BumpAllocator::new(4096);

    // Vec backed by the bump allocator
    let mut v: Vec<i32, &BumpAllocator> = Vec::new_in(&bump);
    v.push(1);
    v.push(2);
    v.push(3);

    // Box backed by the bump allocator
    let boxed: Box<str, &BumpAllocator> = Box::from_str_in("hello world", &bump).unwrap();

    println!("Vec: {:?}", v);
    println!("Box: {}", boxed);
    println!("Bytes used: {}", bump.bytes_used());

    // Reset: reclaim ALL memory at once
    drop(v);
    drop(boxed);
    bump.reset();
    println!("After reset, bytes used: {}", bump.bytes_used());
}
```

**Performance characteristics:**
- Allocation: O(1), just a pointer add and bounds check.
- Deallocation: O(1) no-op.
- Reset: O(1), single store.
- No fragmentation.
- Not suitable for long-lived, individually freed objects.

---

## Arena Allocators

An **arena allocator** extends the bump allocator concept by managing **typed objects** and ensuring their destructors are called when the arena is dropped, even though memory is not freed individually.

### Key Properties

- Objects are allocated into a contiguous or linked pool of memory chunks.
- Individual `free()` calls are not supported.
- When the arena is dropped, all contained objects are properly destructed in reverse order (or all at once).
- Very cache-friendly since related objects are co-located.

### Using the `typed-arena` Crate

The `typed-arena` crate provides a safe, production-ready arena:

```toml
[dependencies]
typed-arena = "2.0"
```

```rust
use typed_arena::Arena;

fn main() {
    let arena: Arena<String> = Arena::new();

    // Allocate into the arena — returns a &mut String
    let s1 = arena.alloc(String::from("hello"));
    let s2 = arena.alloc(String::from("world"));

    s1.push_str(", modified");

    println!("{}", s1); // "hello, modified"
    println!("{}", s2); // "world"

    // When `arena` drops here, all Strings are properly dropped
}
```

### Arena for a Tree Structure

Arenas shine when building graph/tree structures where nodes reference each other:

```rust
use typed_arena::Arena;

struct Node<'arena> {
    value: i32,
    children: Vec<&'arena Node<'arena>>,
}

fn build_tree<'arena>(arena: &'arena Arena<Node<'arena>>) -> &'arena Node<'arena> {
    let leaf1 = arena.alloc(Node { value: 3, children: vec![] });
    let leaf2 = arena.alloc(Node { value: 4, children: vec![] });
    let child = arena.alloc(Node { value: 2, children: vec![leaf1, leaf2] });
    let root  = arena.alloc(Node { value: 1, children: vec![child] });
    root
}

fn print_tree(node: &Node, depth: usize) {
    println!("{}{}", " ".repeat(depth * 2), node.value);
    for child in &node.children {
        print_tree(child, depth + 1);
    }
}

fn main() {
    let arena = Arena::new();
    let root = build_tree(&arena);
    print_tree(root, 0);
    // Output:
    // 1
    //   2
    //     3
    //     4
}
```

> The arena's lifetime `'arena` ties all node references together, preventing dangling pointers while avoiding `Rc`/`RefCell` overhead.

### Building a Multi-Chunk Arena from Scratch

```rust
use std::cell::RefCell;

pub struct Arena {
    chunks: RefCell<Vec<Vec<u8>>>,
    chunk_size: usize,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self {
        Arena {
            chunks: RefCell::new(vec![Vec::with_capacity(chunk_size)]),
            chunk_size,
        }
    }

    pub fn alloc_bytes(&self, n: usize) -> &mut [u8] {
        let mut chunks = self.chunks.borrow_mut();
        let last = chunks.last_mut().unwrap();

        if last.len() + n > last.capacity() {
            let new_size = self.chunk_size.max(n);
            chunks.push(Vec::with_capacity(new_size));
        }

        let chunk = chunks.last_mut().unwrap();
        let start = chunk.len();
        chunk.extend(std::iter::repeat(0).take(n));

        // SAFETY: we are extending the lifetime tied to self's lifetime
        unsafe {
            let ptr = chunk.as_mut_ptr().add(start);
            std::slice::from_raw_parts_mut(ptr, n)
        }
    }
}
```

---

## Allocator-Aware Collections

On **nightly** Rust, standard library collections are generic over an allocator type parameter. This allows passing a custom allocator at construction time.

### `Vec<T, A: Allocator>`

```rust
#![feature(allocator_api)]
use std::alloc::System;

fn main() {
    // Vec backed by the system allocator explicitly
    let mut v: Vec<u32, System> = Vec::new_in(System);
    v.extend([1, 2, 3, 4, 5]);
    println!("{:?}", v);
}
```

### `Box<T, A: Allocator>`

```rust
#![feature(allocator_api)]

fn main() {
    let bump = BumpAllocator::new(1024);

    // Allocate a struct on the bump arena
    let boxed = Box::new_in([1u64; 8], &bump);
    println!("{:?}", &*boxed);

    println!("Arena usage: {} bytes", bump.bytes_used());
}
```

### `HashMap<K, V, S, A: Allocator>` (via `hashbrown`)

The `hashbrown` crate (which backs `std::collections::HashMap`) supports the Allocator API:

```toml
[dependencies]
hashbrown = { version = "0.14", features = ["nightly"] }
```

```rust
use hashbrown::HashMap;

fn main() {
    let bump = BumpAllocator::new(65536);
    let mut map: HashMap<&str, i32, _, &BumpAllocator> =
        HashMap::new_in(&bump);

    map.insert("one", 1);
    map.insert("two", 2);
    map.insert("three", 3);

    println!("{:?}", map.get("two"));
    println!("Arena used: {} bytes", bump.bytes_used());
}
```

### `String` with a Custom Allocator

```rust
#![feature(allocator_api)]

fn main() {
    let bump = BumpAllocator::new(256);
    let mut s = String::new_in(&bump);
    s.push_str("Hello, allocator!");
    println!("{}", s);
}
```

---

## Pool Allocators

A **pool allocator** (or *slab allocator*) pre-allocates a fixed number of same-sized slots and dispenses them on demand. It supports O(1) individual allocation and deallocation:

```rust
use std::mem::MaybeUninit;

pub struct PoolAllocator<T, const N: usize> {
    slots: Box<[MaybeUninit<T>; N]>,
    free_list: Vec<usize>,
}

impl<T, const N: usize> PoolAllocator<T, N> {
    pub fn new() -> Self {
        PoolAllocator {
            // SAFETY: MaybeUninit doesn't require initialization
            slots: Box::new(unsafe { MaybeUninit::uninit().assume_init() }),
            free_list: (0..N).collect(),
        }
    }

    pub fn alloc(&mut self, value: T) -> Option<usize> {
        let index = self.free_list.pop()?;
        self.slots[index].write(value);
        Some(index)
    }

    pub fn get(&self, index: usize) -> &T {
        // SAFETY: caller must ensure slot was allocated
        unsafe { self.slots[index].assume_init_ref() }
    }

    pub fn free(&mut self, index: usize) {
        // SAFETY: caller must ensure slot was allocated and is no longer used
        unsafe { self.slots[index].assume_init_drop() };
        self.free_list.push(index);
    }
}

fn main() {
    let mut pool: PoolAllocator<String, 8> = PoolAllocator::new();

    let i1 = pool.alloc(String::from("Alice")).unwrap();
    let i2 = pool.alloc(String::from("Bob")).unwrap();

    println!("{}", pool.get(i1)); // Alice
    println!("{}", pool.get(i2)); // Bob

    pool.free(i1);

    let i3 = pool.alloc(String::from("Charlie")).unwrap();
    println!("{}", pool.get(i3)); // Charlie — reuses slot from i1
}
```

---

## Tracking and Debugging Allocations

### Counting Total Allocated Bytes

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static TRACKER: TrackingAllocator = TrackingAllocator;

fn allocated_bytes() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
}

fn main() {
    let before = allocated_bytes();
    let v: Vec<u8> = vec![0u8; 1024];
    let after_alloc = allocated_bytes();

    println!("Before: {} bytes", before);
    println!("After vec alloc: {} bytes (delta: {})", after_alloc, after_alloc - before);

    drop(v);
    println!("After drop: {} bytes", allocated_bytes());
}
```

---

## When to Use Custom Allocators

| Scenario | Recommended Strategy |
|---|---|
| **Short-lived request processing** (web servers, parsers) | Bump/arena allocator per request, reset at end |
| **Game frame allocation** | Per-frame arena, reset each frame tick |
| **Many same-sized objects** | Pool/slab allocator |
| **Embedded systems (no OS)** | Bump allocator over a static byte array |
| **Performance-critical hot paths** | Profile first, then try `mimalloc` or `jemalloc` globally |
| **Graph/tree structures** | Typed arena to avoid `Rc`/`RefCell` overhead |
| **Debugging memory issues** | Tracking allocator for counting/logging |

---

## Summary

Rust's allocator system is a layered, composable abstraction:

- **`GlobalAlloc`** (stable) lets you replace the process-wide allocator with one `#[global_allocator]` declaration. It's the right tool for swapping in `mimalloc`, `jemalloc`, or a custom tracking wrapper.

- **`Allocator`** (nightly) is a more powerful trait that allows per-collection custom allocators. Collections like `Vec`, `Box`, `String`, and `HashMap` become generic over `A: Allocator`.

- **Bump allocators** are the fastest possible allocators — O(1) allocation with zero bookkeeping — suited for short-lived, batch-freed workloads.

- **Arena allocators** extend bump allocation with safe lifetime management and proper destructor handling, ideal for graph/tree data structures.

- **Pool allocators** enable O(1) allocation and individual deallocation for fixed-size objects, avoiding fragmentation.

- **Allocator-aware collections** (nightly) make it possible to compose all of the above with the standard data structures you already know.

The key insight is that **the best allocator depends entirely on the allocation pattern**. Knowing your workload — how long objects live, how many are allocated at once, whether sizes are uniform — determines which strategy wins.

---

*See also: [`std::alloc` documentation](https://doc.rust-lang.org/std/alloc/), [`typed-arena` crate](https://crates.io/crates/typed-arena), [`bumpalo` crate](https://crates.io/crates/bumpalo)*