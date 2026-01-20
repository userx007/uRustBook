# When `Rc<T>` is Needed in Rust

## Overview

`Rc<T>` (Reference Counted) provides **shared ownership** of heap-allocated data with **single-threaded** reference counting. Use `Arc<T>` for multi-threaded scenarios.

## 1. **Multiple Owners of the Same Data**

The fundamental use case: when multiple parts of your code need to own the same data.

```rust
use std::rc::Rc;

struct Node {
    value: i32,
    // Can't use Box - would have single owner
    parent: Option<Rc<Node>>,
    children: Vec<Rc<Node>>,
}

// Multiple children can share the same parent
let parent = Rc::new(Node {
    value: 1,
    parent: None,
    children: vec![],
});

let child1 = Rc::new(Node {
    value: 2,
    parent: Some(Rc::clone(&parent)),  // parent now has 2 references
    children: vec![],
});

let child2 = Rc::new(Node {
    value: 3,
    parent: Some(Rc::clone(&parent)),  // parent now has 3 references
    children: vec![],
});
```

## 2. **Graph Data Structures**

Graphs often have multiple edges pointing to the same node, requiring shared ownership.

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct GraphNode {
    value: i32,
    neighbors: Vec<Rc<RefCell<GraphNode>>>,
}

// Multiple nodes can point to the same neighbor
let node_a = Rc::new(RefCell::new(GraphNode {
    value: 1,
    neighbors: vec![],
}));

let node_b = Rc::new(RefCell::new(GraphNode {
    value: 2,
    neighbors: vec![Rc::clone(&node_a)],
}));

let node_c = Rc::new(RefCell::new(GraphNode {
    value: 3,
    neighbors: vec![Rc::clone(&node_a)],  // Also points to node_a
}));
```

## 3. **Shared Immutable Configuration/State**

When multiple components need read-only access to shared configuration.

```rust
use std::rc::Rc;

struct Config {
    database_url: String,
    max_connections: u32,
    timeout: u64,
}

struct Database {
    config: Rc<Config>,
}

struct ApiServer {
    config: Rc<Config>,
}

struct CacheManager {
    config: Rc<Config>,
}

fn main() {
    let config = Rc::new(Config {
        database_url: "localhost:5432".to_string(),
        max_connections: 100,
        timeout: 30,
    });

    let db = Database {
        config: Rc::clone(&config),
    };
    
    let api = ApiServer {
        config: Rc::clone(&config),
    };
    
    let cache = CacheManager {
        config: Rc::clone(&config),
    };
    
    // All components share the same config
}
```

## 4. **Parent-Child Relationships (Trees with Parent Pointers)**

Trees where children need to reference their parent.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TreeNode {
    value: i32,
    parent: RefCell<Weak<TreeNode>>,  // Weak to avoid cycles
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }
    
    fn add_child(parent: &Rc<TreeNode>, child_value: i32) -> Rc<TreeNode> {
        let child = TreeNode::new(child_value);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
}
```

## 5. **Observer Pattern / Event Listeners**

Multiple observers need to hold references to the same subject.

```rust
use std::rc::Rc;
use std::cell::RefCell;

trait Observer {
    fn update(&self, data: &str);
}

struct Subject {
    observers: Vec<Rc<dyn Observer>>,
    state: String,
}

impl Subject {
    fn attach(&mut self, observer: Rc<dyn Observer>) {
        self.observers.push(observer);
    }
    
    fn notify(&self) {
        for observer in &self.observers {
            observer.update(&self.state);
        }
    }
}

struct ConcreteObserver {
    id: String,
}

impl Observer for ConcreteObserver {
    fn update(&self, data: &str) {
        println!("Observer {} received: {}", self.id, data);
    }
}
```

## 6. **Caching / Memoization**

When multiple cache entries might reference the same underlying data.

```rust
use std::rc::Rc;
use std::collections::HashMap;

struct ExpensiveData {
    computation_result: Vec<u8>,
}

struct Cache {
    data: HashMap<String, Rc<ExpensiveData>>,
}

impl Cache {
    fn get_or_compute(&mut self, key: &str) -> Rc<ExpensiveData> {
        self.data
            .entry(key.to_string())
            .or_insert_with(|| {
                // Expensive computation
                Rc::new(ExpensiveData {
                    computation_result: vec![1, 2, 3],
                })
            })
            .clone()
    }
}
```

## 7. **Avoiding Deep Copies of Immutable Data**

When you want to share large immutable data without cloning.

```rust
use std::rc::Rc;

#[derive(Clone)]
struct LargeImmutableData {
    bytes: Vec<u8>,
}

// Without Rc - cloning is expensive
fn process_without_rc(data: LargeImmutableData) {
    // This clones all the data
    let copy1 = data.clone();
    let copy2 = data.clone();
}

// With Rc - only the pointer is cloned
fn process_with_rc(data: Rc<LargeImmutableData>) {
    let copy1 = Rc::clone(&data);  // Cheap - just increments counter
    let copy2 = Rc::clone(&data);  // Cheap
}
```

## 8. **Doubly-Linked Lists**

Nodes need to reference both next and previous nodes.

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct DoublyLinkedNode {
    value: i32,
    next: Option<Rc<RefCell<DoublyLinkedNode>>>,
    prev: Option<Weak<RefCell<DoublyLinkedNode>>>,  // Weak to avoid cycles
}
```

## 9. **Undo/Redo Systems**

Multiple states in history might share unchanged portions.

```rust
use std::rc::Rc;

#[derive(Clone)]
struct DocumentState {
    content: Rc<String>,  // Shared if unchanged
    cursor_position: usize,
}

struct UndoStack {
    states: Vec<DocumentState>,
}

impl UndoStack {
    fn push_state(&mut self, state: DocumentState) {
        self.states.push(state);
    }
    
    // States can share the same content if it hasn't changed
}
```

## 10. **Rc with RefCell for Shared Mutable State**

When you need shared ownership **and** interior mutability.

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct SharedCounter {
    count: Rc<RefCell<i32>>,
}

impl SharedCounter {
    fn new() -> Self {
        SharedCounter {
            count: Rc::new(RefCell::new(0)),
        }
    }
    
    fn increment(&self) {
        *self.count.borrow_mut() += 1;
    }
    
    fn get(&self) -> i32 {
        *self.count.borrow()
    }
}

// Multiple owners can mutate the same data
let counter1 = SharedCounter::new();
let counter2 = SharedCounter {
    count: Rc::clone(&counter1.count),
};

counter1.increment();
counter2.increment();
assert_eq!(counter1.get(), 2);
```

## When **NOT** to Use `Rc<T>`

❌ **Multi-threaded code** - Use `Arc<T>` instead  
❌ **Single ownership is sufficient** - Use `Box<T>` or direct ownership  
❌ **Borrowing works** - Use `&T` or `&mut T`  
❌ **Data needs to be mutable by default** - Consider `Rc<RefCell<T>>` but be aware of runtime borrow checking  
❌ **Performance-critical code** - Reference counting has overhead  
❌ **Creating reference cycles** - Use `Weak<T>` to break cycles  

## Important Caveats

### Memory Leaks via Reference Cycles

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    next: Option<Rc<RefCell<Node>>>,
}

// ❌ This creates a cycle and leaks memory!
let a = Rc::new(RefCell::new(Node { next: None }));
let b = Rc::new(RefCell::new(Node { next: Some(Rc::clone(&a)) }));
a.borrow_mut().next = Some(Rc::clone(&b));
// a and b now reference each other - memory leak!

// ✅ Use Weak to break cycles
struct SafeNode {
    next: Option<Rc<RefCell<SafeNode>>>,
    prev: Option<Weak<RefCell<SafeNode>>>,  // Weak breaks the cycle
}
```

### Runtime Borrow Checking with RefCell

```rust
use std::rc::Rc;
use std::cell::RefCell;

let data = Rc::new(RefCell::new(5));
let borrow1 = data.borrow();
// This will panic at runtime!
// let borrow_mut = data.borrow_mut();  // Can't have mut and immut borrows
```

## Quick Decision Guide

```
Do you need multiple owners? → NO: Use Box<T> or direct ownership
                              ↓ YES
Is this multi-threaded? → YES: Use Arc<T>
                          ↓ NO
Do you need mutability? → NO: Use Rc<T>
                          ↓ YES
Are you okay with runtime borrow checking? → YES: Use Rc<RefCell<T>>
                                              ↓ NO: Redesign
```

## Performance Characteristics

- **Clone cost**: O(1) - just increments reference count
- **Drop cost**: O(1) - just decrements reference count (deallocates if count reaches 0)
- **Memory overhead**: Two `usize` values per allocation (strong count + weak count)
- **Not thread-safe**: No atomic operations, faster than `Arc<T>` in single-threaded code

**Key insight**: `Rc<T>` is for **shared ownership** in **single-threaded** contexts. It allows multiple parts of your code to own the same data without copying, using reference counting to determine when to deallocate.