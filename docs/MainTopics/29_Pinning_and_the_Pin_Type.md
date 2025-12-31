# Pinning and the Pin Type in Rust

## **Core Concepts**

**Pin<T>** is a wrapper type that provides a guarantee: once a value is pinned, it won't move in memory. This is crucial for:
- Self-referential structures (structs with pointers to their own fields)
- Async/await (futures often contain self-references)
- Intrusive data structures

**The Unpin Trait** indicates types that are safe to move even when pinned. Most Rust types are `Unpin` by default. To opt-out (making a type immovable when pinned), you add `PhantomPinned`.

## **Why Pinning is Necessary**

Without pinning, self-referential types would be unsafe because Rust moves values by default. If a struct has a pointer to its own field and gets moved, the pointer becomes invalid (dangling pointer).

The guide includes examples of:
- The fundamental problem with self-referential structs
- How to correctly implement self-referential types using Pin
- Pin projection (accessing fields of pinned structs)
- Intrusive linked lists
- Why async/await needs pinning

## **Key Insight**

Pin is Rust's way of saying "I promise this value won't move" - a guarantee necessary for safe self-referential data structures. It's primarily used behind the scenes in async Rust, but understanding it helps you work with advanced patterns and troubleshoot async code issues.


```rust
// ============================================================================
// PINNING AND THE PIN TYPE IN RUST
// ============================================================================
// Understanding Pin<T>, Unpin, self-referential types, and why pinning matters

// ============================================================================
// 1. THE PROBLEM: SELF-REFERENTIAL STRUCTURES
// ============================================================================

// This is what we CANNOT safely do in Rust without Pin:
struct SelfReferential {
    data: String,
    pointer: *const String, // Points to 'data' field
}

// Why is this dangerous? Let's see:
fn demonstrate_problem() {
    let mut node = SelfReferential {
        data: String::from("Hello"),
        pointer: std::ptr::null(),
    };
    
    // Make pointer reference our own data
    node.pointer = &node.data as *const String;
    
    // PROBLEM: If we move 'node', the pointer becomes invalid!
    // let node2 = node; // The pointer still points to the old location!
    
    println!("This demonstrates why self-referential structs are dangerous");
}

// ============================================================================
// 2. WHAT IS PIN<T>?
// ============================================================================

// Pin<T> is a wrapper that guarantees the pointed-to value won't move in memory.
// It's defined as: pub struct Pin<P> { pointer: P }
// where P is typically a pointer type like Box<T> or &mut T

use std::pin::Pin;
use std::marker::PhantomPinned;

// Example: A simple pinned value
fn basic_pin_example() {
    let value = Box::new(42);
    let pinned = Box::pin(value); // Creates Pin<Box<i32>>
    
    // Pin<Box<T>> ensures the value inside won't move
    println!("Pinned value: {}", pinned);
}

// ============================================================================
// 3. THE UNPIN TRAIT
// ============================================================================

// Most types in Rust implement Unpin automatically.
// Unpin means "it's safe to move this type even when pinned"
// Think of it as "unpinnable" - the type doesn't need pinning guarantees

fn unpin_example() {
    let mut value = 42;
    let pinned = Pin::new(&mut value);
    
    // Because i32 implements Unpin, we can get the value back
    let unpinned = Pin::into_inner(pinned);
    *unpinned = 100;
    
    println!("Value: {}", value); // 100
}

// Types that are !Unpin (NOT Unpin) cannot be safely moved once pinned.
// You opt out of Unpin using PhantomPinned:

struct NotUnpin {
    data: String,
    _pin: PhantomPinned, // This makes the type !Unpin
}

// ============================================================================
// 4. SELF-REFERENTIAL STRUCT WITH PIN
// ============================================================================

// Here's how to properly create a self-referential structure:

struct SelfRef {
    data: String,
    pointer: *const String,
    _pin: PhantomPinned, // Prevents the type from being Unpin
}

impl SelfRef {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfRef {
            data,
            pointer: std::ptr::null(),
            _pin: PhantomPinned,
        });
        
        // SAFETY: We're setting up the self-reference here.
        // The value is pinned, so it won't move.
        unsafe {
            let ptr = &boxed.data as *const String;
            let mut_ref = Pin::get_unchecked_mut(Pin::as_mut(&mut boxed));
            mut_ref.pointer = ptr;
        }
        
        boxed
    }
    
    fn data(&self) -> &str {
        &self.data
    }
    
    fn pointer_data(&self) -> &str {
        // SAFETY: pointer is valid because we're pinned
        unsafe { &*self.pointer }
    }
}

fn self_referential_example() {
    let pinned = SelfRef::new(String::from("Pinned data!"));
    
    println!("Data: {}", pinned.data());
    println!("Pointer data: {}", pinned.pointer_data());
    
    // Both print the same thing because pointer references data
    assert_eq!(pinned.data(), pinned.pointer_data());
}

// ============================================================================
// 5. WHY PINNING IS NECESSARY: ASYNC/AWAIT
// ============================================================================

// The main use case for Pin is in async Rust. Futures can be self-referential:

use std::future::Future;
use std::task::{Context, Poll};

struct MyFuture {
    data: String,
    // In real async code, this might be a reference to 'data'
    // created when the future is polled
    state: Option<*const String>,
    _pin: PhantomPinned,
}

impl Future for MyFuture {
    type Output = String;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We don't move out of self
        unsafe {
            let this = self.get_unchecked_mut();
            
            if this.state.is_none() {
                // Setup self-reference on first poll
                this.state = Some(&this.data as *const String);
            }
            
            // Do async work...
            Poll::Ready(this.data.clone())
        }
    }
}

// ============================================================================
// 6. PIN PROJECTION
// ============================================================================

// When you have a Pin<&mut Struct>, you often need to access pinned fields.
// This is called "pin projection"

struct Container {
    pinned_field: String,
    unpinned_field: i32,
    _pin: PhantomPinned,
}

impl Container {
    // Project to get a pinned reference to a field
    fn project_pinned(self: Pin<&mut Self>) -> Pin<&mut String> {
        // SAFETY: pinned_field doesn't implement Drop and moving it
        // wouldn't break pinning invariants
        unsafe {
            let this = self.get_unchecked_mut();
            Pin::new_unchecked(&mut this.pinned_field)
        }
    }
    
    // For Unpin fields, we can get a regular mutable reference
    fn project_unpinned(self: Pin<&mut Self>) -> &mut i32 {
        // SAFETY: i32 is Unpin, so this is safe
        unsafe {
            &mut self.get_unchecked_mut().unpinned_field
        }
    }
}

// ============================================================================
// 7. PRACTICAL EXAMPLE: INTRUSIVE LINKED LIST
// ============================================================================

// A node that stores pointers to neighbors
struct ListNode {
    value: i32,
    next: Option<*const ListNode>,
    _pin: PhantomPinned,
}

impl ListNode {
    fn new(value: i32) -> Pin<Box<Self>> {
        Box::pin(ListNode {
            value,
            next: None,
            _pin: PhantomPinned,
        })
    }
    
    fn set_next(mut self: Pin<&mut Self>, next: *const ListNode) {
        unsafe {
            self.get_unchecked_mut().next = Some(next);
        }
    }
    
    fn get_next(&self) -> Option<&ListNode> {
        unsafe { self.next.map(|ptr| &*ptr) }
    }
}

fn linked_list_example() {
    let mut node1 = ListNode::new(1);
    let node2 = ListNode::new(2);
    
    // Link node1 to node2
    let node2_ptr = &*node2 as *const ListNode;
    node1.as_mut().set_next(node2_ptr);
    
    println!("Node 1 value: {}", node1.value);
    if let Some(next) = node1.get_next() {
        println!("Node 1's next value: {}", next.value);
    }
}

// ============================================================================
// 8. COMMON PATTERNS AND BEST PRACTICES
// ============================================================================

fn pin_patterns() {
    // Pattern 1: Creating a pinned value
    let pinned = Box::pin(42);
    
    // Pattern 2: Pinning a mutable reference
    let mut value = String::from("hello");
    let pinned_ref = Pin::new(&mut value);
    
    // Pattern 3: Using pin_mut! macro (from futures crate)
    // use futures::pin_mut;
    // let mut value = 42;
    // pin_mut!(value);
    
    println!("Pinned patterns demonstrated");
}

// When to use Pin:
// 1. Implementing Future (async/await)
// 2. Self-referential structs
// 3. Intrusive data structures
// 4. FFI with C++ objects that can't move
// 5. Embedded programming with memory-mapped I/O

// ============================================================================
// MAIN FUNCTION TO DEMONSTRATE ALL EXAMPLES
// ============================================================================

fn main() {
    println!("=== Rust Pinning Examples ===\n");
    
    println!("1. Demonstrating the problem:");
    demonstrate_problem();
    println!();
    
    println!("2. Basic pin example:");
    basic_pin_example();
    println!();
    
    println!("3. Unpin example:");
    unpin_example();
    println!();
    
    println!("4. Self-referential structure:");
    self_referential_example();
    println!();
    
    println!("5. Linked list example:");
    linked_list_example();
    println!();
    
    println!("6. Pin patterns:");
    pin_patterns();
}

// ============================================================================
// KEY TAKEAWAYS
// ============================================================================

/*
1. Pin<T> guarantees a value won't move in memory
2. Most types are Unpin (can be safely moved even when pinned)
3. Use PhantomPinned to make a type !Unpin (not moveable when pinned)
4. Self-referential structs need Pin to be safe
5. Pin is essential for async/await (futures are often self-referential)
6. unsafe code is often needed when working with Pin
7. Pin projection lets you access fields of pinned structs
8. Box::pin() is the easiest way to create a pinned heap value
*/

```