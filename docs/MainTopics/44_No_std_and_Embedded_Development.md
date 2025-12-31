# Rust's no-std and embedded development

**Key Topics:**

1. **No-std Fundamentals** - Understanding the `core`/`alloc`/`std` hierarchy and why we need no-std

2. **Panic Handlers** - Required in no-std environments since there's no runtime to handle panics. I show both basic and debugging-enabled handlers

3. **Custom Allocators** - Including a simple bump allocator implementation and how to use external allocators like `linked_list_allocator` to enable heap allocation

4. **Embedded Constraints** - Real-world challenges like:
   - Working without dynamic allocation
   - Volatile memory access for hardware registers
   - Interrupt-safe shared state
   - Static lifetime requirements

5. **Complete Examples** - Including a full ARM Cortex-M application with peripheral access and memory layout configuration

The guide emphasizes practical patterns you'll actually use in embedded development, like using `heapless` for fixed-size collections, safe interrupt handling with `Mutex<RefCell<T>>`, and proper volatile register access.

**Why No-std Matters:**
- Embedded systems often lack an OS or filesystem
- Memory is severely constrained (sometimes just 20KB RAM)
- Every byte and cycle counts
- Direct hardware access is necessary

# Rust No-std and Embedded Development

## Overview

No-std Rust development refers to writing Rust code without the standard library (`std`). This is essential for embedded systems, operating system kernels, bootloaders, and other resource-constrained environments where the full standard library isn't available or practical.

## Core Concepts

### The Standard Library Hierarchy

Rust has three library layers:

1. **`core`**: Platform-agnostic, no heap allocation, no I/O
2. **`alloc`**: Heap allocation primitives (Vec, Box, String, etc.)
3. **`std`**: Full standard library (files, networking, threads, etc.)

In no-std environments, you typically use `core` and optionally `alloc` if you have a heap allocator.

## Setting Up a No-std Project

### Basic No-std Configuration

```toml
# Cargo.toml
[package]
name = "embedded-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core is implicit, no need to add it

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
```

```rust
// lib.rs or main.rs
#![no_std]
#![no_main]  // For embedded executables

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

## Panic Handlers

In no-std environments, you must provide your own panic handler. The standard library normally provides this.

### Basic Panic Handler

```rust
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In embedded systems, you might:
    // - Halt the processor
    // - Reset the device
    // - Flash an LED
    // - Write to a debug port
    loop {
        // Infinite loop prevents returning
    }
}
```

### Advanced Panic Handler with Debugging

```rust
use core::panic::PanicInfo;
use core::fmt::Write;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Assuming you have a serial output
    if let Some(mut serial) = unsafe { SERIAL_PORT.as_mut() } {
        let _ = writeln!(serial, "PANIC: {}", info);
        
        if let Some(location) = info.location() {
            let _ = writeln!(
                serial,
                "at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
    }
    
    loop {
        // Halt or reset
        unsafe {
            core::arch::asm!("wfi"); // Wait for interrupt (ARM)
        }
    }
}

static mut SERIAL_PORT: Option<SerialPort> = None;
```

## Custom Allocators

When using `alloc` without `std`, you need to provide a global allocator.

### Simple Bump Allocator

```rust
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

struct BumpAllocator {
    heap: UnsafeCell<[u8; 64 * 1024]>, // 64KB heap
    next: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    const fn new() -> Self {
        BumpAllocator {
            heap: UnsafeCell::new([0; 64 * 1024]),
            next: UnsafeCell::new(0),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let next = self.next.get();
        let mut offset = *next;
        
        // Align the offset
        let align = layout.align();
        let remainder = offset % align;
        if remainder != 0 {
            offset += align - remainder;
        }
        
        let new_next = offset + layout.size();
        
        if new_next > 64 * 1024 {
            return core::ptr::null_mut(); // Out of memory
        }
        
        *next = new_next;
        self.heap.get().cast::<u8>().add(offset)
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't deallocate
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation failed: {:?}", layout);
}
```

### Using an External Allocator (linked_list_allocator)

```toml
[dependencies]
linked_list_allocator = "0.10"
```

```rust
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 100 * 1024; // 100KB
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

// Now you can use alloc types!
use alloc::vec::Vec;
use alloc::string::String;

fn use_heap() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    let s = String::from("Hello, embedded world!");
}
```

## Embedded Development Example (ARM Cortex-M)

### Complete Embedded Application

```rust
#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _; // Panic handler

// Entry point
#[entry]
fn main() -> ! {
    // Initialize peripherals
    let peripherals = unsafe { 
        cortex_m::Peripherals::steal() 
    };
    
    // Configure systick for delays
    let mut syst = peripherals.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(8_000_000); // 1 second at 8MHz
    syst.enable_counter();
    
    loop {
        // Wait for systick
        while !syst.has_wrapped() {}
        
        // Toggle LED or do work
        toggle_led();
    }
}

fn toggle_led() {
    // Platform-specific LED toggle
    unsafe {
        let gpio_base = 0x4002_0000 as *mut u32;
        let odr = gpio_base.add(5);
        *odr ^= 1 << 13; // Toggle pin 13
    }
}
```

### Memory Layout (memory.x)

```text
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM   : ORIGIN = 0x20000000, LENGTH = 20K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

## Working with Core Types

### No-std Data Structures

```rust
#![no_std]

use core::cell::Cell;
use core::ptr;

// Using core types instead of std types
fn core_examples() {
    // Cell for interior mutability
    let value = Cell::new(5);
    value.set(10);
    
    // Raw pointers
    let x = 42;
    let ptr = &x as *const i32;
    
    unsafe {
        assert_eq!(*ptr, 42);
    }
    
    // Fixed-size arrays (no Vec without alloc)
    let mut buffer: [u8; 128] = [0; 128];
    buffer[0] = 0xFF;
}

// Option and Result work in no-std
fn error_handling() -> Result<u32, &'static str> {
    let value = Some(42);
    
    match value {
        Some(v) => Ok(v),
        None => Err("No value"),
    }
}
```

## Embedded Constraints

### 1. No Heap Allocation (without custom allocator)

```rust
// Instead of Vec<T>
use heapless::Vec;

let mut buffer: Vec<u8, 32> = Vec::new(); // Max 32 elements
buffer.push(1).unwrap();

// Instead of String
use heapless::String;

let mut s: String<64> = String::new();
s.push_str("Hello").unwrap();
```

### 2. Volatile Memory Access

```rust
use core::ptr::{read_volatile, write_volatile};

// Reading from memory-mapped registers
fn read_register(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

fn write_register(addr: usize, value: u32) {
    unsafe { write_volatile(addr as *mut u32, value) }
}

// Using volatile wrapper
use core::cell::UnsafeCell;

#[repr(C)]
struct Register {
    value: UnsafeCell<u32>,
}

impl Register {
    fn read(&self) -> u32 {
        unsafe { read_volatile(self.value.get()) }
    }
    
    fn write(&self, value: u32) {
        unsafe { write_volatile(self.value.get(), value) }
    }
}
```

### 3. Interrupt Handling

```rust
use cortex_m::interrupt;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

// Shared state between interrupt and main
static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIM2() {
    // Access shared state safely
    interrupt::free(|cs| {
        let mut counter = COUNTER.borrow(cs).borrow_mut();
        *counter += 1;
    });
}

fn main_loop() {
    loop {
        let count = interrupt::free(|cs| {
            *COUNTER.borrow(cs).borrow()
        });
        
        // Use count...
    }
}
```

### 4. Static Lifetimes and No Dynamic Dispatch

```rust
// All data must have static lifetime or be stack-allocated
static LED_STATE: core::sync::atomic::AtomicBool = 
    core::sync::atomic::AtomicBool::new(false);

// No trait objects (dyn Trait) in most embedded code
// Use generics instead
fn process<T: Sensor>(sensor: &T) {
    let value = sensor.read();
    // ...
}

trait Sensor {
    fn read(&self) -> u32;
}
```

## Conditional Compilation

```rust
// Use different implementations for different targets
#[cfg(not(target_arch = "arm"))]
fn platform_init() {
    // Stub for testing on PC
}

#[cfg(target_arch = "arm")]
fn platform_init() {
    // Real ARM initialization
}

// Optional std support
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use heapless::Vec;
```

## Testing No-std Code

```rust
#![cfg_attr(not(test), no_std)]

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
```

## Best Practices

1. **Minimize unsafe code**: Use abstractions when possible
2. **Use const generics**: For compile-time sized buffers
3. **Leverage the type system**: Make invalid states unrepresentable
4. **Profile memory usage**: Every byte counts in embedded systems
5. **Test on host**: Use conditional compilation for unit tests
6. **Document safety invariants**: Explain all unsafe code
7. **Use PACs and HALs**: Peripheral Access Crates and Hardware Abstraction Layers
8. **Handle errors explicitly**: No unwinding in no-std environments

## Common Pitfalls

- Forgetting to initialize the heap before using `alloc` types
- Not providing a panic handler
- Assuming zero-cost abstractions (always measure)
- Using floating point without checking hardware support
- Stack overflow from large stack allocations
- Not considering alignment requirements
- Race conditions in interrupt handlers

## Resources

- **Embedded Rust Book**: https://docs.rust-embedded.org/book/
- **cortex-m crate**: ARM Cortex-M support
- **embedded-hal**: Hardware abstraction traits
- **probe-run**: Modern embedded debugging
- **defmt**: Efficient logging for embedded