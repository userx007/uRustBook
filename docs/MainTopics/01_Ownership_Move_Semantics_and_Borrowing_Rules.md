# Ownership, Move Semantics, and Borrowing Rules in Rust

Rust's ownership system is its most distinctive feature, enabling memory safety without garbage collection. Let me break down these interconnected concepts:

## **1. Ownership Rules**

Rust enforces three fundamental ownership rules at compile time:

1. **Each value has a single owner** (a variable that holds it)
2. **There can only be one owner at a time**
3. **When the owner goes out of scope, the value is dropped** (memory is freed)

### Basic Example:

```rust
fn main() {
    let s1 = String::from("hello"); // s1 owns the String
    
    // When s1 goes out of scope here, the String is dropped
}
```

## **2. Move Semantics**

When you assign a value to another variable or pass it to a function, **ownership moves** by default for non-Copy types.

### Move in Assignment:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // Ownership moves from s1 to s2
    
    // println!("{}", s1); // ERROR! s1 is no longer valid
    println!("{}", s2);    // OK: s2 is the owner now
}
```

### Move in Function Calls:

```rust
fn main() {
    let s = String::from("hello");
    takes_ownership(s); // s is moved into the function
    
    // println!("{}", s); // ERROR! s is no longer valid
}

fn takes_ownership(some_string: String) {
    println!("{}", some_string);
} // some_string goes out of scope and is dropped here
```

### Returning Ownership:

```rust
fn main() {
    let s1 = gives_ownership();         // Function returns ownership
    let s2 = String::from("hello");
    let s3 = takes_and_gives_back(s2);  // s2 moved in, ownership returned
    
    // s1 and s3 are valid, s2 is not
}

fn gives_ownership() -> String {
    String::from("yours")
}

fn takes_and_gives_back(a_string: String) -> String {
    a_string // Ownership moved out to caller
}
```

## **3. Copy vs Move Semantics**

Some types implement the **`Copy` trait**, which means they're copied instead of moved. These are typically simple types stored on the stack.

### Copy Types:
- All integers (`i32`, `u64`, etc.)
- Booleans (`bool`)
- Floats (`f32`, `f64`)
- Characters (`char`)
- Tuples containing only Copy types

```rust
fn main() {
    let x = 5;
    let y = x; // x is copied, not moved
    
    println!("x = {}, y = {}", x, y); // Both valid! Copy semantics
    
    // Same with function calls
    makes_copy(x);
    println!("x is still valid: {}", x);
}

fn makes_copy(some_integer: i32) {
    println!("{}", some_integer);
}
```

## **4. Borrowing: References Without Ownership**

Borrowing allows you to **refer to a value without taking ownership**. This is done using references (`&`).

### Immutable References (`&T`):

```rust
fn main() {
    let s1 = String::from("hello");
    
    let len = calculate_length(&s1); // Borrow s1
    
    println!("The length of '{}' is {}.", s1, len); // s1 still valid!
}

fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope, but since it doesn't own the data, nothing happens
```

### Mutable References (`&mut T`):

```rust
fn main() {
    let mut s = String::from("hello");
    
    change(&mut s); // Mutable borrow
    
    println!("{}", s); // Prints "hello, world"
}

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

## **5. The Borrowing Rules**

Rust enforces strict borrowing rules to prevent data races and ensure memory safety:

### **Rule 1: You can have EITHER multiple immutable references OR one mutable reference**

```rust
fn main() {
    let mut s = String::from("hello");
    
    // Multiple immutable references - OK
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);
    
    // One mutable reference - OK (after immutable refs are done)
    let r3 = &mut s;
    r3.push_str(" world");
}
```

### **Rule Violation Example:**

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s;     // Immutable borrow
    let r2 = &s;     // Another immutable borrow - OK
    let r3 = &mut s; // ERROR! Cannot borrow as mutable while immutable borrows exist
    
    println!("{}, {}, and {}", r1, r2, r3);
}
```

### **Rule 2: References must always be valid (no dangling references)**

```rust
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String { // ERROR! This function tries to return a reference
    let s = String::from("hello");
    &s // s goes out of scope, so this reference would be invalid
} // Fix: Return the String itself, transferring ownership
```

## **6. Reference Scope and Non-Lexical Lifetimes (NLL)**

Modern Rust (2018 edition+) uses **Non-Lexical Lifetimes**, where a reference's scope ends at its last use:

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point
    
    let r3 = &mut s; // OK! Immutable borrows have ended
    println!("{}", r3);
}
```

## **7. Common Patterns and Best Practices**

### Pattern 1: Borrowing for Read-Only Access

```rust
fn print_vector(v: &Vec<i32>) {
    for num in v {
        println!("{}", num);
    }
}

fn main() {
    let vec = vec![1, 2, 3];
    print_vector(&vec); // Borrow
    print_vector(&vec); // Can borrow again
    println!("Still own it: {:?}", vec);
}
```

### Pattern 2: Mutable Borrowing for Modification

```rust
fn append_number(v: &mut Vec<i32>, num: i32) {
    v.push(num);
}

fn main() {
    let mut vec = vec![1, 2, 3];
    append_number(&mut vec, 4);
    println!("{:?}", vec); // [1, 2, 3, 4]
}
```

### Pattern 3: Cloning When You Need Independent Copies

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // Explicit deep copy
    
    println!("s1 = {}, s2 = {}", s1, s2); // Both valid
}
```

## **8. Why These Rules Matter**

These rules prevent:
- **Data races**: No simultaneous mutable and immutable access
- **Use-after-free**: References are always valid
- **Double-free**: Only one owner, freed only once
- **Iterator invalidation**: Can't modify while iterating

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    
    for i in &v {
        // v.push(4); // ERROR! Can't mutate while iterating
        println!("{}", i);
    }
    
    // Now we can mutate
    v.push(4);
}
```

## **Summary**

- **Ownership**: Each value has one owner; dropped when owner goes out of scope
- **Move**: Default for heap-allocated types (String, Vec, etc.)
- **Copy**: Automatic for stack-only types (integers, bools, etc.)
- **Borrowing**: References allow access without ownership transfer
- **Immutable refs**: Multiple allowed, no mutation
- **Mutable refs**: Only one at a time, exclusive access
- **Safety**: All enforced at compile time with zero runtime cost!

This system is what makes Rust both safe and fast—catching memory errors at compile time without needing garbage collection.