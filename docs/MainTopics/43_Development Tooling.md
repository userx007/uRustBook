# Rust Development Tooling

Rust has an excellent ecosystem of development tools that help you write better, safer, and more maintainable code. Here's a comprehensive guide to the essential tools.

## 1. Clippy - The Rust Linter

**Clippy** is Rust's official linter that catches common mistakes and suggests idiomatic improvements.

### Installation & Basic Usage
```bash
# Install (comes with rustup)
rustup component add clippy

# Run clippy
cargo clippy

# Run with warnings as errors
cargo clippy -- -D warnings
```

### Configuration

Create a `clippy.toml` or `.clippy.toml` in your project root:

```toml
# Set the minimum supported Rust version
msrv = "1.70.0"

# Warn on cognitive complexity above 15
cognitive-complexity-threshold = 15

# Allow certain lints
allow = [
    "too_many_arguments",
    "type_complexity",
]

# Deny certain lints (treat as errors)
deny = [
    "unwrap_used",
    "expect_used",
]
```

Or configure via `Cargo.toml`:
```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
indexing_slicing = "warn"
panic = "deny"
```

### Example with Clippy Suggestions

```rust
// Before Clippy
fn process_data(data: &Vec<String>) -> Option<String> {
    if data.len() > 0 {
        Some(data[0].clone())
    } else {
        None
    }
}

// After Clippy suggestions
fn process_data(data: &[String]) -> Option<String> {
    // Use slice instead of &Vec
    // Use !is_empty() instead of len() > 0
    // Use get() and cloned() for safer access
    data.first().cloned()
}
```

## 2. rustfmt - Code Formatter

**rustfmt** automatically formats Rust code according to style guidelines.

### Installation & Usage
```bash
# Install
rustup component add rustfmt

# Format all files in project
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Configuration

Create a `rustfmt.toml` or `.rustfmt.toml`:

```toml
# Maximum line width
max_width = 100

# Indentation
tab_spaces = 4

# Import formatting
imports_granularity = "Crate"
group_imports = "StdExternalCrate"

# Function parameters
fn_params_layout = "Tall"

# Chain formatting
chain_width = 60

# Use field init shorthand
use_field_init_shorthand = true

# Remove nested parens
use_try_shorthand = true

# Format strings
format_strings = true
```

### Example Formatting

```rust
// Before rustfmt
fn calculate(x:i32,y:i32,z:i32)->i32{let result=x+y+z;result}

// After rustfmt
fn calculate(x: i32, y: i32, z: i32) -> i32 {
    let result = x + y + z;
    result
}
```

## 3. rust-analyzer - Language Server

**rust-analyzer** is a Language Server Protocol (LSP) implementation that provides IDE features like autocomplete, go-to-definition, and inline errors.

### Features
- **Intelligent code completion**
- **Type hints and inlay hints**
- **Go to definition/implementation**
- **Find all references**
- **Code actions (quick fixes)**
- **Syntax highlighting**
- **Inline diagnostics**

### Configuration (VS Code settings.json)

```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.inlayHints.typeHints.enable": true,
    "rust-analyzer.inlayHints.parameterHints.enable": true,
    "rust-analyzer.lens.references.adt.enable": true,
    "rust-analyzer.lens.references.method.enable": true,
    "rust-analyzer.completion.autoimport.enable": true,
    "rust-analyzer.procMacro.enable": true
}
```

### Example: What rust-analyzer Shows

```rust
fn process_items(items: Vec<String>) {
    // rust-analyzer shows:
    // - Type hints: items: Vec<String>
    // - Inline errors if types mismatch
    // - Autocomplete for methods
    
    let result = items
        .iter()                    // <- shows: impl Iterator<Item = &String>
        .filter(|s| s.len() > 3)   // <- shows parameter types
        .collect::<Vec<_>>();      // <- suggests collect types
}
```

## 4. cargo-expand - Macro Expansion

**cargo-expand** shows you what your code looks like after macro expansion.

### Installation & Usage
```bash
# Install
cargo install cargo-expand

# Expand entire crate
cargo expand

# Expand specific module
cargo expand module::path

# Expand specific item
cargo expand module::function_name
```

### Example

**Original code:**
```rust
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    println!("Hello, {}!", "world");
}
```

**After `cargo expand`:**
```rust
struct User {
    name: String,
    age: u32,
}

impl ::core::fmt::Debug for User {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "User",
            "name",
            &&self.name,
            "age",
            &&self.age,
        )
    }
}

impl ::core::clone::Clone for User {
    fn clone(&self) -> User {
        User {
            name: ::core::clone::Clone::clone(&self.name),
            age: ::core::clone::Clone::clone(&self.age),
        }
    }
}

fn main() {
    {
        ::std::io::_print(
            ::core::fmt::Arguments::new_v1(
                &["Hello, ", "!\n"],
                &[::core::fmt::ArgumentV1::new_display(&"world")],
            ),
        );
    }
}
```

## 5. Miri - Interpreter for Detecting Undefined Behavior

**Miri** is an interpreter for Rust's mid-level intermediate representation (MIR) that detects undefined behavior.

### Installation & Usage
```bash
# Install
rustup +nightly component add miri

# Run tests with Miri
cargo +nightly miri test

# Run specific binary
cargo +nightly miri run
```

### What Miri Detects
- **Use after free**
- **Invalid pointer arithmetic**
- **Data races**
- **Uninitialized memory access**
- **Invalid enum discriminants**
- **Out-of-bounds array access**

### Example

```rust
fn unsafe_code() {
    let mut x = 42;
    let ptr = &mut x as *mut i32;
    
    unsafe {
        // Miri will catch this!
        let ptr2 = ptr.offset(1); // Out of bounds
        *ptr2 = 100; // Writing to invalid memory
    }
}

fn data_race_example() {
    use std::sync::Arc;
    use std::thread;
    
    let data = Arc::new(0);
    let data1 = data.clone();
    let data2 = data.clone();
    
    // Miri will detect this data race
    thread::spawn(move || {
        let ptr = Arc::as_ptr(&data1) as *mut i32;
        unsafe { *ptr = 1; }
    });
    
    thread::spawn(move || {
        let ptr = Arc::as_ptr(&data2) as *mut i32;
        unsafe { *ptr = 2; }
    });
}
```

**Miri output:**
```
error: Undefined Behavior: dereferencing pointer failed: 
pointer must be in-bounds at offset 4, but is outside bounds
```

## 6. cargo-audit - Security Vulnerability Scanner

**cargo-audit** checks your dependencies for known security vulnerabilities.

### Installation & Usage
```bash
# Install
cargo install cargo-audit

# Check for vulnerabilities
cargo audit

# Check and automatically update vulnerable dependencies
cargo audit fix

# Generate JSON report
cargo audit --json
```

### Example Output

```bash
$ cargo audit

    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 523 security advisories (from rustsec-advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (125 crate dependencies)

Crate:     time
Version:   0.1.43
Warning:   potential segfault in `time` crate
Title:     Potential segfault in the time crate
Date:      2020-11-18
ID:        RUSTSEC-2020-0071
URL:       https://rustsec.org/advisories/RUSTSEC-2020-0071
Solution:  Upgrade to >=0.2.23

error: 1 vulnerability found!
```

### Configuration

Create a `.cargo/audit.toml`:

```toml
[advisories]
# Ignore specific advisories
ignore = [
    "RUSTSEC-2020-0071",
]

# Set severity threshold
severity-threshold = "medium"

[output]
# Deny warnings
deny = ["warnings"]

# Output format
format = "json"
```

## Complete CI/CD Integration Example

Here's a GitHub Actions workflow using all these tools:

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
          
      - name: Check formatting
        run: cargo fmt -- --check
        
      - name: Run Clippy
        run: cargo clippy -- -D warnings
        
      - name: Run tests
        run: cargo test
        
      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit
          
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Miri
        run: |
          rustup toolchain install nightly --component miri
          rustup override set nightly
          cargo miri setup
          
      - name: Run Miri tests
        run: cargo miri test
```

These tools together create a robust development environment that catches bugs early, maintains code quality, and ensures security. They're essential for any serious Rust project.