# Custom Error Types and Error Trait in Rust

## Understanding the Error Trait

The `Error` trait in Rust's standard library (`std::error::Error`) is the foundation for error handling. Any type that implements this trait can be used as an error type with the `Result` type.

### Basic Error Trait Definition

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
    fn backtrace(&self) -> Option<&Backtrace> { None }
}
```

## 1. Manual Implementation of Error Trait

```
            Custom Error Types (Rust)
        ┌───────────────────────────────┐
        │      std::error::Error        │
        │   (trait, public interface)   │
        └───────────────────────────────┘
                         ▲
                         │ requires
            ┌────────────┴────────────┐
            │                         │
    ┌───────────────┐        ┌────────────────┐
    │  fmt::Debug   │        │  fmt::Display  │
    │  (developer)  │        │  (user output) │
    └───────────────┘        └────────────────┘


        ┌───────────────────────────────┐
        │      Custom Error Type        │
        │        (struct / enum)        │
        └───────────────────────────────┘
                     │
        ┌────────────┼────────────────┐
        │            │                │
        ▼            ▼                ▼
  derive Debug   impl Display     impl Error
                                 (no methods)


        ┌───────────────────────────────┐
        │     Error Usage Patterns      │
        └───────────────────────────────┘
         │              │            │
     Result<T, E>    dyn Error       ?
         │              │            │
         ▼              ▼            ▼
  concrete error   trait object   error propagation


        ┌───────────────────────────────┐
        │     Optional Enhancements     │
        └───────────────────────────────┘
           source()  → error chaining
           From<T>   → automatic conversion
           thiserror / anyhow → ergonomics

Legend:
───────
struct / enum → define error shape
Display       → user-friendly message
Debug         → internal diagnostics
Error         → interoperability contract
```

### Minimal code (irreducible)

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error")
    }
}

impl Error for MyError {}
```

### Simple Custom Error

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
    Overflow,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Cannot divide by zero"),
            MathError::NegativeSquareRoot => write!(f, "Cannot take square root of negative number"),
            MathError::Overflow => write!(f, "Arithmetic overflow occurred"),
        }
    }
}

impl Error for MathError {}

// Using the custom error
fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 0.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Complex Custom Error with Context

```rust
use std::fmt;
use std::error::Error;
use std::io;

#[derive(Debug)]
enum DatabaseError {
    ConnectionFailed(String),
    QueryError { query: String, cause: String },
    IoError(io::Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(host) => {
                write!(f, "Failed to connect to database at {}", host)
            }
            DatabaseError::QueryError { query, cause } => {
                write!(f, "Query failed: '{}'. Reason: {}", query, cause)
            }
            DatabaseError::IoError(e) => {
                write!(f, "Database I/O error: {}", e)
            }
        }
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DatabaseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Convert from io::Error to DatabaseError
impl From<io::Error> for DatabaseError {
    fn from(error: io::Error) -> Self {
        DatabaseError::IoError(error)
    }
}
```

## 2. Using `thiserror` Crate

`thiserror` provides derive macros to simplify error type creation.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {message}")]
    Database { message: String },
    
    #[error("User {user} not found")]
    UserNotFound { user: String },
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Usage example
use std::fs;

fn load_config(path: &str) -> Result<String, AppError> {
    // The ? operator automatically converts io::Error to AppError
    let content = fs::read_to_string(path)?;
    
    if content.is_empty() {
        return Err(AppError::Config("Config file is empty".to_string()));
    }
    
    Ok(content)
}

fn parse_user_id(s: &str) -> Result<u32, AppError> {
    // ParseIntError is automatically converted to AppError
    Ok(s.parse()?)
}
```

### Advanced `thiserror` Example

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ServiceError {
    #[error("Authentication failed for user '{username}'")]
    AuthFailed { username: String },
    
    #[error("Rate limit exceeded: {requests} requests in {window} seconds")]
    RateLimitExceeded { requests: u32, window: u32 },
    
    #[error("Invalid API key format")]
    InvalidApiKey,
    
    #[error("Network error: {0}")]
    Network(#[source] std::io::Error),
    
    #[error("JSON parsing failed")]
    JsonError(#[from] serde_json::Error),
}

fn authenticate(username: &str, api_key: &str) -> Result<(), ServiceError> {
    if api_key.len() < 32 {
        return Err(ServiceError::InvalidApiKey);
    }
    
    // Simulate authentication
    if username == "invalid" {
        return Err(ServiceError::AuthFailed {
            username: username.to_string(),
        });
    }
    
    Ok(())
}
```

## 3. Using `anyhow` Crate

`anyhow` is ideal for application code where you want simplified error handling without defining custom types for every error.

### Basic `anyhow` Usage

```rust
use anyhow::{Context, Result, anyhow, bail};

fn read_username_from_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read username file")?;
    
    let username = content.trim();
    
    if username.is_empty() {
        bail!("Username cannot be empty");
    }
    
    Ok(username.to_string())
}

fn process_user(id: u32) -> Result<()> {
    if id == 0 {
        return Err(anyhow!("User ID cannot be zero"));
    }
    
    let username = read_username_from_file("user.txt")
        .context(format!("Failed to process user {}", id))?;
    
    println!("Processing user: {}", username);
    Ok(())
}

fn main() {
    if let Err(e) = process_user(42) {
        eprintln!("Error: {:?}", e);
        // Prints the full error chain
    }
}
```

### `anyhow` with Error Context Chain

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn read_config() -> Result<String> {
    fs::read_to_string("config.json")
        .context("Failed to read config.json")?;
    Ok("config content".to_string())
}

fn parse_config(content: &str) -> Result<serde_json::Value> {
    serde_json::from_str(content)
        .context("Failed to parse JSON configuration")?;
    Ok(serde_json::json!({}))
}

fn initialize_app() -> Result<()> {
    let config = read_config()
        .context("Configuration initialization failed")?;
    
    let parsed = parse_config(&config)
        .context("Configuration parsing failed")?;
    
    println!("App initialized with config: {:?}", parsed);
    Ok(())
}

fn main() {
    if let Err(e) = initialize_app() {
        // This prints the full error chain:
        // Error: Configuration initialization failed
        // Caused by: Failed to read config.json
        // Caused by: No such file or directory (os error 2)
        eprintln!("Error: {:?}", e);
        
        // For production, you might want:
        eprintln!("Error: {}", e);
        for cause in e.chain().skip(1) {
            eprintln!("  Caused by: {}", cause);
        }
    }
}
```

## 4. Combining `thiserror` and `anyhow`

Use `thiserror` for library code and `anyhow` for application code.

```rust
// Library code (mylib.rs)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Processing failed: {0}")]
    ProcessingError(String),
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

pub fn library_function(input: &str) -> Result<String, LibError> {
    if input.is_empty() {
        return Err(LibError::InvalidInput("Input cannot be empty".to_string()));
    }
    
    // Process input
    Ok(format!("Processed: {}", input))
}

// Application code (main.rs)
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let result = library_function("")
        .context("Failed to process user input")?;
    
    println!("Result: {}", result);
    Ok(())
}
```

## 5. Error Backtraces

Backtraces help debug where errors originated.

### Manual Backtrace Implementation

```rust
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct MyError {
    message: String,
    backtrace: Backtrace,
}

impl MyError {
    fn new(message: String) -> Self {
        Self {
            message,
            backtrace: Backtrace::capture(),
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.backtrace)
    }
}

fn might_fail() -> Result<(), MyError> {
    Err(MyError::new("Something went wrong".to_string()))
}

fn main() {
    // Set RUST_BACKTRACE=1 environment variable to see backtraces
    if let Err(e) = might_fail() {
        eprintln!("Error: {}", e);
        if let Some(backtrace) = e.backtrace() {
            eprintln!("Backtrace:\n{}", backtrace);
        }
    }
}
```

### Backtraces with `anyhow`

```rust
use anyhow::Result;

fn main() -> Result<()> {
    // Backtraces are automatically captured when RUST_BACKTRACE=1
    // and displayed with {:?} formatting
    
    inner_function()?;
    Ok(())
}

fn inner_function() -> Result<()> {
    deep_function()?;
    Ok(())
}

fn deep_function() -> Result<()> {
    anyhow::bail!("Deep error occurred");
}

// Run with: RUST_BACKTRACE=1 cargo run
// The error will include the full backtrace
```

## 6. Complete Real-World Example

```rust
use thiserror::Error;
use std::fs;
use std::io;
use std::num::ParseIntError;

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Configuration file not found at '{path}'")]
    FileNotFound { path: String },
    
    #[error("Failed to read configuration file")]
    ReadError(#[from] io::Error),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid port number")]
    InvalidPort(#[from] ParseIntError),
}

struct Config {
    host: String,
    port: u16,
    database: String,
}

impl Config {
    fn from_file(path: &str) -> Result<Self, ConfigError> {
        // Check if file exists
        if !std::path::Path::new(path).exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string(),
            });
        }
        
        // Read file (io::Error automatically converted)
        let content = fs::read_to_string(path)?;
        
        // Parse configuration
        Self::parse(&content)
    }
    
    fn parse(content: &str) -> Result<Self, ConfigError> {
        let mut host = None;
        let mut port = None;
        let mut database = None;
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::InvalidFormat(line.to_string()));
            }
            
            match parts[0].trim() {
                "host" => host = Some(parts[1].trim().to_string()),
                "port" => port = Some(parts[1].trim().parse()?), // ParseIntError converted
                "database" => database = Some(parts[1].trim().to_string()),
                _ => {}
            }
        }
        
        Ok(Config {
            host: host.ok_or(ConfigError::MissingField {
                field: "host".to_string(),
            })?,
            port: port.ok_or(ConfigError::MissingField {
                field: "port".to_string(),
            })?,
            database: database.ok_or(ConfigError::MissingField {
                field: "database".to_string(),
            })?,
        })
    }
}

fn main() {
    match Config::from_file("config.txt") {
        Ok(config) => {
            println!("Configuration loaded:");
            println!("  Host: {}", config.host);
            println!("  Port: {}", config.port);
            println!("  Database: {}", config.database);
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            
            // Print the cause chain
            let mut current = e.source();
            while let Some(cause) = current {
                eprintln!("  Caused by: {}", cause);
                current = cause.source();
            }
            
            std::process::exit(1);
        }
    }
}
```

## Key Takeaways

1. **`thiserror`**: Use for library code where you want explicit, typed errors
2. **`anyhow`**: Use for application code where convenience matters more than specific error types
3. **Error trait**: Implement manually for full control, or use derive macros for convenience
4. **Context**: Add context to errors as they propagate up the call stack
5. **Backtraces**: Enable with `RUST_BACKTRACE=1` for debugging
6. **`From` trait**: Automatically convert between error types with `?` operator
7. **Error chains**: Use `source()` method to access underlying causes