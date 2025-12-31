# Guide to advanced Rust type system concepts

## **1. Zero-Sized Types (ZSTs)**
ZSTs are types that occupy zero memory but provide compile-time type information. Common examples:
- **Unit type `()`** - the most basic ZST
- **Empty structs** - useful as marker types
- **PhantomData** - for type-level programming

**Key use case**: Type-state pattern where the compiler enforces state machine transitions (like the `Door<Locked>` / `Door<Unlocked>` example).

## **2. PhantomData**
A special ZST that tells the compiler "act as if this struct owns/uses type T" without actually storing it. Essential for:
- **Unused type parameters** - when a type parameter doesn't appear in fields
- **Variance control** - managing lifetime and type relationships
- **Drop check** - ensuring proper drop semantics
- **Type-level programming** - like the `Distance<Kilometers>` example preventing unit mix-ups

## **3. Unsized Types (DSTs)**
Types whose size isn't known at compile time. You can only use them behind pointers/references:
- **`[T]`** - slice type (accessed as `&[T]`)
- **`str`** - string slice (accessed as `&str`)
- **`dyn Trait`** - trait objects

The **`?Sized`** bound allows working with both sized and unsized types in generic functions.

## **4. Type Aliases**
Simple names for existing types using the `type` keyword. They don't create new types - they're just convenient names. Useful for:
- Improving code readability (`type UserId = u64`)
- Simplifying complex types (`type Result<T> = std::result::Result<T, Error>`)
- Reducing verbosity in generic code

## **5. Newtype Pattern**
Wrapping existing types in single-field tuple structs to create **distinct types**. This provides:
- **Type safety** - prevents mixing up similar types (like `Email` vs `Password`)
- **Trait implementations** - implement traits on external types
- **Domain modeling** - encode invariants in the type system
- **Zero-cost abstraction** - the wrapper is optimized away at runtime

The key difference from type aliases: newtypes create **new, incompatible types**, while aliases are just alternate names.

You can compile and run this code with `cargo run` to see all the examples in action!

```rust
// ============================================================================
// RUST TYPE SYSTEM DEEP DIVE
// ============================================================================

// ----------------------------------------------------------------------------
// 1. ZERO-SIZED TYPES (ZSTs)
// ----------------------------------------------------------------------------
// ZSTs are types that occupy zero bytes of memory. The compiler completely
// optimizes them away at runtime, but they still provide type-level information.

use std::marker::PhantomData;
use std::mem::size_of;

// Unit type () is the most common ZST
fn demonstrate_unit_type() {
    println!("Size of (): {} bytes", size_of::<()>());
    // Output: 0 bytes
}

// Empty structs are ZSTs
struct EmptyStruct;
struct EmptyTupleStruct();
struct EmptyBracedStruct {}

// Arrays of ZSTs are also ZSTs
fn zst_arrays() {
    let arr: [(); 1000000] = [(); 1000000];
    println!("Size of [(); 1000000]: {} bytes", size_of::<[(); 1000000]>());
    // Output: 0 bytes - even a million unit types take no space!
}

// Practical use: Marker types for compile-time state tracking
struct Locked;
struct Unlocked;

struct Door<State> {
    // PhantomData here (explained later)
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        println!("Door created in locked state");
        Door { _state: PhantomData }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        println!("Door unlocked!");
        Door { _state: PhantomData }
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

// This won't compile: door.open() only works on Door<Unlocked>
fn zst_state_machine() {
    let door = Door::<Locked>::new();
    // door.open(); // ERROR: method not found
    let door = door.unlock();
    door.open(); // OK!
}

// ----------------------------------------------------------------------------
// 2. PHANTOM DATA
// ----------------------------------------------------------------------------
// PhantomData<T> is a ZST that tells the compiler to act as if the struct
// owns a value of type T, without actually storing it.

// Use case 1: Unused type parameters
struct Slice<'a, T> {
    start: *const T,
    end: *const T,
    // Without PhantomData, compiler doesn't know about lifetime 'a and type T
    _marker: PhantomData<&'a T>,
}

// Use case 2: Variance and drop check
struct MyBox<T> {
    ptr: *const T,
    // Tells compiler this struct "owns" T for drop checking
    _marker: PhantomData<T>,
}

// Use case 3: Type-level programming
struct Kilometers;
struct Miles;

struct Distance<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

impl Distance<Kilometers> {
    fn new(value: f64) -> Self {
        Distance { value, _unit: PhantomData }
    }
    
    fn to_miles(self) -> Distance<Miles> {
        Distance {
            value: self.value * 0.621371,
            _unit: PhantomData,
        }
    }
}

impl Distance<Miles> {
    fn to_kilometers(self) -> Distance<Kilometers> {
        Distance {
            value: self.value * 1.60934,
            _unit: PhantomData,
        }
    }
}

fn phantom_data_example() {
    let dist_km = Distance::<Kilometers>::new(100.0);
    println!("100 km = {} miles", dist_km.to_miles().value);
    
    // Type safety: can't accidentally mix units
    // let mixed = dist_km.value + Distance::<Miles>::new(10.0).value; // Different types!
}

// ----------------------------------------------------------------------------
// 3. UNSIZED TYPES (DYNAMICALLY SIZED TYPES - DSTs)
// ----------------------------------------------------------------------------
// DSTs are types whose size is not known at compile time.
// You can only interact with them through references or pointers.

// Common DSTs:
// - [T] (slices)
// - str (string slices)
// - dyn Trait (trait objects)

fn dst_examples() {
    // [T] - slice type (unsized)
    let array = [1, 2, 3, 4, 5];
    let slice: &[i32] = &array[..]; // &[T] is sized (fat pointer: ptr + len)
    println!("Slice length: {}", slice.len());
    
    // str - string slice (unsized)
    let string = String::from("Hello");
    let str_slice: &str = &string[..]; // &str is sized (fat pointer)
    println!("String slice: {}", str_slice);
    
    // We can't do this:
    // let x: [i32]; // ERROR: size cannot be known at compile-time
    // let y: str;   // ERROR: size cannot be known at compile-time
}

// Trait objects are DSTs
trait Animal {
    fn make_sound(&self) -> &str;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn make_sound(&self) -> &str { "Woof!" }
}

impl Animal for Cat {
    fn make_sound(&self) -> &str { "Meow!" }
}

fn trait_objects() {
    let dog = Dog;
    let cat = Cat;
    
    // dyn Animal is a DST
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(dog),
        Box::new(cat),
    ];
    
    for animal in &animals {
        println!("{}", animal.make_sound());
    }
}

// Custom DST with ?Sized bound
fn work_with_dst<T: ?Sized>(value: &T) {
    // ?Sized means "T may or may not be Sized"
    // Without ?Sized, T must be Sized
    println!("Size of &T: {} bytes", size_of::<&T>());
}

fn dst_bounds() {
    let sized_value = 42;
    let slice: &[i32] = &[1, 2, 3];
    
    work_with_dst(&sized_value); // T = i32 (Sized)
    work_with_dst(slice);         // T = [i32] (Unsized)
}

// ----------------------------------------------------------------------------
// 4. TYPE ALIASES
// ----------------------------------------------------------------------------
// Type aliases create a new name for an existing type.

// Basic type alias
type Kilometers = f64;
type Result<T> = std::result::Result<T, String>;

// Complex type aliases for readability
type UserId = u64;
type UserName = String;
type UserMap = std::collections::HashMap<UserId, UserName>;

fn type_alias_basics() {
    let distance: Kilometers = 42.0; // More meaningful than just f64
    let users: UserMap = UserMap::new();
    
    // Note: type aliases don't create new types, just new names
    let km: Kilometers = 10.0;
    let meters: f64 = 20.0;
    let sum = km + meters; // This works - they're both f64
    println!("Sum: {}", sum);
}

// Generic type aliases
type NodeBox<T> = Box<Node<T>>;

struct Node<T> {
    value: T,
    next: Option<NodeBox<T>>,
}

// Associated type aliases
trait Container {
    type Item;
    type Iter;
}

// Existential types (impl Trait in type aliases) - unstable feature
// type ReturnIterator = impl Iterator<Item = i32>;

// ----------------------------------------------------------------------------
// 5. NEWTYPE PATTERN
// ----------------------------------------------------------------------------
// The newtype pattern wraps existing types in a tuple struct to create
// distinct types at compile time, providing type safety.

// Basic newtype
struct Email(String);
struct Password(String);

impl Email {
    fn new(email: String) -> Result<Self, &'static str> {
        if email.contains('@') {
            Ok(Email(email))
        } else {
            Err("Invalid email")
        }
    }
    
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Password {
    fn new(password: String) -> Result<Self, &'static str> {
        if password.len() >= 8 {
            Ok(Password(password))
        } else {
            Err("Password too short")
        }
    }
}

// Type safety with newtypes
fn send_email(email: &Email, password: &Password) {
    println!("Sending email to: {}", email.as_str());
}

fn newtype_safety() {
    let email = Email::new("user@example.com".to_string()).unwrap();
    let password = Password::new("secret123".to_string()).unwrap();
    
    send_email(&email, &password);
    // send_email(&password, &email); // ERROR: mismatched types!
}

// Newtype for implementing external traits on external types
struct MyVec(Vec<i32>);

impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyVec[{}]", self.0.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", "))
    }
}

// Newtype for additional type safety (units)
struct Meters(f64);
struct Seconds(f64);
struct MetersPerSecond(f64);

impl std::ops::Div<Seconds> for Meters {
    type Output = MetersPerSecond;
    
    fn div(self, rhs: Seconds) -> MetersPerSecond {
        MetersPerSecond(self.0 / rhs.0)
    }
}

fn newtype_units() {
    let distance = Meters(100.0);
    let time = Seconds(10.0);
    let speed = distance / time;
    
    println!("Speed: {} m/s", speed.0);
    
    // Can't accidentally mix up units:
    // let wrong = distance / distance; // ERROR: no implementation for Meters / Meters
}

// Newtype for hiding implementation details
pub struct Isbn(String);

impl Isbn {
    pub fn new(isbn: String) -> Result<Self, &'static str> {
        // Validation logic
        if isbn.len() == 13 {
            Ok(Isbn(isbn))
        } else {
            Err("ISBN must be 13 digits")
        }
    }
    
    // Public API doesn't expose inner String directly
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Newtype with Deref for transparent access
use std::ops::Deref;

struct Username(String);

impl Deref for Username {
    type Target = String;
    
    fn deref(&self) -> &String {
        &self.0
    }
}

fn newtype_deref() {
    let username = Username("alice".to_string());
    // Can use String methods directly via Deref coercion
    println!("Username length: {}", username.len());
    println!("Uppercase: {}", username.to_uppercase());
}

// ----------------------------------------------------------------------------
// MAIN FUNCTION - Demonstrates all concepts
// ----------------------------------------------------------------------------

fn main() {
    println!("=== Zero-Sized Types ===");
    demonstrate_unit_type();
    zst_arrays();
    println!("Size of EmptyStruct: {} bytes", size_of::<EmptyStruct>());
    zst_state_machine();
    
    println!("\n=== Phantom Data ===");
    phantom_data_example();
    println!("Size of PhantomData<String>: {} bytes", size_of::<PhantomData<String>>());
    
    println!("\n=== Unsized Types (DSTs) ===");
    dst_examples();
    trait_objects();
    dst_bounds();
    
    println!("\n=== Type Aliases ===");
    type_alias_basics();
    
    println!("\n=== Newtype Pattern ===");
    newtype_safety();
    
    let my_vec = MyVec(vec![1, 2, 3, 4, 5]);
    println!("Display: {}", my_vec);
    
    newtype_units();
    newtype_deref();
    
    println!("\n=== Advanced Example: Combining Concepts ===");
    advanced_example();
}

// Advanced example combining multiple concepts
struct TypedBuffer<T, State> {
    data: Vec<u8>,
    _type: PhantomData<T>,
    _state: PhantomData<State>,
}

struct Initialized;
struct Uninitialized;

impl<T> TypedBuffer<T, Uninitialized> {
    fn new(capacity: usize) -> Self {
        TypedBuffer {
            data: Vec::with_capacity(capacity),
            _type: PhantomData,
            _state: PhantomData,
        }
    }
    
    fn initialize(mut self, value: T) -> TypedBuffer<T, Initialized> 
    where
        T: Clone,
    {
        // Simplified initialization
        TypedBuffer {
            data: self.data,
            _type: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<T> TypedBuffer<T, Initialized> {
    fn len(&self) -> usize {
        self.data.len()
    }
}

fn advanced_example() {
    let buffer = TypedBuffer::<i32, Uninitialized>::new(100);
    // buffer.len(); // ERROR: method not found in this state
    
    let buffer = buffer.initialize(42);
    println!("Buffer size: {}", buffer.len()); // OK!
}
```