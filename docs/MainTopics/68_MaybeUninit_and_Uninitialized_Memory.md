# MaybeUninit and Uninitialized Memory


1. **Basic usage** — `uninit()` → `write()` → `assume_init()`
2. **Array initialization** — for types without `Default`, including the modern `.map()` pattern
3. **FFI** — passing a `MaybeUninit` pointer to a C function to fill
4. **Partial struct initialization** — using `ptr::write` + `addr_of_mut!` to build fields one-by-one
5. **Safe cleanup on failure** — manually dropping initialized fields if partial init fails
6. **`zeroed()`** — when all-zero bytes is a valid state (C-compatible structs)

It also includes a pitfalls table, a comparison of approaches, and explains *why* `mem::uninitialized()` was deprecated and is UB.

## Overview

In Rust, every variable must be initialized before it is used. This is enforced at compile time and is one of the key guarantees of the language. However, there are situations — particularly in systems programming, FFI, and performance-critical code — where you need to work with memory that has not yet been initialized. Doing this naively (e.g., via unsafe transmutes or raw pointer tricks) is Undefined Behavior (UB).

The standard library provides `std::mem::MaybeUninit<T>` as the **safe, idiomatic way** to handle uninitialized memory. It is a union that tells the compiler "this memory may or may not contain a valid `T`", preventing the optimizer from making incorrect assumptions.

---

## Why Uninitialized Memory Exists

There are legitimate reasons to defer initialization:

- **Performance**: Avoid zeroing a large buffer only to overwrite it immediately.
- **FFI**: C functions often take a pointer to a struct that *they* will fill in.
- **Partial initialization**: Build up complex structures field by field before they are "live".
- **Custom allocators / arenas**: Manage raw memory without always constructing objects immediately.

The challenge is doing this without invoking Undefined Behavior.

---

## The Problem: Undefined Behavior with Uninitialized Memory

Consider the following **broken** approach:

```rust
use std::mem;

// ❌ DO NOT DO THIS — Undefined Behavior!
let x: u32 = unsafe { mem::uninitialized() }; // deprecated and UB
println!("{}", x); // Reading uninitialized memory is UB
```

`mem::uninitialized()` was removed from stable use precisely because it caused subtle UB. The compiler is allowed to assume that a `bool`, for example, is always `0` or `1` — reading an uninitialized one violates that invariant, causing miscompilation.

---

## `MaybeUninit<T>`: The Safe Abstraction

```rust
use std::mem::MaybeUninit;

let mut x: MaybeUninit<u32> = MaybeUninit::uninit();

// Write a value into it
x.write(42);

// Now it is safe to read
let value: u32 = unsafe { x.assume_init() };
println!("{}", value); // 42
```

### Key API

| Method | Description |
|---|---|
| `MaybeUninit::uninit()` | Create uninitialized storage |
| `MaybeUninit::new(val)` | Create initialized storage |
| `MaybeUninit::zeroed()` | Create zero-initialized storage |
| `.write(val)` | Write a value; returns `&mut T` |
| `unsafe .assume_init()` | Assert it's init and move out the value |
| `unsafe .assume_init_ref()` | Get a `&T` without moving |
| `unsafe .assume_init_mut()` | Get a `&mut T` without moving |
| `.as_ptr()` | Raw const pointer to inner data |
| `.as_mut_ptr()` | Raw mutable pointer to inner data |

> **The safety contract**: You must ensure the value is genuinely initialized before calling any `assume_init*` method. Violating this is UB.

---

## Example 1: Basic Usage

```rust
use std::mem::MaybeUninit;

fn create_value() -> u32 {
    let mut val = MaybeUninit::<u32>::uninit();
    val.write(100);
    // SAFETY: We just wrote 100 into `val`, so it is fully initialized.
    unsafe { val.assume_init() }
}

fn main() {
    println!("{}", create_value()); // 100
}
```

---

## Example 2: Initializing an Array Without Default

A common use case is initializing a fixed-size array of a type that doesn't implement `Default` or `Copy`.

```rust
use std::mem::MaybeUninit;

#[derive(Debug)]
struct NonDefault {
    value: i32,
}

fn init_array() -> [NonDefault; 5] {
    // Create an array of uninitialized MaybeUninit values
    let mut arr: [MaybeUninit<NonDefault>; 5] = [
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
    ];

    // Initialize each element
    for (i, slot) in arr.iter_mut().enumerate() {
        slot.write(NonDefault { value: i as i32 * 10 });
    }

    // SAFETY: Every element has been written above.
    // We use pointer casting to reinterpret the array type.
    unsafe {
        // transmute works here because [MaybeUninit<T>; N] has the same
        // layout as [T; N] once all elements are initialized.
        std::mem::transmute::<[MaybeUninit<NonDefault>; 5], [NonDefault; 5]>(arr)
    }
}

fn main() {
    let arr = init_array();
    for item in &arr {
        println!("{:?}", item);
    }
}
```

### A Cleaner Array Pattern (Rust 1.82+)

```rust
use std::mem::MaybeUninit;

fn init_array_v2() -> [i32; 10] {
    let mut arr = [const { MaybeUninit::<i32>::uninit() }; 10];
    for (i, slot) in arr.iter_mut().enumerate() {
        slot.write(i as i32 * i as i32);
    }
    // SAFETY: all elements initialized in the loop above.
    arr.map(|x| unsafe { x.assume_init() })
}
```

---

## Example 3: FFI — Letting C Fill a Struct

A very common real-world use: calling a C function that populates a struct via pointer.

```rust
use std::mem::MaybeUninit;

// Imagine this is an extern "C" function from a C library
// that fills in the struct at the given pointer.
unsafe fn c_get_info(out: *mut u64) {
    // Simulating what C would do
    *out = 0xDEADBEEF;
}

fn main() {
    let mut info = MaybeUninit::<u64>::uninit();

    // SAFETY: c_get_info writes a valid u64 into the pointer before we read it.
    unsafe {
        c_get_info(info.as_mut_ptr());
    }

    // SAFETY: c_get_info has fully initialized `info`.
    let value = unsafe { info.assume_init() };
    println!("0x{:X}", value); // 0xDEADBEEF
}
```

---

## Example 4: Partially Initialized Structs

Sometimes you want to build a struct field by field. This is tricky because you must track which fields are initialized and ensure all are done before reading the whole struct.

```rust
use std::mem::MaybeUninit;
use std::ptr;

struct Config {
    width: u32,
    height: u32,
    title: String,
}

fn build_config() -> Config {
    let mut config = MaybeUninit::<Config>::uninit();
    let ptr = config.as_mut_ptr();

    unsafe {
        // Initialize each field via raw pointer field access
        ptr::write(std::ptr::addr_of_mut!((*ptr).width), 1920);
        ptr::write(std::ptr::addr_of_mut!((*ptr).height), 1080);
        ptr::write(
            std::ptr::addr_of_mut!((*ptr).title),
            String::from("My Window"),
        );

        // SAFETY: All three fields have been written.
        config.assume_init()
    }
}

fn main() {
    let cfg = build_config();
    println!("{}x{} - {}", cfg.width, cfg.height, cfg.title);
}
```

> **Critical rule**: When writing fields of a `MaybeUninit` struct, always use `std::ptr::write` (or `addr_of_mut!`) — never use `(*ptr).field = val` directly, as that would attempt to drop the old (uninitialized) value first, which is UB.

---

## Example 5: Avoiding UB with Drop

If initialization fails partway through and you allocated heap memory (or anything with a `Drop` impl), you must carefully avoid double-free or use-after-free.

```rust
use std::mem::MaybeUninit;
use std::ptr;

struct Wrapper {
    data: Vec<u32>,
    name: String,
}

fn safe_partial_init() -> Option<Wrapper> {
    let mut w = MaybeUninit::<Wrapper>::uninit();
    let ptr = w.as_mut_ptr();

    // Step 1: initialize `data`
    unsafe {
        ptr::write(ptr::addr_of_mut!((*ptr).data), vec![1, 2, 3]);
    }

    // Step 2: suppose name initialization might fail
    let name_result: Result<String, _> = "hello".parse::<String>(); // always ok here

    match name_result {
        Ok(name) => {
            unsafe {
                ptr::write(ptr::addr_of_mut!((*ptr).name), name);
                // SAFETY: Both fields are now initialized.
                Some(w.assume_init())
            }
        }
        Err(_) => {
            // SAFETY: We must drop `data` since it was initialized.
            // `name` was never written, so we must NOT drop it.
            unsafe {
                ptr::drop_in_place(ptr::addr_of_mut!((*ptr).data));
            }
            None
        }
    }
}

fn main() {
    if let Some(w) = safe_partial_init() {
        println!("name={}, data={:?}", w.name, w.data);
    }
}
```

This is advanced territory. In practice, you should prefer simpler patterns when possible and resort to this only when performance demands it.

---

## Example 6: `MaybeUninit::zeroed()`

For types where all-zero bytes is a valid state (e.g., integer types, C-compatible structs), you can use `zeroed()` instead of `uninit()`:

```rust
use std::mem::MaybeUninit;

#[repr(C)]
struct CRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

fn main() {
    // SAFETY: CRect is a C struct where all-zeros is valid.
    let rect: CRect = unsafe { MaybeUninit::zeroed().assume_init() };
    println!("({}, {}) {}x{}", rect.x, rect.y, rect.w, rect.h); // (0, 0) 0x0
}
```

Do **not** use `zeroed()` for types that have invariants (e.g., `bool`, references, `NonZero*`, `Box<T>`), as all-zero bytes may be invalid for them.

---

## Common Pitfalls and Rules

### ❌ Don't read before writing
```rust
let x: MaybeUninit<u32> = MaybeUninit::uninit();
// UB! x has not been written to.
let _ = unsafe { x.assume_init() };
```

### ❌ Don't use assignment on uninit struct fields
```rust
let mut s = MaybeUninit::<MyStruct>::uninit();
unsafe {
    // WRONG: This drops the "old" value at the field, which is garbage.
    (*s.as_mut_ptr()).field = value;
}
```
Always use `ptr::write` instead.

### ❌ Don't forget to drop initialized fields on early exit
If you partially init a struct and bail out, manually drop any heap-owning fields you've already written.

### ✅ Document your safety invariants
Every `unsafe { x.assume_init() }` call should have a comment explaining *why* the memory is guaranteed to be initialized at that point.

---

## `MaybeUninit` vs Raw Pointers vs `mem::uninitialized`

| Approach | Status | Notes |
|---|---|---|
| `mem::uninitialized()` | **Deprecated / UB** | Never use. Removed from safe code. |
| `mem::zeroed()` (direct) | Sometimes OK | Only for types where zero is valid. |
| Raw `*mut T` + `alloc` | OK but complex | You manage everything manually. |
| `MaybeUninit<T>` | **Recommended** | Compiler-aware, no false assumptions made. |

---

## Summary

`MaybeUninit<T>` is the correct tool whenever you need to work with memory that may not be initialized yet. It prevents the compiler from assuming the memory holds a valid `T`, and its API guides you toward safe patterns:

- Use `.write()` to initialize the memory.
- Use `unsafe { .assume_init() }` only after you've guaranteed full initialization.
- Use `ptr::write` and `addr_of_mut!` for struct fields.
- Be careful with partial initialization and `Drop` types — you are responsible for cleanup.
- Prefer higher-level abstractions whenever possible; reach for `MaybeUninit` when you genuinely need to avoid the cost of default initialization or must interop with C.

By following these rules, you can work with uninitialized memory safely, without undefined behavior, and without sacrificing performance.