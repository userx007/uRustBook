# Memory Layout and Representations in Rust

## Stack vs Heap

### Stack Memory
The stack stores data with a known, fixed size at compile time. It's fast, automatically managed, and follows LIFO (Last In, First Out) order.

```rust
fn stack_example() {
    let x = 5;           // stored on stack
    let y = [1, 2, 3];   // array on stack
    let z = (10, 20);    // tuple on stack
    
    // When function exits, all these are automatically cleaned up
}
```

**Characteristics:**
- Fast allocation/deallocation
- Fixed size known at compile time
- Automatic cleanup (no need for manual memory management)
- Limited size (typically a few MB)

### Heap Memory
The heap stores data with dynamic or unknown size at compile time. Accessed through pointers, it's slower but more flexible.

```rust
fn heap_example() {
    let s = String::from("hello");  // String data on heap
    let v = vec![1, 2, 3, 4, 5];    // Vec data on heap
    let b = Box::new(42);            // Boxed value on heap
    
    // Stack stores: pointer, length, capacity
    // Heap stores: actual string/vector data
}
```

**Characteristics:**
- Dynamic size
- Slower allocation/deallocation
- Manual ownership tracking (handled by Rust's ownership system)
- Much larger capacity

### Practical Example

```rust
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn demonstrate_stack_heap() {
    // Stack: fixed size, known at compile time
    let point_stack = Point { x: 1.0, y: 2.0 };
    
    // Heap: allows dynamic sizing and sharing
    let point_heap = Box::new(Point { x: 3.0, y: 4.0 });
    
    // Vec grows dynamically - data on heap
    let mut points = Vec::new();
    points.push(Point { x: 5.0, y: 6.0 });
    points.push(Point { x: 7.0, y: 8.0 });
    
    println!("Stack: {:?}", point_stack);
    println!("Heap: {:?}", point_heap);
    println!("Vec: {:?}", points);
}
```

---

## Alignment and Padding

### Alignment
Alignment ensures data is stored at memory addresses that are multiples of the type's alignment requirement. This optimizes CPU access.

```rust
use std::mem::{size_of, align_of};

fn alignment_examples() {
    // Alignment requirements
    println!("u8 alignment: {}", align_of::<u8>());     // 1
    println!("u16 alignment: {}", align_of::<u16>());   // 2
    println!("u32 alignment: {}", align_of::<u32>());   // 4
    println!("u64 alignment: {}", align_of::<u64>());   // 8
    
    // Struct alignment is max of field alignments
    struct Example {
        a: u8,
        b: u32,
    }
    println!("Example alignment: {}", align_of::<Example>()); // 4
}
```

### Padding
Rust adds padding bytes to ensure proper alignment, which can increase struct size.

```rust
use std::mem::size_of;

// Without optimization
struct Unoptimized {
    a: u8,   // 1 byte
    // 3 bytes padding
    b: u32,  // 4 bytes
    c: u8,   // 1 byte
    // 3 bytes padding
}
// Total: 12 bytes

// Optimized by reordering
struct Optimized {
    b: u32,  // 4 bytes
    a: u8,   // 1 byte
    c: u8,   // 1 byte
    // 2 bytes padding
}
// Total: 8 bytes

fn padding_example() {
    println!("Unoptimized size: {}", size_of::<Unoptimized>()); // 12
    println!("Optimized size: {}", size_of::<Optimized>());     // 8
    
    // Visualizing memory layout
    struct Visual {
        x: u8,   // [0]
        // padding: [1, 2, 3]
        y: u32,  // [4, 5, 6, 7]
        z: u16,  // [8, 9]
        // padding: [10, 11]
    }
    println!("Visual size: {}", size_of::<Visual>()); // 12
}
```

---

## repr(C)

`repr(C)` guarantees C-compatible memory layout, essential for FFI (Foreign Function Interface).

```rust
// Default Rust layout - no guarantees about field order
#[derive(Debug)]
struct RustLayout {
    a: u8,
    b: u32,
    c: u16,
}

// C-compatible layout - fields in declaration order
#[repr(C)]
#[derive(Debug)]
struct CLayout {
    a: u8,
    b: u32,
    c: u16,
}

// For FFI with C
#[repr(C)]
struct FFIStruct {
    count: i32,
    data: *const u8,
}

// C enums
#[repr(C)]
enum CEnum {
    A,
    B = 100,
    C,
}

// Practical FFI example
#[repr(C)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

extern "C" {
    // Hypothetical C function
    // void process_point(Point3D* p);
}

fn repr_c_example() {
    use std::mem::size_of;
    
    println!("RustLayout size: {}", size_of::<RustLayout>());
    println!("CLayout size: {}", size_of::<CLayout>());
    
    // Both might be same size, but CLayout guarantees field order
    // Rust can reorder RustLayout fields for optimization
}
```

**When to use `repr(C)`:**
- Interfacing with C code
- Reading/writing binary formats
- Memory-mapped I/O
- Ensuring specific field ordering

---

## repr(transparent)

`repr(transparent)` makes a wrapper type have the same memory layout as its single field.

```rust
use std::mem::size_of;

// Wrapper around u32 with same layout
#[repr(transparent)]
struct UserId(u32);

// Multiple fields - ERROR!
// #[repr(transparent)]
// struct Invalid(u32, u32); // Won't compile

// With zero-sized types is OK
#[repr(transparent)]
struct ValidWrapper<T> {
    value: T,
    _marker: std::marker::PhantomData<()>,
}

fn repr_transparent_example() {
    println!("u32 size: {}", size_of::<u32>());         // 4
    println!("UserId size: {}", size_of::<UserId>());   // 4
    
    // Can safely transmute
    let user_id = UserId(42);
    let raw: u32 = unsafe { std::mem::transmute(user_id) };
    println!("Raw value: {}", raw); // 42
}

// Practical use case: type-safe FFI
#[repr(transparent)]
struct FileDescriptor(i32);

extern "C" {
    // fn close(fd: FileDescriptor) -> i32;
}

// Another example: newtype pattern with guaranteed layout
#[repr(transparent)]
struct Seconds(f64);

#[repr(transparent)]
struct Meters(f64);
```

**When to use `repr(transparent)`:**
- Type-safe wrappers for FFI
- Zero-cost abstractions
- Newtype pattern with layout guarantees

---

## repr(packed)

`repr(packed)` removes all padding, making fields tightly packed. **Use with caution!**

```rust
use std::mem::{size_of, align_of};

// Normal struct with padding
struct Normal {
    a: u8,
    b: u32,
    c: u8,
}

// Packed - no padding
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,
    c: u8,
}

fn repr_packed_example() {
    println!("Normal size: {}", size_of::<Normal>());  // 12
    println!("Normal align: {}", align_of::<Normal>()); // 4
    
    println!("Packed size: {}", size_of::<Packed>());  // 6
    println!("Packed align: {}", align_of::<Packed>()); // 1
}

// Dangers of packed
#[repr(packed)]
struct Dangerous {
    a: u8,
    b: u32,  // Misaligned!
}

fn packed_dangers() {
    let d = Dangerous { a: 1, b: 2 };
    
    // Direct field access is fine
    let a = d.a;
    
    // Taking reference to misaligned field - UNSAFE!
    // let b_ref = &d.b; // This creates UB if dereferenced
    
    // Safe way - copy the value
    let b = d.b;
    println!("a: {}, b: {}", a, b);
}

// Specify custom alignment
#[repr(packed(2))]  // Align to 2 bytes
struct PackedCustom {
    a: u8,
    b: u32,
}
```

**Dangers:**
- Misaligned access is undefined behavior on some platforms
- Performance penalties
- Cannot safely borrow packed fields

**When to use `repr(packed)`:**
- Binary file formats with no padding
- Network protocols
- Hardware registers
- When you absolutely must match specific layout

---

## Memory Layout Guarantees

### Default Rust Layout (`repr(Rust)`)

```rust
// Rust can reorder these fields for optimization
struct DefaultLayout {
    small: u8,
    big: u64,
    medium: u32,
}

// Might actually be laid out as:
// [big: u64][medium: u32][small: u8][padding]
// for better packing
```

**Guarantees:**
- Undefined field order
- May be reordered for optimization
- No ABI stability

### Layout Comparison

```rust
use std::mem::{size_of, align_of};

struct Data {
    a: u8,
    b: u32,
    c: u16,
}

#[repr(C)]
struct DataC {
    a: u8,
    b: u32,
    c: u16,
}

#[repr(packed)]
struct DataPacked {
    a: u8,
    b: u32,
    c: u16,
}

fn layout_comparison() {
    println!("Default:");
    println!("  size: {}, align: {}", 
        size_of::<Data>(), align_of::<Data>());
    
    println!("repr(C):");
    println!("  size: {}, align: {}", 
        size_of::<DataC>(), align_of::<DataC>());
    
    println!("repr(packed):");
    println!("  size: {}, align: {}", 
        size_of::<DataPacked>(), align_of::<DataPacked>());
}
```

### Practical Complete Example

```rust
use std::mem::{size_of, align_of};

// Network packet - needs exact layout
#[repr(C, packed)]
struct PacketHeader {
    version: u8,
    length: u16,
    flags: u8,
}

// FFI type - C-compatible but aligned
#[repr(C)]
struct FFIData {
    id: u32,
    name_ptr: *const u8,
    name_len: usize,
}

// Type-safe wrapper
#[repr(transparent)]
struct Handle(usize);

// Optimized Rust struct
struct GameEntity {
    // Compiler can reorder these
    active: bool,      // 1 byte
    health: f32,       // 4 bytes
    id: u64,          // 8 bytes
    position: [f32; 3], // 12 bytes
}

fn complete_example() {
    println!("PacketHeader: {} bytes, {} align", 
        size_of::<PacketHeader>(), 
        align_of::<PacketHeader>());
    
    println!("FFIData: {} bytes, {} align", 
        size_of::<FFIData>(), 
        align_of::<FFIData>());
    
    println!("Handle: {} bytes, {} align", 
        size_of::<Handle>(), 
        align_of::<Handle>());
    
    println!("GameEntity: {} bytes, {} align", 
        size_of::<GameEntity>(), 
        align_of::<GameEntity>());
}

fn main() {
    complete_example();
}
```

## Key Takeaways

1. **Stack vs Heap**: Use stack for fixed-size data, heap for dynamic data
2. **Alignment**: CPU performance optimization, causes padding
3. **Padding**: Trade memory for alignment, reorder fields to minimize
4. **`repr(C)`**: Use for FFI and guaranteed field order
5. **`repr(transparent)`**: Zero-cost type-safe wrappers
6. **`repr(packed)`**: Use sparingly, beware of misalignment
7. **Default Rust**: Optimized but no layout guarantees

Understanding these concepts is crucial for systems programming, FFI, and performance-critical code in Rust.