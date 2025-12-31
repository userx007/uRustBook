# Rust Compiler Internals: A Deep Dive

## Key Topics Covered:

**1. Compilation Stages**: The journey from source code through AST, HIR, MIR, to machine code

**2. HIR (High-Level IR)**: The desugared, name-resolved representation used for type checking and trait resolution

**3. MIR (Mid-Level IR)**: The control flow graph representation with:
   - Basic blocks and explicit control flow
   - How it enables precise borrow checking
   - Optimization opportunities

**4. Borrow Checker Algorithm**: 
   - Non-Lexical Lifetimes (NLL)
   - The three-phase process: constraint generation, region computation, conflict detection
   - Real examples of what fails and why

**5. Trait Resolution**:
   - How the compiler selects trait implementations
   - Associated types and generic resolution
   - Coherence rules and ambiguity handling

**6. Reading Compiler Errors**:
   - Anatomy of error messages
   - Common error codes (E0382, E0499, E0502, E0597)
   - How to use error explanations effectively

**7. Practical Tools**:
   - Viewing MIR output
   - Using `cargo-expand`, `miri`, and other debugging tools

The guide includes numerous code examples showing both what works and what doesn't, with explanations of the compiler's reasoning.

# Rust Compiler Internals: A Deep Dive

## Overview of Rust Compilation Stages

The Rust compiler (rustc) transforms your source code through several intermediate representations before producing machine code:

```
Source Code → Tokens → AST → HIR → MIR → LLVM IR → Machine Code
```

Each stage serves a specific purpose and enables different analyses and optimizations.

## 1. High-Level Intermediate Representation (HIR)

### What is HIR?

HIR is created after parsing and macro expansion. It's a more compiler-friendly version of the Abstract Syntax Tree (AST) that:

- Desugars syntactic sugar (for loops → loop with iterators)
- Resolves names and imports
- Type checks expressions
- Still maintains structure close to source code

### HIR Example

**Source Code:**
```rust
fn main() {
    for i in 0..5 {
        println!("{}", i);
    }
}
```

**Conceptual HIR (simplified):**
```rust
fn main() {
    {
        let mut iter = IntoIterator::into_iter(Range { start: 0, end: 5 });
        loop {
            match Iterator::next(&mut iter) {
                Some(i) => {
                    println!("{}", i);
                }
                None => break,
            }
        }
    }
}
```

### Key HIR Responsibilities

- **Name Resolution**: Connecting identifiers to their definitions
- **Type Checking**: Ensuring type correctness
- **Trait Resolution**: Determining which trait implementations to use
- **Lifetime Elision**: Adding implicit lifetimes

## 2. Mid-Level Intermediate Representation (MIR)

### What is MIR?

MIR is a Control Flow Graph (CFG) based representation that:

- Uses basic blocks and explicit control flow
- Has no nested expressions (everything is flattened)
- Makes borrowing and lifetime analysis straightforward
- Enables sophisticated optimizations

### MIR Structure

MIR consists of:
- **Basic Blocks**: Sequences of statements ending in a terminator
- **Statements**: Assignments, storage operations
- **Terminators**: Control flow (goto, return, call, switch)
- **Places**: Locations in memory (variables, fields, derefs)
- **Rvalues**: Values being computed

### MIR Example

**Source Code:**
```rust
fn add_one(x: i32) -> i32 {
    let y = x + 1;
    y
}
```

**Conceptual MIR:**
```
fn add_one(_1: i32) -> i32 {
    let mut _0: i32;        // return place
    let mut _2: i32;        // temporary
    
    bb0: {
        _2 = _1;            // copy x
        _0 = Add(_2, const 1i32);  // x + 1
        return;             // return _0
    }
}
```

### Why MIR Matters

1. **Borrow Checking**: MIR's explicit control flow makes it easier to track borrows across all possible execution paths
2. **Optimizations**: Dead code elimination, constant propagation, inlining
3. **Debugging**: Clearer representation for analyzing program behavior
4. **Safety Checks**: Uninitialized variables, moved values

## 3. The Borrow Checker Algorithm

### Core Principles

The borrow checker enforces Rust's ownership rules at compile time:

1. Each value has a single owner
2. You can have either one mutable reference OR multiple immutable references
3. References must always be valid

### Non-Lexical Lifetimes (NLL)

Modern Rust uses NLL, which operates on MIR and determines borrow lifetimes based on actual usage, not just lexical scope.

### Borrow Checker Process

1. **Loan Analysis**: Track where borrows begin and end
2. **Path Analysis**: Determine which memory locations are accessed
3. **Liveness Analysis**: Find where variables are actually used
4. **Conflict Detection**: Check for simultaneous incompatible borrows

### Example: Borrow Checker in Action

**This fails:**
```rust
fn main() {
    let mut vec = vec![1, 2, 3];
    let first = &vec[0];        // immutable borrow
    vec.push(4);                // ERROR: mutable borrow
    println!("{}", first);
}
```

**Error explanation:**
- `&vec[0]` creates an immutable borrow of `vec`
- `vec.push(4)` needs a mutable borrow of `vec`
- These borrows conflict because `first` is still used later

**This works (NLL):**
```rust
fn main() {
    let mut vec = vec![1, 2, 3];
    let first = &vec[0];        // immutable borrow
    println!("{}", first);      // last use of 'first'
    vec.push(4);                // OK: immutable borrow ended
}
```

### Borrow Checker Algorithm Phases

**Phase 1: Generate Constraints**
```rust
let mut x = 5;
let r1 = &x;      // 'r1 lifetime starts
let r2 = &x;      // 'r2 lifetime starts
println!("{} {}", r1, r2);  // both used here
// 'r1 and 'r2 lifetimes end
```

**Phase 2: Compute Loan Regions**
- Track each borrow's region (set of program points where it's live)
- Region = from borrow creation to last use

**Phase 3: Check Conflicts**
- For each use of borrowed data, ensure no conflicting borrows exist
- Mutable + immutable = conflict
- Mutable + mutable = conflict
- Immutable + immutable = OK

### Complex Example: Polonius

Rust is moving toward "Polonius," a next-generation borrow checker that's even more precise:

```rust
fn get_default<'a>(map: &'a mut HashMap<i32, String>, key: i32) 
    -> &'a mut String 
{
    match map.get_mut(&key) {
        Some(value) => value,
        None => {
            map.insert(key, String::new());
            map.get_mut(&key).unwrap()
        }
    }
}
```

This pattern (checking then inserting) is rejected by current NLL but accepted by Polonius because it can prove the borrows don't actually conflict.

## 4. Trait Resolution

### What is Trait Resolution?

Trait resolution determines which trait implementation to use for generic code. It happens during HIR type checking.

### Resolution Process

1. **Candidate Collection**: Find all possible implementations
2. **Candidate Selection**: Choose the most specific implementation
3. **Confirmation**: Verify the selected implementation is valid

### Example: Basic Trait Resolution

```rust
trait Drawable {
    fn draw(&self);
}

struct Circle;
struct Square;

impl Drawable for Circle {
    fn draw(&self) { println!("Drawing circle"); }
}

impl Drawable for Square {
    fn draw(&self) { println!("Drawing square"); }
}

fn render<T: Drawable>(shape: &T) {
    shape.draw();  // Compiler resolves which impl to use
}

fn main() {
    let c = Circle;
    let s = Square;
    render(&c);  // Resolves to Circle's impl
    render(&s);  // Resolves to Square's impl
}
```

### Trait Resolution with Associated Types

```rust
trait Container {
    type Item;
    fn get(&self) -> &Self::Item;
}

impl Container for Vec<i32> {
    type Item = i32;
    fn get(&self) -> &i32 { &self[0] }
}

fn first<C: Container>(container: &C) -> &C::Item {
    container.get()
}
```

The compiler must resolve:
- Which `Container` impl to use
- What `Item` type corresponds to
- Which `get` method to call

### Trait Resolution Ambiguity

**Ambiguous case:**
```rust
trait A {
    fn foo(&self);
}

trait B {
    fn foo(&self);
}

struct S;
impl A for S { fn foo(&self) { println!("A"); } }
impl B for S { fn foo(&self) { println!("B"); } }

fn main() {
    let s = S;
    s.foo();  // ERROR: ambiguous
    A::foo(&s);  // OK: explicit
}
```

### Coherence Rules

Rust enforces coherence (no overlapping impls) to make trait resolution deterministic:

```rust
trait MyTrait {}

// This would be an error if uncommented:
// impl<T> MyTrait for T {}
// impl MyTrait for i32 {}  // Overlaps with above!
```

## 5. Reading Compiler Errors

### Anatomy of a Rust Error

```rust
error[E0502]: cannot borrow `vec` as mutable because it is also borrowed as immutable
  --> src/main.rs:4:5
   |
3  |     let first = &vec[0];
   |                  --- immutable borrow occurs here
4  |     vec.push(4);
   |     ^^^^^^^^^^^ mutable borrow occurs here
5  |     println!("{}", first);
   |                    ----- immutable borrow later used here
```

**Components:**
- **Error Code** (E0502): Searchable in error index
- **Message**: What went wrong
- **Location**: File, line, column
- **Context**: Visual representation with carets/underlines
- **Explanation**: Why the error occurred

### Common Error Patterns

**E0382: Use of Moved Value**
```rust
let s1 = String::from("hello");
let s2 = s1;
println!("{}", s1);  // ERROR: value used after move
```

**Fix:** Clone or use references
```rust
let s1 = String::from("hello");
let s2 = s1.clone();
println!("{}", s1);  // OK
```

**E0499: Cannot Borrow as Mutable More Than Once**
```rust
let mut s = String::from("hello");
let r1 = &mut s;
let r2 = &mut s;  // ERROR
```

**Fix:** Ensure borrows don't overlap
```rust
let mut s = String::from("hello");
{
    let r1 = &mut s;
} // r1 dropped here
let r2 = &mut s;  // OK
```

**E0597: Borrowed Value Does Not Live Long Enough**
```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // ERROR: s dropped at end of function
}
```

**Fix:** Return owned value
```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // ownership transferred
}
```

### Using Error Explanations

```bash
rustc --explain E0502
```

Provides detailed documentation about the error, including examples and solutions.

### Reading Complex Errors

**Lifetime errors:**
```rust
struct Holder<'a> {
    data: &'a str,
}

impl<'a> Holder<'a> {
    fn get_data(&self) -> &str {
        self.data
    }
}

fn process() -> &str {
    let holder = Holder { data: "test" };
    holder.get_data()  // ERROR: returns reference to local
}
```

**Error message will show:**
- Lifetime annotations
- Where data is created
- Where reference escapes
- Suggested fixes

## 6. Debugging with MIR

### Viewing MIR Output

```bash
rustc +nightly -Z dump-mir=all example.rs
```

This generates `.mir` files showing MIR at various optimization stages.

### MIR Flags

```bash
RUSTFLAGS="-Z dump-mir-graphviz" cargo build
# Generates GraphViz .dot files of control flow
```

### Example: Analyzing Optimization

**Source:**
```rust
fn square(x: i32) -> i32 {
    x * x
}

fn main() {
    let result = square(4);
    println!("{}", result);
}
```

At MIR level, you can see:
- Constant propagation (4 * 4 = 16 at compile time)
- Inlining of `square` into `main`
- Dead code elimination

## 7. Practical Tips

### Understanding Compilation Errors

1. **Read the full error message**: Don't just look at the first line
2. **Check the error code**: Use `rustc --explain EXXXX`
3. **Look at the annotations**: Carets show exactly what's wrong
4. **Read suggestions**: Compiler often provides fixes
5. **Understand the why**: Don't just apply fixes blindly

### Working with the Borrow Checker

1. **Think in terms of ownership**: Who owns what?
2. **Minimize borrow scope**: Use borrows only as long as needed
3. **Use blocks** to end borrows early
4. **Consider cloning**: For small types, cloning may be clearer than complex borrowing
5. **Understand the difference**: Moves vs. copies vs. borrows

### Trait Resolution Tips

1. **Use explicit paths**: `Trait::method()` when ambiguous
2. **Check trait bounds**: Ensure all constraints are satisfied
3. **Use where clauses**: For complex bounds
4. **Consider trait objects**: `dyn Trait` for runtime polymorphism

## 8. Advanced Tools

### Cargo Expand

View macro-expanded code:
```bash
cargo install cargo-expand
cargo expand
```

### Miri

Experimental MIR interpreter for detecting undefined behavior:
```bash
cargo install cargo-miri
cargo miri test
```

### Chalk

Next-generation trait solver (being integrated into rustc):
- More powerful trait resolution
- Better error messages
- Handles complex scenarios

## Conclusion

Understanding Rust's compiler internals helps you:
- Write better, more idiomatic code
- Debug issues faster
- Understand why code compiles or doesn't
- Appreciate the guarantees Rust provides
- Contribute to the compiler itself

The compiler is your friend—it catches bugs at compile time that would be runtime errors in other languages. Learning to read and understand its messages is key to mastering Rust.