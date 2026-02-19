# Compile-Time Computation and `const fn`

- **`const` items** — named constants, inlining behavior, and explicit typing rules
- **`const fn`** — how functions can be evaluated at compile time, and when they are *guaranteed* to be
- **The const evaluation context** — what CTFE allows and forbids (no heap, no I/O, no trait objects)
- **Loops in `const fn`** — `while`/`loop` support since Rust 1.46
- **Lookup tables** — three practical examples: powers of two, ASCII uppercase, and CRC32
- **`const` vs `static`** — memory layout, lifetimes, and when to prefer each
- **Const generics** — `Matrix<ROWS, COLS>` style zero-cost generic types
- **Inline `const {}` blocks** — forcing compile-time evaluation anywhere (stable since 1.79)
- **Compile-time safety guarantees** — overflow and bounds checking at compile time
- **`const fn` in `impl` blocks** — `Point::new()` and `distance_squared()` examples
- **Limitations and workarounds** — no heap, no `dyn Trait`, floating-point caveats
- **Real-world use cases** — CRC tables, S-boxes, protocol constants, validated config

## Overview

Rust provides powerful mechanisms for performing computations at compile time, allowing values to be computed once during compilation rather than repeatedly at runtime. This leads to zero-cost abstractions, improved performance, and stronger compile-time guarantees. The primary tools are `const fn`, `const` items, and `static` items.

---

## 1. Constants and `const` Items

A `const` item is a named compile-time constant. Its value must be known at compile time and is inlined wherever it is used.

```rust
const MAX_SIZE: usize = 1024;
const PI: f64 = 3.14159265358979;
const GREETING: &str = "Hello, world!";

fn main() {
    let buffer = [0u8; MAX_SIZE]; // Array size must be a const
    println!("PI ≈ {}", PI);
    println!("{}", GREETING);
}
```

Key properties of `const`:
- **Always inlined** — the value is substituted at every use site.
- **No fixed memory address** — unlike `static`.
- **Must be typed explicitly**.
- **Cannot be mutable**.

---

## 2. `const fn` — Constant Functions

A `const fn` is a function that *can* be evaluated at compile time when called in a const context. It can also be called at runtime like a regular function.

```rust
const fn square(x: u32) -> u32 {
    x * x
}

const SQUARED: u32 = square(12); // Evaluated at compile time

fn main() {
    println!("12² = {}", SQUARED); // 144

    let n = 7u32;
    println!("7² = {}", square(n)); // Evaluated at runtime
}
```

### When is a `const fn` evaluated at compile time?

A `const fn` is **guaranteed** to run at compile time only when used in a **const evaluation context**:
- Assigned to a `const` or `static` item.
- Used as an array length.
- Used as a generic constant argument.

```rust
const fn add(a: usize, b: usize) -> usize {
    a + b
}

const SUM: usize = add(10, 20);       // compile-time
static TOTAL: usize = add(100, 200);  // compile-time

fn main() {
    let arr = [0i32; add(3, 4)];       // compile-time (array length)
    println!("Array length: {}", arr.len()); // 7
}
```

---

## 3. The Const Evaluation Context

The **const evaluation context** is a sandboxed environment in which only a restricted subset of Rust is permitted. The compiler's **CTFE (Compile-Time Function Evaluation)** engine enforces these restrictions.

### What is allowed in `const fn`:

| Feature | Allowed |
|---|---|
| Arithmetic and logic | ✅ |
| Conditionals (`if`/`else`) | ✅ |
| Loops (`loop`, `while`, `for`) | ✅ (since Rust 1.46+) |
| Local variables and patterns | ✅ |
| References and borrows (limited) | ✅ |
| Calling other `const fn` | ✅ |
| Struct/enum creation | ✅ |
| Raw pointers (limited) | ✅ (stable since Rust 1.58+) |

### What is NOT allowed in `const fn`:

| Feature | Reason |
|---|---|
| Heap allocation (`Box`, `Vec`, etc.) | Heap does not exist at compile time |
| Trait objects (`dyn Trait`) | Dynamic dispatch not const-evaluable |
| Floating-point operations (historically) | Now partially allowed |
| `impl Trait` in return position | Not yet stabilized in const context |
| Closures (in many cases) | Limited support |
| I/O, system calls | Not available at compile time |
| Mutable references (mostly) | Restricted |

```rust
// ✅ This works
const fn factorial(n: u64) -> u64 {
    if n == 0 { 1 } else { n * factorial(n - 1) }
}

// ❌ This does NOT work — Vec is heap allocated
// const fn make_vec() -> Vec<i32> {
//     vec![1, 2, 3]
// }
```

---

## 4. Loops in `const fn`

Since Rust 1.46, `loop`, `while`, and `for` loops over ranges are allowed in `const fn`.

```rust
const fn sum_to(n: u32) -> u32 {
    let mut total = 0;
    let mut i = 1;
    while i <= n {
        total += i;
        i += 1;
    }
    total
}

const SUM_100: u32 = sum_to(100);

fn main() {
    println!("Sum 1..=100 = {}", SUM_100); // 5050
}
```

---

## 5. Compile-Time Lookup Tables

One of the most powerful uses of `const fn` is generating lookup tables (LUT) at compile time, avoiding runtime initialization entirely.

### Example: Precomputed Powers of Two

```rust
const fn build_pow2_table() -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        table[i] = 1u64 << i;
        i += 1;
    }
    table
}

const POW2: [u64; 64] = build_pow2_table();

fn main() {
    for i in [0, 1, 8, 16, 32, 63] {
        println!("2^{} = {}", i, POW2[i]);
    }
}
```

### Example: ASCII Uppercase Lookup Table

```rust
const fn build_uppercase_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0u8;
    loop {
        table[i as usize] = if i >= b'a' && i <= b'z' {
            i - 32
        } else {
            i
        };
        if i == 255 { break; }
        i += 1;
    }
    table
}

const UPPERCASE: [u8; 256] = build_uppercase_table();

fn fast_to_upper(c: u8) -> u8 {
    UPPERCASE[c as usize]
}

fn main() {
    let result: Vec<u8> = b"hello rust".iter().map(|&c| fast_to_upper(c)).collect();
    println!("{}", String::from_utf8(result).unwrap()); // HELLO RUST
}
```

### Example: CRC32 Lookup Table

```rust
const fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32_TABLE: [u32; 256] = crc32_table();

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    crc ^ 0xFFFFFFFF
}

fn main() {
    println!("CRC32: 0x{:08X}", crc32(b"hello world"));
}
```

---

## 6. `const` vs `static`

Both `const` and `static` are evaluated at compile time, but they differ importantly:

| Feature | `const` | `static` |
|---|---|---|
| Memory address | No fixed address (inlined) | Fixed address |
| Mutability | Immutable only | Can be `static mut` (unsafe) |
| Lifetime | Inlined at use site | `'static` lifetime |
| Use for large data | May bloat binary | More efficient (single copy) |
| Interior mutability | Not possible | Possible with `Mutex`, `AtomicXxx` |

```rust
// For large tables, prefer static to avoid multiple copies:
static LARGE_TABLE: [u32; 256] = crc32_table(); // one copy in binary

// For small scalar constants, const is fine:
const BUFFER_SIZE: usize = 4096;
```

---

## 7. Generic Const Expressions

Rust supports **const generics**, allowing types to be parameterized by constant values. Combined with `const fn`, this enables powerful zero-cost generic abstractions.

```rust
struct Matrix<const ROWS: usize, const COLS: usize> {
    data: [[f64; COLS]; ROWS],
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> {
    const fn new() -> Self {
        Self { data: [[0.0; COLS]; ROWS] }
    }

    const fn size() -> usize {
        ROWS * COLS
    }
}

fn main() {
    let m: Matrix<3, 4> = Matrix::new();
    println!("Matrix size: {}", Matrix::<3, 4>::size()); // 12
}
```

---

## 8. `const` Blocks (Inline Const)

Rust 1.79+ stabilized **inline `const` blocks**, allowing you to force compile-time evaluation anywhere in your code:

```rust
fn main() {
    // Force compile-time evaluation inline
    let x = const { 2u32.pow(10) };
    println!("{}", x); // 1024

    // Useful for complex inline initialization
    let threshold = const {
        let base = 100usize;
        base * base + base / 2
    };
    println!("Threshold: {}", threshold); // 10050
}
```

---

## 9. Compile-Time Guarantees and Safety

`const fn` and compile-time evaluation provide important safety guarantees:

### Overflow Detection

Arithmetic overflow in `const` contexts is a **compile error**, not undefined behavior:

```rust
// This will fail to compile with overflow error:
// const OVERFLOW: u8 = 200u8 + 100u8;

// This is safe:
const MAX: u8 = u8::MAX; // 255
```

### Bounds Checking

Array indexing in const contexts is bounds-checked at compile time:

```rust
const ARR: [i32; 3] = [1, 2, 3];
// const OUT_OF_BOUNDS: i32 = ARR[5]; // compile error!
```

### Determinism

Const evaluation is **completely deterministic** — there is no randomness, no I/O, no side effects. This means results are reproducible and platform-independent (with minor exceptions for target-dependent sizes like `usize`).

---

## 10. `const fn` in `impl` Blocks and Traits

`const fn` can be used in `impl` blocks:

```rust
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub const fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    pub const fn distance_squared(&self, other: &Point) -> i64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

const ORIGIN: Point = Point::new(0, 0);
const TARGET: Point = Point::new(3, 4);
const DIST_SQ: i64 = ORIGIN.distance_squared(&TARGET);

fn main() {
    println!("Distance squared: {}", DIST_SQ); // 25
}
```

---

## 11. Limitations and Workarounds

### Limitation: No Heap Allocation

`Vec`, `String`, `Box`, and other heap-allocated types cannot be used in const contexts. The workaround is to use fixed-size arrays:

```rust
// ❌ Cannot do this:
// const fn make_list() -> Vec<u32> { vec![1, 2, 3] }

// ✅ Use a fixed-size array instead:
const fn make_list() -> [u32; 3] {
    [1, 2, 3]
}
```

### Limitation: No Trait Objects

Dynamic dispatch via `dyn Trait` is not allowed. Use generics instead:

```rust
// ❌ Not allowed in const context:
// const fn process(op: &dyn Fn(u32) -> u32, x: u32) -> u32 { op(x) }

// ✅ Use generic const fn:
const fn apply<F: ~const Fn(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}
// Note: ~const Fn is nightly-only; this pattern is still evolving
```

### Limitation: Floating-Point Precision

Floating-point in const contexts is now mostly stable, but results can vary subtly. For truly portable lookup tables, consider using integer fixed-point arithmetic.

---

## 12. Real-World Use Cases

| Use Case | Benefit |
|---|---|
| Hash table seeds | Precomputed at compile time, no runtime cost |
| CRC/checksum tables | Fast data integrity checks with zero init cost |
| Trigonometry tables | Replace `sin`/`cos` calls with table lookups |
| Protocol constants | Packet sizes, bitmasks computed from specs |
| Validated config | Ensure config values are in range at compile time |
| Cryptographic S-boxes | S-boxes for AES, DES precomputed statically |

---

## Summary

| Concept | Key Point |
|---|---|
| `const` items | Named compile-time constants, inlined at use |
| `const fn` | Functions that can run at compile time |
| Const evaluation | Sandboxed: no heap, no I/O, no side effects |
| Lookup tables | Pre-generate arrays of data at compile time |
| `static` for large data | Single copy, fixed address, `'static` lifetime |
| Const generics | Types parameterized by compile-time values |
| Overflow safety | Const arithmetic overflow is a compile error |
| Inline `const {}` | Force const evaluation anywhere (Rust 1.79+) |

Compile-time computation in Rust is a zero-cost abstraction: you pay nothing at runtime for work done at compile time. Mastering `const fn` and const evaluation allows you to write safer, faster code by pushing more logic out of the hot path and into the compiler.