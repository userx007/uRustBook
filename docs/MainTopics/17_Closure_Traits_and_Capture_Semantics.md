# Closure Traits and Capture Semantics in Rust

Rust's closure system is built around three traits that define how closures can be called and what they can do with captured variables. Understanding these traits is essential for working effectively with closures.

## The Three Closure Traits

**FnOnce** is the most permissive trait. A closure that implements FnOnce can be called at least once and may consume captured variables. Every closure implements FnOnce because every closure can be called at least once. The trait definition looks conceptually like `fn call_once(self)` - notice it takes `self` by value, meaning it consumes the closure.

**FnMut** is more restrictive. Closures implementing FnMut can be called multiple times and can mutate captured variables. FnMut closures can be called repeatedly because they don't consume themselves - the trait method takes `&mut self`. Any closure implementing FnMut also implements FnOnce.

**Fn** is the most restrictive. Closures implementing Fn can be called multiple times without mutating captured variables. They only need an immutable reference to themselves (`&self`) to be called. Any closure implementing Fn also implements FnMut and FnOnce, forming a hierarchy.

```rust
                ┌──────────────────────────┐
                │          Fn              │ "most restrictive"
                │──────────────────────────│
                │ call(&self)              │
                │ • no mutation            │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │         FnMut            │ "more restrictive"
                │──────────────────────────│
                │ call(&mut self)          │
                │ • may mutate captures    │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │        FnOnce            │ "most permissive"
                │──────────────────────────│
                │ call_once(self)          │
                │ • may consume captures   │
                │ • callable at least once │
                └──────────────────────────┘

"Everything that is Fn is also FnMut and FnOnce.                

```

## Capture Modes

Rust closures automatically capture variables from their environment in the least restrictive way possible:

**By immutable reference (&T)** - The closure borrows the variable immutably. This happens when the closure only reads the value. The closure will implement Fn.

**By mutable reference (&mut T)** - The closure borrows the variable mutably. This happens when the closure modifies the value. The closure will implement FnMut but not Fn.

**By value (T)** - The closure takes ownership of the variable. This happens when the closure consumes the value or when you use the `move` keyword. If the closure consumes a captured value, it only implements FnOnce.

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

Here's an example showing the different capture modes:

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

## Move Closures

The `move` keyword forces a closure to take ownership of all captured variables, even if the closure would normally borrow them. This is essential when:

- Returning closures from functions
- Spawning threads (the closure must own its data to be 'static)
- Ensuring the closure lives longer than the original scope

```rust
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    // Without 'move', x would be borrowed, but x is going out of scope
    move |y| x + y
}

let add_five = make_adder(5);
println!("{}", add_five(3)); // 8
```

Even with `move`, the closure's trait implementation depends on what it does with the captured values. A `move` closure that only reads captured values still implements Fn.

## Practical Implications

When writing functions that accept closures, you specify which trait bound you need. Use Fn when possible for maximum flexibility, FnMut when the closure needs to mutate state, and FnOnce when you only need to call it once or when accepting the most general closures:

```rust
fn call_twice<F>(mut func: F) where F: FnMut() {
    func();
    func();
}

fn call_once<F>(func: F) where F: FnOnce() {
    func(); // Can only call once
}
```

The compiler automatically determines which traits a closure implements based on its body. You can't manually implement these traits - they're compiler magic that makes closures work seamlessly with Rust's ownership system.

Understanding these traits helps you write more flexible APIs and understand why certain closure patterns work or don't work in your code.