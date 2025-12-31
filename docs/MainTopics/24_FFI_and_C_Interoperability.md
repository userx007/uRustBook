# FFI and C Interoperability in Rust

Foreign Function Interface (FFI) in Rust provides the ability to interact with code written in other languages, particularly C. This is crucial for integrating with existing libraries, system APIs, and creating language bindings. Rust's FFI capabilities are built around C ABI (Application Binary Interface) compatibility, making it possible to create seamless interoperability between Rust and C codebases.

## C ABI Compatibility

The C ABI defines how functions are called at the binary level, including how arguments are passed, return values are handled, and how data structures are laid out in memory. Rust can be made ABI-compatible with C using specific attributes and types.

### Extern Functions and Blocks

To call C functions from Rust, you use `extern` blocks:

```rust
// Declaring external C functions
extern "C" {
    fn abs(input: i32) -> i32;
    fn sqrt(input: f64) -> f64;
}

fn main() {
    unsafe {
        println!("Absolute value of -3 is: {}", abs(-3));
        println!("Square root of 16 is: {}", sqrt(16.0));
    }
}
```

The `"C"` calling convention tells Rust to use C's calling convention. All FFI functions are inherently unsafe because Rust cannot verify the safety guarantees of foreign code.

### Exposing Rust Functions to C

To export Rust functions for use by C code, you use `#[no_mangle]` and `extern "C"`:

```rust
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn rust_greet(name: *const c_char) {
    let c_str = unsafe {
        assert!(!name.is_null());
        std::ffi::CStr::from_ptr(name)
    };
    
    if let Ok(name_str) = c_str.to_str() {
        println!("Hello, {}!", name_str);
    }
}
```

The `#[no_mangle]` attribute prevents Rust's name mangling, ensuring the function name remains predictable for C code to link against.

## repr(C) and Data Layout

Rust's default struct layout is undefined and may be optimized. To ensure C-compatible memory layout, use `#[repr(C)]`:

```rust
use std::os::raw::c_int;

// C-compatible struct
#[repr(C)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

// NOT C-compatible (default Rust layout)
pub struct RustPoint {
    pub x: i32,
    pub y: i32,
}

#[no_mangle]
pub extern "C" fn create_point(x: c_int, y: c_int) -> Point {
    Point { x, y }
}

#[no_mangle]
pub extern "C" fn point_distance(p: Point) -> f64 {
    ((p.x * p.x + p.y * p.y) as f64).sqrt()
}
```

## cbindgen: Generating C Headers from Rust

`cbindgen` is a tool that automatically generates C/C++ header files from Rust code. This is essential when you're writing a library in Rust that will be used by C/C++ code.

### Installation and Usage

```bash
cargo install cbindgen
```

Example Rust library (`src/lib.rs`):

```rust
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct Database {
    connection_string: *mut c_char,
    timeout: c_int,
}

#[no_mangle]
pub extern "C" fn db_create(conn_str: *const c_char, timeout: c_int) -> *mut Database {
    let db = Box::new(Database {
        connection_string: std::ptr::null_mut(),
        timeout,
    });
    Box::into_raw(db)
}

#[no_mangle]
pub extern "C" fn db_query(db: *mut Database, query: *const c_char) -> c_int {
    if db.is_null() || query.is_null() {
        return -1;
    }
    // Query logic here
    0
}

#[no_mangle]
pub extern "C" fn db_destroy(db: *mut Database) {
    if !db.is_null() {
        unsafe {
            let _ = Box::from_raw(db);
        }
    }
}
```

Create `cbindgen.toml`:

```toml
language = "C"
include_guard = "MY_LIBRARY_H"
pragma_once = true
```

Run cbindgen:

```bash
cbindgen --config cbindgen.toml --crate my_library --output my_library.h
```

Generated header (`my_library.h`):

```c
#ifndef MY_LIBRARY_H
#define MY_LIBRARY_H

#include <stdint.h>

typedef struct Database {
    char *connection_string;
    int32_t timeout;
} Database;

Database* db_create(const char *conn_str, int32_t timeout);
int32_t db_query(Database *db, const char *query);
void db_destroy(Database *db);

#endif // MY_LIBRARY_H
```

## bindgen: Generating Rust Bindings from C

`bindgen` does the opposite of cbindgen—it generates Rust FFI bindings from C header files. This is useful when you want to call existing C libraries from Rust.

### Basic bindgen Usage

Given a C header file (`math_ops.h`):

```c
#ifndef MATH_OPS_H
#define MATH_OPS_H

typedef struct {
    double x;
    double y;
} Vector2D;

Vector2D vector_add(Vector2D a, Vector2D b);
double vector_magnitude(Vector2D v);
void vector_normalize(Vector2D* v);

#endif
```

Create a build script (`build.rs`):

```rust
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=math_ops.h");
    println!("cargo:rustc-link-lib=mathops");
    
    let bindings = bindgen::Builder::default()
        .header("math_ops.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

In your Rust code:

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let v1 = Vector2D { x: 3.0, y: 4.0 };
    let v2 = Vector2D { x: 1.0, y: 2.0 };
    
    unsafe {
        let result = vector_add(v1, v2);
        println!("Result: ({}, {})", result.x, result.y);
        
        let mag = vector_magnitude(v1);
        println!("Magnitude: {}", mag);
    }
}
```

## Calling Conventions

Different platforms and compilers use different calling conventions for how function arguments are passed and cleaned up.

### Common Calling Conventions

```rust
// Standard C calling convention (most portable)
extern "C" fn c_convention(x: i32) -> i32 { x }

// Windows-specific conventions
#[cfg(windows)]
extern "stdcall" fn win_stdcall(x: i32) -> i32 { x }

#[cfg(windows)]
extern "system" fn win_system(x: i32) -> i32 { x }

// Rust's native calling convention (default)
extern "Rust" fn rust_convention(x: i32) -> i32 { x }
```

The `"C"` calling convention is the most common and portable. On Windows, you might encounter `"stdcall"` for Win32 API functions. The `"system"` convention adapts to the platform's preferred convention.

## Cross-Language Safety Considerations

FFI introduces significant safety challenges because you're crossing the boundary between Rust's strict safety guarantees and potentially unsafe C code.

### Safe Wrapper Pattern

Always wrap unsafe FFI calls in safe Rust interfaces:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

extern "C" {
    fn c_strlen(s: *const c_char) -> usize;
    fn c_malloc(size: usize) -> *mut u8;
    fn c_free(ptr: *mut u8);
}

// Unsafe, direct FFI
pub unsafe fn strlen_unsafe(s: *const c_char) -> usize {
    c_strlen(s)
}

// Safe wrapper
pub fn strlen_safe(s: &str) -> usize {
    let c_string = CString::new(s).expect("CString conversion failed");
    unsafe { c_strlen(c_string.as_ptr()) }
}

// RAII wrapper for memory management
pub struct CBuffer {
    ptr: *mut u8,
    size: usize,
}

impl CBuffer {
    pub fn new(size: usize) -> Option<Self> {
        let ptr = unsafe { c_malloc(size) };
        if ptr.is_null() {
            None
        } else {
            Some(CBuffer { ptr, size })
        }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
}

impl Drop for CBuffer {
    fn drop(&mut self) {
        unsafe {
            c_free(self.ptr);
        }
    }
}
```

### Handling Strings Across FFI

String handling is particularly tricky because C uses null-terminated strings while Rust uses length-encoded UTF-8:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Passing Rust string to C
pub fn rust_to_c_string(rust_str: &str) -> *mut c_char {
    let c_string = CString::new(rust_str).expect("CString conversion failed");
    c_string.into_raw() // Transfer ownership to C
}

// Receiving string from C (non-owning)
pub unsafe fn c_to_rust_str<'a>(c_str: *const c_char) -> Option<&'a str> {
    if c_str.is_null() {
        return None;
    }
    CStr::from_ptr(c_str).to_str().ok()
}

// Receiving string from C (owning)
pub unsafe fn c_to_rust_string(c_str: *mut c_char) -> Option<String> {
    if c_str.is_null() {
        return None;
    }
    let c_str = CString::from_raw(c_str);
    c_str.to_str().ok().map(|s| s.to_owned())
}

// Example usage
#[no_mangle]
pub extern "C" fn process_string(input: *const c_char) -> *mut c_char {
    let input_str = unsafe {
        if input.is_null() {
            return std::ptr::null_mut();
        }
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    let output = format!("Processed: {}", input_str);
    rust_to_c_string(&output)
}
```

### Opaque Types and Handles

Use opaque pointers to hide Rust types from C:

```rust
pub struct InternalState {
    data: Vec<u8>,
    count: usize,
}

// Opaque type for C
#[repr(C)]
pub struct OpaqueHandle {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn handle_create() -> *mut OpaqueHandle {
    let state = Box::new(InternalState {
        data: Vec::new(),
        count: 0,
    });
    Box::into_raw(state) as *mut OpaqueHandle
}

#[no_mangle]
pub extern "C" fn handle_process(handle: *mut OpaqueHandle, value: u8) {
    if handle.is_null() {
        return;
    }
    
    let state = unsafe { &mut *(handle as *mut InternalState) };
    state.data.push(value);
    state.count += 1;
}

#[no_mangle]
pub extern "C" fn handle_destroy(handle: *mut OpaqueHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut InternalState);
        }
    }
}
```

## Complete Example: Building a C-Compatible Library

Here's a complete example of a Rust library that can be called from C:

```rust
// lib.rs
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

#[repr(C)]
pub struct ImageBuffer {
    width: u32,
    height: u32,
    data: *mut u8,
}

#[no_mangle]
pub extern "C" fn image_create(width: u32, height: u32) -> *mut ImageBuffer {
    let size = (width * height * 4) as usize; // RGBA
    let mut data = vec![0u8; size];
    
    let buffer = Box::new(ImageBuffer {
        width,
        height,
        data: data.as_mut_ptr(),
    });
    
    std::mem::forget(data); // Prevent Rust from dropping the vector
    Box::into_raw(buffer)
}

#[no_mangle]
pub extern "C" fn image_set_pixel(
    img: *mut ImageBuffer,
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> c_int {
    if img.is_null() {
        return -1;
    }
    
    let img = unsafe { &mut *img };
    
    if x >= img.width || y >= img.height {
        return -1;
    }
    
    let offset = ((y * img.width + x) * 4) as isize;
    unsafe {
        *img.data.offset(offset) = r;
        *img.data.offset(offset + 1) = g;
        *img.data.offset(offset + 2) = b;
        *img.data.offset(offset + 3) = a;
    }
    
    0
}

#[no_mangle]
pub extern "C" fn image_save(img: *const ImageBuffer, path: *const c_char) -> c_int {
    if img.is_null() || path.is_null() {
        return -1;
    }
    
    let img = unsafe { &*img };
    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };
    
    // Image saving logic here
    println!("Saving image to: {}", path_str);
    0
}

#[no_mangle]
pub extern "C" fn image_destroy(img: *mut ImageBuffer) {
    if img.is_null() {
        return;
    }
    
    unsafe {
        let img_box = Box::from_raw(img);
        let size = (img_box.width * img_box.height * 4) as usize;
        let _ = Vec::from_raw_parts(img_box.data, size, size);
    }
}
```

This example demonstrates proper memory management, null checks, error handling, and safe wrapping of potentially unsafe operations. FFI in Rust requires careful attention to safety, but when done correctly, it provides powerful interoperability with C while maintaining Rust's safety guarantees at the boundaries.