
**Rust Iterator Traits:**
- **`Iterator`**: The core trait with `next()` method returning `Option<&T>`
- **`ExactSizeIterator`**: Provides exact length information
- **`DoubleEndedIterator`**: Allows iteration from both ends
- **`IntoIterator`**: Enables use in `for` loops

**Three Types of Iteration:**
1. **Immutable** (`iter()`): Borrows elements as `&T`
2. **Mutable** (`iter_mut()`): Borrows elements as `&mut T` for modification
3. **Consuming** (`into_iter()`): Takes ownership and consumes the collection

**Key Rust Concepts:**
- **Lifetimes** (`'a`): Ensures borrowed data lives long enough
- **Ownership**: Iterator borrows don't outlive the container
- **Option type**: `next()` returns `Option<T>` instead of using sentinel values
- **Zero-cost abstractions**: Iterator chains compile to efficient code

**Benefits:**
- Works seamlessly with Rust's `for` loops
- Compatible with iterator adapters like `map`, `filter`, `take`
- Type-safe with compile-time guarantees
- No runtime overhead

The Rust version is more idiomatic and leverages Rust's ownership system to prevent common iterator invalidation bugs that can occur in C++.

```rust
use std::ops::Index;

// A simple dynamic array container with custom iterators
struct MyVector<T> {
    data: Vec<T>,
}

impl<T> MyVector<T> {
    // Constructor
    fn new() -> Self {
        MyVector { data: Vec::new() }
    }

    // Add element
    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    // Get size
    fn len(&self) -> usize {
        self.data.len()
    }

    // Check if empty
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // Create iterator
    fn iter(&self) -> MyVectorIterator<T> {
        MyVectorIterator {
            data: &self.data,
            index: 0,
        }
    }

    // Create mutable iterator
    fn iter_mut(&mut self) -> MyVectorIteratorMut<T> {
        MyVectorIteratorMut {
            data: &mut self.data,
            index: 0,
        }
    }
}

// Implement Index trait for array-like access
impl<T> Index<usize> for MyVector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

// Immutable Iterator
struct MyVectorIterator<'a, T> {
    data: &'a Vec<T>,
    index: usize,
}

impl<'a, T> Iterator for MyVectorIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    // Optional: provide size_hint for better performance
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.index;
        (remaining, Some(remaining))
    }
}

// Implement ExactSizeIterator for precise length
impl<'a, T> ExactSizeIterator for MyVectorIterator<'a, T> {
    fn len(&self) -> usize {
        self.data.len() - self.index
    }
}

// Implement DoubleEndedIterator to iterate from both ends
impl<'a, T> DoubleEndedIterator for MyVectorIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.data.len() - 1];
            Some(item)
        } else {
            None
        }
    }
}

// Mutable Iterator
struct MyVectorIteratorMut<'a, T> {
    data: &'a mut Vec<T>,
    index: usize,
}

impl<'a, T> Iterator for MyVectorIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let ptr = self.data.as_mut_ptr();
            let item = unsafe { &mut *ptr.add(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Implement IntoIterator for owned iteration (consuming)
impl<T> IntoIterator for MyVector<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// Implement IntoIterator for borrowed iteration
impl<'a, T> IntoIterator for &'a MyVector<T> {
    type Item = &'a T;
    type IntoIter = MyVectorIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn main() {
    let mut vec = MyVector::new();

    // Add elements
    for i in 1..=10 {
        vec.push(i * 10);
    }

    // Iterate using custom iterator
    print!("Elements: ");
    for val in vec.iter() {
        print!("{} ", val);
    }
    println!();

    // Using for loop directly (thanks to IntoIterator)
    print!("Using for loop: ");
    for val in &vec {
        print!("{} ", val);
    }
    println!();

    // Access by index
    println!("Third element: {}", vec[2]);

    // Using iterator adapters
    println!("First 5 elements:");
    for val in vec.iter().take(5) {
        print!("{} ", val);
    }
    println!();

    // Filter and map
    print!("Even values doubled: ");
    for val in vec.iter().filter(|&&x| x % 20 == 0).map(|&x| x * 2) {
        print!("{} ", val);
    }
    println!();

    // Mutable iteration
    println!("\nModifying elements...");
    for val in vec.iter_mut() {
        *val += 5;
    }

    print!("After modification: ");
    for val in &vec {
        print!("{} ", val);
    }
    println!();

    // Collecting into a new vector
    let doubled: Vec<i32> = vec.iter().map(|&x| x * 2).collect();
    print!("Doubled collection: ");
    for val in &doubled {
        print!("{} ", val);
    }
    println!();

    // Using iterator methods
    let sum: i32 = vec.iter().sum();
    println!("Sum of all elements: {}", sum);

    let max = vec.iter().max();
    println!("Max element: {:?}", max);
}
```