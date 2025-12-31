# Rust traits 

**Core Concepts:**
- **Trait Definition**: How to define shared behavior using the `trait` keyword
- **Implementation**: The `impl TraitName for TypeName` pattern with concrete examples using shapes
- **Default Implementations**: Showing how traits can provide optional default behavior
- **Associated Functions**: Functions without `self` that work like constructors or static methods

**Advanced Topics:**
- **Trait Bounds**: Using traits with generics to constrain types
- **Multiple Traits**: Implementing and requiring multiple traits on a single type
- **Trait Objects**: Dynamic dispatch using `dyn` for runtime polymorphism

Each section includes working code examples that demonstrate practical use cases, from basic geometric shapes to UI elements and animals. The examples progressively build in complexity, making it easy to understand how traits enable code reuse and polymorphism in Rust.

The key advantage of Rust's trait system is that it provides both static dispatch (zero-cost abstractions) and dynamic dispatch (trait objects) depending on your needs, all while maintaining strong type safety.

# Rust Traits: Definition and Implementation

## What are Traits?

Traits in Rust are similar to interfaces in other languages. They define shared behavior by specifying a set of methods that types must implement. Traits enable polymorphism and code reuse while maintaining Rust's strong type safety.

## 1. Defining Traits

A trait is defined using the `trait` keyword followed by method signatures:

```rust
// Define a trait for drawable objects
trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
}

// Define a trait for comparable objects
trait Describable {
    fn describe(&self) -> String;
}
```

## 2. Implementing Traits for Types

To implement a trait for a type, use the `impl TraitName for TypeName` syntax:

```rust
struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

// Implement Drawable for Circle
impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing a circle with radius {}", self.radius);
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

// Implement Drawable for Rectangle
impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing a rectangle {}x{}", self.width, self.height);
    }
    
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

// Usage
fn main() {
    let circle = Circle { radius: 5.0 };
    let rectangle = Rectangle { width: 4.0, height: 6.0 };
    
    circle.draw();
    println!("Circle area: {}", circle.area());
    
    rectangle.draw();
    println!("Rectangle area: {}", rectangle.area());
}
```

## 3. Default Implementations

Traits can provide default method implementations that types can use or override:

```rust
trait Summarizable {
    // Method without default implementation (must be implemented)
    fn author(&self) -> String;
    
    // Method with default implementation (optional to override)
    fn summary(&self) -> String {
        format!("(Read more from {}...)", self.author())
    }
    
    // Another default method
    fn full_summary(&self) -> String {
        format!("Summary by {}: {}", self.author(), self.summary())
    }
}

struct Article {
    title: String,
    author: String,
    content: String,
}

struct Tweet {
    username: String,
    content: String,
}

// Using default implementation
impl Summarizable for Article {
    fn author(&self) -> String {
        self.author.clone()
    }
    // summary() and full_summary() use default implementations
}

// Overriding default implementation
impl Summarizable for Tweet {
    fn author(&self) -> String {
        format!("@{}", self.username)
    }
    
    // Override the default summary
    fn summary(&self) -> String {
        format!("{}: {}", self.author(), self.content)
    }
}

// Usage
fn main() {
    let article = Article {
        title: String::from("Rust Programming"),
        author: String::from("Jane Doe"),
        content: String::from("Rust is amazing..."),
    };
    
    let tweet = Tweet {
        username: String::from("rustacean"),
        content: String::from("Learning Rust traits!"),
    };
    
    println!("{}", article.summary());  // Uses default
    println!("{}", tweet.summary());    // Uses custom override
}
```

## 4. Associated Functions

Traits can include associated functions (functions that don't take `self` as a parameter). These are often used as constructors or utility functions:

```rust
trait Shape {
    // Associated function (no self parameter)
    fn new(dimension: f64) -> Self;
    
    // Regular method
    fn area(&self) -> f64;
    
    // Another associated function with default implementation
    fn type_name() -> &'static str {
        "Generic Shape"
    }
}

struct Square {
    side: f64,
}

impl Shape for Square {
    fn new(dimension: f64) -> Self {
        Square { side: dimension }
    }
    
    fn area(&self) -> f64 {
        self.side * self.side
    }
    
    fn type_name() -> &'static str {
        "Square"
    }
}

// Usage
fn main() {
    let square = Square::new(5.0);
    println!("Area: {}", square.area());
    println!("Type: {}", Square::type_name());
}
```

## 5. Trait Bounds and Generic Functions

Traits are commonly used with generics to constrain types:

```rust
trait Animal {
    fn make_sound(&self) -> String;
    fn name(&self) -> String;
}

struct Dog {
    name: String,
}

struct Cat {
    name: String,
}

impl Animal for Dog {
    fn make_sound(&self) -> String {
        String::from("Woof!")
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Animal for Cat {
    fn make_sound(&self) -> String {
        String::from("Meow!")
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}

// Generic function with trait bound
fn introduce_animal<T: Animal>(animal: &T) {
    println!("{} says: {}", animal.name(), animal.make_sound());
}

// Alternative syntax using 'where' clause
fn compare_animals<T, U>(animal1: &T, animal2: &U) 
where
    T: Animal,
    U: Animal,
{
    println!("{} and {} are friends!", animal1.name(), animal2.name());
}

// Usage
fn main() {
    let dog = Dog { name: String::from("Buddy") };
    let cat = Cat { name: String::from("Whiskers") };
    
    introduce_animal(&dog);
    introduce_animal(&cat);
    compare_animals(&dog, &cat);
}
```

## 6. Multiple Trait Implementation

A type can implement multiple traits:

```rust
trait Printable {
    fn print(&self);
}

trait Cloneable {
    fn clone_item(&self) -> Self;
}

struct Document {
    content: String,
}

impl Printable for Document {
    fn print(&self) {
        println!("Document: {}", self.content);
    }
}

impl Cloneable for Document {
    fn clone_item(&self) -> Self {
        Document {
            content: self.content.clone(),
        }
    }
}

// Function requiring multiple traits
fn print_and_clone<T: Printable + Cloneable>(item: &T) -> T {
    item.print();
    item.clone_item()
}
```

## 7. Trait Objects (Dynamic Dispatch)

You can use trait objects for dynamic polymorphism:

```rust
trait Renderable {
    fn render(&self);
}

struct Button {
    label: String,
}

struct TextBox {
    text: String,
}

impl Renderable for Button {
    fn render(&self) {
        println!("Rendering button: {}", self.label);
    }
}

impl Renderable for TextBox {
    fn render(&self) {
        println!("Rendering textbox: {}", self.text);
    }
}

// Using trait objects
fn render_all(items: &[Box<dyn Renderable>]) {
    for item in items {
        item.render();
    }
}

fn main() {
    let ui_elements: Vec<Box<dyn Renderable>> = vec![
        Box::new(Button { label: String::from("Submit") }),
        Box::new(TextBox { text: String::from("Enter name") }),
    ];
    
    render_all(&ui_elements);
}
```

## Key Takeaways

- **Traits** define shared behavior through method signatures
- **Implementation** uses `impl TraitName for TypeName` syntax
- **Default implementations** provide reusable behavior that can be overridden
- **Associated functions** don't take `self` and are called on the type itself
- **Trait bounds** enable generic programming with type constraints
- **Trait objects** allow runtime polymorphism using `dyn TraitName`

Traits are fundamental to Rust's type system and enable powerful abstractions while maintaining zero-cost abstractions through static dispatch.