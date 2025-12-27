# Documentation and Rustdoc in Rust

Documentation is a first-class citizen in Rust. The language provides built-in tools for writing and generating beautiful, searchable documentation directly from your code using **rustdoc**.

## Doc Comments

Rust has special comment syntax for documentation:

- `///` - Documents the item that follows (outer doc comments)
- `//!` - Documents the enclosing item (inner doc comments)

### Basic Example

```rust
/// Calculates the area of a rectangle.
///
/// # Arguments
///
/// * `width` - The width of the rectangle
/// * `height` - The height of the rectangle
///
/// # Examples
///
/// ```
/// let area = calculate_area(5, 10);
/// assert_eq!(area, 50);
/// ```
fn calculate_area(width: u32, height: u32) -> u32 {
    width * height
}

/// A rectangle with width and height dimensions.
pub struct Rectangle {
    /// The width of the rectangle in pixels
    pub width: u32,
    /// The height of the rectangle in pixels
    pub height: u32,
}

impl Rectangle {
    /// Creates a new Rectangle instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::Rectangle;
    /// let rect = Rectangle::new(10, 20);
    /// assert_eq!(rect.width, 10);
    /// ```
    pub fn new(width: u32, height: u32) -> Self {
        Rectangle { width, height }
    }
}
```

### Inner Doc Comments

Use `//!` for module-level or crate-level documentation:

```rust
//! # My Geometry Library
//!
//! This crate provides utilities for geometric calculations.
//!
//! ## Features
//!
//! - Calculate areas and perimeters
//! - Work with various shapes
//! - High-performance implementations

/// Module containing shape definitions
pub mod shapes {
    //! This module defines various geometric shapes.
    //!
    //! All shapes implement the `Shape` trait.
}
```

## Common Documentation Sections

Rust documentation follows conventions with standard sections:

```rust
/// Divides two numbers.
///
/// # Arguments
///
/// * `numerator` - The number to be divided
/// * `denominator` - The number to divide by
///
/// # Returns
///
/// Returns the quotient as a floating-point number.
///
/// # Examples
///
/// ```
/// let result = divide(10.0, 2.0);
/// assert_eq!(result, 5.0);
/// ```
///
/// # Panics
///
/// Panics if the denominator is zero.
///
/// ```should_panic
/// divide(10.0, 0.0); // This will panic!
/// ```
///
/// # Errors
///
/// This function doesn't return errors, but panics instead.
///
/// # Safety
///
/// This function is safe to call with any valid f64 values.
pub fn divide(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        panic!("Cannot divide by zero!");
    }
    numerator / denominator
}
```

## Documentation Tests (Doctests)

Code examples in documentation are automatically tested by `cargo test`. This ensures your examples stay up-to-date:

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// Basic addition:
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
///
/// Works with negative numbers:
///
/// ```
/// let result = add(-5, 3);
/// assert_eq!(result, -2);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Doctest Attributes

Control how doctests run with special annotations:

```rust
/// # Examples
///
/// This example should panic:
///
/// ```should_panic
/// panic!("This is expected");
/// ```
///
/// This example won't compile (useful for showing incorrect code):
///
/// ```compile_fail
/// let x: u32 = "not a number"; // Type error
/// ```
///
/// This example is ignored during testing:
///
/// ```ignore
/// // Requires network access or external resources
/// let data = fetch_from_api();
/// ```
///
/// This example doesn't run but is syntax-highlighted:
///
/// ```no_run
/// loop {
///     // Infinite loop - we don't want to actually run this
/// }
/// ```
pub fn example_function() {}
```

### Hidden Lines in Examples

Use `#` to hide setup code from documentation while still running it:

```rust
/// Processes a user record.
///
/// # Examples
///
/// ```
/// # struct User { name: String, age: u32 }
/// # let user = User { name: "Alice".to_string(), age: 30 };
/// #
/// println!("User: {} is {} years old", user.name, user.age);
/// ```
pub fn process_user() {
    // implementation
}
```

The lines starting with `#` run during testing but don't appear in the rendered documentation.

## Real-World Example

Here's a comprehensive example showing best practices:

```rust
//! # Authentication Module
//!
//! Provides user authentication and authorization utilities.

use std::collections::HashMap;

/// Represents a user in the system.
///
/// Users have a unique username and encrypted password.
///
/// # Examples
///
/// ```
/// # use my_crate::User;
/// let user = User::new("alice", "secret123");
/// assert_eq!(user.username(), "alice");
/// ```
#[derive(Debug, Clone)]
pub struct User {
    username: String,
    password_hash: String,
}

impl User {
    /// Creates a new user with the given credentials.
    ///
    /// # Arguments
    ///
    /// * `username` - The unique username for this user
    /// * `password` - The plaintext password (will be hashed)
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::User;
    /// let user = User::new("bob", "mypassword");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the username is empty.
    ///
    /// ```should_panic
    /// # use my_crate::User;
    /// User::new("", "password"); // Panics!
    /// ```
    pub fn new(username: &str, password: &str) -> Self {
        assert!(!username.is_empty(), "Username cannot be empty");
        
        User {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
        }
    }

    /// Returns the username.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::User;
    /// let user = User::new("charlie", "pass");
    /// assert_eq!(user.username(), "charlie");
    /// ```
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Verifies if the provided password matches.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to verify
    ///
    /// # Returns
    ///
    /// Returns `true` if the password matches, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::User;
    /// let user = User::new("dave", "secret");
    /// assert!(user.verify_password("secret"));
    /// assert!(!user.verify_password("wrong"));
    /// ```
    pub fn verify_password(&self, password: &str) -> bool {
        self.password_hash == Self::hash_password(password)
    }

    // Private helper - not documented publicly
    fn hash_password(password: &str) -> String {
        // Simplified for example - use proper hashing in production
        format!("hashed_{}", password)
    }
}

/// A simple authentication manager.
///
/// Manages user registration and login.
///
/// # Examples
///
/// ```
/// # use my_crate::{AuthManager, User};
/// let mut auth = AuthManager::new();
/// auth.register(User::new("alice", "password123"));
///
/// assert!(auth.login("alice", "password123"));
/// assert!(!auth.login("alice", "wrong_password"));
/// ```
pub struct AuthManager {
    users: HashMap<String, User>,
}

impl AuthManager {
    /// Creates a new empty authentication manager.
    pub fn new() -> Self {
        AuthManager {
            users: HashMap::new(),
        }
    }

    /// Registers a new user.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to register
    ///
    /// # Returns
    ///
    /// Returns `true` if registration succeeded, `false` if username exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::{AuthManager, User};
    /// let mut auth = AuthManager::new();
    /// assert!(auth.register(User::new("alice", "pass")));
    /// assert!(!auth.register(User::new("alice", "different"))); // Duplicate
    /// ```
    pub fn register(&mut self, user: User) -> bool {
        if self.users.contains_key(user.username()) {
            return false;
        }
        self.users.insert(user.username().to_string(), user);
        true
    }

    /// Attempts to login with credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::{AuthManager, User};
    /// let mut auth = AuthManager::new();
    /// auth.register(User::new("bob", "secret"));
    ///
    /// assert!(auth.login("bob", "secret"));
    /// assert!(!auth.login("bob", "wrong"));
    /// assert!(!auth.login("nonexistent", "pass"));
    /// ```
    pub fn login(&self, username: &str, password: &str) -> bool {
        self.users
            .get(username)
            .map(|user| user.verify_password(password))
            .unwrap_or(false)
    }
}
```

## Generating Documentation

Run these commands to generate and view documentation:

```bash
# Generate documentation for your crate
cargo doc

# Generate and open in browser
cargo doc --open

# Include private items
cargo doc --document-private-items

# Generate docs for dependencies too
cargo doc --no-deps
```

## Best Practices

**1. Document public APIs thoroughly**
```rust
/// Public function - well documented
pub fn public_api() {}

// Private function - basic comment is fine
fn internal_helper() {}
```

**2. Always include examples**
```rust
/// Formats a name in title case.
///
/// # Examples
///
/// ```
/// let name = format_name("john doe");
/// assert_eq!(name, "John Doe");
/// ```
pub fn format_name(name: &str) -> String {
    // implementation
    name.to_string()
}
```

**3. Document edge cases and panics**
```rust
/// Gets an element from a slice.
///
/// # Panics
///
/// Panics if index is out of bounds.
pub fn get_element(slice: &[i32], index: usize) -> i32 {
    slice[index]
}
```

**4. Use links to other items**
```rust
/// Processes data using [`User`] credentials.
///
/// See also: [`AuthManager::login`]
pub fn process_with_auth(user: &User) {
    // implementation
}
```

**5. Keep examples testable**
All examples should compile and run successfully unless marked with special attributes.

Documentation in Rust is powerful because it's integrated into the development workflow, automatically tested, and generates professional-looking output with minimal effort. The convention of including examples in documentation helps ensure code remains correct and maintainable over time.