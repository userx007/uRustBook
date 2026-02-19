# Conditional Compilation and `cfg`

1. **`#[cfg(...)]` attribute** — excluding whole items at compile time
2. **`cfg!()` macro** — boolean branching inside expressions
3. **Built-in cfg keys** — OS, arch, pointer width, endianness, debug assertions
4. **Logical combinators** — `all()`, `any()`, `not()` for complex conditions
5. **Feature flags** — declaring and using optional features via `Cargo.toml`
6. **`cfg_attr`** — conditionally applying other attributes (like `derive`)
7. **Custom cfg keys** — setting them via `--cfg` or `build.rs`
8. **`CARGO_CFG_*` env vars** — using target cfg inside build scripts
9. **`#[cfg(test)]`** — keeping test helpers out of production binaries
10. **Cross-platform abstraction pattern** — splitting platform code into gated submodules
11. **`compile_error!`** — failing loudly for unsupported or conflicting configurations
12. **Inspecting active cfg values** — using `rustc --print cfg`

# 54. Conditional Compilation and `cfg` in Rust

Conditional compilation lets you include or exclude code at compile time based on configuration
flags, target platform properties, feature flags, or custom conditions. Rust provides this
mechanism through `cfg` attributes and the `cfg!()` macro, making it possible to write a single
codebase that adapts to many environments without runtime overhead.

---

## 1. The `#[cfg(...)]` Attribute

The `#[cfg(...)]` attribute is placed on any Rust item (function, struct, module, `impl` block,
`use` statement, etc.) and instructs the compiler to include that item **only when** the given
condition is true. If the condition is false, the item is completely absent from the compiled
binary — it is not compiled at all.

```rust
#[cfg(target_os = "linux")]
fn platform_info() -> &'static str {
    "Running on Linux"
}

#[cfg(target_os = "windows")]
fn platform_info() -> &'static str {
    "Running on Windows"
}

#[cfg(target_os = "macos")]
fn platform_info() -> &'static str {
    "Running on macOS"
}

fn main() {
    println!("{}", platform_info());
}
```

Only one of the three `platform_info` definitions will exist in the final binary.

---

## 2. The `cfg!()` Macro

While `#[cfg(...)]` controls whole items, the `cfg!()` macro evaluates to a `bool` at compile time
inside an expression. It is useful when you want to branch on a condition inside a function body.

```rust
fn main() {
    if cfg!(debug_assertions) {
        println!("Debug build — extra checks are active.");
    } else {
        println!("Release build — optimized for speed.");
    }

    let os = if cfg!(target_os = "windows") { "Windows" }
             else if cfg!(target_os = "linux") { "Linux" }
             else { "Other" };

    println!("OS family: {}", os);
}
```

> **Key difference:** `cfg!()` keeps *both* branches in the source, but the compiler can still
> optimize away the dead branch. `#[cfg(...)]` removes code before type-checking, so unreachable
> branches may not even need to compile.

---

## 3. Built-in Configuration Keys

Rust's compiler populates a rich set of cfg keys automatically. The most commonly used ones are:

### Target architecture and OS

```rust
#[cfg(target_arch = "x86_64")]
fn simd_add(a: f32, b: f32) -> f32 { a + b } // placeholder

#[cfg(target_arch = "aarch64")]
fn simd_add(a: f32, b: f32) -> f32 { a + b } // ARM NEON version

#[cfg(target_os = "linux")]
fn get_pid() -> u32 {
    // Linux-specific syscall
    unsafe { libc::getpid() as u32 }
}

#[cfg(target_family = "unix")]
fn is_unix() -> bool { true }

#[cfg(target_family = "windows")]
fn is_unix() -> bool { false }
```

### Pointer width

```rust
#[cfg(target_pointer_width = "64")]
type USize64 = u64;

#[cfg(target_pointer_width = "32")]
type USize64 = u32;
```

### Endianness

```rust
#[cfg(target_endian = "little")]
fn read_u16(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

#[cfg(target_endian = "big")]
fn read_u16(bytes: [u8; 2]) -> u16 {
    u16::from_be_bytes(bytes)
}
```

### Debug assertions and test mode

```rust
#[cfg(debug_assertions)]
fn validate(x: i32) {
    assert!(x > 0, "x must be positive in debug mode");
}

#[cfg(not(debug_assertions))]
fn validate(_x: i32) {} // no-op in release

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        validate(5);
    }
}
```

---

## 4. Logical Combinators: `all()`, `any()`, `not()`

`cfg` conditions can be combined with logical operators.

```rust
// Only on 64-bit Linux
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
fn linux_64_only() {
    println!("64-bit Linux code path");
}

// On either macOS or iOS
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn apple_platform() {
    println!("Apple platform");
}

// Anything that is NOT Windows
#[cfg(not(target_os = "windows"))]
fn non_windows_code() {
    println!("Not Windows");
}

// Complex nested condition
#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    target_arch = "x86_64",
    not(debug_assertions)
))]
fn optimized_unix_x86() {
    println!("Optimized release build on x86_64 Unix");
}
```

---

## 5. Feature Flags

Feature flags let library authors and application developers opt into optional functionality.
They are declared in `Cargo.toml` and activated at compile time.

### Declaring features in `Cargo.toml`

```toml
[package]
name = "my_crate"
version = "0.1.0"
edition = "2021"

[features]
# Default features are enabled unless the user opts out
default = ["logging"]

# Optional features
logging   = ["dep:log"]
async     = ["dep:tokio"]
serde     = ["dep:serde", "dep:serde_json"]
advanced  = ["logging", "serde"]  # a feature can enable other features

[dependencies]
log       = { version = "0.4", optional = true }
tokio     = { version = "1",   optional = true, features = ["full"] }
serde     = { version = "1",   optional = true }
serde_json = { version = "1",  optional = true }
```

### Using features in code

```rust
// src/lib.rs

/// Always available
pub fn core_function() -> u32 { 42 }

/// Only compiled when the "logging" feature is enabled
#[cfg(feature = "logging")]
pub fn logged_function(msg: &str) {
    log::info!("logged_function called: {}", msg);
}

/// Async support only available with the "async" feature
#[cfg(feature = "async")]
pub async fn async_fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

/// Serialization support
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub name: String,
    pub value: u32,
}
```

### Enabling features in downstream `Cargo.toml`

```toml
[dependencies]
my_crate = { version = "0.1", features = ["async", "serde"] }

# To disable default features and pick only what you need:
my_crate = { version = "0.1", default-features = false, features = ["logging"] }
```

### Enabling features from the command line

```bash
cargo build --features "async serde"
cargo build --all-features
cargo build --no-default-features --features logging
cargo test --features serde
```

---

## 6. The `cfg_attr` Attribute

`cfg_attr` conditionally applies *another attribute* only when a cfg condition is satisfied. This
avoids duplicating the entire item declaration.

```rust
// Derive Serialize/Deserialize only when the "serde" feature is active
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UserProfile {
    pub user_id: u64,
    pub display_name: String,
    pub email: String,
}

// Allow dead code only in test builds to silence warnings on test helpers
#[cfg_attr(test, allow(dead_code))]
fn test_helper() -> Vec<u8> {
    vec![1, 2, 3]
}

// Add Clone/Debug only in debug builds (useful for huge types)
#[cfg_attr(debug_assertions, derive(Debug, Clone))]
pub struct HeavyData {
    pub buffer: Vec<u8>,
}
```

---

## 7. Custom `cfg` Keys via `--cfg` and `build.rs`

You can define entirely custom configuration keys, either on the command line or from a build
script.

### Via the command line

```bash
RUSTFLAGS='--cfg my_feature' cargo build
```

```rust
#[cfg(my_feature)]
fn only_with_my_feature() {
    println!("Custom feature enabled!");
}
```

### Via `build.rs`

`build.rs` is a build script that runs on the host machine before compilation. It can emit
`cargo:rustc-cfg=` instructions to set cfg keys based on detected system properties.

```rust
// build.rs
fn main() {
    // Detect SIMD support
    if is_x86_feature_detected_at_build_time() {
        println!("cargo:rustc-cfg=has_avx2");
    }

    // Set a cfg based on an environment variable
    if std::env::var("ENABLE_PROFILING").is_ok() {
        println!("cargo:rustc-cfg=profiling");
    }

    // Check OS version or installed libraries, etc.
    let target = std::env::var("TARGET").unwrap();
    if target.contains("musl") {
        println!("cargo:rustc-cfg=musl_target");
    }
}

fn is_x86_feature_detected_at_build_time() -> bool {
    // Real detection would use the `cc` or `cpuid` crates
    cfg!(target_arch = "x86_64")
}
```

```rust
// src/lib.rs
#[cfg(has_avx2)]
fn fast_hash(data: &[u8]) -> u64 {
    // Use AVX2 SIMD intrinsics
    todo!()
}

#[cfg(not(has_avx2))]
fn fast_hash(data: &[u8]) -> u64 {
    // Portable fallback
    data.iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

#[cfg(profiling)]
fn record_span(name: &str) {
    eprintln!("[PROFILE] {}", name);
}

#[cfg(not(profiling))]
fn record_span(_name: &str) {}
```

### Emitting link instructions from `build.rs`

```rust
// build.rs — link a native library conditionally
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "linux" => println!("cargo:rustc-link-lib=pthread"),
        "macos" => println!("cargo:rustc-link-lib=framework=CoreFoundation"),
        _ => {}
    }
}
```

---

## 8. `CARGO_CFG_*` Environment Variables in `build.rs`

Inside a `build.rs`, Cargo automatically provides environment variables of the form
`CARGO_CFG_<KEY>` for every cfg key of the target. This lets you make decisions based on the
target even though the build script runs on the host.

```rust
// build.rs
fn main() {
    let target_os  = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let ptr_width  = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap();

    println!("cargo:warning=Building for {target_os}/{target_arch} ({ptr_width}-bit)");

    if target_os == "android" {
        println!("cargo:rustc-cfg=android_build");
    }
}
```

---

## 9. Checking cfg Values in Tests

The `#[cfg(test)]` attribute is so common it deserves its own mention. The entire `tests` module
is excluded from production builds.

```rust
pub fn add(a: i32, b: i32) -> i32 { a + b }

// This whole block is only compiled during `cargo test`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, -2), -3);
    }

    // A helper only needed in tests — no production overhead
    fn make_test_data() -> Vec<i32> {
        vec![1, 2, 3, 4, 5]
    }

    #[test]
    fn test_sum() {
        let sum: i32 = make_test_data().iter().sum();
        assert_eq!(sum, 15);
    }
}
```

---

## 10. Real-World Pattern: Cross-Platform Abstraction

A common pattern is to split platform-specific code into submodules gated by `cfg`.

```
src/
├── lib.rs
└── platform/
    ├── mod.rs
    ├── unix.rs
    └── windows.rs
```

```rust
// src/platform/mod.rs
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

// Fallback for unsupported platforms
#[cfg(not(any(unix, windows)))]
compile_error!("This crate only supports Unix and Windows targets.");
```

```rust
// src/platform/unix.rs
pub fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

pub fn path_separator() -> char { '/' }
```

```rust
// src/platform/windows.rs
pub fn home_dir() -> std::path::PathBuf {
    std::env::var("USERPROFILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("C:\\Users\\Default"))
}

pub fn path_separator() -> char { '\\' }
```

```rust
// src/lib.rs
mod platform;

fn main() {
    let home = platform::home_dir();
    println!("Home: {}", home.display());
    println!("Separator: {}", platform::path_separator());
}
```

---

## 11. `compile_error!` for Unsupported Configurations

You can use `compile_error!` alongside `cfg` to emit a helpful message when someone tries to
compile your crate in an unsupported configuration.

```rust
#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
compile_error!("Unsupported pointer width — only 32-bit and 64-bit targets are supported.");

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!("Features `async` and `blocking` are mutually exclusive. Enable only one.");
```

---

## 12. Viewing Active cfg Values

You can inspect what cfg flags are active for a given target with:

```bash
# See all cfg flags for the current host target
rustc --print cfg

# See cfg flags for a specific cross-compile target
rustc --print cfg --target aarch64-unknown-linux-gnu
```

Example output (abbreviated):

```
debug_assertions
target_arch="x86_64"
target_endian="little"
target_env="gnu"
target_family="unix"
target_feature="fxsr"
target_feature="sse"
target_feature="sse2"
target_os="linux"
target_pointer_width="64"
unix
```

---

## Summary

| Mechanism | Purpose |
|---|---|
| `#[cfg(...)]` | Include/exclude whole items at compile time |
| `cfg!(...)` | Boolean expression evaluated at compile time inside expressions |
| `#[cfg_attr(...)]` | Conditionally apply another attribute |
| `[features]` in `Cargo.toml` | Declare optional functionality, enabled by users |
| `build.rs` + `cargo:rustc-cfg=` | Set cfg keys based on environment, system detection, or env vars |
| `compile_error!` with `cfg` | Fail compilation with a helpful message for invalid configs |
| `--cfg` via `RUSTFLAGS` | Set arbitrary cfg keys from the command line or CI |

Conditional compilation is one of Rust's most powerful zero-cost abstractions: all branching
decisions happen before the binary is produced, leaving no runtime overhead and ensuring that
unsupported code paths can't accidentally be reached on the wrong platform.