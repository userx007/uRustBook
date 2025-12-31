# Panic Handling and Unwinding in Rust

## Overview

Rust has two main error handling mechanisms: **recoverable errors** (using `Result<T, E>`) and **unrecoverable errors** (using `panic!`). Understanding when and how to use each is crucial for writing robust Rust code.

## Panic vs Recoverable Errors

### Recoverable Errors (Result)

Use `Result<T, E>` when an error is expected and the caller should handle it:

```rust
use std::fs::File;
use std::io::Read;

fn read_username_from_file() -> Result<String, std::io::Error> {
    let mut file = File::open("username.txt")?;
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username)
}

fn main() {
    match read_username_from_file() {
        Ok(name) => println!("Username: {}", name),
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}
```

### Unrecoverable Errors (Panic)

Use `panic!` when the program reaches an unrecoverable state:

```rust
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero!");
    }
    a / b
}

fn get_element(vec: &Vec<i32>, index: usize) -> i32 {
    if index >= vec.len() {
        panic!("Index {} out of bounds for vector of length {}", index, vec.len());
    }
    vec[index]
}
```

## Stack Unwinding

When a panic occurs, Rust begins **unwinding** the stack by default:

1. The current function stops executing
2. Rust walks back up the stack, cleaning up data from each function
3. Destructors (`Drop` implementations) are called for all values
4. The process continues until the panic is caught or the thread terminates

```rust
struct Guard {
    name: String,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("Cleaning up: {}", self.name);
    }
}

fn example_unwinding() {
    let _guard1 = Guard { name: "Guard 1".to_string() };
    let _guard2 = Guard { name: "Guard 2".to_string() };
    let _guard3 = Guard { name: "Guard 3".to_string() };
    
    panic!("Something went wrong!");
    // During unwinding, Drop will be called in reverse order:
    // "Cleaning up: Guard 3"
    // "Cleaning up: Guard 2"
    // "Cleaning up: Guard 1"
}
```

## catch_unwind

The `std::panic::catch_unwind` function allows you to catch panics and recover from them:

```rust
use std::panic;

fn might_panic(should_panic: bool) -> i32 {
    if should_panic {
        panic!("I panicked!");
    }
    42
}

fn main() {
    // Catching a panic
    let result = panic::catch_unwind(|| {
        might_panic(true)
    });

    match result {
        Ok(value) => println!("Success: {}", value),
        Err(_) => println!("Caught a panic!"),
    }

    // Normal execution continues
    println!("Program continues running");

    // Example without panic
    let result = panic::catch_unwind(|| {
        might_panic(false)
    });

    match result {
        Ok(value) => println!("Success: {}", value),
        Err(_) => println!("Caught a panic!"),
    }
}
```

### Practical use case: FFI boundaries

```rust
use std::panic;

// Catching panics at FFI boundary
#[no_mangle]
pub extern "C" fn safe_rust_function(x: i32) -> i32 {
    let result = panic::catch_unwind(|| {
        // Rust code that might panic
        risky_operation(x)
    });

    match result {
        Ok(value) => value,
        Err(_) => -1, // Return error code to C
    }
}

fn risky_operation(x: i32) -> i32 {
    if x < 0 {
        panic!("Negative numbers not allowed!");
    }
    x * 2
}
```

## UnwindSafe and RefUnwindSafe

These traits indicate that a type is safe to use across an unwinding boundary:

### UnwindSafe

A type is `UnwindSafe` if it's safe to access after a panic has been caught:

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::cell::RefCell;

fn main() {
    let mut counter = 0;
    
    // This works - i32 is UnwindSafe
    let result = catch_unwind(|| {
        counter += 1;
        if counter == 1 {
            panic!("Panic!");
        }
    });
    
    // RefCell is NOT UnwindSafe by default
    let cell = RefCell::new(vec![1, 2, 3]);
    
    // This won't compile without AssertUnwindSafe
    // let result = catch_unwind(|| {
    //     cell.borrow_mut().push(4);
    // });
    
    // Use AssertUnwindSafe to bypass the check (use with caution!)
    let result = catch_unwind(AssertUnwindSafe(|| {
        cell.borrow_mut().push(4);
    }));
}
```

### Why some types aren't UnwindSafe

```rust
use std::sync::Mutex;
use std::panic::catch_unwind;

fn main() {
    let mutex = Mutex::new(vec![1, 2, 3]);
    
    // Mutex is NOT UnwindSafe because a panic could leave it
    // in an inconsistent state (e.g., partially modified data)
    
    let result = catch_unwind(|| {
        let mut guard = mutex.lock().unwrap();
        guard.push(4);
        panic!("Panic with mutex locked!");
        // Mutex might be in inconsistent state
    });
    
    // After panic, the mutex might be poisoned
    match mutex.lock() {
        Ok(_) => println!("Mutex is fine"),
        Err(e) => println!("Mutex is poisoned: {}", e),
    }
}
```

### Working with UnwindSafe

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn safe_example() {
    // Arc<AtomicUsize> is UnwindSafe
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    let result = catch_unwind(move || {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        panic!("Panic!");
    });
    
    // Counter is still in consistent state
    println!("Counter value: {}", counter.load(Ordering::SeqCst));
}
```

## Abort Behavior

Instead of unwinding, you can configure Rust to **abort** immediately on panic:

### In Cargo.toml

```toml
[profile.release]
panic = 'abort'

[profile.dev]
panic = 'abort'  # Optional for dev builds
```

### Setting panic hook to abort

```rust
use std::panic;

fn main() {
    // Set a custom panic hook that aborts
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
        std::process::abort();
    }));
    
    panic!("This will abort!");
}
```

### Comparing Unwinding vs Abort

```rust
// With unwinding (default)
fn with_unwinding() {
    let _guard = Guard { id: 1 };
    panic!("Panic!");
    // Guard's Drop is called during unwinding
}

// With abort (panic = 'abort')
fn with_abort() {
    let _guard = Guard { id: 1 };
    panic!("Panic!");
    // Guard's Drop is NOT called - process terminates immediately
}

struct Guard {
    id: i32,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("Dropping guard {}", self.id);
    }
}
```

### Benefits of Abort

1. **Smaller binary size** - no unwinding machinery needed
2. **Faster panic** - immediate termination
3. **Simpler** - no need to worry about unwind safety
4. **Better for embedded systems** - less memory overhead

## Best Practices

### 1. Prefer Result over panic for library code

```rust
// Good - library function
pub fn parse_config(data: &str) -> Result<Config, ConfigError> {
    // Return errors that callers can handle
}

// Bad - library function
pub fn parse_config(data: &str) -> Config {
    // panic!("Invalid config"); // Don't panic in libraries
}
```

### 2. Use panic for programmer errors

```rust
pub fn get_user_by_id(id: usize, users: &[User]) -> &User {
    // Panic for invariant violations
    assert!(id < users.len(), "User ID out of bounds");
    &users[id]
}
```

### 3. Document panic conditions

```rust
/// Returns the element at the given index.
///
/// # Panics
///
/// Panics if the index is out of bounds.
pub fn get(&self, index: usize) -> &T {
    if index >= self.len() {
        panic!("index out of bounds");
    }
    &self.data[index]
}
```

### 4. Use catch_unwind sparingly

```rust
// Good use case: FFI boundaries, plugin systems, test frameworks
fn plugin_system() {
    let result = catch_unwind(|| {
        run_untrusted_plugin()
    });
}

// Bad: using it for normal control flow
fn bad_example() {
    // Don't do this - use Result instead
    let result = catch_unwind(|| {
        might_fail()?
    });
}
```

## Summary

- **Result**: For expected, recoverable errors
- **Panic**: For unrecoverable errors and invariant violations
- **Unwinding**: Default cleanup mechanism that runs destructors
- **catch_unwind**: Catches panics at boundaries (FFI, plugins)
- **UnwindSafe**: Indicates types safe to use across panic boundaries
- **Abort**: Alternative to unwinding for smaller binaries and simpler behavior

Choose the right tool based on whether the error is expected and recoverable, and always document panic conditions in your public APIs.