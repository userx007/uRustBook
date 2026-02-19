# Plugin Systems and Dynamic Loading in Rust

**What's inside:**

- **`cdylib` crate type** — how to compile a Rust plugin as a C-compatible shared library
- **`libloading` walkthrough** — opening `.so`/`.dll` files, resolving symbols, and the `Symbol<T>` type
- **ABI stability deep dive** — why Rust's ABI is unstable, and how `#[repr(C)]`, raw pointers, and shared type crates solve it
- **Full working example** — a host + plugin audio processor with a shared API crate
- **Patterns** — trait-object bridges, plugin registries, version negotiation, and hot-reloading
- **Safety checklist** — covering library lifetime, panic handling across FFI, and allocator mismatches
- **Alternatives** — `abi_stable` for richer Rust types without raw C, and WASM-based plugins for sandboxed portability


## Overview

Plugin systems allow applications to be extended at runtime without recompilation. In Rust, this typically involves compiling plugins as shared libraries (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows) and loading them dynamically using the `libloading` crate. This pattern is powerful, but comes with unique challenges around ABI stability, memory safety, and error handling that are critical to understand.

---

## 1. Core Concepts

### What Is Dynamic Loading?

Dynamic loading is the process of loading a compiled shared library into a running process at runtime. Unlike static linking (where everything is resolved at compile time), dynamic loading allows:

- Adding new features without restarting or recompiling the host application.
- Third-party developers to ship plugins independently.
- Hot-reloading during development.

### The `cdylib` Crate Type

To build a plugin in Rust, you declare the crate type as `cdylib` in `Cargo.toml`. This produces a C-compatible dynamic library with a stable exported symbol table.

```toml
# plugin/Cargo.toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
```

The `cdylib` type strips Rust-specific metadata and produces a pure C ABI shared library — this is essential for `libloading` to find and call your exported functions.

---

## 2. The `libloading` Crate

`libloading` is the de facto standard for loading shared libraries in Rust. It wraps platform-specific APIs (`dlopen`/`LoadLibrary`) with a safe(r) interface.

### Adding the Dependency

```toml
# host/Cargo.toml
[dependencies]
libloading = "0.8"
```

### Basic Usage: Loading a Symbol

```rust
// host/src/main.rs
use libloading::{Library, Symbol};

fn main() {
    unsafe {
        // Load the shared library
        let lib = Library::new("./target/debug/libmy_plugin.so")
            .expect("Failed to load library");

        // Look up an exported function by name
        let greet: Symbol<unsafe extern "C" fn()> = lib
            .get(b"greet\0")
            .expect("Symbol not found");

        greet();
    }
}
```

```rust
// plugin/src/lib.rs
#[no_mangle]
pub extern "C" fn greet() {
    println!("Hello from the plugin!");
}
```

Key points:
- `Library::new` opens the shared library by path.
- `lib.get(b"symbol_name\0")` retrieves a function pointer by its exported name (null-terminated byte string).
- `#[no_mangle]` prevents Rust from mangling the symbol name.
- `extern "C"` ensures the function uses the C calling convention.
- The entire block is `unsafe` because Rust cannot verify the type or validity of the loaded symbol.

---

## 3. ABI Stability Challenges

This is the most critical and complex aspect of Rust plugin systems.

### What Is ABI?

The Application Binary Interface (ABI) defines how data and functions are laid out in memory at the binary level — struct field ordering, calling conventions, enum representation, etc.

### Why Rust's ABI Is Unstable

Rust **does not guarantee a stable ABI** between compiler versions, or even between compilations of the same code. The compiler is free to:

- Reorder struct fields for optimization.
- Change the size/alignment of types.
- Alter enum discriminant representation.

This means if your host and plugin are compiled with different Rust versions or even different compiler flags, your program may silently read garbage data or crash.

### The Safe Path: Use `repr(C)`

The C ABI is stable and well-defined. You can opt into it for types that cross the plugin boundary:

```rust
// shared/src/lib.rs  (a shared types crate used by both host and plugin)

/// A stable, C-compatible struct for passing data across the plugin boundary.
#[repr(C)]
pub struct PluginInfo {
    pub name: *const std::os::raw::c_char, // Use raw pointers, not &str or String
    pub version: u32,
}

/// A stable vtable for the plugin interface.
#[repr(C)]
pub struct PluginVTable {
    pub init: extern "C" fn() -> *mut PluginState,
    pub process: extern "C" fn(state: *mut PluginState, input: f64) -> f64,
    pub destroy: extern "C" fn(state: *mut PluginState),
}

// Opaque state type — host never inspects its internals
pub struct PluginState {
    _private: (),
}
```

**Rules for ABI-safe types:**
- Prefer `#[repr(C)]` on all structs/enums crossing the boundary.
- Use raw pointers (`*const T`, `*mut T`) instead of references.
- Never pass `String`, `Vec`, `Box`, or other heap types directly — their internal layout is not stable.
- Use `std::os::raw::c_char` for strings; pass them as null-terminated C strings.
- Use `#[repr(u32)]` or similar on enums to fix their representation.

---

## 4. A Full Working Example

Let's build a simple extensible audio processing pipeline.

### Shared Interface Crate

```rust
// plugin_api/src/lib.rs

#[repr(C)]
pub struct ProcessorInfo {
    pub name: *const std::os::raw::c_char,
}

/// The C-compatible plugin interface
#[repr(C)]
pub struct AudioProcessor {
    /// Process a buffer of samples in-place
    pub process: extern "C" fn(samples: *mut f32, count: usize),
    /// Return a null-terminated name string
    pub name: extern "C" fn() -> *const std::os::raw::c_char,
}

/// The symbol name the host will look for in every plugin
pub const PLUGIN_ENTRY: &[u8] = b"create_processor\0";

/// The signature of the plugin entry point
pub type CreateProcessorFn = extern "C" fn() -> AudioProcessor;
```

### The Plugin

```rust
// gain_plugin/src/lib.rs
use std::ffi::CStr;
use plugin_api::AudioProcessor;

extern "C" fn process_samples(samples: *mut f32, count: usize) {
    let buf = unsafe { std::slice::from_raw_parts_mut(samples, count) };
    for s in buf.iter_mut() {
        *s *= 1.5; // Apply 1.5x gain
    }
}

extern "C" fn get_name() -> *const std::os::raw::c_char {
    b"Gain 1.5x\0".as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub extern "C" fn create_processor() -> AudioProcessor {
    AudioProcessor {
        process: process_samples,
        name: get_name,
    }
}
```

### The Host Application

```rust
// host/src/main.rs
use libloading::{Library, Symbol};
use plugin_api::{AudioProcessor, CreateProcessorFn, PLUGIN_ENTRY};

struct LoadedPlugin {
    processor: AudioProcessor,
    _lib: Library, // Keep the library alive as long as the processor is used
}

fn load_plugin(path: &str) -> Result<LoadedPlugin, Box<dyn std::error::Error>> {
    unsafe {
        let lib = Library::new(path)?;
        let constructor: Symbol<CreateProcessorFn> = lib.get(PLUGIN_ENTRY)?;
        let processor = constructor();

        Ok(LoadedPlugin {
            processor,
            _lib: lib, // IMPORTANT: must outlive processor
        })
    }
}

fn main() {
    let plugin = load_plugin("./target/debug/libgain_plugin.so")
        .expect("Failed to load plugin");

    let mut samples: Vec<f32> = vec![0.1, 0.5, 0.8, -0.3];

    // Call into the plugin
    unsafe {
        (plugin.processor.process)(samples.as_mut_ptr(), samples.len());
    }

    println!("Processed samples: {:?}", samples);
    // Output: [0.15, 0.75, 1.2, -0.45]

    // Print plugin name
    let name_ptr = (plugin.processor.name)();
    let name = unsafe { std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap() };
    println!("Plugin: {}", name);
}
```

> **Critical gotcha:** The `Library` must be kept alive (not dropped) for as long as you're using any symbols loaded from it. Dropping `Library` unloads the `.so`, leaving dangling function pointers that will cause undefined behavior when called. The pattern of keeping `_lib` inside the struct ensures this.

---

## 5. Patterns for Extensible Rust Applications

### Pattern 1: The Trait Object + FFI Bridge

A common ergonomic pattern wraps the raw C function pointers in a Rust trait on the host side:

```rust
// host side: wrap the C vtable in a Rust trait object
use plugin_api::AudioProcessor;

trait Processor {
    fn process(&self, samples: &mut [f32]);
    fn name(&self) -> &str;
}

struct FfiProcessor {
    inner: AudioProcessor,
    name_cache: String,
}

impl FfiProcessor {
    fn new(inner: AudioProcessor) -> Self {
        let name = unsafe {
            std::ffi::CStr::from_ptr((inner.name)())
                .to_str()
                .unwrap_or("unknown")
                .to_owned()
        };
        FfiProcessor { inner, name_cache: name }
    }
}

impl Processor for FfiProcessor {
    fn process(&self, samples: &mut [f32]) {
        unsafe {
            (self.inner.process)(samples.as_mut_ptr(), samples.len());
        }
    }

    fn name(&self) -> &str {
        &self.name_cache
    }
}
```

Now the rest of the application works with `Box<dyn Processor>` and never needs to touch `unsafe` again.

### Pattern 2: Plugin Registry

```rust
use std::collections::HashMap;

struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Processor>>,
    _libs: Vec<Library>, // Keeps all libraries alive
}

impl PluginRegistry {
    fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
            _libs: Vec::new(),
        }
    }

    fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let loaded = load_plugin(path)?;
        let name = (loaded.processor.name)();
        let name_str = unsafe { std::ffi::CStr::from_ptr(name).to_str()?.to_owned() };
        let wrapper = FfiProcessor::new(loaded.processor);
        self.plugins.insert(name_str, Box::new(wrapper));
        self._libs.push(loaded._lib);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<&dyn Processor> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
}
```

### Pattern 3: Version Negotiation

To handle plugins compiled at different times, embed version info and negotiate at load time:

```rust
#[repr(C)]
pub struct PluginManifest {
    pub api_version: u32,       // Must match host's expected version
    pub plugin_version: u32,
    pub name: *const std::os::raw::c_char,
}

pub const CURRENT_API_VERSION: u32 = 1;

// Host validates before use:
fn validate_plugin(manifest: &PluginManifest) -> bool {
    manifest.api_version == CURRENT_API_VERSION
}
```

### Pattern 4: Hot Reloading (Development)

During development, you can reload plugins on file change:

```rust
use std::time::SystemTime;
use notify::{Watcher, RecursiveMode, watcher};

struct HotPlugin {
    path: String,
    last_modified: SystemTime,
    plugin: Option<LoadedPlugin>,
}

impl HotPlugin {
    fn check_reload(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.path) {
            if let Ok(modified) = meta.modified() {
                if modified > self.last_modified {
                    println!("Reloading plugin: {}", self.path);
                    // Drop old plugin first (unloads the library)
                    self.plugin = None;
                    self.plugin = load_plugin(&self.path).ok();
                    self.last_modified = modified;
                }
            }
        }
    }
}
```

> **Note:** On some platforms (especially Linux), the old `.so` file may stay mapped until all references are dropped. You may need to copy the compiled library to a unique temp path before loading to allow true hot-swap.

---

## 6. Safety Checklist

When building a Rust plugin system, validate the following:

| Concern | Mitigation |
|---|---|
| Plugin ABI mismatch | Use `#[repr(C)]` on all shared types; embed API version number |
| Library outlives symbols | Keep `Library` in same struct as loaded symbols |
| Panics crossing FFI | Catch panics with `std::panic::catch_unwind` at the plugin boundary |
| Allocator mismatch | Always free memory in the same crate that allocated it |
| Null pointer from plugin | Always check raw pointers before dereferencing |
| Thread safety | Annotate or document which functions are thread-safe |

### Handling Panics Across the FFI Boundary

Panics must not unwind across `extern "C"` boundaries — doing so is undefined behavior. Wrap plugin logic:

```rust
// In the plugin, wrap all exported functions:
use std::panic;

#[no_mangle]
pub extern "C" fn safe_process(samples: *mut f32, count: usize) {
    let result = panic::catch_unwind(|| {
        // plugin logic here
        let buf = unsafe { std::slice::from_raw_parts_mut(samples, count) };
        for s in buf.iter_mut() {
            *s *= 2.0;
        }
    });

    if result.is_err() {
        eprintln!("Plugin panicked! Recovering gracefully.");
    }
}
```

---

## 7. Alternatives to Manual `libloading`

### `abi_stable` Crate

The [`abi_stable`](https://crates.io/crates/abi_stable) crate provides a framework for building Rust plugin systems with stable-ABI Rust types (stable strings, vecs, etc.) without dropping all the way to raw C:

```toml
[dependencies]
abi_stable = "0.11"
```

It offers types like `RString`, `RVec<T>`, `RBox<T>`, `ROption<T>` which have a guaranteed stable layout, and a `#[sabi_trait]` macro for defining stable trait objects.

### `wasmer` / `wasmtime` (WASM Plugins)

For maximum safety and portability, compile plugins to **WebAssembly** instead of native code. This eliminates ABI issues entirely — WASM has a well-defined, stable ABI — and provides sandboxing:

```toml
[dependencies]
wasmtime = "19"
```

WASM plugins are compiled with `crate-type = ["cdylib"]` targeting `wasm32-wasi`, and the host uses a WASM runtime to execute them. This approach is increasingly popular in production systems (e.g., Extism, plugin systems in databases, etc.).

---

## 8. Summary

Building a plugin system in Rust involves several layered concerns:

1. **Compilation:** Plugins are `cdylib` crates that export C-ABI symbols with `#[no_mangle]` and `extern "C"`.
2. **Loading:** `libloading` loads the `.so`/`.dll` and resolves function pointers at runtime.
3. **ABI Stability:** Use `#[repr(C)]` on all shared types; never pass Rust-native heap types across boundaries; embed version numbers.
4. **Safety:** Keep `Library` alive as long as symbols are in use; catch panics at the boundary; free memory in the same allocator that created it.
5. **Ergonomics:** Wrap raw FFI in safe Rust traits on the host side; use a plugin registry pattern for managing multiple plugins.
6. **Alternatives:** Consider `abi_stable` for richer Rust types, or WASM-based plugins for sandboxing and maximum portability.

Rust's plugin story requires more manual discipline than higher-level languages, but rewards careful design with a system that is both extremely performant and — when done correctly — reasonably safe.