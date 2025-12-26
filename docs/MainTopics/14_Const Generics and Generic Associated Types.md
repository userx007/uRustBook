# Const Generics and Generic Associated Types (GATs) in Rust

## Const Generics

**Const generics** allow you to parameterize types and functions over constant values (like integers) rather than just types. This enables compile-time computations and type-safe abstractions based on values.

### Basic Const Generics

Before const generics, working with fixed-size arrays was cumbersome:

```rust
// Old approach - needed separate implementations for each size
fn print_array_3(arr: [i32; 3]) { /* ... */ }
fn print_array_5(arr: [i32; 5]) { /* ... */ }

// With const generics - one implementation for all sizes!
fn print_array<T: std::fmt::Display, const N: usize>(arr: [T; N]) {
    for item in arr.iter() {
        println!("{}", item);
    }
}

fn main() {
    print_array([1, 2, 3]);
    print_array([1, 2, 3, 4, 5]);
    print_array(["hello", "world"]);
}
```

### Practical Example: Fixed-Size Buffer

```rust
use std::fmt;

// A buffer with compile-time known capacity
struct Buffer<T, const CAPACITY: usize> {
    data: [Option<T>; CAPACITY],
    len: usize,
}

impl<T, const CAPACITY: usize> Buffer<T, CAPACITY> {
    fn new() -> Self {
        Self {
            data: [(); CAPACITY].map(|_| None),
            len: 0,
        }
    }
    
    fn push(&mut self, item: T) -> Result<(), T> {
        if self.len < CAPACITY {
            self.data[self.len] = Some(item);
            self.len += 1;
            Ok(())
        } else {
            Err(item) // Buffer full
        }
    }
    
    fn len(&self) -> usize {
        self.len
    }
    
    fn capacity(&self) -> usize {
        CAPACITY
    }
}

fn main() {
    let mut small_buf: Buffer<i32, 3> = Buffer::new();
    small_buf.push(10).unwrap();
    small_buf.push(20).unwrap();
    
    let mut large_buf: Buffer<String, 100> = Buffer::new();
    large_buf.push("Hello".to_string()).unwrap();
    
    println!("Small buffer: {}/{}", small_buf.len(), small_buf.capacity());
    println!("Large buffer: {}/{}", large_buf.len(), large_buf.capacity());
}
```

### Matrix Example with Const Generics

```rust
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    fn new() -> Self {
        Self {
            data: [[T::default(); COLS]; ROWS],
        }
    }
}

// Matrix addition - dimensions must match
impl<T: Add<Output = T> + Copy, const ROWS: usize, const COLS: usize> 
    Add for Matrix<T, ROWS, COLS> 
{
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        let mut result = self;
        for i in 0..ROWS {
            for j in 0..COLS {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}

// Matrix multiplication - inner dimensions must match
impl<T, const M: usize, const N: usize, const P: usize> 
    Matrix<T, M, N>
where
    T: Default + Copy + Mul<Output = T> + Add<Output = T>,
{
    fn matmul(self, other: Matrix<T, N, P>) -> Matrix<T, M, P> {
        let mut result = Matrix::new();
        for i in 0..M {
            for j in 0..P {
                let mut sum = T::default();
                for k in 0..N {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }
}

fn main() {
    let m1: Matrix<i32, 2, 3> = Matrix {
        data: [[1, 2, 3], [4, 5, 6]],
    };
    
    let m2: Matrix<i32, 3, 2> = Matrix {
        data: [[7, 8], [9, 10], [11, 12]],
    };
    
    // Type-safe: the compiler ensures dimensions are compatible
    let result = m1.matmul(m2); // Results in Matrix<i32, 2, 2>
    
    // This would not compile:
    // let bad = m2.matmul(m1); // Error: dimensions don't match
}
```

---

## Generic Associated Types (GATs)

**GATs** allow associated types in traits to have generic parameters. This enables more flexible and expressive trait designs, particularly for types that need to be generic over lifetimes or other type parameters.

### Basic GAT Example: Lending Iterator

A classic use case is a "lending iterator" that can lend references with different lifetimes:

```rust
// Without GATs, this is impossible to express properly
trait LendingIterator {
    type Item<'a> where Self: 'a;
    
    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

// Example: Iterator that lends windows into a slice
struct WindowsMut<'data, T> {
    data: &'data mut [T],
    size: usize,
    pos: usize,
}

impl<'data, T> WindowsMut<'data, T> {
    fn new(data: &'data mut [T], size: usize) -> Self {
        Self { data, size, pos: 0 }
    }
}

impl<'data, T> LendingIterator for WindowsMut<'data, T> {
    type Item<'a> = &'a mut [T] where Self: 'a;
    
    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        if self.pos + self.size <= self.data.len() {
            let window = &mut self.data[self.pos..self.pos + self.size];
            self.pos += 1;
            // SAFETY: We're returning a reference tied to 'a
            Some(unsafe { 
                std::slice::from_raw_parts_mut(
                    window.as_mut_ptr(), 
                    window.len()
                ) 
            })
        } else {
            None
        }
    }
}

fn main() {
    let mut data = vec![1, 2, 3, 4, 5];
    let mut windows = WindowsMut::new(&mut data, 3);
    
    // Each window is borrowed for a different lifetime
    if let Some(w) = windows.next() {
        w[0] = 10;
        println!("Window: {:?}", w);
    }
    
    if let Some(w) = windows.next() {
        w[1] = 20;
        println!("Window: {:?}", w);
    }
}
```

### GATs for Container Abstractions

```rust
// A trait for types that can be viewed as containers
trait Container {
    type Item<'a> where Self: 'a;
    
    fn get<'a>(&'a self, index: usize) -> Option<Self::Item<'a>>;
}

// Implementation for Vec - returns references
impl<T> Container for Vec<T> {
    type Item<'a> = &'a T where Self: 'a;
    
    fn get<'a>(&'a self, index: usize) -> Option<Self::Item<'a>> {
        self.as_slice().get(index)
    }
}

// Implementation for a computed container - returns owned values
struct ComputedContainer {
    base: i32,
}

impl Container for ComputedContainer {
    type Item<'a> = i32; // Returns owned values, not references!
    
    fn get<'a>(&'a self, index: usize) -> Option<Self::Item<'a>> {
        if index < 10 {
            Some(self.base + index as i32)
        } else {
            None
        }
    }
}

fn print_first_three<C: Container>(container: &C) 
where
    for<'a> C::Item<'a>: std::fmt::Display,
{
    for i in 0..3 {
        if let Some(item) = container.get(i) {
            println!("{}", item);
        }
    }
}

fn main() {
    let vec = vec![10, 20, 30, 40];
    print_first_three(&vec);
    
    let computed = ComputedContainer { base: 100 };
    print_first_three(&computed);
}
```

### Advanced GAT: Generic Pointer Types

```rust
// A trait for smart pointer types
trait PointerFamily {
    type Pointer<T>: std::ops::Deref<Target = T>;
    
    fn new<T>(value: T) -> Self::Pointer<T>;
}

// Box pointer family
struct BoxFamily;

impl PointerFamily for BoxFamily {
    type Pointer<T> = Box<T>;
    
    fn new<T>(value: T) -> Self::Pointer<T> {
        Box::new(value)
    }
}

// Rc pointer family
use std::rc::Rc;

struct RcFamily;

impl PointerFamily for RcFamily {
    type Pointer<T> = Rc<T>;
    
    fn new<T>(value: T) -> Self::Pointer<T> {
        Rc::new(value)
    }
}

// Generic data structure parameterized over pointer family
struct Tree<T, P: PointerFamily> {
    value: T,
    left: Option<P::Pointer<Tree<T, P>>>,
    right: Option<P::Pointer<Tree<T, P>>>,
}

impl<T, P: PointerFamily> Tree<T, P> {
    fn leaf(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }
    
    fn with_children(
        value: T,
        left: Tree<T, P>,
        right: Tree<T, P>,
    ) -> Self {
        Self {
            value,
            left: Some(P::new(left)),
            right: Some(P::new(right)),
        }
    }
}

fn main() {
    // Tree with Box pointers
    let box_tree: Tree<i32, BoxFamily> = Tree::with_children(
        1,
        Tree::leaf(2),
        Tree::leaf(3),
    );
    
    // Tree with Rc pointers (for sharing)
    let rc_tree: Tree<i32, RcFamily> = Tree::with_children(
        1,
        Tree::leaf(2),
        Tree::leaf(3),
    );
}
```

### Combining Const Generics and GATs

```rust
// A trait for fixed-size collections with GATs
trait FixedCollection<const N: usize> {
    type Item<'a> where Self: 'a;
    type Iterator<'a>: Iterator<Item = Self::Item<'a>> where Self: 'a;
    
    fn iter<'a>(&'a self) -> Self::Iterator<'a>;
}

// Implementation for fixed-size array
impl<T, const N: usize> FixedCollection<N> for [T; N] {
    type Item<'a> = &'a T where Self: 'a;
    type Iterator<'a> = std::slice::Iter<'a, T> where Self: 'a;
    
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
        self.as_slice().iter()
    }
}

fn process_fixed<C, const N: usize>(collection: &C)
where
    C: FixedCollection<N>,
    for<'a> C::Item<'a>: std::fmt::Debug,
{
    println!("Processing collection of size {}", N);
    for item in collection.iter() {
        println!("  {:?}", item);
    }
}

fn main() {
    let arr = [1, 2, 3, 4, 5];
    process_fixed(&arr);
}
```

## Key Benefits

**Const Generics:**
- Type-safe compile-time dimensions and sizes
- Eliminate code duplication for different sizes
- Enable better optimizations (size known at compile time)
- Catch size mismatches at compile time

**GATs:**
- More flexible trait abstractions
- Express lending/borrowing patterns properly
- Generic over lifetimes in associated types
- Enable advanced patterns like streaming iterators and type families

Both features significantly enhance Rust's type system expressiveness while maintaining zero-cost abstractions!