# Structs, Enums, and Pattern Matching in Rust

I'll create a comprehensive guide with detailed examples covering these fundamental Rust concepts.I've created a comprehensive guide covering structs, enums, and pattern matching in Rust. Let me break down the key concepts:

## **Structs**

Rust has three types of structs:

1. **Classic structs** - Named fields (like `Person` with `name`, `age`, `email`)
2. **Tuple structs** - Ordered, unnamed fields (like `Point(i32, i32, i32)`)
3. **Unit structs** - No fields, useful as markers or types

## **Enums**

Enums in Rust are powerful because variants can hold data:

- **Simple variants**: `Direction::North` (no data)
- **Tuple variants**: `Message::Write(String)` (unnamed fields)
- **Struct variants**: `Message::Move { x: i32, y: i32 }` (named fields)

The built-in `Option<T>` and `Result<T, E>` enums are fundamental to Rust's approach to null safety and error handling.

## **Pattern Matching**

**Exhaustive matching** is a key feature - the compiler ensures you handle all possible cases:

```rust
match direction {
    Direction::North => { /* ... */ },
    Direction::South => { /* ... */ },
    // Must handle East and West, or use _ for "everything else"
}
```

**Destructuring** lets you extract values from structs and enums directly in patterns.

**Match guards** add conditional logic with `if`:
```rust
Some(x) if x < 0 => println!("Negative"),
```

## **Key Benefits**

- **Safety**: Exhaustive matching prevents missed cases
- **Expressiveness**: Complex data patterns handled elegantly
- **Refactoring**: Adding enum variants causes compilation errors at unhandled match sites
- **No null**: `Option<T>` forces explicit handling of "no value" cases

The code includes practical examples using `Option`, `Result`, `if let`, and `while let` patterns for more concise code when you don't need exhaustive matching.

```rust
// ==================== STRUCTS ====================

// 1. Classic Structs (Named Fields)
struct Person {
    name: String,
    age: u32,
    email: String,
}

// 2. Tuple Structs (Unnamed Fields)
struct Point(i32, i32, i32);
struct Color(u8, u8, u8);

// 3. Unit Structs (No Fields)
struct Marker;

// Struct with methods
impl Person {
    fn new(name: String, age: u32, email: String) -> Self {
        Person { name, age, email }
    }
    
    fn greet(&self) -> String {
        format!("Hello, I'm {} and I'm {} years old", self.name, self.age)
    }
}

// ==================== ENUMS ====================

// Basic enum with simple variants
enum Direction {
    North,
    South,
    East,
    West,
}

// Enum with data attached to variants
enum Message {
    Quit,                       // No data
    Move { x: i32, y: i32 },   // Named fields (like a struct)
    Write(String),              // Single value
    ChangeColor(u8, u8, u8),   // Tuple-like
}

// The Option enum (built into Rust)
// enum Option<T> {
//     Some(T),
//     None,
// }

// The Result enum (built into Rust)
// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

// Complex enum example
enum WebEvent {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click { x: i64, y: i64 },
}

// ==================== PATTERN MATCHING ====================

fn basic_matching_example() {
    let direction = Direction::North;
    
    // Exhaustive matching - must cover all variants
    match direction {
        Direction::North => println!("Heading north!"),
        Direction::South => println!("Heading south!"),
        Direction::East => println!("Heading east!"),
        Direction::West => println!("Heading west!"),
    }
}

fn enum_with_data_matching() {
    let msg = Message::Move { x: 10, y: 20 };
    
    match msg {
        Message::Quit => {
            println!("Quit message received");
        }
        Message::Move { x, y } => {
            println!("Move to coordinates: ({}, {})", x, y);
        }
        Message::Write(text) => {
            println!("Text message: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Change color to RGB({}, {}, {})", r, g, b);
        }
    }
}

// ==================== DESTRUCTURING ====================

fn destructuring_structs() {
    let person = Person {
        name: String::from("Alice"),
        age: 30,
        email: String::from("alice@example.com"),
    };
    
    // Destructure in match
    match person {
        Person { name, age, email } => {
            println!("Name: {}, Age: {}, Email: {}", name, age, email);
        }
    }
    
    // Destructure with pattern
    let Person { name, age, .. } = Person::new(
        String::from("Bob"),
        25,
        String::from("bob@example.com"),
    );
    println!("Name: {}, Age: {}", name, age);
    
    // Tuple struct destructuring
    let point = Point(1, 2, 3);
    let Point(x, y, z) = point;
    println!("Point coordinates: ({}, {}, {})", x, y, z);
}

fn destructuring_in_function_params(Point(x, y, z): Point) {
    println!("Destructured in params: x={}, y={}, z={}", x, y, z);
}

// ==================== MATCH GUARDS ====================

fn match_guards_example(num: Option<i32>) {
    match num {
        Some(x) if x < 0 => println!("Negative number: {}", x),
        Some(x) if x == 0 => println!("Zero!"),
        Some(x) if x < 10 => println!("Small positive number: {}", x),
        Some(x) => println!("Large number: {}", x),
        None => println!("No value"),
    }
}

fn complex_match_guards() {
    let pair = (2, -5);
    
    match pair {
        (x, y) if x == y => println!("Both values are equal"),
        (x, y) if x + y == 0 => println!("They sum to zero!"),
        (x, _) if x % 2 == 0 => println!("First value is even"),
        _ => println!("No special pattern matched"),
    }
}

// ==================== ADVANCED PATTERNS ====================

fn advanced_patterns() {
    // Multiple patterns with |
    let number = 4;
    match number {
        1 | 2 => println!("One or two"),
        3..=5 => println!("Three through five"),
        _ => println!("Something else"),
    }
    
    // Matching ranges
    let age = 25;
    match age {
        0..=12 => println!("Child"),
        13..=19 => println!("Teen"),
        20..=64 => println!("Adult"),
        _ => println!("Senior"),
    }
    
    // @ binding - capture value while testing
    let msg = Message::Write(String::from("Hello"));
    match msg {
        Message::Write(text @ _) if text.len() > 10 => {
            println!("Long message: {}", text);
        }
        Message::Write(text) => {
            println!("Short message: {}", text);
        }
        _ => println!("Other message type"),
    }
}

// ==================== PRACTICAL EXAMPLES ====================

// Using Option<T>
fn divide(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn handle_division_result() {
    let result = divide(10.0, 2.0);
    
    match result {
        Some(x) => println!("Result: {}", x),
        None => println!("Cannot divide by zero"),
    }
    
    // Alternative: if let pattern
    if let Some(x) = divide(8.0, 2.0) {
        println!("Division successful: {}", x);
    }
}

// Using Result<T, E>
fn parse_number(s: &str) -> Result<i32, String> {
    match s.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(format!("'{}' is not a valid number", s)),
    }
}

fn handle_result() {
    match parse_number("42") {
        Ok(n) => println!("Parsed number: {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

// ==================== IF LET AND WHILE LET ====================

fn if_let_example() {
    let some_option = Some(7);
    
    // Instead of verbose match
    if let Some(x) = some_option {
        println!("Got a value: {}", x);
    } else {
        println!("No value");
    }
}

fn while_let_example() {
    let mut stack = vec![1, 2, 3, 4, 5];
    
    // Pop values while Some
    while let Some(top) = stack.pop() {
        println!("Popped: {}", top);
    }
}

// ==================== MAIN FUNCTION ====================

fn main() {
    println!("=== Basic Matching ===");
    basic_matching_example();
    
    println!("\n=== Enum with Data ===");
    enum_with_data_matching();
    
    println!("\n=== Destructuring ===");
    destructuring_structs();
    destructuring_in_function_params(Point(5, 10, 15));
    
    println!("\n=== Match Guards ===");
    match_guards_example(Some(-5));
    match_guards_example(Some(0));
    match_guards_example(Some(7));
    match_guards_example(Some(42));
    complex_match_guards();
    
    println!("\n=== Advanced Patterns ===");
    advanced_patterns();
    
    println!("\n=== Option and Result ===");
    handle_division_result();
    handle_result();
    
    println!("\n=== If Let and While Let ===");
    if_let_example();
    while_let_example();
}
```