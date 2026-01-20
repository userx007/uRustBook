# Rust's reference counting and smart pointers 

**Core Smart Pointers:**
- **Box<T>** - Simple heap allocation, perfect for large data, recursive types, and trait objects
- **Rc<T>** - Reference counting for single-threaded shared ownership
- **Arc<T>** - Thread-safe atomic reference counting for concurrent code
- **Weak<T>** - Weak references to prevent memory leaks from circular references
- **Cow<T>** - Clone-on-write optimization for read-heavy workloads

**Advanced Topics:**
- Creating custom smart pointers with `Deref` and `Drop` traits
- Understanding deref coercion and how Rust automatically dereferences smart pointers
- Combining smart pointers with interior mutability (`RefCell`, `Mutex`)

**Key Takeaways:**
- Use `Box` when you need simple heap allocation
- Use `Rc`/`Arc` when multiple parts of your code need to own the same data
- Always use `Weak` for back-references to avoid memory leaks
- Use `Cow` to avoid unnecessary cloning in functions that might not need to modify data
- `Rc::clone()` is cheap - it only increments a counter, not a deep copy

# Rust Smart Pointers and Reference Counting

## Overview

Smart pointers in Rust are data structures that act like pointers but have additional metadata and capabilities. They implement the `Deref` and often `Drop` traits, enabling automatic memory management while maintaining Rust's safety guarantees.

---

## 1. Box\<T\> - Heap Allocation

`Box<T>` is the simplest smart pointer, providing heap allocation for values that would otherwise be stored on the stack.

### When to Use Box
- Store data on the heap instead of the stack
- Transfer ownership of large data without copying
- Enable recursive types (linked lists, trees)
- Store trait objects for dynamic dispatch

### Example: Basic Usage

```rust
fn main() {
    // Simple heap allocation
    let boxed_value = Box::new(5);
    println!("Boxed value: {}", boxed_value);
    
    // Large data on heap to avoid stack overflow
    let large_array = Box::new([0; 1000000]);
    
    // Box is automatically deallocated when it goes out of scope
}
```

### Example: Recursive Types

```rust
// This would cause infinite size without Box
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    let list = Cons(1, 
        Box::new(Cons(2, 
            Box::new(Cons(3, 
                Box::new(Nil))))));
}
```

### Example: Trait Objects

```rust
trait Animal {
    fn make_sound(&self) -> &str;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn make_sound(&self) -> &str { "Woof!" }
}

impl Animal for Cat {
    fn make_sound(&self) -> &str { "Meow!" }
}

fn main() {
    // Store different types implementing the same trait
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog),
        Box::new(Cat),
    ];
    
    for animal in animals {
        println!("{}", animal.make_sound());
    }
}
```
[More `Box<T>` examples here...](../Other/103_When_Box_is_Needed_in_Rust.md)

---

## 2. Rc\<T\> - Reference Counting (Single-threaded)

`Rc<T>` (Reference Counted) enables multiple ownership in single-threaded scenarios by tracking the number of references to a value.

### Key Characteristics
- **Not thread-safe** (use `Arc<T>` for threads)
- Provides **shared ownership** with immutable access
- Uses **non-atomic** reference counting for performance
- Automatically deallocates when reference count reaches zero

### Example: Multiple Ownership

```rust
use std::rc::Rc;

struct Data {
    value: i32,
}

fn main() {
    let data = Rc::new(Data { value: 42 });
    println!("Reference count: {}", Rc::strong_count(&data)); // 1
    
    let data_clone1 = Rc::clone(&data);
    println!("Reference count: {}", Rc::strong_count(&data)); // 2
    
    let data_clone2 = Rc::clone(&data);
    println!("Reference count: {}", Rc::strong_count(&data)); // 3
    
    println!("Value: {}", data.value);
    println!("Value from clone: {}", data_clone1.value);
    
    drop(data_clone1);
    println!("After drop: {}", Rc::strong_count(&data)); // 2
}
```

### Example: Shared Data Structure

```rust
use std::rc::Rc;

enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    let shared = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    
    // Multiple lists can share the same tail
    let list1 = Cons(3, Rc::clone(&shared));
    let list2 = Cons(4, Rc::clone(&shared));
    
    println!("Shared list reference count: {}", Rc::strong_count(&shared)); // 3
}
```

[More `Rc<T>` examples here...](../Other/104_When_Rc_is_Needed_in_Rust.md)

---

## 3. Arc\<T\> - Atomic Reference Counting (Thread-safe)

`Arc<T>` (Atomically Reference Counted) is the thread-safe version of `Rc<T>`, using atomic operations for reference counting.

### Key Characteristics
- **Thread-safe** reference counting
- Slight performance overhead compared to `Rc<T>`
- Required for sharing data across threads
- Provides **immutable** shared access

### Example: Sharing Data Across Threads

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];
    
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let sum: i32 = data_clone.iter().sum();
            println!("Thread {}: sum = {}", i, sum);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Original data still accessible: {:?}", data);
}
```

### Example: Arc with Mutex for Mutable Shared State

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
    
    println!("Final count: {}", *counter.lock().unwrap()); // 10
}
```

[More `Arc<T>` examples here...](../Other/105_When_Arc_is_Needed_in_Rust.md)

---

## 4. Weak\<T\> - Weak References

`Weak<T>` provides non-owning references that don't prevent deallocation. This is crucial for breaking reference cycles.

### Key Characteristics
- Doesn't increment **strong reference count**
- Must be upgraded to `Rc<T>` or `Arc<T>` to access data
- Returns `None` if the value has been deallocated
- Used to prevent **memory leaks** from circular references

### Example: Breaking Reference Cycles

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn main() {
    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    
    println!("Leaf strong count: {}", Rc::strong_count(&leaf)); // 1
    println!("Leaf weak count: {}", Rc::weak_count(&leaf));     // 0
    
    {
        let branch = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![Rc::clone(&leaf)]),
        });
        
        // Set parent without creating a cycle
        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
        
        println!("Branch strong count: {}", Rc::strong_count(&branch)); // 1
        println!("Leaf strong count: {}", Rc::strong_count(&leaf));     // 2
        println!("Leaf weak count: {}", Rc::weak_count(&leaf));         // 0
        println!("Branch weak count: {}", Rc::weak_count(&branch));     // 1
        
        // Try to access parent
        if let Some(parent) = leaf.parent.borrow().upgrade() {
            println!("Leaf's parent value: {}", parent.value);
        }
    }
    
    // branch is dropped here, but leaf is still valid
    println!("After branch dropped:");
    println!("Leaf strong count: {}", Rc::strong_count(&leaf)); // 1
    
    // Parent is now deallocated
    assert!(leaf.parent.borrow().upgrade().is_none());
}
```

### Example: Observer Pattern

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Subject {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

trait Observer {
    fn update(&self, message: &str);
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: RefCell::new(vec![]),
        }
    }
    
    fn attach(&self, observer: Weak<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }
    
    fn notify(&self, message: &str) {
        // Clean up dead observers and notify live ones
        self.observers.borrow_mut().retain(|weak| {
            if let Some(observer) = weak.upgrade() {
                observer.update(message);
                true
            } else {
                false // Remove dead observer
            }
        });
    }
}
```
[More `Weak<T>` examples here...](../Other/106_When_Weak_is_Needed_in_Rust.md)

---

## 5. Cow\<T\> - Clone on Write

`Cow<T>` (Clone on Write) provides efficient handling of data that might be borrowed or owned, cloning only when modification is needed.

### Key Characteristics
- Stores either borrowed or owned data
- Delays cloning until mutation is required
- Useful for optimizing read-heavy workloads
- Common with strings: `Cow<'a, str>`

### Example: String Processing

```rust
use std::borrow::Cow;

fn process_text(input: &str) -> Cow<str> {
    if input.contains("error") {
        // Need to modify, so clone
        Cow::Owned(input.replace("error", "ERROR"))
    } else {
        // No modification needed, just borrow
        Cow::Borrowed(input)
    }
}

fn main() {
    let text1 = "This is fine";
    let text2 = "This has an error";
    
    let result1 = process_text(text1);
    let result2 = process_text(text2);
    
    println!("Result 1 (borrowed): {}", result1);
    println!("Result 2 (owned): {}", result2);
    
    // Check which variant we have
    match result1 {
        Cow::Borrowed(s) => println!("No allocation for: {}", s),
        Cow::Owned(_) => println!("Allocated new string"),
    }
}
```

### Example: Efficient Configuration

```rust
use std::borrow::Cow;

struct Config<'a> {
    name: Cow<'a, str>,
    path: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn new(name: &'a str) -> Self {
        Config {
            name: Cow::Borrowed(name),
            path: Cow::Borrowed("/default/path"),
        }
    }
    
    fn with_custom_path(mut self, path: String) -> Self {
        self.path = Cow::Owned(path);
        self
    }
    
    fn normalize_name(mut self) -> Self {
        if self.name.contains(char::is_whitespace) {
            self.name = Cow::Owned(self.name.replace(' ', "_"));
        }
        self
    }
}

fn main() {
    let config1 = Config::new("my_config");
    println!("Config 1: {} at {}", config1.name, config1.path);
    
    let config2 = Config::new("my config")
        .normalize_name()
        .with_custom_path("/custom/path".to_string());
    println!("Config 2: {} at {}", config2.name, config2.path);
}
```

---

## 6. Custom Smart Pointers

You can create custom smart pointers by implementing the `Deref` and optionally `Drop` traits.

### Example: Simple Smart Pointer

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(value: T) -> MyBox<T> {
        MyBox(value)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        println!("Dropping MyBox!");
    }
}

fn main() {
    let x = MyBox::new(42);
    
    // Deref coercion allows us to use * operator
    assert_eq!(42, *x);
    
    // Can be used like a reference
    fn print_value(val: &i32) {
        println!("Value: {}", val);
    }
    
    print_value(&x); // Automatically dereferenced
}
```

### Example: Reference Counted Smart Pointer with Interior Mutability

```rust
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;

struct SharedMut<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> SharedMut<T> {
    fn new(value: T) -> Self {
        SharedMut {
            inner: Rc::new(RefCell::new(value)),
        }
    }
    
    fn get(&self) -> std::cell::Ref<T> {
        self.inner.borrow()
    }
    
    fn get_mut(&self) -> std::cell::RefMut<T> {
        self.inner.borrow_mut()
    }
}

impl<T> Clone for SharedMut<T> {
    fn clone(&self) -> Self {
        SharedMut {
            inner: Rc::clone(&self.inner),
        }
    }
}

fn main() {
    let value = SharedMut::new(vec![1, 2, 3]);
    let clone = value.clone();
    
    // Mutate through one reference
    value.get_mut().push(4);
    
    // See changes through another reference
    println!("Values: {:?}", *clone.get()); // [1, 2, 3, 4]
}
```

---

## 7. Deref Coercion

Deref coercion is a convenience feature that automatically converts references to smart pointers into references to their inner values.

### How It Works

When you pass a reference to a smart pointer where a reference is expected, Rust automatically calls `deref()` as many times as needed to match the expected type.

### Example: Automatic Dereferencing

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(value: T) -> MyBox<T> {
        MyBox(value)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn hello(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    let m = MyBox::new(String::from("Rust"));
    
    // Without deref coercion, we'd need:
    // hello(&(*m)[..]);
    
    // With deref coercion:
    hello(&m); // MyBox<String> -> String -> str
}
```

### Example: Method Call Deref Coercion

```rust
fn main() {
    let s = Box::new(String::from("hello"));
    
    // Box<String> -> String -> str
    println!("Length: {}", s.len());
    
    // Multiple levels of dereferencing
    let rc_box = std::rc::Rc::new(Box::new(String::from("world")));
    println!("Uppercase: {}", rc_box.to_uppercase());
    // Rc -> Box -> String -> str
}
```

### Deref Coercion Rules

1. From `&T` to `&U` when `T: Deref<Target=U>`
2. From `&mut T` to `&mut U` when `T: DerefMut<Target=U>`
3. From `&mut T` to `&U` when `T: Deref<Target=U>` (mutable to immutable)

---

## Comparison Table

| Smart Pointer | Thread-Safe | Multiple Owners | Mutability | Use Case |
|---------------|-------------|-----------------|------------|----------|
| `Box<T>` | Yes | No | Yes (with `mut`) | Heap allocation, trait objects |
| `Rc<T>` | No | Yes | No (use `RefCell`) | Single-threaded shared ownership |
| `Arc<T>` | Yes | Yes | No (use `Mutex`) | Multi-threaded shared ownership |
| `Weak<T>` | Depends | No | No | Breaking reference cycles |
| `Cow<T>` | Yes | No | Clone on write | Efficient read-heavy workloads |

---

## Best Practices

1. **Start with ownership**: Use regular ownership unless you need shared ownership
2. **Box for simple heap allocation**: Use `Box<T>` for simple cases
3. **Rc vs Arc**: Use `Rc<T>` for single-threaded, `Arc<T>` for multi-threaded
4. **Break cycles with Weak**: Always use `Weak<T>` for parent pointers or caches
5. **Interior mutability**: Combine with `RefCell` (single-threaded) or `Mutex` (multi-threaded) for mutation
6. **Cow for optimization**: Use `Cow<T>` when you might not need to clone
7. **Avoid unnecessary cloning**: `Rc::clone()` is cheap (increments counter), not a deep copy

---

## Common Patterns

### Pattern 1: Graph with Rc and Weak

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    edges: RefCell<Vec<Rc<Node>>>,
    back_edges: RefCell<Vec<Weak<Node>>>,
}
```

### Pattern 2: Thread-safe Cache

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type Cache<K, V> = Arc<Mutex<HashMap<K, V>>>;
```

### Pattern 3: Configuration with Cow

```rust
use std::borrow::Cow;

fn process<'a>(input: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(input.to_uppercase())
    } else {
        Cow::Borrowed(input)
    }
}
```

This comprehensive guide covers the essential smart pointers in Rust and their practical applications!