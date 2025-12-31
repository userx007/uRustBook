# Rust Optimization Techniques

Optimization in Rust involves multiple strategies to improve performance, reduce binary size, and minimize runtime overhead. Let me walk you through the key techniques with practical examples.

## 1. Avoiding Allocations

Heap allocations are expensive operations. Minimizing them can significantly improve performance.

**Stack vs Heap:**

```rust
// ❌ Heap allocation (slower)
fn process_data_heap() -> Vec<i32> {
    let mut data = Vec::new();
    for i in 0..100 {
        data.push(i);
    }
    data
}

// ✅ Stack allocation (faster)
fn process_data_stack() -> [i32; 100] {
    let mut data = [0; 100];
    for i in 0..100 {
        data[i] = i;
    }
    data
}
```

**Using Iterators (Zero-Cost Abstractions):**

```rust
// ❌ Allocates intermediate Vec
fn sum_squares_allocating(nums: &[i32]) -> i32 {
    nums.iter()
        .map(|&x| x * x)
        .collect::<Vec<_>>()  // Allocates!
        .iter()
        .sum()
}

// ✅ No allocation - iterator chain
fn sum_squares_optimized(nums: &[i32]) -> i32 {
    nums.iter()
        .map(|&x| x * x)
        .sum()  // Direct consumption
}
```

**Reusing Allocations:**

```rust
// ❌ Allocates on every call
fn process_items(items: &[String]) {
    for item in items {
        let mut buffer = String::new();
        buffer.push_str(item);
        // process buffer
    }
}

// ✅ Reuse allocation
fn process_items_optimized(items: &[String]) {
    let mut buffer = String::new();
    for item in items {
        buffer.clear();
        buffer.push_str(item);
        // process buffer
    }
}
```

**Using `Cow` (Clone-on-Write):**

```rust
use std::borrow::Cow;

fn process_string(s: &str) -> Cow<str> {
    if s.contains("bad") {
        // Only allocate if modification needed
        Cow::Owned(s.replace("bad", "good"))
    } else {
        // No allocation
        Cow::Borrowed(s)
    }
}
```

## 2. LTO (Link-Time Optimization)

LTO enables cross-crate optimizations by performing optimization during linking rather than just compilation.

**Cargo.toml Configuration:**

```toml
[profile.release]
lto = true           # Enable full LTO (slow compile, best performance)
# or
lto = "thin"         # Faster compile, good performance balance
# or
lto = "fat"          # Same as true, explicit full LTO
```

**What LTO Does:**
- Inlines functions across crate boundaries
- Removes dead code globally
- Optimizes function calls across the entire program
- Can reduce binary size by 10-30%
- Improves runtime performance by 5-15%

**Trade-offs:**
- Significantly increases compile time
- Requires more memory during linking
- Best used for final release builds

```toml
# Example optimized release profile
[profile.release]
lto = true
codegen-units = 1    # Better optimization but slower compile
opt-level = 3
strip = true         # Remove debug symbols
```

## 3. PGO (Profile-Guided Optimization)

PGO uses runtime profiling data to guide compiler optimizations.

**Three-Step Process:**

**Step 1: Build with instrumentation**
```bash
# Set environment variable
export RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data"
cargo build --release
```

**Step 2: Run typical workloads**
```bash
# Run your application with representative data
./target/release/myapp
# This generates profiling data in /tmp/pgo-data
```

**Step 3: Rebuild with profile data**
```bash
# Merge profile data (requires llvm-profdata)
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Build with PGO
export RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata"
cargo build --release
```

**PGO Benefits:**
- Optimizes hot paths (frequently executed code)
- Better branch prediction
- Improved code layout and caching
- Can improve performance by 10-30% for compute-heavy workloads

**Example with Cargo Configuration:**

```toml
# .cargo/config.toml
[profile.release-pgo]
inherits = "release"

[build]
rustflags = ["-Cprofile-use=/path/to/merged.profdata"]
```

## 4. Compiler Optimization Levels

Rust provides several optimization levels via the `opt-level` flag.

```toml
[profile.dev]
opt-level = 0        # No optimization (default for dev)

[profile.release]
opt-level = 3        # Maximum optimization (default for release)

# Custom profiles
[profile.release-small]
inherits = "release"
opt-level = "z"      # Optimize for size
lto = true
codegen-units = 1

[profile.release-fast]
inherits = "release"
opt-level = 3        # Optimize for speed
lto = "thin"
codegen-units = 16
```

**Optimization Levels:**
- `0`: No optimization, fast compilation
- `1`: Basic optimizations
- `2`: Some optimizations
- `3`: Maximum optimization for speed (default release)
- `"s"`: Optimize for size (like -Os in C)
- `"z"`: Aggressively optimize for size

**Example Impact:**

```rust
// This kind of code benefits greatly from optimization
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// At opt-level=0: ~5 seconds for fib(40)
// At opt-level=3: ~2 seconds for fib(40)
// With memoization: ~microseconds
```

## 5. Target-CPU Flags

The `target-cpu` flag enables CPU-specific optimizations and instruction sets.

**Basic Usage:**

```bash
# Use native CPU features
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

**Cargo Configuration:**

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "target-cpu=native"]

# Or target specific CPUs
# rustflags = ["-C", "target-cpu=skylake"]
# rustflags = ["-C", "target-cpu=haswell"]
```

**Common Target CPUs:**
- `native`: Uses features of the CPU you're compiling on
- `generic`: Conservative baseline (default)
- Specific CPUs: `skylake`, `haswell`, `znver2`, `apple-m1`, etc.

**Example with SIMD Benefits:**

```rust
// This code can benefit from SIMD instructions with target-cpu
fn sum_array(data: &[f32]) -> f32 {
    data.iter().sum()  // Can be auto-vectorized with SIMD
}

// With target-cpu=native, the compiler might use:
// - AVX2 instructions (8 f32s at once)
// - AVX-512 instructions (16 f32s at once) on newer CPUs
```

**Checking Available Features:**

```bash
# See what features your CPU supports
rustc --print target-features

# Enable specific features
RUSTFLAGS="-C target-feature=+avx2,+fma" cargo build --release
```

## Complete Optimization Example

Here's a comprehensive `Cargo.toml` with aggressive optimizations:

```toml
[package]
name = "optimized-app"
version = "0.1.0"
edition = "2021"

[profile.release]
opt-level = 3                # Maximum optimization
lto = true                   # Enable full LTO
codegen-units = 1            # Single unit for best optimization
strip = true                 # Strip symbols
panic = "abort"              # Smaller binary, faster panics

[profile.release-size]
inherits = "release"
opt-level = "z"              # Optimize for size
lto = true
codegen-units = 1
strip = true

[profile.release-fast]
inherits = "release"
opt-level = 3
lto = "thin"                 # Balance speed/compile time
codegen-units = 16
```

**Build commands:**

```bash
# Maximum performance
RUSTFLAGS="-C target-cpu=native" cargo build --release

# With PGO (after gathering profile data)
RUSTFLAGS="-C target-cpu=native -C profile-use=merged.profdata" \
  cargo build --release

# Size-optimized build
cargo build --profile release-size
```

## Performance Measurement

Always measure before and after optimization:

```rust
use std::time::Instant;

fn benchmark<F: Fn()>(name: &str, f: F) {
    let start = Instant::now();
    f();
    let duration = start.elapsed();
    println!("{}: {:?}", name, duration);
}

fn main() {
    benchmark("Process data", || {
        // Your code here
    });
}
```

Or use the `criterion` crate for more rigorous benchmarking:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

These optimization techniques can dramatically improve Rust application performance, with potential speedups ranging from 2-10x depending on your use case. The key is to profile first, optimize second, and always measure the impact of your changes.