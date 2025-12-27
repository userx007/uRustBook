# Generics and Monomorphization in Rust


## **What Are Generics?**

Generics allow you to write code that works with multiple types without duplicating logic. Instead of writing separate `largest_int()` and `largest_char()` functions, you write one generic `largest<T>()` function that works with any comparable type.

## **What Is Monomorphization?**

Monomorphization is Rust's compile-time process of converting generic code into concrete, type-specific code. When you use a generic function with different types, the compiler generates separate versions of that function for each type.

**Example**: If you write:
```rust
let a = add(5, 10);      // i32
let b = add(2.5, 3.7);   // f64
```

The compiler generates two separate functions:
- `add_i32(i32, i32) -> i32`
- `add_f64(f64, f64) -> f64`

## **Key Benefits**

1. **Zero-cost abstraction**: Generic code runs as fast as hand-written type-specific code with no runtime overhead
2. **Type safety**: Errors caught at compile time, not runtime
3. **Code reuse**: Write once, use with many types

## **Trade-offs**

1. **Compile time**: Longer compilation as the compiler generates code for each type combination
2. **Binary size**: Larger executables due to code duplication (though often marginal)

## **How It Differs from Other Languages**

- **C++**: Similar template instantiation
- **Java/C#**: Use type erasure - single implementation with runtime type checking (slower but smaller binaries)
- **Rust**: Compile-time specialization - fastest but larger code

The artifact demonstrates everything from basic generic functions to complex containers, showing how monomorphization enables Rust's "zero-cost abstractions" philosophy.

```rust
// ============================================
// GENERICS AND MONOMORPHIZATION IN RUST
// ============================================

// 1. BASIC GENERIC FUNCTIONS
// Generic functions work with multiple types using type parameters

fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// Generic function with multiple type parameters
fn display_pair<T: std::fmt::Display, U: std::fmt::Display>(a: T, b: U) {
    println!("Pair: {} and {}", a, b);
}

// 2. GENERIC STRUCTS
// Structs can be generic over one or more types

struct Point<T> {
    x: T,
    y: T,
}

// Multiple type parameters
struct Pair<T, U> {
    first: T,
    second: U,
}

// 3. GENERIC IMPLEMENTATIONS
// Implementing methods on generic types

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Implementation for specific types only
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// Generic methods with different type parameters
impl<T, U> Pair<T, U> {
    fn mix_up<V, W>(self, other: Pair<V, W>) -> Pair<T, W> {
        Pair {
            first: self.first,
            second: other.second,
        }
    }
}

// 4. GENERIC ENUMS
// Enums can also be generic

enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Option<T> {
    Some(T),
    None,
}

// 5. TRAIT BOUNDS
// Constraining generic types with traits

use std::fmt::Display;

fn print_and_return<T: Display + Clone>(item: T) -> T {
    println!("Got: {}", item);
    item.clone()
}

// Multiple trait bounds with where clause
fn complex_function<T, U>(t: &T, u: &U) -> String
where
    T: Display + Clone,
    U: Display + Clone,
{
    format!("{} and {}", t, u)
}

// 6. MONOMORPHIZATION DEMONSTRATION
// This shows how Rust generates concrete types at compile time

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

// 7. PERFORMANCE EXAMPLE
// Generic container that demonstrates zero-cost abstraction

struct Container<T> {
    items: Vec<T>,
}

impl<T> Container<T> {
    fn new() -> Self {
        Container { items: Vec::new() }
    }
    
    fn push(&mut self, item: T) {
        self.items.push(item);
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
}

// 8. PRACTICAL EXAMPLE: Generic Stack
#[derive(Debug)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    
    fn push(&mut self, item: T) {
        self.items.push(item);
    }
    
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn len(&self) -> usize {
        self.items.len()
    }
}

fn main() {
    println!("=== BASIC GENERICS ===\n");
    
    // Using generic function with different types
    let numbers = vec![34, 50, 25, 100, 65];
    let result = largest(&numbers);
    println!("Largest number: {}", result);
    
    let chars = vec!['y', 'm', 'a', 'q'];
    let result = largest(&chars);
    println!("Largest char: {}\n", result);
    
    display_pair(5, "hello");
    display_pair(3.14, true);
    
    println!("\n=== GENERIC STRUCTS ===\n");
    
    // Point with integers
    let int_point = Point::new(5, 10);
    println!("Integer point: ({}, {})", int_point.x, int_point.y);
    
    // Point with floats
    let float_point = Point::new(1.5, 4.2);
    println!("Float point: ({}, {})", float_point.x, float_point.y);
    println!("Distance from origin: {:.2}", float_point.distance_from_origin());
    
    // Mixed type pair
    let pair = Pair { first: 5, second: "hello" };
    println!("Pair: {} and {}\n", pair.first, pair.second);
    
    println!("=== MONOMORPHIZATION DEMONSTRATION ===\n");
    
    // When we call add() with different types, Rust generates
    // separate concrete implementations for each type:
    
    let int_sum = add(5, 10);        // Generates add_i32(i32, i32) -> i32
    let float_sum = add(2.5, 3.7);   // Generates add_f64(f64, f64) -> f64
    
    println!("Integer sum: {}", int_sum);
    println!("Float sum: {}", float_sum);
    
    // At runtime, there's no generic code - only type-specific code
    // This is zero-cost abstraction: no runtime overhead!
    
    println!("\n=== GENERIC STACK EXAMPLE ===\n");
    
    // Integer stack - monomorphized to Stack_i32
    let mut int_stack = Stack::new();
    int_stack.push(1);
    int_stack.push(2);
    int_stack.push(3);
    println!("Integer stack: {:?}", int_stack);
    println!("Popped: {:?}", int_stack.pop());
    println!("Peek: {:?}", int_stack.peek());
    
    // String stack - monomorphized to Stack_String
    let mut string_stack = Stack::new();
    string_stack.push(String::from("hello"));
    string_stack.push(String::from("world"));
    println!("\nString stack length: {}", string_stack.len());
    println!("String stack: {:?}", string_stack);
    
    // Struct stack - monomorphized to Stack_Point_f64
    let mut point_stack: Stack<Point<f64>> = Stack::new();
    point_stack.push(Point::new(1.0, 2.0));
    point_stack.push(Point::new(3.0, 4.0));
    println!("\nPoint stack length: {}", point_stack.len());
    
    println!("\n=== COMPILE-TIME EXPLOSION ===\n");
    println!("Each unique type combination generates separate machine code:");
    println!("- Stack<i32>");
    println!("- Stack<String>");
    println!("- Stack<Point<f64>>");
    println!("- add<i32>");
    println!("- add<f64>");
    println!("\nBenefit: Zero runtime cost, maximum performance");
    println!("Trade-off: Larger binary size (code bloat)");
}

// ============================================
// KEY CONCEPTS:
// ============================================
// 
// 1. GENERICS:
//    - Allow code to work with multiple types
//    - Defined with type parameters like <T>
//    - Can have trait bounds to constrain types
//
// 2. MONOMORPHIZATION:
//    - Rust's process of turning generic code into specific code
//    - Happens at compile time
//    - Creates separate copy of code for each concrete type used
//    - Example: Vec<i32> and Vec<String> become two different types
//
// 3. ZERO-COST ABSTRACTION:
//    - Generic code runs as fast as hand-written type-specific code
//    - No runtime overhead for using generics
//    - No dynamic dispatch unless explicitly requested (trait objects)
//
// 4. TRADE-OFFS:
//    - Pros: Maximum runtime performance, type safety, code reuse
//    - Cons: Longer compile times, larger binary size
//    - Compare to C++ templates (similar) vs Java generics (different)
//
// 5. WHEN MONOMORPHIZATION HAPPENS:
//    - During compilation, not at runtime
//    - For each unique combination of type parameters
//    - Results in specialized machine code for each type
// ============================================
```