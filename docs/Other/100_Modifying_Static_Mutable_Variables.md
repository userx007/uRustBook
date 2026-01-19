# Modifying Static Mutable Variables

## Core Principle

**Modifying a `static mut` variable in Rust must be done within an `unsafe` block**. This is a fundamental safety requirement in Rust's type system.

## Why This Restriction Exists

Rust enforces this restriction because `static mut` variables violate Rust's core safety guarantees:

**Data Race Prevention**: Static mutable variables have a 'static lifetime and can be accessed from any thread. Without synchronization, multiple threads could read and write simultaneously, causing data races.

**No Ownership Tracking**: Unlike normal variables, the compiler cannot track who owns or borrows a static mutable variable. This means Rust's borrow checker cannot verify that accesses are safe.

**Global Mutable State**: Mutable global state makes reasoning about program behavior difficult and error-prone, as any part of the code could potentially modify it.

## Syntax and Examples

### Basic Example

```rust
static mut COUNTER: u32 = 0;

fn increment_counter() {
    unsafe {
        COUNTER += 1;  // Must be in unsafe block
    }
}

fn main() {
    unsafe {
        COUNTER = 5;
        println!("Counter: {}", COUNTER);
    }
    
    increment_counter();
    
    unsafe {
        println!("Counter after increment: {}", COUNTER);
    }
}
```

### Reading is Also Unsafe

Interestingly, even **reading** from a `static mut` requires an `unsafe` block, because the act of reading could race with a write happening elsewhere:

```rust
static mut CONFIG: i32 = 100;

fn read_config() -> i32 {
    unsafe {
        CONFIG  // Reading also requires unsafe
    }
}
```

### Multi-threaded Danger

Here's an example showing why `static mut` is dangerous:

```rust
use std::thread;

static mut SHARED: i32 = 0;

fn main() {
    let handles: Vec<_> = (0..10).map(|_| {
        thread::spawn(|| {
            for _ in 0..1000 {
                unsafe {
                    SHARED += 1;  // Data race! Multiple threads writing
                }
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    unsafe {
        println!("Final value: {}", SHARED);
        // Expected: 10000, but likely less due to race conditions
    }
}
```

## Better Alternatives

Rust provides safer alternatives to `static mut`:

--- 

### Using Atomic Types

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn increment_counter() {
    COUNTER.fetch_add(1, Ordering::SeqCst);  // No unsafe needed!
}
```
---

### Using Mutex for Complex Types

```rust
use std::sync::Mutex;

static CONFIG: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn add_config(item: String) {
    CONFIG.lock().unwrap().push(item);  // Thread-safe, no unsafe
}
```

#### Full example of using Mutex for Complex Types

```rust
use std::sync::Mutex;
use std::thread;

// Global configuration using Mutex
static CONFIG: Mutex<Vec<String>> = Mutex::new(Vec::new());

// A more complex example with a struct
#[derive(Debug, Clone)]
struct AppConfig {
    settings: Vec<String>,
    user_count: u32,
    enabled: bool,
}

static APP_STATE: Mutex<AppConfig> = Mutex::new(AppConfig {
    settings: Vec::new(),
    user_count: 0,
    enabled: false,
});

// Function to add configuration items
fn add_config(item: String) {
    // Lock the mutex, add item, automatically unlocks when guard drops
    CONFIG.lock().unwrap().push(item);
}

// Function to read configuration
fn get_config() -> Vec<String> {
    // Lock, clone the data, return it
    CONFIG.lock().unwrap().clone()
}

// Function to update app state
fn update_app_state(setting: String, increment_users: bool) {
    let mut state = APP_STATE.lock().unwrap();
    state.settings.push(setting);
    if increment_users {
        state.user_count += 1;
    }
    state.enabled = true;
}

// Function to read app state
fn print_app_state() {
    let state = APP_STATE.lock().unwrap();
    println!("App State: {:?}", *state);
}

fn main() {
    println!("=== Simple Mutex Example ===\n");
    
    // Single-threaded usage
    add_config("debug=true".to_string());
    add_config("port=8080".to_string());
    println!("Config: {:?}", get_config());
    
    println!("\n=== Multi-threaded Mutex Example ===\n");
    
    // Spawn multiple threads that all modify CONFIG
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..3 {
                    add_config(format!("thread-{}-item-{}", i, j));
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final config ({}items):", get_config().len());
    for item in get_config() {
        println!("  - {}", item);
    }
    
    println!("\n=== Complex State Example ===\n");
    
    // Spawn threads that modify complex state
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                update_app_state(format!("feature-{}", i), true);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    print_app_state();
    
    println!("\n=== Error Handling Example ===\n");
    
    // Proper error handling with match
    match CONFIG.lock() {
        Ok(mut guard) => {
            guard.push("safely-added".to_string());
            println!("Successfully added item");
        }
        Err(poisoned) => {
            println!("Mutex was poisoned: {:?}", poisoned);
            // Can recover the data even from poisoned mutex
            let mut guard = poisoned.into_inner();
            guard.push("recovered".to_string());
        }
    }
    
    println!("\n=== Scoped Lock Example ===\n");
    
    // Demonstrate explicit scope for lock
    {
        let mut config = CONFIG.lock().unwrap();
        config.push("scoped-item".to_string());
        println!("Lock held in this scope");
    } // Lock is dropped here
    
    println!("Lock released, can lock again");
    println!("Final count: {}", CONFIG.lock().unwrap().len());
}
```

***This example demonstrates:***
- Basic thread-safe configuration storage using `Mutex<Vec<String>>`
- Complex state management with a struct inside a Mutex
- Multi-threaded access patterns with multiple threads safely modifying shared state
- Proper error handling for poisoned mutexes
- Scoped locks and automatic unlocking via RAII

***Key points:***
- The `Mutex` ensures only one thread can modify the data at a time
- No `unsafe` code needed
- Locks are automatically released when the `MutexGuard` goes out of scope
- Works perfectly for complex types like `Vec`, `HashMap`, or custom structs

---

### Using LazyLock for Initialization

```rust
use std::sync::LazyLock;

static SETTINGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec!["default".to_string()]
});
```

#### Full example of using LazyLock for initialization

```rust
use std::sync::LazyLock;
use std::collections::HashMap;

// Simple LazyLock example
static SETTINGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    println!("Initializing SETTINGS (this runs only once)");
    vec!["default".to_string(), "production".to_string()]
});

// LazyLock with complex initialization
static DATABASE_CONFIG: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    println!("Initializing DATABASE_CONFIG (expensive operation)");
    let mut config = HashMap::new();
    config.insert("host".to_string(), "localhost".to_string());
    config.insert("port".to_string(), "5432".to_string());
    config.insert("database".to_string(), "myapp".to_string());
    
    // Simulate expensive initialization
    std::thread::sleep(std::time::Duration::from_millis(100));
    config
});

// LazyLock with computed value
static COMPILED_REGEX: LazyLock<Vec<String>> = LazyLock::new(|| {
    println!("Compiling regex patterns");
    vec![
        r"\d{3}-\d{2}-\d{4}".to_string(),  // SSN pattern
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),  // Email
    ]
});

// LazyLock with file reading simulation
static APP_VERSION: LazyLock<String> = LazyLock::new(|| {
    println!("Reading application version");
    // In real code, this might read from a file or environment
    "1.2.3".to_string()
});

// Struct with LazyLock
#[derive(Debug)]
struct AppMetadata {
    name: String,
    version: String,
    features: Vec<String>,
}

static METADATA: LazyLock<AppMetadata> = LazyLock::new(|| {
    println!("Building application metadata");
    AppMetadata {
        name: "MyApp".to_string(),
        version: APP_VERSION.clone(),
        features: vec![
            "authentication".to_string(),
            "logging".to_string(),
            "caching".to_string(),
        ],
    }
});

// LazyLock with environment-based initialization
static ENVIRONMENT: LazyLock<String> = LazyLock::new(|| {
    println!("Detecting environment");
    // In real code: std::env::var("ENV").unwrap_or_else(|_| "development".to_string())
    "development".to_string()
});

// Combining LazyLock with Mutex for mutable state
use std::sync::Mutex;

static CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| {
    println!("Initializing cache");
    Mutex::new(HashMap::new())
});

fn get_cached_or_compute(key: &str) -> String {
    let mut cache = CACHE.lock().unwrap();
    
    if let Some(value) = cache.get(key) {
        println!("Cache hit for '{}'", key);
        return value.clone();
    }
    
    println!("Cache miss for '{}', computing...", key);
    let computed = format!("computed_value_for_{}", key);
    cache.insert(key.to_string(), computed.clone());
    computed
}

fn main() {
    println!("=== LazyLock Basic Example ===\n");
    
    println!("Program started, SETTINGS not yet initialized");
    println!("Accessing SETTINGS for first time:");
    println!("Settings: {:?}", *SETTINGS);
    
    println!("\nAccessing SETTINGS again:");
    println!("Settings: {:?}", *SETTINGS);
    println!("(Notice initialization message only appeared once)\n");
    
    println!("=== Complex Initialization ===\n");
    
    println!("Accessing DATABASE_CONFIG:");
    println!("Host: {}", DATABASE_CONFIG.get("host").unwrap());
    println!("Port: {}", DATABASE_CONFIG.get("port").unwrap());
    
    println!("\n=== Multiple LazyLock Dependencies ===\n");
    
    println!("Accessing METADATA (which uses APP_VERSION):");
    println!("Metadata: {:?}", *METADATA);
    
    println!("\n=== Compiled Patterns ===\n");
    
    println!("Regex patterns:");
    for (i, pattern) in COMPILED_REGEX.iter().enumerate() {
        println!("  Pattern {}: {}", i, pattern);
    }
    
    println!("\n=== Environment Detection ===\n");
    
    println!("Running in: {} mode", *ENVIRONMENT);
    
    println!("\n=== LazyLock with Mutex (Mutable Cache) ===\n");
    
    // First access - cache miss
    let result1 = get_cached_or_compute("user_123");
    println!("Result: {}", result1);
    
    // Second access - cache hit
    let result2 = get_cached_or_compute("user_123");
    println!("Result: {}", result2);
    
    // Different key - cache miss
    let result3 = get_cached_or_compute("user_456");
    println!("Result: {}", result3);
    
    println!("\n=== Practical Usage Example ===\n");
    
    // Simulating a real application flow
    fn process_request(user_id: &str) {
        println!("Processing request for user: {}", user_id);
        println!("  Environment: {}", *ENVIRONMENT);
        println!("  App Version: {}", *APP_VERSION);
        
        let cached_data = get_cached_or_compute(user_id);
        println!("  User data: {}", cached_data);
    }
    
    process_request("alice");
    println!();
    process_request("bob");
    println!();
    process_request("alice"); // Will use cache
    
    println!("\n=== Performance Note ===");
    println!("LazyLock is initialized exactly once, thread-safely.");
    println!("After initialization, access is as fast as a regular static.");
}
```

***This example shows:***
- Simple lazy initialization with `LazyLock<Vec<String>>`
- Complex initialization with expensive operations
- Dependency between multiple `LazyLock` statics
- Combining `LazyLock` with `Mutex` for lazy-initialized mutable state
- Real-world patterns like caching and configuration loading

***Key benefits:***
- Initialization happens exactly once, thread-safely
- Perfect for expensive computations that should only run when first needed
- After initialization, access is as fast as a regular static reference
- Great for reading configuration, compiling regex patterns, or loading resources

---

### Using OnceLock for Initialization

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

// OnceLock for manual initialization
static CONFIG: OnceLock<HashMap<String, String>> = OnceLock::new();

// OnceLock for logger initialization
static LOGGER: OnceLock<String> = OnceLock::new();

// OnceLock for runtime-determined values
static API_KEY: OnceLock<String> = OnceLock::new();

// OnceLock with complex type
#[derive(Debug, Clone)]
struct DatabaseConnection {
    host: String,
    port: u16,
    connected: bool,
}

static DB_CONNECTION: OnceLock<DatabaseConnection> = OnceLock::new();

// Multiple OnceLock cells that depend on runtime conditions
static FEATURE_FLAGS: OnceLock<Vec<String>> = OnceLock::new();

/// Initialize configuration - can be called from anywhere, anytime
fn init_config() {
    let mut config = HashMap::new();
    config.insert("app_name".to_string(), "MyApp".to_string());
    config.insert("version".to_string(), "2.0.0".to_string());
    config.insert("max_connections".to_string(), "100".to_string());
    
    match CONFIG.set(config) {
        Ok(_) => println!("✓ Config initialized successfully"),
        Err(_) => println!("⚠ Config was already initialized"),
    }
}

/// Get config value, initializing if needed
fn get_config(key: &str) -> Option<String> {
    // get_or_init ensures initialization happens exactly once
    CONFIG
        .get_or_init(|| {
            println!("Lazy initializing config in get_config");
            let mut config = HashMap::new();
            config.insert("default".to_string(), "value".to_string());
            config
        })
        .get(key)
        .cloned()
}

/// Initialize logger with specific configuration
fn init_logger(log_level: &str) -> Result<(), String> {
    let logger_config = format!("Logger[level={}]", log_level);
    
    LOGGER.set(logger_config).map_err(|_| {
        "Logger already initialized".to_string()
    })
}

/// Get logger, returns None if not initialized
fn get_logger() -> Option<&'static String> {
    LOGGER.get()
}

/// Initialize database connection
fn connect_database(host: &str, port: u16) {
    println!("Connecting to database at {}:{}...", host, port);
    
    let connection = DatabaseConnection {
        host: host.to_string(),
        port,
        connected: true,
    };
    
    DB_CONNECTION.get_or_init(|| {
        println!("Database connection established");
        connection
    });
}

/// Get database connection, initialize with defaults if needed
fn get_db() -> &'static DatabaseConnection {
    DB_CONNECTION.get_or_init(|| {
        println!("Initializing default database connection");
        DatabaseConnection {
            host: "localhost".to_string(),
            port: 5432,
            connected: true,
        }
    })
}

/// Initialize feature flags based on environment
fn init_features(environment: &str) {
    let features = match environment {
        "production" => vec!["auth".to_string(), "logging".to_string()],
        "development" => vec![
            "auth".to_string(),
            "logging".to_string(),
            "debug".to_string(),
            "hot_reload".to_string(),
        ],
        _ => vec!["auth".to_string()],
    };
    
    FEATURE_FLAGS.set(features).ok();
}

/// Check if a feature is enabled
fn is_feature_enabled(feature: &str) -> bool {
    FEATURE_FLAGS
        .get()
        .map(|flags| flags.contains(&feature.to_string()))
        .unwrap_or(false)
}

/// Example of conditional initialization
fn setup_api_key(key: Option<String>) {
    if let Some(key) = key {
        if API_KEY.set(key).is_ok() {
            println!("✓ API key configured");
        }
    } else {
        println!("⚠ No API key provided, using guest mode");
    }
}

/// Safe getter with fallback
fn get_api_key() -> &'static str {
    API_KEY.get().map(|s| s.as_str()).unwrap_or("GUEST_KEY")
}

fn main() {
    println!("=== OnceLock Basic Usage ===\n");
    
    // Check before initialization
    println!("Config initialized? {}", CONFIG.get().is_some());
    
    // Initialize explicitly
    init_config();
    
    // Try to initialize again (will fail)
    init_config();
    
    // Access the value
    if let Some(value) = CONFIG.get() {
        println!("Config app_name: {:?}", value.get("app_name"));
    }
    
    println!("\n=== get_or_init Pattern ===\n");
    
    // This will use existing config
    println!("Getting 'version': {:?}", get_config("version"));
    
    // This would initialize if CONFIG wasn't already set
    println!("Getting 'default': {:?}", get_config("default"));
    
    println!("\n=== Logger Initialization ===\n");
    
    // Check if logger exists
    println!("Logger exists? {}", get_logger().is_some());
    
    // Initialize logger
    match init_logger("INFO") {
        Ok(_) => println!("✓ Logger initialized"),
        Err(e) => println!("✗ Logger error: {}", e),
    }
    
    // Try to reinitialize (will fail)
    match init_logger("DEBUG") {
        Ok(_) => println!("✓ Logger initialized"),
        Err(e) => println!("✗ Logger error: {}", e),
    }
    
    // Use logger
    if let Some(logger) = get_logger() {
        println!("Current logger: {}", logger);
    }
    
    println!("\n=== Database Connection ===\n");
    
    // First call initializes with custom values
    connect_database("db.example.com", 3306);
    
    // Subsequent calls use existing connection
    let db = get_db();
    println!("Connected to: {}:{}", db.host, db.port);
    
    println!("\n=== Feature Flags ===\n");
    
    // Initialize based on environment
    init_features("development");
    
    // Check features
    println!("Auth enabled? {}", is_feature_enabled("auth"));
    println!("Debug enabled? {}", is_feature_enabled("debug"));
    println!("Analytics enabled? {}", is_feature_enabled("analytics"));
    
    println!("\n=== Conditional Initialization ===\n");
    
    // Simulate getting API key from environment
    let api_key_from_env = Some("sk_live_abc123xyz".to_string());
    setup_api_key(api_key_from_env);
    
    println!("Using API key: {}", get_api_key());
    
    println!("\n=== Thread Safety Example ===\n");
    
    use std::thread;
    
    static SHARED_COUNTER: OnceLock<u32> = OnceLock::new();
    
    // Spawn multiple threads trying to initialize
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                // Only one thread will successfully initialize
                match SHARED_COUNTER.set(i * 10) {
                    Ok(_) => println!("Thread {} initialized counter to {}", i, i * 10),
                    Err(_) => println!("Thread {} found counter already initialized", i),
                }
                
                // All threads can read the value
                if let Some(value) = SHARED_COUNTER.get() {
                    println!("Thread {} reads counter: {}", i, value);
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("\nFinal counter value: {:?}", SHARED_COUNTER.get());
    
    println!("\n=== Practical Pattern: Singleton ===\n");
    
    struct AppConfig {
        name: String,
        debug: bool,
    }
    
    static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();
    
    fn get_app_config() -> &'static AppConfig {
        APP_CONFIG.get_or_init(|| {
            println!("Creating singleton AppConfig");
            AppConfig {
                name: "MyApplication".to_string(),
                debug: true,
            }
        })
    }
    
    let config1 = get_app_config();
    println!("Config name: {}", config1.name);
    
    let config2 = get_app_config();
    println!("Same instance? {}", std::ptr::eq(config1, config2));
    
    println!("\n=== OnceLock vs LazyLock ===");
    println!("OnceLock: Manual initialization, can fail if already set");
    println!("LazyLock: Automatic initialization with closure, always succeeds");
    println!("Use OnceLock when: initialization depends on runtime values");
    println!("Use LazyLock when: initialization is deterministic");
}
```

***Key Features Shown:***

1. **Manual Initialization** - You control when and how initialization happens using `set()`
2. **Lazy Initialization** - Use `get_or_init()` to initialize on first access
3. **Initialization Checking** - Check if already initialized with `get().is_some()`
4. **Error Handling** - `set()` returns `Result`, failing if already initialized
5. **Thread Safety** - Multiple threads can safely race to initialize (only one wins)
6. **Runtime Configuration** - Perfect for values determined at runtime (API keys, environment-based settings)
7. **Singleton Pattern** - Create true singletons with guaranteed single initialization

---

## OnceLock vs LazyLock:

**OnceLock (formerly OnceCell):**
- Manual control over initialization
- `set()` can fail if already initialized
- Great for runtime-dependent values
- More flexible, but requires explicit initialization logic

**LazyLock:**
- Automatic initialization with a closure
- Always succeeds (closure runs exactly once)
- Great for deterministic initialization
- Less flexible, but simpler to use

All three patterns above are vastly superior to `static mut` for nearly all use cases, providing thread safety without requiring `unsafe` blocks!


## When Static Mut Might Be Acceptable

Despite the risks, `static mut` can be appropriate in specific scenarios:

**Single-threaded embedded systems** where you control all access patterns and no threading exists.

**FFI (Foreign Function Interface)** when interfacing with C libraries that expect mutable global variables.

**Low-level system programming** where you're implementing synchronization primitives themselves.

Even in these cases, consider whether thread-local storage or other patterns might work instead.

## Key Takeaways

The requirement that `static mut` modifications occur in `unsafe` blocks serves as a critical reminder that you're taking responsibility for preventing data races and maintaining invariants that Rust normally guarantees. This is intentional friction that encourages you to use safer abstractions like atomics, mutexes, or immutable statics whenever possible. When you do use `static mut`, the `unsafe` keyword signals to anyone reading your code that special care and scrutiny are required.