# When `Box<T>` is Needed in Rust

## 1. **Recursive Types**

You **must** use `Box<T>` for recursive data structures because the compiler needs to know the size of types at compile time, and recursive types would otherwise have infinite size.

```rust
// ❌ This won't compile - infinite size
struct Node {
    value: i32,
    next: Node,  // Error!
}

// ✅ This works - Box has known size (pointer size)
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
```

**Common examples:**
- Linked lists
- Trees (binary trees, BSTs, etc.)
- Graph nodes
- AST (Abstract Syntax Tree) nodes

## 2. **Large Stack Allocations**

Use `Box<T>` when you have large data that would overflow the stack or when you want to avoid expensive stack copies.

```rust
// This array is 400KB on the stack
let large_array = [0u8; 400_000];

// This puts it on the heap - only pointer on stack
let large_array = Box::new([0u8; 400_000]);
```

**When this matters:**
- Large structs (multiple MB)
- Deeply nested function calls with large local variables
- Embedded systems with limited stack space

## 3. **Trait Objects (Dynamic Dispatch)**

When you need dynamic dispatch and don't know the concrete type at compile time, you need `Box<dyn Trait>` because trait objects are unsized types.

```rust
trait Animal {
    fn make_sound(&self);
}

// ❌ Can't have Vec<dyn Animal> - unsized type
// ✅ Need Box for trait objects
let animals: Vec<Box<dyn Animal>> = vec![
    Box::new(Dog {}),
    Box::new(Cat {}),
];
```

## 4. **Transferring Ownership of Large Data**

When you want to transfer ownership without copying large amounts of data.

```rust
struct LargeStruct {
    data: [u8; 1_000_000],
}

// Moving this copies 1MB on the stack
fn consume(data: LargeStruct) { }

// Moving this only copies a pointer (8 bytes)
fn consume(data: Box<LargeStruct>) { }
```

## 5. **When You Need Stable Addresses**

`Box<T>` guarantees that the data won't move in memory, even when the `Box` itself is moved. This is important for:

```rust
struct SelfReferential {
    data: String,
    ptr: *const String,  // Dangerous without pinning!
}

// Box + Pin ensures the data doesn't move
use std::pin::Pin;
let pinned: Pin<Box<SelfReferential>> = /* ... */;
```

**Use cases:**
- Self-referential structs (often with `Pin`)
- FFI when C code expects stable pointers
- Intrusive data structures

## 6. **Reducing Enum Size**

When one variant of an enum is much larger than others, box the large variant to keep the overall enum size small.

```rust
// Bad: entire enum is size of largest variant
enum Message {
    Quit,
    LargePayload([u8; 10_000]),  // Enum is 10KB!
}

// Good: enum is size of pointer + discriminant
enum Message {
    Quit,
    LargePayload(Box<[u8; 10_000]>),  // Enum is ~16 bytes
}
```

## 7. **Escaping Stack Lifetimes**

When you need to return data that outlives the function scope but can't use references.

```rust
fn create_node() -> Box<Node> {
    Box::new(Node {
        value: 42,
        next: None,
    })
    // Data lives beyond function return
}
```

## 8. **Allocator-Backed Collections**

When you need precise control over allocations or are implementing custom data structures.

```rust
// Manual memory management for custom collection
struct MyVec<T> {
    ptr: Box<[T]>,
    len: usize,
}
```

## 9. **FFI (Foreign Function Interface)**

When interfacing with C code that expects heap-allocated pointers.

```rust
#[no_mangle]
pub extern "C" fn create_object() -> *mut MyObject {
    Box::into_raw(Box::new(MyObject::new()))
}
```

## When **NOT** to Use `Box<T>`

- **Small types**: Stack allocation is faster for small data
- **When `&T` or `&mut T` suffices**: References are cheaper
- **Performance-critical code**: Heap allocation has overhead
- **When `Rc<T>` or `Arc<T>` is needed**: For shared ownership
- **Simple cases**: Don't prematurely optimize

## Quick Decision Guide

```
Do you have a recursive type? → YES: Use Box<T>
Is your data >1KB and copied often? → YES: Consider Box<T>
Do you need trait objects? → YES: Use Box<dyn Trait>
Do you need shared ownership? → NO: Don't use Box, use Rc/Arc
Is it a small type (<100 bytes)? → NO: Probably don't need Box
```

The key insight: **`Box<T>` is Rust's simplest heap allocation tool for single ownership**. Use it when stack allocation isn't feasible or when you need a level of indirection with exclusive ownership.