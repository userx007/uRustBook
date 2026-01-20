# 1. Ownership, Move Semantics, and Borrowing Rules in Rust

## Rust enforces three fundamental ownership rules at compile time:

1. **Each value has a single owner** (a variable that holds it)
    - Emphasizes *existence and identity*:
        - A value is *never unowned*
        - Ownership is *explicit*
        - There is always a *specific* variable responsible for cleanup

2. **There can only be one owner at a time**
    - Emphasizes *exclusivity over time*:
        - Ownership *cannot be shared*
        - Ownership *can move*, but never duplicate
        - Prevents double-free and data races

3. **When the owner goes out of scope, the value is dropped** (memory is freed)

First 2 rules can be replced with: 
- Each value has exactly one owner at any point in time.

## Move Semantics

When you assign a value to another variable or pass it to a function, **ownership moves** by default for non-Copy types.

## Copy vs Move Semantics

Some types implement the **`Copy` trait**, which means they're copied instead of moved. 
These are typically simple types stored on the stack.

## The Borrowing Rules

1. **You can have EITHER multiple immutable references OR one mutable reference**
2. **References must always be valid (no dangling references)**

These rules prevent:
- **Data races**: No simultaneous mutable and immutable access
- **Use-after-free**: References are always valid
- **Double-free**: Only one owner, freed only once
- **Iterator invalidation**: Can't modify while iterating

## Summary

- **Ownership**: Each value has one owner; dropped when owner goes out of scope
- **Move**: Default for heap-allocated types (String, Vec, etc.)
- **Copy**: Automatic for stack-only types (integers, bools, etc.)
- **Borrowing**: References allow access without ownership transfer
- **Immutable refs**: Multiple allowed, no mutation
- **Mutable refs**: Only one at a time, exclusive access
- **Safety**: All enforced at compile time with zero runtime cost!

---

# 2. Lifetimes 

- Lifetimes are Rust's way of tracking how long references are valid. 
- Every reference in Rust has a lifetime - the scope for which that reference is valid. 
- The borrow checker uses lifetimes to ensure references don't outlive the data they point to

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {}

// Return value is tied only to 'a, not 'b
fn complex<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { x }
```
- The 'a annotation means: the returned reference will live at least as long as the shortest-lived input reference.

## Lifetime Elision Rules (3)

- **Rule 1:** Each elided input lifetime gets its own lifetime parameter
- **Rule 2:** If there's exactly one input lifetime, it's assigned to all output lifetimes
- **Rule 3:** If there are multiple input lifetimes but one is `&self` or `&mut self`, the lifetime of `self` is assigned to all output lifetimes

```rust
// Rules 1 and 2
fn first_word(s: &str) -> &str

// Compiler interprets as:
fn first_word<'a>(s: &'a str) -> &'a str

// Rule 3
impl<'a> ImportantExcerpt<'a> {
    // Written (elision):
    fn level(&self) -> &str {
        self.part
    }
    
    // Compiler interprets as:
    fn level<'b>(&'b self) -> &'b str {
        self.part
    }
}
```
## When Elision Doesn't Apply

```rust
// This requires explicit annotations:
fn longest(x: &str, y: &str) -> &str {  // ERROR!
    if x.len() > y.len() { x } else { y }
}

// Must be written as:
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

## Lifetime Bounds on Generic Types

- You can specify that a generic type parameter must outlive a certain lifetime:

```rust
// T must live at least as long as 'a
struct Ref<'a, T: 'a> {
    reference: &'a T,
}

// Modern Rust (2018+) allows this to be implicit:
struct Ref<'a, T> {
    reference: &'a T,
}
```

## The `'static` Lifetime

The `'static` lifetime is special - it means the reference can live for the entire duration of the program:

```rust
// String literals have 'static lifetime
let s: &'static str = "I have a static lifetime.";

// This is stored in the program's binary
static HELLO: &str = "Hello, world!";

// Requiring 'static in function signatures
fn print_it(input: &'static str) {
    println!("{}", input);
}
```


## Higher-Ranked Trait Bounds (HRTBs)

HRTBs use the `for<'a>` syntax to express that a trait bound must hold for any lifetime:
**for<'a>** means: “this function must work for any lifetime, not one specific lifetime.”

```rust
// meaning: There exists some lifetime 'a such that F works for &'a i32
// Problem: Too weak for many cases.
F: Fn(&'a i32) -> &'a i32
```

```rust
// meaning: For ALL lifetimes 'a, F must work for &'a i32
// A much stronger guarantee.
F: for<'a> Fn(&'a i32) -> &'a i32
```

### Why this matters (intuition)

- Without `for<'a>` (too specific) callback only works for ONE particular lifetime

```rust
// This closure **does NOT work for arbitrary references** — only for ones compatible with `local`.
let local = 5;
let f = |_: &i32| &local; // returns reference tied to outer scope
```

- With `for<'a>` (universally valid) callback must work for ANY reference lifetime

```rust
// This forces the closure to behave like an **identity function**:
// It cannot capture or return references tied to a specific scope.
|x| x
```

### Example

```rust
fn call_with_ref<F>(callback: F)
where
    F: for<'a> Fn(&'a i32) -> &'a i32,
{
    let value = 42;
    let result = callback(&value);
    println!("Result: {}", result);
}
```

- What this function demands is that the callback must:
	- accept a reference with ANY lifetime
	- return a reference with the SAME lifetime
- This ensures:
	- The returned reference is always tied to the input
	- No hidden borrowing from elsewhere
	- No dangling references

```rust
call_with_ref(|x| x);
// Works because:
// - input lifetime 'a → output lifetime 'a
// - (no capture, no extension, no restriction)

```

```rust
let outside = 10;
call_with_ref(|_| &outside); 
// Fails because:
// - output lifetime ≠ input lifetime
// - the function is no longer valid for *all* `'a`.
```

### Real-World HRTB Example

```rust
use std::fmt::Debug;

// Function that works with any function that can process any reference
// - processor must accept &T with ANY lifetime 'a
// - and return an owned String (no borrowing back)
fn process_items<F, T>(items: Vec<T>, processor: F)
where
    T: Debug,
    F: for<'a> Fn(&'a T) -> String,
{

    // - Each item reference:
    // - Has a fresh lifetime per loop iteration
    // - Is not known in advance
    // - Must be accepted by processor
    // - The HRTB guarantees the closure cannot assume anything about the lifetime of item.    

    for item in &items {
        let result = processor(item);
        println!("Processed: {}", result);
    }
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];#

    // |n| format!("Number: {}", n)
    // - Takes &T
    // - Does not store or return the reference
    // - Works for any lifetime
    // - Produces owned data
    // - This is exactly what for<'a> enforces.

    process_items(numbers, |n| format!("Number: {}", n));
}

// Note: In this specific case, the HRTB is technically stronger than necessary.
// F: Fn(&T) -> String
// - would still work, because:
//   - no reference escapes
//   - no lifetime relationship is expressed in the return type
```

### When HRTB is truly required

- input lifetime == output lifetime

```rust
F: for<'a> Fn(&'a T) -> &'a T
```

## Non-Lexical Lifetimes (NLL) 

```rust
// Before NLL (Pre-2018): this would NOT compile in old Rust:
fn main() {
    let mut scores = vec![1, 2, 3];
    
    let first = &scores[0];  // Borrow starts
    println!("First score: {}", first);
    // In old Rust, borrow would last until end of scope
    
    scores.push(4);  // ERROR in old Rust: can't mutate while borrowed
}
```

```rust
// With NLL (Rust 2018+) This DOES compile
fn main() {
    let mut scores = vec![1, 2, 3];
    
    let first = &scores[0];  // Borrow starts
    println!("First score: {}", first);
    // Borrow ends here (last use of 'first')
    // With NLL, the compiler sees that 'first' isn't used after this point
    
    scores.push(4);  // OK! Borrow is no longer active
    println!("Scores: {:?}", scores);
}

```rust
fn process_or_default(data: &mut Vec<i32>) -> i32 {
    if let Some(&first) = data.first() {
        // Immutable borrow happens here
        if first > 10 {
            return first;  // Borrow ends here
        }
    }
    
    // Mutable borrow is allowed here because the immutable borrow
    // ended in the conditional above
    data.push(0);  // OK with NLL!
    0
}
```

## Addons

- Container<'a, T: 'a> does NOT mean that T is a reference or must contain references.
- What it means is: T may contain references, and if it does, they must live at least as long as 'a.
- Lifetimes do not imply references exist — they only constrain them if they do.
- `T: 'a` means: any references inside T (if there are any) must outlive 'a.
- lifetimes appear even when no reference is visible because Rust is being generic and defensive.
- lifetime annotation means “constraints on references”, not “references exist”.

### Example 1: Foo contains no references, the constraint `T: 'a` is trivially satisfied

```rust
struct Foo {
    x: i32,
}

Container<'a, Foo>
```

### Example 2: `T` contains references, this is only allowed if `'b: 'a`

```rust
struct Bar<'b> {
    x: &'b i32,
}

Container<'a, Bar<'b>>
// Rust prevents dangling references.
```

### Example 3: `T` is a reference, this is only allowed if `'b: 'a`

```rust
T = &'b i32

Container<'a, &'b i32>
```

---

# 3. Interior Mutability Patterns

- Rust's way of allowing mutation of data even when there are immutable references to it

### **Cell\<T>** - Simple & Fast
- Allows mutation of a Copy type inside an immutable struct	
- **Only for `Copy` types** (integers, bools, chars)
- Zero runtime overhead
- Single-threaded only
- Replaces the entire value with `get()` and `set()`
- **Use when**: Tracking metadata in immutable structs (counters, flags)

```rust
struct Counter {
    count: Cell<i32>,
}

let counter = Counter {
    count: Cell::new(0),
};

// counter is immutable, but we can mutate count
counter.count.set(counter.count.get() + 1);
counter.count.set(counter.count.get() + 1);
```

### **RefCell\<T>** - Flexible Runtime Checking
- Allows mutation through immutable reference
- Works with **any type**
- Enforces borrowing rules at runtime (panics if violated)
- Can borrow mutably (`borrow_mut()`) or immutably (`borrow()`)
- Single-threaded only
- **Use when**: Building graphs/trees with cycles, mock objects, or complex shared data structures in single-threaded code

```rust
let data = RefCell::new(vec![1, 2, 3]);

// Borrow mutably and modify
data.borrow_mut().push(4);
data.borrow_mut().push(5);

// Borrow immutably to read
println!("Data: {:?}", *data.borrow());
```

### **Mutex\<T>** - Thread-Safe Exclusivity
- Thread-safe interior mutability via mutual exclusion
- Only one thread can access at a time
- Blocks waiting threads
- Use with `Arc` for shared ownership
- **Use when**: Multiple threads need exclusive access to shared data (caches, counters, queues)

```rust
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
```


### **RwLock\<T>** - Optimized for Reads
- Allows **multiple simultaneous readers** OR one writer
- Better performance when reads vastly outnumber writes
- Writers block all access, readers only block writers
- **Use when**: Configuration systems, read-heavy data structures, rarely-updated shared state

```rust
let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
let mut handles = vec![];

// Spawn multiple reader threads
for i in 0..5 {
    let data = Arc::clone(&data);
    let handle = thread::spawn(move || {
        let read_guard = data.read().unwrap();
        println!("Reader {} sees: {:?}", i, *read_guard);
    });
    handles.push(handle);
}

// Spawn a writer thread
let data_writer = Arc::clone(&data);
let writer_handle = thread::spawn(move || {
    let mut write_guard = data_writer.write().unwrap();
    write_guard.push(6);
    println!("Writer added value");
});
handles.push(writer_handle);

for handle in handles {
    handle.join().unwrap();
}
```

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


### **Decision Factors:**

1. **Thread safety needed?** → `Mutex` / `RwLock` (multi-threaded) vs `Cell` / `RefCell` (single-threaded)
2. **Copy type?** → `Cell` is simplest
3. **Read-heavy workload?** → `RwLock` over `Mutex`
4. **Need runtime flexibility?** → `RefCell` for graphs/cycles


---


# Trait Bounds?

Without trait bounds, generic types can't do anything useful because Rust doesn't know what operations they support. 
Trait bounds tell the compiler what methods and behaviors a type must implement.
The `T: Trait` syntax means "T must implement Trait." This allows you to call trait methods on `T`.


```rust

fn function<T: Trait>(param: T) { }  

fn function<T: Trait1 + Trait2 + Trait3>(param: T) { }

fn function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Debug + PartialEq,
{ }

```

You can provide different implementations based on what traits a type implements:

```rust

impl<T: Display> MyStruct<T> { }               // All T with Display

impl<T: Display + PartialOrd> MyStruct<T> { }  // Additional methods for comparable types

```

Constrain not just the generic type, but its associated types:

```rust
where
    T: Iterator,
    T::Item: Display,  // The items produced must be displayable


           ┌─────────────────────────────┐
           │     SpiTransfer (trait)     │
           │─────────────────────────────│
           │  transfer(...)              │
           │  write(...)                 │
           │  read(...)                  │
           └───────────────▲─────────────┘
                           │
           (T must implement this trait)
                           │
           ┌───────────────┴─────────────┐
           │  AdvancedLoopbackTester<T>  │
           │─────────────────────────────│
           │  T : SpiTransfer            │
           └─────────────────────────────┘

                    Compile Time
     ┌──────────────┐         ┌──────────────┐
     │ HardwareSpi  │         │   MockSpi    │
     └───────▲──────┘         └───────▲──────┘
             │ implements             │ implements
             │                        │
     ┌───────┴────────────────────────┴───────┐
     │           SpiTransfer trait            │
     └────────────────────────────────────────┘

     ┌────────────────────────────────────────┐
     │ AdvancedLoopbackTester<HardwareSpi>    │
     └────────────────────────────────────────┘

     ┌────────────────────────────────────────┐
     │ AdvancedLoopbackTester<MockSpi>        │
     └────────────────────────────────────────┘

//_______________________________________________

struct AdvancedLoopbackTester<T: SpiTransfer>{
	spi: T,
}
     ┌───────────────────────────────────────┐
     │   AdvancedLoopbackTester<HardwareSpi> │
     │───────────────────────────────────────│
     │  spi ───────────────┐                 │
     │                     ▼                 │
     │              HardwareSpi              │
     └───────────────────────────────────────┘
"The tester owns the SPI device."
// _______________________________________________

struct AdvancedLoopbackTester<'a, T: SpiTransfer>{
    spi: &'a mut T,
}

	┌───────────────────────────────────────┐
	│ AdvancedLoopbackTester<'a, T>         │
	│───────────────────────────────────────│
	│ spi ──&'a mut T────────────────────┐  │
	└────────────────────────────────────│──┘  
	                                     ▼
	                              ┌─────────────┐
	                              │ HardwareSpi │
	                              └─────────────┘
"The tester temporarily borrow the SPI device, doesn’t own it."

_______________________________________________
```

---

# The Three Closure Traits

```rust
                ┌──────────────────────────┐
                │          Fn              │
                │──────────────────────────│
                │ call(&self)              │
                │ • no mutation            │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │         FnMut            │
                │──────────────────────────│
                │ call(&mut self)          │
                │ • may mutate captures    │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │        FnOnce            │
                │──────────────────────────│
                │ call_once(self)          │
                │ • may consume captures   │
                │ • callable at least once │
                └──────────────────────────┘
``` 

```rust
fn call_twice<F>(mut func: F) 
where 
	F: FnMut() 
{
    func();
    func();
}

fn call_once<F>(func: F) 
where 
	F: FnOnce() 
{
    func(); // Can only call once
}
```

## Capture Modes

```
                    Rust Closure Capture Analysis

               ┌────────────────────────────────────┐
               │   Variable used inside closure?    │
               └────────────────────────────────────┘
                              │
                              ▼
               ┌────────────────────────────────────┐
               │   How does the closure use it?     │
               └────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
          ▼                   ▼                   ▼
┌───────────────────┐ ┌───────────────────┐ ┌─────────────────────┐
│ Read-only access  │ │ Mutable access    │ │ Ownership required  │
│ (no mutation)     │ │ (value modified)  │ │ (consumed or `move`)│
└───────────────────┘ └───────────────────┘ └─────────────────────┘
          │                   │                   │
          ▼                   ▼                   ▼
┌───────────────────┐ ┌───────────────────┐ ┌─────────────────────┐
│ Capture as &T     │ │ Capture as &mut T │ │ Capture as T        │
│ Immutable borrow  │ │ Mutable borrow    │ │ Move / ownership    │
└───────────────────┘ └───────────────────┘ └─────────────────────┘
          │                   │                   │
          ▼                   ▼                   ▼
┌───────────────────┐ ┌───────────────────┐ ┌─────────────────────┐
│ Implements: Fn    │ │ Implements: FnMut │ │ Implements: FnOnce  │
│                   │ │ (not Fn)          │ │ (only once)         │
└───────────────────┘ └───────────────────┘ └─────────────────────┘


Legend:
───────
&T      = immutable borrow (shared access)
&mut T  = mutable borrow (exclusive access)
T       = ownership moved into closure

Mental model shortcut
─────────────────────
Read    → borrow (&T)     → Fn
Modify  → borrow (&mut T) → FnMut
Consume → move (T)        → FnOnce
```

## Examples

```rust
let x = String::from("hello");
let mut y = 5;

// Captures x by immutable reference - implements Fn
let read_only = || println!("{}", x);

// Captures y by mutable reference - implements FnMut
let mutating = || { y += 1; };

// Consumes x - only implements FnOnce
let consuming = || drop(x);
```

---

# Blanket Implementations

Implement Summary for any type that implements Display

``` rust
trait Summary {
    fn summarize(&self) -> String;
}

impl<T: Display> Summary for T {
    fn summarize(&self) -> String {
        format!("Summary: {}", self)
    }
}
```

---

# Basic Error Trait Definition

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
    fn backtrace(&self) -> Option<&Backtrace> { None }
}
```

- Debug + Display (supertraits)
- Any type that implements Error must also implement both Debug and Display
- Any public error type must be both human-readable and developer-debuggable.

```
            ┌────────────┐
            │   Error    │
            └────────────┘
                 ▲
         ┌───────┴─────────┐
         │                 │
     ┌───────┐         ┌────────┐
     │ Debug │         │ Display│
     └───────┘         └────────┘

"An Error must be printable for users and inspectable by developers"
```

```
            Custom Error Types (Rust)
        ┌───────────────────────────────┐
        │      std::error::Error        │
        │   (trait, public interface)   │
        └───────────────────────────────┘
                         ▲
                         │ requires
            ┌────────────┴────────────┐
            │                         │
    ┌───────────────┐        ┌────────────────┐
    │  fmt::Debug   │        │  fmt::Display  │
    │  (developer)  │        │  (user output) │
    └───────────────┘        └────────────────┘


        ┌───────────────────────────────┐
        │      Custom Error Type        │
        │        (struct / enum)        │
        └───────────────────────────────┘
                     │
        ┌────────────┼────────────────┐
        │            │                │
        ▼            ▼                ▼
  derive Debug   impl Display     impl Error
                                 (no methods)


        ┌───────────────────────────────┐
        │     Error Usage Patterns      │
        └───────────────────────────────┘
         │              │            │
     Result<T, E>    dyn Error       ?
         │              │            │
         ▼              ▼            ▼
  concrete error   trait object   error propagation


        ┌───────────────────────────────┐
        │     Optional Enhancements     │
        └───────────────────────────────┘
           source()  → error chaining
           From<T>   → automatic conversion
           thiserror / anyhow → ergonomics

Legend:
───────
struct / enum → define error shape
Display       → user-friendly message
Debug         → internal diagnostics
Error         → interoperability contract
```

## Minimal code (irreducible)

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error")
    }
}

impl Error for MyError {}
```
