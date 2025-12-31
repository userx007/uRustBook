# Profiling and Benchmarking in Rust

Profiling and benchmarking are essential practices for understanding and optimizing your Rust code's performance. While Rust provides memory safety and zero-cost abstractions, you still need to measure actual performance to identify bottlenecks and validate optimizations.

## Benchmarking with Criterion

Criterion is the de facto standard benchmarking library for Rust. It provides statistical analysis, protection against measurement noise, and beautiful HTML reports.

First, add criterion to your `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

Here's a practical example benchmarking different string concatenation approaches:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn string_concat_plus(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result = result + &i.to_string();
    }
    result
}

fn string_concat_push_str(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result.push_str(&i.to_string());
    }
    result
}

fn string_concat_format(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result = format!("{}{}", result, i);
    }
    result
}

fn benchmark_string_concat(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_concatenation");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("plus_operator", size), size, 
            |b, &size| b.iter(|| string_concat_plus(black_box(size))));
        
        group.bench_with_input(BenchmarkId::new("push_str", size), size,
            |b, &size| b.iter(|| string_concat_push_str(black_box(size))));
        
        group.bench_with_input(BenchmarkId::new("format", size), size,
            |b, &size| b.iter(|| string_concat_format(black_box(size))));
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_string_concat);
criterion_main!(benches);
```

The `black_box` function prevents the compiler from optimizing away your benchmark code. Run benchmarks with `cargo bench`, and criterion generates detailed reports showing mean execution time, standard deviation, and performance comparisons.

## CPU Profiling with perf

`perf` is a powerful Linux profiling tool that can identify hot spots in your code. To use it with Rust, you need debug symbols in your release build:

```toml
[profile.release]
debug = true
```

Build and profile your application:

```bash
cargo build --release
perf record --call-graph=dwarf ./target/release/myapp
perf report
```

The `perf record` command captures performance data while your program runs, and `perf report` shows where CPU time is spent. You'll see output like:

```
Overhead  Command  Shared Object       Symbol
  45.23%  myapp    myapp               [.] myapp::process_data
  23.45%  myapp    myapp               [.] myapp::parse_input
  12.34%  myapp    libc.so.6           [.] memcpy
```

This reveals that `process_data` consumes 45% of CPU time, making it a prime optimization candidate.

## Flame Graphs

Flame graphs visualize profiling data as interactive hierarchical charts. Install the tools:

```bash
cargo install flamegraph
```

Generate a flame graph directly:

```bash
cargo flamegraph --bin myapp
```

This creates an SVG file showing call stacks, with wider bars indicating more time spent. You can also combine perf data with the flamegraph script:

```bash
perf record --call-graph=dwarf -F 997 ./target/release/myapp
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

Here's code that demonstrates optimization opportunities visible in flame graphs:

```rust
use std::collections::HashMap;

fn inefficient_lookup(data: &Vec<String>, queries: &[String]) -> Vec<bool> {
    // This shows up as a hot spot in flame graphs
    queries.iter()
        .map(|q| data.iter().any(|d| d == q))  // O(n*m) - terrible!
        .collect()
}

fn efficient_lookup(data: &Vec<String>, queries: &[String]) -> Vec<bool> {
    // Much better - build HashSet once
    let set: HashMap<&String, ()> = data.iter()
        .map(|s| (s, ()))
        .collect();
    
    queries.iter()
        .map(|q| set.contains_key(q))  // O(1) lookups
        .collect()
}
```

A flame graph would show `inefficient_lookup` with a wide bar containing nested `iter().any()` calls, while `efficient_lookup` shows minimal time in the lookup phase.

## Memory Profiling

Memory profiling helps identify leaks, excessive allocations, and memory bloat. Several tools are available for Rust.

**Valgrind/Massif** tracks heap allocations:

```bash
cargo build --release
valgrind --tool=massif ./target/release/myapp
ms_print massif.out.12345
```

**DHAT** (part of Valgrind) provides detailed allocation analysis:

```bash
valgrind --tool=dhat ./target/release/myapp
```

**Heaptrack** is a Linux tool with excellent visualization:

```bash
heaptrack ./target/release/myapp
heaptrack_gui heaptrack.myapp.12345.gz
```

For production monitoring, consider `jemalloc` with profiling enabled:

```toml
[dependencies]
jemallocator = "0.5"
```

```rust
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn memory_intensive_operation() {
    // Allocate vectors that grow over time
    let mut data: Vec<Vec<u8>> = Vec::new();
    
    for i in 0..1000 {
        let mut vec = Vec::with_capacity(1024 * 1024);
        vec.extend(std::iter::repeat(i as u8).take(1024 * 1024));
        data.push(vec);
    }
    
    // Memory profiler shows this holding ~1GB
    println!("Peak memory usage here");
    
    // Clearing frees memory
    data.clear();
    data.shrink_to_fit();
}
```

**Using `cargo-instruments` on macOS:**

```bash
cargo install cargo-instruments
cargo instruments -t time --release --bin myapp
```

This launches Xcode Instruments with comprehensive profiling including memory, CPU, and system calls.

## Interpreting Performance Data

When analyzing profiling results, look for these patterns:

**Hot paths**: Functions appearing at the top of perf reports or with wide flame graph bars are your primary optimization targets. Even small improvements here yield significant gains.

**Unexpected allocations**: Memory profilers revealing allocations in tight loops often indicate opportunities to reuse buffers or use stack allocation. For example, replacing `format!` in a loop with a reused `String` buffer.

**Cache misses**: Tools like `perf stat` show cache miss rates:

```bash
perf stat -e cache-references,cache-misses ./target/release/myapp
```

High cache miss rates suggest poor data locality. Consider restructuring data layouts (Array of Structs vs Struct of Arrays) or processing data in chunks.

**Lock contention**: Profilers showing threads waiting on mutexes indicate parallelism issues. Consider finer-grained locking, lock-free structures, or redesigning for less shared state.

Here's an example showing before and after optimization based on profiling data:

```rust
use std::time::Instant;

// Before: Profiling showed excessive allocations
fn process_lines_slow(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim().to_uppercase())  // Allocates for each line
        .filter(|line| !line.is_empty())
        .collect()
}

// After: Reduced allocations by ~50%
fn process_lines_fast(text: &str) -> Vec<String> {
    let line_count = text.lines().count();
    let mut result = Vec::with_capacity(line_count);
    
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_uppercase());
        }
    }
    result
}

// Micro-benchmark to validate
fn benchmark_both() {
    let text = "line 1\n  line 2  \n\nline 3\n".repeat(10000);
    
    let start = Instant::now();
    let _ = process_lines_slow(&text);
    println!("Slow: {:?}", start.elapsed());
    
    let start = Instant::now();
    let _ = process_lines_fast(&text);
    println!("Fast: {:?}", start.elapsed());
}
```

## Complete Profiling Workflow

A typical optimization workflow combines these tools:

1. **Start with criterion benchmarks** to establish baselines and validate that changes improve performance
2. **Use perf or flamegraph** to identify hot functions consuming the most CPU time
3. **Apply memory profilers** if allocations appear problematic in flame graphs or if memory usage is high
4. **Optimize the identified bottleneck** with targeted changes
5. **Re-benchmark** to verify improvements and ensure you didn't regress other paths
6. **Profile again** since fixing one bottleneck often reveals the next one

Remember that profiling data is only meaningful with representative workloads. Profile with realistic inputs, not toy examples, and always measure in release mode since debug builds have completely different performance characteristics. The combination of statistical benchmarking through criterion and detailed profiling with perf/flamegraph/memory tools gives you a complete picture of your application's performance and clear direction for optimization efforts.