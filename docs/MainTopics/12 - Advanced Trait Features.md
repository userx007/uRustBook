

## **1. Supertraits**
Dependencies between traits - if `trait A: B`, you must implement `B` before implementing `A`. This enables trait composition and ensures certain capabilities exist before allowing others.

## **2. Blanket Implementations**
Powerful feature that implements a trait for *all types* meeting certain criteria. For example, Rust's standard library implements `ToString` for any type implementing `Display`. This dramatically reduces code duplication.

## **3. Orphan Rules**
A fundamental restriction: you can only implement a trait if **either** the trait **or** the type is local to your crate. This prevents different crates from creating conflicting implementations. The newtype pattern (`struct Wrapper(ExternalType)`) is the standard workaround.

## **4. Coherence**
Rust guarantees exactly one implementation of any trait for any type. This prevents ambiguity and ensures predictable behavior. The compiler enforces this by detecting overlapping implementations, even with generic parameters.

## **5. Sealed Trait Pattern**
A design pattern that prevents external crates from implementing your trait by using a private supertrait. This gives you control over which types can implement your trait and allows adding methods without breaking compatibility.

These features work together to make Rust's trait system both powerful and safe, preventing common issues like the diamond problem while enabling extensive code reuse through blanket implementations.

```rust
// ============================================================================
// ADVANCED TRAIT FEATURES IN RUST
// ============================================================================

// ----------------------------------------------------------------------------
// 1. SUPERTRAITS
// ----------------------------------------------------------------------------
// A supertrait is a trait that another trait depends on. When implementing
// the dependent trait, you must also implement its supertrait(s).

use std::fmt;

// Display is a supertrait of Summary
trait Summary: fmt::Display {
    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self)
    }
}

struct Article {
    headline: String,
    content: String,
}

// Must implement Display before implementing Summary
impl fmt::Display for Article {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.headline)
    }
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.headline, &self.content[..50])
    }
}

// Multiple supertraits example
trait Printable: fmt::Display + fmt::Debug {
    fn print(&self) {
        println!("Display: {}", self);
        println!("Debug: {:?}", self);
    }
}

// ----------------------------------------------------------------------------
// 2. BLANKET IMPLEMENTATIONS
// ----------------------------------------------------------------------------
// Blanket implementations provide trait implementations for any type that
// satisfies certain trait bounds.

// Example from std: ToString is implemented for all types that implement Display
// impl<T: Display> ToString for T { ... }

trait Describable {
    fn describe(&self) -> String;
}

// Blanket implementation: any type implementing Display gets Describable
impl<T: fmt::Display> Describable for T {
    fn describe(&self) -> String {
        format!("This item displays as: {}", self)
    }
}

// Now any type with Display automatically has Describable
fn demonstrate_blanket() {
    let number = 42;
    let text = "Hello";
    
    // Both work because i32 and &str implement Display
    println!("{}", number.describe());
    println!("{}", text.describe());
}

// Custom blanket implementation example
trait Doubler {
    fn double(&self) -> Self;
}

// Blanket impl for all Copy types that implement Add
use std::ops::Add;

impl<T> Doubler for T 
where 
    T: Add<Output = T> + Copy 
{
    fn double(&self) -> Self {
        *self + *self
    }
}

fn test_doubler() {
    assert_eq!(5.double(), 10);
    assert_eq!(3.14.double(), 6.28);
}

// ----------------------------------------------------------------------------
// 3. ORPHAN RULES
// ----------------------------------------------------------------------------
// The orphan rule states: you can implement a trait for a type only if either
// the trait or the type is local to your crate.
//
// Valid implementations:
// - Local trait on foreign type: impl MyTrait for Vec<T>
// - Foreign trait on local type: impl Display for MyStruct
// - Local trait on local type: impl MyTrait for MyStruct
//
// INVALID (orphan):
// - Foreign trait on foreign type: impl Display for Vec<T>

// Local trait
trait LocalTrait {
    fn local_method(&self);
}

// Local type
struct LocalType {
    value: i32,
}

// ✅ Valid: Local trait on local type
impl LocalTrait for LocalType {
    fn local_method(&self) {
        println!("Local method on local type");
    }
}

// ✅ Valid: Local trait on foreign type
impl LocalTrait for Vec<i32> {
    fn local_method(&self) {
        println!("Vector with {} elements", self.len());
    }
}

// ✅ Valid: Foreign trait on local type
impl fmt::Display for LocalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LocalType({})", self.value)
    }
}

// ❌ INVALID: Cannot implement foreign trait on foreign type
// impl fmt::Display for Vec<i32> { ... }  // This would violate orphan rule

// Workaround: Use newtype pattern
struct MyVec(Vec<i32>);

impl fmt::Display for MyVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

// ----------------------------------------------------------------------------
// 4. COHERENCE
// ----------------------------------------------------------------------------
// Coherence ensures there's only one implementation of a trait for any given
// type. This prevents conflicts and ensures predictable behavior.

trait Processor {
    fn process(&self) -> String;
}

struct Data(i32);

// First implementation
impl Processor for Data {
    fn process(&self) -> String {
        format!("Processing: {}", self.0)
    }
}

// ❌ This would cause a coherence error:
// impl Processor for Data {
//     fn process(&self) -> String {
//         format!("Different processing: {}", self.0)
//     }
// }

// Coherence with generics - these don't conflict due to different bounds
trait Computer<T> {
    fn compute(&self, input: T) -> String;
}

impl Computer<i32> for Data {
    fn compute(&self, input: i32) -> String {
        format!("Computing with i32: {}", self.0 + input)
    }
}

impl Computer<String> for Data {
    fn compute(&self, input: String) -> String {
        format!("Computing with String: {} + {}", self.0, input)
    }
}

// Coherence ensures no overlapping implementations
trait Handler<T> {
    fn handle(&self, value: T);
}

// ❌ These would overlap (coherence violation):
// impl<T> Handler<T> for Data { ... }
// impl<T> Handler<Vec<T>> for Data { ... }
// Because Vec<T> matches T

// ✅ These don't overlap:
impl Handler<i32> for Data {
    fn handle(&self, value: i32) {
        println!("Handling i32: {}", value);
    }
}

impl Handler<String> for Data {
    fn handle(&self, value: String) {
        println!("Handling String: {}", value);
    }
}

// ----------------------------------------------------------------------------
// 5. SEALED TRAIT PATTERN
// ----------------------------------------------------------------------------
// Sealed traits prevent external crates from implementing a trait.
// This is useful for maintaining control over trait implementations.

// Private module with a public trait
mod sealed {
    pub trait Sealed {}
}

// Public trait that requires Sealed supertrait
pub trait Operation: sealed::Sealed {
    fn operate(&self) -> i32;
}

// Types that can implement Operation
pub struct TypeA(i32);
pub struct TypeB(i32);

// Implement sealed trait for allowed types (in same crate)
impl sealed::Sealed for TypeA {}
impl sealed::Sealed for TypeB {}

// Now implement the public trait
impl Operation for TypeA {
    fn operate(&self) -> i32 {
        self.0 * 2
    }
}

impl Operation for TypeB {
    fn operate(&self) -> i32 {
        self.0 + 10
    }
}

// External crates cannot implement Operation because they cannot
// implement the sealed::Sealed supertrait (it's in a private module)

// More complex sealed trait pattern with type parameters
mod sealed_generic {
    pub trait Sealed<T> {}
}

pub trait SafeOperation<T>: sealed_generic::Sealed<T> {
    fn safe_op(&self, value: T) -> T;
}

impl sealed_generic::Sealed<i32> for TypeA {}

impl SafeOperation<i32> for TypeA {
    fn safe_op(&self, value: i32) -> i32 {
        self.0 + value
    }
}

// Real-world example: Sealed Iterator-like trait
mod iter_sealed {
    pub trait Sealed {}
}

pub trait CustomIterator: iter_sealed::Sealed {
    type Item;
    fn next_item(&mut self) -> Option<Self::Item>;
    
    fn count_items(mut self) -> usize 
    where 
        Self: Sized 
    {
        let mut count = 0;
        while self.next_item().is_some() {
            count += 1;
        }
        count
    }
}

pub struct RangeIter {
    current: i32,
    end: i32,
}

impl RangeIter {
    pub fn new(start: i32, end: i32) -> Self {
        RangeIter { current: start, end }
    }
}

impl iter_sealed::Sealed for RangeIter {}

impl CustomIterator for RangeIter {
    type Item = i32;
    
    fn next_item(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}

// ----------------------------------------------------------------------------
// DEMONSTRATION
// ----------------------------------------------------------------------------

fn main() {
    println!("=== SUPERTRAITS ===");
    let article = Article {
        headline: "Rust 2.0 Released".to_string(),
        content: "The Rust team is excited to announce a new version...".to_string(),
    };
    println!("{}", article.summarize());
    
    println!("\n=== BLANKET IMPLEMENTATIONS ===");
    demonstrate_blanket();
    println!("Doubled: {}", 7.double());
    
    println!("\n=== ORPHAN RULES (Newtype Pattern) ===");
    let my_vec = MyVec(vec![1, 2, 3]);
    println!("MyVec: {}", my_vec);
    
    println!("\n=== COHERENCE ===");
    let data = Data(42);
    println!("{}", data.process());
    println!("{}", data.compute(8));
    println!("{}", data.compute("test".to_string()));
    
    println!("\n=== SEALED TRAITS ===");
    let type_a = TypeA(5);
    let type_b = TypeB(3);
    println!("TypeA operation: {}", type_a.operate());
    println!("TypeB operation: {}", type_b.operate());
    
    let mut range_iter = RangeIter::new(0, 5);
    println!("Custom iterator count: {}", range_iter.count_items());
}

// ============================================================================
// KEY TAKEAWAYS
// ============================================================================
//
// 1. SUPERTRAITS: Define trait dependencies, enabling trait composition
//
// 2. BLANKET IMPLEMENTATIONS: Provide implementations for entire categories
//    of types, increasing code reuse
//
// 3. ORPHAN RULES: Prevent conflicting implementations across crates by
//    requiring local trait or type (use newtype pattern as workaround)
//
// 4. COHERENCE: Guarantees exactly one implementation per type-trait pair,
//    ensuring predictable behavior
//
// 5. SEALED TRAITS: Control trait implementation scope by preventing external
//    crates from implementing your traits
// ============================================================================
```