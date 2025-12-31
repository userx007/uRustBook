# Rust Iterator Trait and Lazy Evaluation

## Core Concepts Covered:

**Iterator Protocol** - The simple foundation (just implement `next()`) that unlocks 50+ powerful methods through the trait's default implementations.

**Lazy Evaluation** - Iterators don't compute anything until consumed, enabling memory-efficient processing of potentially infinite sequences and avoiding unnecessary work.

**Iterator Combinators** - Methods like `map`, `filter`, `fold`, `zip`, and many others that chain together to create expressive data transformations.

**Custom Iterators** - Examples ranging from simple counters to Fibonacci sequences and custom collection iterators, showing how to implement your own.

**Zero-Cost Abstractions** - How iterator chains compile down to the same efficient machine code as hand-written loops, with no runtime overhead.

The guide includes practical examples throughout, demonstrating patterns like:
- Chaining multiple transformations
- Error handling with iterators
- Partitioning and grouping data
- Creating infinite sequences
- Working with custom types

This is one of Rust's most elegant features—allowing you to write high-level, functional-style code that performs as efficiently as low-level imperative code!

# Rust Iterator Trait and Lazy Evaluation

## Overview

The Iterator trait is one of Rust's most powerful abstractions, enabling efficient, composable, and expressive data processing. Iterators in Rust are **lazy** by default, meaning they don't do any work until you consume them. This design, combined with Rust's zero-cost abstraction principle, allows you to write high-level functional code that compiles down to efficient machine code.

## The Iterator Protocol

At its core, the Iterator trait requires implementing just one method:

```rust
pub trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
    
    // ... many provided methods
}
```

### Key Components

- **Associated Type `Item`**: Specifies the type of elements the iterator yields
- **`next()` method**: Returns `Some(item)` for the next element, or `None` when exhausted
- **Provided methods**: The trait includes 50+ default methods built on top of `next()`

### Basic Example

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];
    let mut iter = v.iter();
    
    println!("{:?}", iter.next()); // Some(1)
    println!("{:?}", iter.next()); // Some(2)
    println!("{:?}", iter.next()); // Some(3)
    println!("{:?}", iter.next()); // Some(4)
    println!("{:?}", iter.next()); // Some(5)
    println!("{:?}", iter.next()); // None
}
```

## Lazy Evaluation

Iterators in Rust are lazy—they only compute values when explicitly requested. This enables:
- **Memory efficiency**: No intermediate collections are created
- **Performance**: Only necessary computations are performed
- **Infinite sequences**: You can work with potentially infinite iterators

### Lazy vs Eager Evaluation

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // This is LAZY - nothing happens yet!
    let lazy_iter = numbers.iter()
        .map(|x| {
            println!("Mapping {}", x);
            x * 2
        })
        .filter(|x| {
            println!("Filtering {}", x);
            x % 3 == 0
        });
    
    println!("Iterator created, but no work done yet!");
    
    // Only when we CONSUME the iterator does work happen
    let result: Vec<_> = lazy_iter.collect();
    println!("Result: {:?}", result); // [6, 12, 18]
}
```

Output demonstrates lazy evaluation:
```
Iterator created, but no work done yet!
Mapping 1
Filtering 2
Mapping 2
Filtering 4
Mapping 3
Filtering 6
Mapping 4
Filtering 8
...
```

## Iterator Combinators

Combinators are methods that transform iterators into new iterators. They're the building blocks of functional-style data processing.

### Common Combinators

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    
    // map: Transform each element
    let doubled: Vec<_> = numbers.iter()
        .map(|x| x * 2)
        .collect();
    println!("Doubled: {:?}", doubled); // [2, 4, 6, 8, 10, 12]
    
    // filter: Keep only matching elements
    let evens: Vec<_> = numbers.iter()
        .filter(|x| *x % 2 == 0)
        .collect();
    println!("Evens: {:?}", evens); // [2, 4, 6]
    
    // filter_map: Combine filter and map
    let parsed: Vec<_> = vec!["1", "two", "3", "four"]
        .iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    println!("Parsed: {:?}", parsed); // [1, 3]
    
    // take: Limit number of elements
    let first_three: Vec<_> = numbers.iter()
        .take(3)
        .collect();
    println!("First three: {:?}", first_three); // [1, 2, 3]
    
    // skip: Skip elements
    let skip_two: Vec<_> = numbers.iter()
        .skip(2)
        .collect();
    println!("Skip two: {:?}", skip_two); // [3, 4, 5, 6]
    
    // enumerate: Add index
    for (i, val) in numbers.iter().enumerate() {
        println!("Index {}: {}", i, val);
    }
    
    // zip: Combine two iterators
    let letters = vec!['a', 'b', 'c'];
    let zipped: Vec<_> = numbers.iter()
        .zip(letters.iter())
        .collect();
    println!("Zipped: {:?}", zipped); // [(1, 'a'), (2, 'b'), (3, 'c')]
    
    // flat_map: Map and flatten
    let nested = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
    let flattened: Vec<_> = nested.iter()
        .flat_map(|v| v.iter())
        .collect();
    println!("Flattened: {:?}", flattened); // [1, 2, 3, 4, 5, 6]
    
    // chain: Concatenate iterators
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let chained: Vec<_> = a.iter()
        .chain(b.iter())
        .collect();
    println!("Chained: {:?}", chained); // [1, 2, 3, 4, 5, 6]
}
```

### Consuming Adaptors

These methods consume the iterator and produce a final result:

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // collect: Build a collection
    let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
    
    // sum: Add all elements
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum); // 15
    
    // product: Multiply all elements
    let product: i32 = numbers.iter().product();
    println!("Product: {}", product); // 120
    
    // fold: Reduce with accumulator
    let factorial = (1..=5).fold(1, |acc, x| acc * x);
    println!("5! = {}", factorial); // 120
    
    // reduce: Like fold but starts with first element
    let max = numbers.iter().copied().reduce(|a, b| a.max(b));
    println!("Max: {:?}", max); // Some(5)
    
    // find: Get first matching element
    let first_even = numbers.iter().find(|&&x| x % 2 == 0);
    println!("First even: {:?}", first_even); // Some(2)
    
    // any: Check if any element matches
    let has_even = numbers.iter().any(|&x| x % 2 == 0);
    println!("Has even: {}", has_even); // true
    
    // all: Check if all elements match
    let all_positive = numbers.iter().all(|&x| x > 0);
    println!("All positive: {}", all_positive); // true
    
    // count: Count elements
    let count = numbers.iter().filter(|&&x| x > 2).count();
    println!("Count > 2: {}", count); // 3
}
```

## Creating Custom Iterators

You can create your own iterators by implementing the Iterator trait.

### Simple Custom Iterator

```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Counter {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

fn main() {
    let counter = Counter::new(5);
    
    for num in counter {
        println!("{}", num); // Prints 1, 2, 3, 4, 5
    }
    
    // Can use all iterator methods
    let sum: u32 = Counter::new(10).sum();
    println!("Sum of 1-10: {}", sum); // 55
}
```

### Fibonacci Iterator

```rust
struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        let new_next = self.curr.checked_add(self.next)?;
        
        self.curr = self.next;
        self.next = new_next;
        
        Some(self.curr)
    }
}

fn main() {
    // Take first 10 Fibonacci numbers
    let fibs: Vec<_> = Fibonacci::new()
        .take(10)
        .collect();
    println!("Fibonacci: {:?}", fibs);
    // [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
    
    // Sum of first 20 Fibonacci numbers
    let sum: u64 = Fibonacci::new()
        .take(20)
        .sum();
    println!("Sum: {}", sum);
}
```

### Iterator Over Custom Collection

```rust
struct Book {
    title: String,
    pages: u32,
}

struct Library {
    books: Vec<Book>,
}

impl Library {
    fn iter(&self) -> LibraryIter {
        LibraryIter {
            books: &self.books,
            index: 0,
        }
    }
}

struct LibraryIter<'a> {
    books: &'a [Book],
    index: usize,
}

impl<'a> Iterator for LibraryIter<'a> {
    type Item = &'a Book;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.books.len() {
            let book = &self.books[self.index];
            self.index += 1;
            Some(book)
        } else {
            None
        }
    }
}

fn main() {
    let library = Library {
        books: vec![
            Book { title: "1984".to_string(), pages: 328 },
            Book { title: "Brave New World".to_string(), pages: 268 },
            Book { title: "Fahrenheit 451".to_string(), pages: 249 },
        ],
    };
    
    for book in library.iter() {
        println!("{}: {} pages", book.title, book.pages);
    }
    
    let total_pages: u32 = library.iter()
        .map(|book| book.pages)
        .sum();
    println!("Total pages: {}", total_pages);
}
```

## Zero-Cost Abstractions

One of Rust's key promises is "zero-cost abstractions"—high-level abstractions that compile down to the same code you'd write by hand. Iterators are a prime example.

### Comparison: Iterator vs Manual Loop

```rust
fn sum_of_squares_iter(numbers: &[i32]) -> i32 {
    numbers.iter()
        .map(|x| x * x)
        .sum()
}

fn sum_of_squares_manual(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in numbers {
        sum += x * x;
    }
    sum
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    println!("Iterator: {}", sum_of_squares_iter(&numbers));
    println!("Manual: {}", sum_of_squares_manual(&numbers));
    
    // Both compile to nearly identical assembly!
}
```

When compiled with optimizations, both functions produce virtually identical assembly code. The iterator version:
- Has no runtime overhead
- Often gets optimized better due to clearer intent
- Is more maintainable and expressive

### Why Zero-Cost Works

1. **Monomorphization**: Generic code is specialized at compile time
2. **Inlining**: Small functions like `map` and `filter` are inlined
3. **Dead code elimination**: Unused code paths are removed
4. **LLVM optimizations**: The compiler can see the full picture and optimize aggressively

## Advanced Patterns

### Chaining Complex Transformations

```rust
fn main() {
    let text = "Hello, World! How are you today?";
    
    let word_lengths: Vec<_> = text
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| (word, word.len()))
        .collect();
    
    for (word, len) in word_lengths {
        println!("{}: {} letters", word, len);
    }
}
```

### Using `Iterator::from_fn`

```rust
fn main() {
    // Create iterator from a closure
    let mut counter = 0;
    let mut iter = std::iter::from_fn(move || {
        counter += 1;
        if counter <= 5 {
            Some(counter * counter)
        } else {
            None
        }
    });
    
    println!("{:?}", iter.collect::<Vec<_>>()); // [1, 4, 9, 16, 25]
}
```

### Infinite Iterators

```rust
fn main() {
    // Generate infinite sequence
    let powers_of_two = std::iter::successors(Some(1u64), |&n| n.checked_mul(2));
    
    // Take only what we need
    let first_ten: Vec<_> = powers_of_two.take(10).collect();
    println!("{:?}", first_ten);
    // [1, 2, 4, 8, 16, 32, 64, 128, 256, 512]
}
```

## Performance Tips

1. **Avoid unnecessary `collect()`**: Keep iterators lazy as long as possible
2. **Use `copied()` or `cloned()` explicitly**: Makes intent clear
3. **Prefer iterator methods over manual loops**: Allows better optimization
4. **Use `into_iter()` when ownership transfer is acceptable**: Avoids cloning
5. **Consider `par_iter()` from Rayon**: For parallel iteration on large datasets

## Common Patterns

### Error Handling with Iterators

```rust
fn main() {
    let strings = vec!["1", "2", "foo", "4"];
    
    // collect() can return Result<Vec<_>, _>
    let result: Result<Vec<i32>, _> = strings
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();
    
    match result {
        Ok(numbers) => println!("All parsed: {:?}", numbers),
        Err(e) => println!("Parse error: {}", e),
    }
    
    // Use filter_map to skip errors
    let numbers: Vec<i32> = strings
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    println!("Successfully parsed: {:?}", numbers); // [1, 2, 4]
}
```

### Grouping and Partitioning

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Partition into two groups
    let (evens, odds): (Vec<_>, Vec<_>) = numbers
        .into_iter()
        .partition(|&x| x % 2 == 0);
    
    println!("Evens: {:?}", evens); // [2, 4, 6, 8, 10]
    println!("Odds: {:?}", odds);   // [1, 3, 5, 7, 9]
}
```

## Summary

The Iterator trait embodies Rust's philosophy of zero-cost abstractions:
- **Expressive**: Write functional-style code that clearly expresses intent
- **Efficient**: Compiles to optimal machine code with no runtime overhead
- **Composable**: Chain operations to build complex transformations
- **Lazy**: Compute only what's needed, when it's needed
- **Type-safe**: Compile-time guarantees about iterator behavior

Mastering iterators is essential for idiomatic Rust programming, enabling you to write code that is both elegant and performant.