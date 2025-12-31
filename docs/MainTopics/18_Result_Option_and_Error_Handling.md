# Result, Option, and Error Handling in Rust

## **Option Type**

`Option<T>` represents a value that might be absent. It has two variants:
- `Some(T)` - contains a value
- `None` - no value present

This replaces null pointers from other languages and forces you to explicitly handle the absence case, preventing null pointer errors at compile time.

## **Result Type**

`Result<T, E>` represents operations that can fail:
- `Ok(T)` - successful operation with value
- `Err(E)` - failed operation with error

This is Rust's primary error handling mechanism, making errors explicit and impossible to ignore.

## **The ? Operator**

The `?` operator is syntactic sugar for error propagation. It:
- Unwraps `Ok` or `Some` values automatically
- Returns `Err` or `None` immediately if encountered
- Converts error types using the `From` trait

This transforms verbose match statements into clean, linear code.

## **Key Patterns**

1. **Pattern Matching**: Use `match` for comprehensive handling of all cases
2. **if let**: Cleaner syntax when you only care about one variant
3. **Combinators**: Methods like `map()`, `and_then()`, `unwrap_or()` for functional-style error handling
4. **Error Conversion**: Implement `From` trait to enable `?` with different error types
5. **Custom Errors**: Create enums for domain-specific error types with detailed context

## **Best Practices**

- Avoid `unwrap()` and `expect()` in production code
- Use `?` for concise error propagation
- Create custom error types for library code
- Return `Result` from functions that can fail
- Use `Option` for truly optional values, not errors

This system ensures robust error handling while keeping code readable and maintainable.

```rust
// ============================================
// OPTION TYPE - Handling Absence of Values
// ============================================

// Option<T> represents an optional value: either Some(T) or None
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

// Pattern matching with Option
fn greet_user(id: u32) {
    match find_user(id) {
        Some(name) => println!("Hello, {}!", name),
        None => println!("User not found"),
    }
}

// Using if let for cleaner code when you only care about one case
fn greet_user_if_let(id: u32) {
    if let Some(name) = find_user(id) {
        println!("Hello, {}!", name);
    }
}

// Common Option methods
fn option_methods_demo() {
    let x = Some(5);
    let y: Option<i32> = None;
    
    // unwrap_or: provide a default value
    println!("x or 0: {}", x.unwrap_or(0));
    println!("y or 0: {}", y.unwrap_or(0));
    
    // map: transform the value if present
    let doubled = x.map(|n| n * 2);
    println!("Doubled: {:?}", doubled); // Some(10)
    
    // and_then: chain operations that return Options
    let result = x.and_then(|n| if n > 3 { Some(n) } else { None });
    println!("Result: {:?}", result); // Some(5)
}

// ============================================
// RESULT TYPE - Handling Operations That Can Fail
// ============================================

use std::fs::File;
use std::io::{self, Read};

// Result<T, E> represents success (Ok(T)) or failure (Err(E))
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// Pattern matching with Result
fn calculate() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

// ============================================
// THE ? OPERATOR - Elegant Error Propagation
// ============================================

// Without ? operator - verbose error handling
fn read_file_verbose(path: &str) -> Result<String, io::Error> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}

// With ? operator - clean and concise
// ? automatically returns the error if operation fails
fn read_file_clean(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// ? works with Option too
fn get_first_char(text: Option<String>) -> Option<char> {
    text?.chars().next()
}

// ============================================
// ERROR PROPAGATION PATTERNS
// ============================================

// Chaining operations with ?
fn process_data(filename: &str) -> Result<usize, io::Error> {
    let contents = read_file_clean(filename)?;
    Ok(contents.lines().count())
}

// Converting between error types with map_err
fn read_number_from_file(path: &str) -> Result<i32, String> {
    let contents = read_file_clean(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    contents.trim()
        .parse::<i32>()
        .map_err(|e| format!("Failed to parse number: {}", e))
}

// ============================================
// CUSTOM ERROR TYPES
// ============================================

use std::fmt;

#[derive(Debug)]
enum DataError {
    IoError(io::Error),
    ParseError(String),
    ValidationError(String),
}

// Implement Display for better error messages
impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::IoError(e) => write!(f, "IO error: {}", e),
            DataError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DataError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

// Implement From to enable ? operator with different error types
impl From<io::Error> for DataError {
    fn from(error: io::Error) -> Self {
        DataError::IoError(error)
    }
}

// Using custom error type
fn load_and_validate(path: &str) -> Result<i32, DataError> {
    let contents = read_file_clean(path)?; // io::Error automatically converts to DataError
    
    let number: i32 = contents.trim()
        .parse()
        .map_err(|_| DataError::ParseError("Invalid number format".to_string()))?;
    
    if number < 0 {
        return Err(DataError::ValidationError("Number must be positive".to_string()));
    }
    
    Ok(number)
}

// ============================================
// COMBINING OPTION AND RESULT
// ============================================

fn find_and_parse(id: u32) -> Result<i32, String> {
    let user = find_user(id)
        .ok_or_else(|| format!("User {} not found", id))?;
    
    user.parse::<i32>()
        .map_err(|e| format!("Parse error: {}", e))
}

// ============================================
// COMMON PATTERNS AND BEST PRACTICES
// ============================================

fn demonstrate_patterns() {
    // 1. unwrap() - panics if None/Err (use only when sure it won't fail)
    let x = Some(5);
    let value = x.unwrap();
    
    // 2. expect() - like unwrap but with custom panic message
    let y = Some(10);
    let value2 = y.expect("Value should exist");
    
    // 3. unwrap_or_default() - use type's default value
    let z: Option<i32> = None;
    let value3 = z.unwrap_or_default(); // 0 for i32
    
    // 4. ok_or() / ok_or_else() - convert Option to Result
    let opt = Some(42);
    let res: Result<i32, &str> = opt.ok_or("No value");
    
    // 5. Combining Results with and_then
    let result = divide(10.0, 2.0)
        .and_then(|x| divide(x, 2.0))
        .and_then(|x| divide(x, 2.5));
}

// Early return pattern for cleaner code
fn complex_operation(value: i32) -> Result<i32, String> {
    if value < 0 {
        return Err("Value must be positive".to_string());
    }
    
    if value > 100 {
        return Err("Value too large".to_string());
    }
    
    Ok(value * 2)
}

// ============================================
// MAIN FUNCTION DEMONSTRATING USAGE
// ============================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Option Examples ===");
    greet_user(1);
    greet_user(2);
    option_methods_demo();
    
    println!("\n=== Result Examples ===");
    calculate();
    
    println!("\n=== Error Propagation ===");
    match read_number_from_file("data.txt") {
        Ok(num) => println!("Number: {}", num),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== Custom Errors ===");
    match load_and_validate("number.txt") {
        Ok(num) => println!("Valid number: {}", num),
        Err(e) => println!("Error: {}", e),
    }
    
    // Using ? in main is possible with Result return type
    Ok(())
}
```