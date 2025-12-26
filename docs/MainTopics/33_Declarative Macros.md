# Declarative Macros in Rust (macro_rules!)

I'll create a comprehensive guide covering all aspects of declarative macros with practical examples.I've created a comprehensive guide to Rust's declarative macros. Here's an overview of the key concepts:

## **What are Declarative Macros?**

Declarative macros (`macro_rules!`) are code generators that work by pattern matching. They let you write code that writes code, reducing repetition and creating domain-specific syntax.

## **Core Concepts:**

### **1. Pattern Matching Syntax**
Macros match input against patterns, similar to `match` expressions. Each pattern has an arm with `=>` that defines the expansion.

### **2. Fragment Specifiers**
These define what kind of syntax each captured part accepts:
- `expr` - expressions (most common)
- `ident` - identifiers (variable/function names)
- `ty` - types
- `stmt` - statements
- `pat` - patterns
- `block` - code blocks
- `item` - items (functions, structs, etc.)
- `tt` - token trees (any single token)

### **3. Repetition**
- `$(...)*` - zero or more repetitions
- `$(...)+` - one or more repetitions  
- `$(...)?` - zero or one repetition
- Can include separators: `$(...),*`

### **4. Hygiene**
Rust macros are hygienic - they don't accidentally capture variables from the surrounding scope. Variables created in a macro exist in their own scope.

### **5. Debugging Techniques**
- Use `stringify!()` to see what the macro receives
- Use `compile_error!()` for custom compile-time errors
- Nightly features: `trace_macros!()` and `log_syntax!()`

The artifact includes practical examples like custom assertions, builder patterns, enum generators, and advanced patterns like TT munchers and internal rules. These patterns form the foundation for powerful compile-time metaprogramming in Rust!

```rust
// ============================================
// DECLARATIVE MACROS IN RUST (macro_rules!)
// ============================================

// 1. BASIC SYNTAX AND PATTERN MATCHING
// =====================================

// Simple macro with no arguments
macro_rules! hello {
    () => {
        println!("Hello, Rust macros!");
    };
}

// Macro with single pattern
macro_rules! create_number {
    ($val:expr) => {
        $val
    };
}

// Multiple patterns (pattern matching)
macro_rules! calculate {
    // Pattern 1: Addition
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    // Pattern 2: Subtraction
    (sub $a:expr, $b:expr) => {
        $a - $b
    };
    // Pattern 3: Multiplication
    (mul $a:expr, $b:expr) => {
        $a * $b
    };
}

// 2. FRAGMENT SPECIFIERS
// =======================
// Fragment specifiers determine what kind of syntax the macro accepts

macro_rules! fragment_examples {
    // expr: expressions (most common)
    ($e:expr) => { println!("Expression: {}", $e); };
    
    // ident: identifiers (variable/function names)
    ($i:ident) => { 
        let $i = 42;
        println!("Identifier created: {}", $i);
    };
    
    // ty: types
    ($t:ty) => { 
        let _var: $t = Default::default();
        println!("Type: {}", stringify!($t));
    };
    
    // stmt: statements
    ($s:stmt) => { $s };
    
    // pat: patterns
    ($p:pat) => { 
        match Some(5) {
            $p => println!("Pattern matched!"),
            _ => println!("No match"),
        }
    };
    
    // block: block expressions
    ($b:block) => { $b };
    
    // item: items (functions, structs, etc.)
    ($it:item) => { $it };
    
    // meta: attribute contents
    ($m:meta) => { };
    
    // tt: token tree (any single token)
    ($tt:tt) => { println!("Token: {}", stringify!($tt)); };
}

// 3. REPETITION PATTERNS
// =======================

// Simple repetition: zero or more
macro_rules! vec_creation {
    ($($element:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push($element);
            )*
            v
        }
    };
}

// Repetition with separator
macro_rules! hash_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

// One or more repetition (+)
macro_rules! sum {
    ($first:expr $(, $rest:expr)+) => {
        $first $(+ $rest)+
    };
}

// Nested repetition
macro_rules! matrix {
    ($([$($element:expr),*]),*) => {
        vec![
            $(
                vec![$($element),*]
            ),*
        ]
    };
}

// 4. ADVANCED PATTERN MATCHING
// =============================

// Matching different types of input
macro_rules! smart_print {
    // No arguments
    () => {
        println!("Nothing to print!");
    };
    // Single expression
    ($val:expr) => {
        println!("{}", $val);
    };
    // Format string with arguments
    ($fmt:expr, $($arg:expr),+) => {
        println!($fmt, $($arg),+);
    };
}

// Recursive macros
macro_rules! count_items {
    () => { 0 };
    ($head:tt $($tail:tt)*) => {
        1 + count_items!($($tail)*)
    };
}

// 5. MACRO HYGIENE
// ================
// Rust macros are hygienic: they don't accidentally capture variables

macro_rules! hygienic_example {
    () => {
        let x = "macro's x";
        println!("Inside macro: {}", x);
    };
}

// This demonstrates that macros create their own scope
macro_rules! using_dollar_crate {
    ($name:ident) => {
        // $crate refers to the crate where the macro is defined
        pub struct $name {
            data: Vec<i32>,
        }
        
        impl $name {
            pub fn new() -> Self {
                Self { data: Vec::new() }
            }
        }
    };
}

// 6. MACRO DEBUGGING
// ==================

// Using trace_macros! (nightly only)
// #![feature(trace_macros)]
// trace_macros!(true);
// some_macro!();
// trace_macros!(false);

// Using log_syntax! (nightly only)
// log_syntax!(This will be printed at compile time);

// Using stringify! to debug
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        {
            println!("Macro input: {}", stringify!($($tt)*));
            // actual implementation
        }
    };
}

// Using compile_error! for custom errors
macro_rules! check_value {
    (0) => {
        compile_error!("Value cannot be zero!");
    };
    ($val:expr) => {
        $val
    };
}

// 7. PRACTICAL EXAMPLES
// ======================

// Custom assert macro
macro_rules! assert_near {
    ($left:expr, $right:expr, $epsilon:expr) => {
        {
            let (left_val, right_val, eps) = ($left, $right, $epsilon);
            if (left_val - right_val).abs() > eps {
                panic!(
                    "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n epsilon: `{:?}`",
                    left_val, right_val, eps
                );
            }
        }
    };
}

// Builder pattern macro
macro_rules! builder {
    ($struct_name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        pub struct $struct_name {
            $($field: Option<$type>),*
        }
        
        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: None),*
                }
            }
            
            $(
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = Some(value);
                    self
                }
            )*
            
            pub fn build(self) -> Result<(), &'static str> {
                $(
                    if self.$field.is_none() {
                        return Err(concat!("Missing field: ", stringify!($field)));
                    }
                )*
                Ok(())
            }
        }
    };
}

// Enum with associated values
macro_rules! create_enum {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum $enum_name {
            $($variant),*
        }
        
        impl $enum_name {
            pub fn variants() -> &'static [Self] {
                &[$(Self::$variant),*]
            }
            
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }
        }
    };
}

// Test generator macro
macro_rules! test_suite {
    ($($test_name:ident: $test_expr:expr),* $(,)?) => {
        $(
            #[test]
            fn $test_name() {
                assert!($test_expr);
            }
        )*
    };
}

// 8. USAGE EXAMPLES
// =================

fn main() {
    // Basic usage
    hello!();
    println!("Number: {}", create_number!(42));
    
    // Pattern matching
    println!("Add: {}", calculate!(add 5, 3));
    println!("Sub: {}", calculate!(sub 10, 4));
    println!("Mul: {}", calculate!(mul 6, 7));
    
    // Repetition
    let v = vec_creation!(1, 2, 3, 4, 5);
    println!("Vector: {:?}", v);
    
    let map = hash_map! {
        "name" => "Rust",
        "type" => "Language",
        "year" => "2010"
    };
    println!("HashMap: {:?}", map);
    
    println!("Sum: {}", sum!(1, 2, 3, 4, 5));
    
    // Nested repetition
    let mat = matrix!([1, 2, 3], [4, 5, 6], [7, 8, 9]);
    println!("Matrix: {:?}", mat);
    
    // Smart print
    smart_print!();
    smart_print!("Hello");
    smart_print!("Name: {}, Age: {}", "Alice", 30);
    
    // Count items
    println!("Count: {}", count_items!(a b c d e));
    
    // Hygiene demonstration
    let x = "outer x";
    hygienic_example!();
    println!("Outside macro: {}", x);  // Still "outer x"
    
    // Assert near
    assert_near!(3.14, 3.14159, 0.01);
    
    // Builder pattern
    builder! {
        Config {
            host: String,
            port: u16,
            timeout: u64,
        }
    }
    
    let config = Config::new()
        .host("localhost".to_string())
        .port(8080)
        .timeout(30);
    
    match config.build() {
        Ok(_) => println!("Config built successfully"),
        Err(e) => println!("Error: {}", e),
    }
    
    // Enum creation
    create_enum! {
        Color {
            Red,
            Green,
            Blue,
        }
    }
    
    for color in Color::variants() {
        println!("Color: {}", color.name());
    }
}

// 9. COMMON PATTERNS AND IDIOMS
// ==============================

// Internal rules (helper rules)
macro_rules! with_internal {
    // Public interface
    ($val:expr) => {
        with_internal!(@internal $val)
    };
    
    // Internal helper (prefixed with @)
    (@internal $val:expr) => {
        $val * 2
    };
}

// TT muncher pattern (processing token trees one at a time)
macro_rules! tt_muncher {
    // Base case
    () => { 0 };
    
    // Recursive case: process one item at a time
    ($first:tt $($rest:tt)*) => {
        1 + tt_muncher!($($rest)*)
    };
}

// Push-down accumulation
macro_rules! reverse {
    // Start with empty accumulator
    ([$($rev:tt)*] $first:tt $($rest:tt)*) => {
        reverse!([$first $($rev)*] $($rest)*)
    };
    
    // Base case: all items reversed
    ([$($rev:tt)*]) => {
        ($($rev)*)
    };
    
    // Entry point
    ($($forward:tt)*) => {
        reverse!([] $($forward)*)
    };
}

// ============================================
// KEY CONCEPTS SUMMARY
// ============================================
// 
// 1. Pattern Matching: Match different input patterns
// 2. Fragment Specifiers: Define what syntax to accept
//    - expr, ident, ty, stmt, pat, block, item, meta, tt
// 3. Repetition: $(...),* (0+), $(...),+ (1+), $(...)?  (0-1)
// 4. Hygiene: Macros don't capture surrounding scope
// 5. Debugging: Use stringify!, compile_error!, or nightly features
// 6. Advanced: Internal rules (@), TT munchers, recursion
//
// ============================================
```