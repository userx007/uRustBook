# Common Rust design patterns 

## **1. Builder Pattern**
Creates complex objects step-by-step with a fluent API. Perfect for structs with many optional fields, avoiding massive constructors. Each method returns `self`, allowing method chaining.

## **2. Type State Pattern**
Uses Rust's type system to enforce valid state transitions at compile time. Different states are represented by distinct types, making illegal state transitions impossible to compile. Great for state machines and protocols.

## **3. RAII (Resource Acquisition Is Initialization)**
Resources are tied to object lifetimes. When an object goes out of scope, its `Drop` implementation automatically cleans up resources. This is fundamental to Rust's memory safety - think file handles, network connections, locks, etc.

## **4. Visitor Pattern**
Separates operations from the data structures they operate on. Allows adding new operations without modifying existing types. Uses double dispatch through the `accept` method and visitor trait.

## **5. Strategy Pattern**
Defines a family of interchangeable algorithms through a trait. The algorithm can be swapped at runtime, enabling flexible behavior changes. Common in compression, sorting, or any scenario requiring pluggable algorithms.

## **6. Extension Traits**
Adds methods to existing types (even foreign types from other crates) without modifying them. This is Rust's solution to extension methods, respecting orphan rules while maintaining flexibility.

## **7. Newtype Pattern**
Wraps existing types in tuple structs to:
- Add type safety (prevent mixing similar types like `UserId` vs `ProductId`)
- Implement traits on foreign types (orphan rule workaround)
- Hide implementation details
- Add semantic meaning to primitive types

Each pattern leverages Rust's unique features like ownership, traits, zero-cost abstractions, and compile-time guarantees to create safe, efficient, and expressive code.

```rust
// ============================================================================
// 1. BUILDER PATTERN
// ============================================================================
// Used to construct complex objects step by step, especially when there are
// many optional parameters. Avoids telescoping constructors.

#[derive(Debug, Clone)]
struct Server {
    host: String,
    port: u16,
    timeout: u64,
    max_connections: usize,
    ssl_enabled: bool,
}

struct ServerBuilder {
    host: String,
    port: u16,
    timeout: u64,
    max_connections: usize,
    ssl_enabled: bool,
}

impl ServerBuilder {
    fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 8080,
            timeout: 30,
            max_connections: 100,
            ssl_enabled: false,
        }
    }

    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    fn ssl_enabled(mut self, enabled: bool) -> Self {
        self.ssl_enabled = enabled;
        self
    }

    fn build(self) -> Server {
        Server {
            host: self.host,
            port: self.port,
            timeout: self.timeout,
            max_connections: self.max_connections,
            ssl_enabled: self.ssl_enabled,
        }
    }
}

fn builder_example() {
    let server = ServerBuilder::new("localhost")
        .port(3000)
        .ssl_enabled(true)
        .max_connections(500)
        .build();
    
    println!("Builder Pattern: {:?}", server);
}

// ============================================================================
// 2. TYPE STATE PATTERN (Session Types)
// ============================================================================
// Uses Rust's type system to enforce state transitions at compile time.
// Different states are represented by different types.

use std::marker::PhantomData;

// States as marker types
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        println!("Door created in locked state");
        Door { _state: PhantomData }
    }

    fn unlock(self, key: &str) -> Result<Door<Unlocked>, String> {
        if key == "correct_key" {
            println!("Door unlocked!");
            Ok(Door { _state: PhantomData })
        } else {
            Err("Wrong key!".to_string())
        }
    }
}

impl Door<Unlocked> {
    fn open(&self) {
        println!("Door opened!");
    }

    fn lock(self) -> Door<Locked> {
        println!("Door locked!");
        Door { _state: PhantomData }
    }
}

fn typestate_example() {
    let locked_door = Door::<Locked>::new();
    
    // This would fail to compile:
    // locked_door.open(); // Error: no method `open` found for `Door<Locked>`
    
    let unlocked_door = locked_door.unlock("correct_key").unwrap();
    unlocked_door.open();
    
    let _locked_again = unlocked_door.lock();
}

// ============================================================================
// 3. RAII (Resource Acquisition Is Initialization)
// ============================================================================
// Resources are tied to object lifetime. Cleanup happens automatically
// when objects go out of scope. This is Rust's core memory safety principle.

use std::fs::File;
use std::io::Write;

struct TempFile {
    path: String,
    file: File,
}

impl TempFile {
    fn new(path: &str) -> std::io::Result<Self> {
        println!("Creating temporary file: {}", path);
        let file = File::create(path)?;
        Ok(TempFile {
            path: path.to_string(),
            file,
        })
    }

    fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.file.write_all(data)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        println!("Cleaning up temporary file: {}", self.path);
        let _ = std::fs::remove_file(&self.path);
    }
}

fn raii_example() {
    {
        let mut temp = TempFile::new("temp.txt").unwrap();
        temp.write(b"Hello, RAII!").unwrap();
        // File is automatically closed and deleted when temp goes out of scope
    }
    println!("Temp file has been cleaned up automatically");
}

// ============================================================================
// 4. VISITOR PATTERN
// ============================================================================
// Separates algorithms from the objects they operate on. Enables adding
// new operations without modifying existing types.

trait Shape {
    fn accept(&self, visitor: &dyn ShapeVisitor);
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn accept(&self, visitor: &dyn ShapeVisitor) {
        visitor.visit_circle(self);
    }
}

impl Shape for Rectangle {
    fn accept(&self, visitor: &dyn ShapeVisitor) {
        visitor.visit_rectangle(self);
    }
}

trait ShapeVisitor {
    fn visit_circle(&self, circle: &Circle);
    fn visit_rectangle(&self, rect: &Rectangle);
}

struct AreaCalculator {
    total_area: std::cell::RefCell<f64>,
}

impl AreaCalculator {
    fn new() -> Self {
        Self {
            total_area: std::cell::RefCell::new(0.0),
        }
    }

    fn get_total(&self) -> f64 {
        *self.total_area.borrow()
    }
}

impl ShapeVisitor for AreaCalculator {
    fn visit_circle(&self, circle: &Circle) {
        let area = std::f64::consts::PI * circle.radius * circle.radius;
        *self.total_area.borrow_mut() += area;
        println!("Circle area: {:.2}", area);
    }

    fn visit_rectangle(&self, rect: &Rectangle) {
        let area = rect.width * rect.height;
        *self.total_area.borrow_mut() += area;
        println!("Rectangle area: {:.2}", area);
    }
}

fn visitor_example() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 10.0, height: 20.0 }),
    ];

    let calculator = AreaCalculator::new();
    
    for shape in &shapes {
        shape.accept(&calculator);
    }
    
    println!("Total area: {:.2}", calculator.get_total());
}

// ============================================================================
// 5. STRATEGY PATTERN
// ============================================================================
// Defines a family of algorithms, encapsulates each one, and makes them
// interchangeable. Uses traits to define the algorithm interface.

trait CompressionStrategy {
    fn compress(&self, data: &str) -> String;
}

struct ZipCompression;
struct GzipCompression;
struct NoCompression;

impl CompressionStrategy for ZipCompression {
    fn compress(&self, data: &str) -> String {
        format!("[ZIP compressed: {}]", data)
    }
}

impl CompressionStrategy for GzipCompression {
    fn compress(&self, data: &str) -> String {
        format!("[GZIP compressed: {}]", data)
    }
}

impl CompressionStrategy for NoCompression {
    fn compress(&self, data: &str) -> String {
        data.to_string()
    }
}

struct FileCompressor {
    strategy: Box<dyn CompressionStrategy>,
}

impl FileCompressor {
    fn new(strategy: Box<dyn CompressionStrategy>) -> Self {
        Self { strategy }
    }

    fn set_strategy(&mut self, strategy: Box<dyn CompressionStrategy>) {
        self.strategy = strategy;
    }

    fn compress_file(&self, filename: &str, data: &str) {
        let compressed = self.strategy.compress(data);
        println!("File '{}': {}", filename, compressed);
    }
}

fn strategy_example() {
    let mut compressor = FileCompressor::new(Box::new(ZipCompression));
    compressor.compress_file("doc.txt", "Hello World");

    compressor.set_strategy(Box::new(GzipCompression));
    compressor.compress_file("data.txt", "Important data");

    compressor.set_strategy(Box::new(NoCompression));
    compressor.compress_file("notes.txt", "Quick notes");
}

// ============================================================================
// 6. EXTENSION TRAITS
// ============================================================================
// Add methods to existing types (even foreign types) without modifying them.
// This is Rust's answer to extension methods.

trait StringExtensions {
    fn to_title_case(&self) -> String;
    fn word_count(&self) -> usize;
}

impl StringExtensions for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn word_count(&self) -> usize {
        self.split_whitespace().count()
    }
}

// Extension trait for Vec
trait VecExtensions<T> {
    fn second(&self) -> Option<&T>;
    fn second_mut(&mut self) -> Option<&mut T>;
}

impl<T> VecExtensions<T> for Vec<T> {
    fn second(&self) -> Option<&T> {
        self.get(1)
    }

    fn second_mut(&mut self) -> Option<&mut T> {
        self.get_mut(1)
    }
}

fn extension_traits_example() {
    let text = "hello world from rust";
    println!("Title case: {}", text.to_title_case());
    println!("Word count: {}", text.word_count());

    let mut numbers = vec![1, 2, 3, 4, 5];
    println!("Second element: {:?}", numbers.second());
    
    if let Some(second) = numbers.second_mut() {
        *second = 42;
    }
    println!("Modified vec: {:?}", numbers);
}

// ============================================================================
// 7. NEWTYPE PATTERN
// ============================================================================
// Wraps an existing type in a new type to add type safety, implement traits
// on foreign types, or hide implementation details.

// Type safety: prevent mixing up different kinds of IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProductId(u64);

impl UserId {
    fn new(id: u64) -> Self {
        UserId(id)
    }

    fn value(&self) -> u64 {
        self.0
    }
}

impl ProductId {
    fn new(id: u64) -> Self {
        ProductId(id)
    }

    fn value(&self) -> u64 {
        self.0
    }
}

// Implement trait on foreign type through newtype
struct Meters(f64);

impl std::ops::Add for Meters {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Meters(self.0 + other.0)
    }
}

impl std::fmt::Display for Meters {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}m", self.0)
    }
}

// Hide implementation details
struct Password(String);

impl Password {
    fn new(password: String) -> Result<Self, &'static str> {
        if password.len() < 8 {
            Err("Password must be at least 8 characters")
        } else {
            Ok(Password(password))
        }
    }

    // Prevent accidental printing of password
    fn verify(&self, input: &str) -> bool {
        self.0 == input
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Password([REDACTED])")
    }
}

fn newtype_example() {
    let user_id = UserId::new(42);
    let product_id = ProductId::new(100);
    
    // This would fail to compile - type safety!
    // if user_id == product_id { ... }
    
    println!("User ID: {:?}", user_id);
    println!("Product ID: {:?}", product_id);

    let d1 = Meters(10.5);
    let d2 = Meters(5.5);
    println!("Total distance: {}", d1 + d2);

    let password = Password::new("secure_password_123".to_string()).unwrap();
    println!("Password object: {:?}", password);
    println!("Password valid: {}", password.verify("secure_password_123"));
}

// ============================================================================
// MAIN FUNCTION - Run all examples
// ============================================================================

fn main() {
    println!("=== BUILDER PATTERN ===");
    builder_example();
    println!();

    println!("=== TYPE STATE PATTERN ===");
    typestate_example();
    println!();

    println!("=== RAII PATTERN ===");
    raii_example();
    println!();

    println!("=== VISITOR PATTERN ===");
    visitor_example();
    println!();

    println!("=== STRATEGY PATTERN ===");
    strategy_example();
    println!();

    println!("=== EXTENSION TRAITS ===");
    extension_traits_example();
    println!();

    println!("=== NEWTYPE PATTERN ===");
    newtype_example();
}
```