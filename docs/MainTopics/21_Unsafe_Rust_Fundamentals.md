# Unsafe Rust Fundamentals

## What is Unsafe Rust?

Rust's core promise is memory safety without garbage collection, enforced at compile time. However, there are situations where the compiler's conservative checks prevent certain valid operations, or where you need to interface with hardware, other languages, or perform low-level optimizations. **Unsafe Rust** provides an escape hatch that lets you bypass some of Rust's safety guarantees while still leveraging the rest of the language.

**Important:** Unsafe doesn't turn off the borrow checker or disable all safety checks—it simply allows you to perform operations that the compiler cannot verify as safe.

---

## The Five Unsafe Superpowers

When you enter an `unsafe` block or function, you gain access to five additional capabilities:

1. **Dereference raw pointers**
2. **Call unsafe functions or methods**
3. **Access or modify mutable static variables**
4. **Implement unsafe traits**
5. **Access fields of unions**

---

## 1. Unsafe Blocks

An `unsafe` block tells the compiler "I know what I'm doing here, trust me."

```rust
fn main() {
    let mut num = 5;

    // Creating raw pointers is safe
    let r1 = &num as *const i32;
    let r2 = &mut num as *mut i32;

    // Dereferencing raw pointers requires unsafe
    unsafe {
        println!("r1 is: {}", *r1);
        println!("r2 is: {}", *r2);
    }
}
```

**Key Points:**
- Keep `unsafe` blocks as small as possible
- Encapsulate unsafe code in safe abstractions
- Document why the code is safe

---

## 2. Unsafe Functions

Functions that contain unsafe operations or require certain invariants can be marked as `unsafe`:

```rust
unsafe fn dangerous() {
    // This function can only be called from unsafe blocks
    println!("Doing dangerous things!");
}

fn main() {
    unsafe {
        dangerous();
    }
}
```

**Example: Split at Mutable**

Here's a simplified version of how you might implement splitting a slice:

```rust
use std::slice;

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    assert!(mid <= len);

    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

fn main() {
    let mut vector = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = split_at_mut(&mut vector, 3);
    
    println!("Left: {:?}", left);   // [1, 2, 3]
    println!("Right: {:?}", right); // [4, 5, 6]
}
```

This is safe because we ensure the two slices don't overlap.

---

## 3. Raw Pointers

Raw pointers (`*const T` and `*mut T`) are similar to references but with fewer guarantees:

```rust
fn main() {
    let mut num = 5;

    // Immutable raw pointer
    let r1 = &num as *const i32;
    
    // Mutable raw pointer
    let r2 = &mut num as *mut i32;

    // Can create raw pointers to arbitrary memory (dangerous!)
    let address = 0x012345usize;
    let r3 = address as *const i32;

    unsafe {
        println!("r1 is: {}", *r1);
        
        // Modifying through mutable raw pointer
        *r2 = 10;
        println!("num is now: {}", num);
        
        // Dereferencing arbitrary memory - will likely crash!
        // println!("r3 is: {}", *r3);
    }
}
```

**Differences from References:**
- Can be null
- Can ignore borrowing rules
- Not guaranteed to point to valid memory
- No automatic cleanup
- Can create both immutable and mutable pointers to the same location

---

## 4. Accessing/Modifying Mutable Static Variables

Static variables have a fixed address in memory. Accessing mutable statics is unsafe because it could create data races:

```rust
static HELLO_WORLD: &str = "Hello, world!";

static mut COUNTER: u32 = 0;

fn add_to_count(inc: u32) {
    unsafe {
        COUNTER += inc;
    }
}

fn main() {
    // Safe static access
    println!("name is: {}", HELLO_WORLD);

    // Unsafe mutable static access
    add_to_count(3);

    unsafe {
        println!("COUNTER: {}", COUNTER);
    }
}
```

**Better Alternative:** Use thread-safe types like `AtomicU32` or `Mutex`:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn add_to_count(inc: u32) {
    COUNTER.fetch_add(inc, Ordering::SeqCst);
}

fn main() {
    add_to_count(3);
    println!("COUNTER: {}", COUNTER.load(Ordering::SeqCst));
}
```

---

## 5. Implementing Unsafe Traits

Some traits are marked `unsafe` because implementing them incorrectly could cause undefined behavior:

```rust
unsafe trait Foo {
    // methods go here
}

unsafe impl Foo for i32 {
    // implementation goes here
}
```

**Example: Send and Sync**

The `Send` and `Sync` marker traits are unsafe:

```rust
use std::marker::PhantomData;

// A type that contains a raw pointer
struct MyBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

// We must manually implement Send if we want to transfer between threads
unsafe impl<T: Send> Send for MyBox<T> {}

// We must manually implement Sync if we want to share references between threads
unsafe impl<T: Sync> Sync for MyBox<T> {}
```

---

## 6. Accessing Union Fields

Unions allow you to store different types in the same memory location, but accessing them is unsafe:

```rust
union MyUnion {
    f1: u32,
    f2: f32,
}

fn main() {
    let u = MyUnion { f1: 1 };
    
    unsafe {
        let f = u.f1;
        println!("u.f1: {}", f);
        
        // Reading f2 is reading the bytes of f1 as if they were f32
        let f = u.f2;
        println!("u.f2: {}", f); // Likely garbage!
    }
}
```

---

## Foreign Function Interface (FFI)

A common use of unsafe is calling functions from other languages:

```rust
extern "C" {
    fn abs(input: i32) -> i32;
}

fn main() {
    unsafe {
        println!("Absolute value of -3 according to C: {}", abs(-3));
    }
}
```

**Calling Rust from C:**

```rust
#[no_mangle]
pub extern "C" fn call_from_c() {
    println!("Just called a Rust function from C!");
}
```

---

## Real-World Example: Manual Memory Management

Here's an example that demonstrates multiple unsafe superpowers:

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> MyVec<T> {
    fn new() -> Self {
        MyVec {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };
        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                let ptr = alloc(new_layout) as *mut T;
                ptr::copy_nonoverlapping(self.ptr, ptr, self.len);
                dealloc(self.ptr as *mut u8, old_layout);
                ptr
            }
        };

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                for i in 0..self.len {
                    ptr::drop_in_place(self.ptr.add(i));
                }
                let layout = Layout::array::<T>(self.capacity).unwrap();
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

fn main() {
    let mut v = MyVec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    println!("First element: {:?}", v.get(0));
    println!("Second element: {:?}", v.get(1));
}
```

---

## Best Practices

1. **Minimize Unsafe Code:** Keep unsafe blocks as small as possible
2. **Encapsulate:** Wrap unsafe code in safe abstractions
3. **Document Invariants:** Explain why the code is safe
4. **Test Thoroughly:** Unsafe code can cause undefined behavior
5. **Use Tools:** `miri` can detect some undefined behavior
6. **Prefer Safe Abstractions:** Use existing safe libraries when possible

```rust
// Good: Small unsafe block with clear purpose
fn safe_wrapper(values: &[i32], index: usize) -> Option<i32> {
    if index < values.len() {
        unsafe {
            Some(*values.as_ptr().add(index))
        }
    } else {
        None
    }
}
```

---

## When to Use Unsafe

**Valid reasons:**
- Interfacing with C libraries (FFI)
- Building fundamental data structures
- Performance-critical code after profiling
- Implementing low-level abstractions
- Hardware/OS interaction

**Invalid reasons:**
- "The borrow checker is annoying"
- "I need it to compile"
- "I don't understand the error"

Remember: **Unsafe Rust is about taking responsibility for upholding invariants that the compiler cannot verify.** It's a powerful tool, but with great power comes great responsibility!