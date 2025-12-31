# Rust testing strategies 

## **1. Unit Tests**

Unit tests in Rust live alongside your code in a `tests` module marked with `#[cfg(test)]`. This attribute ensures test code is only compiled when running tests, not in production builds.

**Key features:**
- Access to private functions and types
- Fast execution
- Use `assert!`, `assert_eq!`, and `assert_ne!` macros
- `#[should_panic]` for testing error conditions
- `#[ignore]` for expensive tests you want to skip by default

## **2. Integration Tests**

Located in the `tests/` directory at your project root, these test your crate as external users would:

**Characteristics:**
- Only access public APIs
- Each file in `tests/` is compiled as a separate crate
- Best for testing module interactions
- Can share code via `tests/common/mod.rs`

## **3. Doc Tests**

Written in `///` documentation comments, these serve dual purposes as documentation and tests:

**Advantages:**
- Ensures examples stay up-to-date
- Provides executable documentation
- Supports `should_panic` and `compile_fail` annotations
- Automatically runs with `cargo test`

## **4. Property-Based Testing (proptest)**

Instead of hand-picking test cases, proptest generates hundreds of random inputs to verify properties:

**Benefits:**
- Finds edge cases you might miss
- Tests mathematical properties (commutativity, identity, etc.)
- Custom strategies for complex types
- Shrinking: automatically minimizes failing cases

**Common strategies:**
- Range constraints: `1..100`
- Collections: `prop::collection::vec()`
- Complex types: compose simpler strategies

## **5. Mocking Strategies**

Rust doesn't have built-in mocking, but offers several approaches:

**Manual trait-based mocking:**
- Define traits for dependencies
- Create mock implementations for tests
- Full control, no external dependencies

**Mockall crate:**
- Automatic mock generation with `#[automock]`
- Expectation setting and verification
- Supports complex scenarios with `.with()`, `.times()`, `.returning()`

## **Best Practices**

**Test Organization:**
- Use descriptive test names: `test_divide_by_zero_returns_error`
- Group related tests in modules
- Use `cargo test --lib` for unit tests only, `cargo test --test` for integration tests

**Test Coverage:**
- Run `cargo tarpaulin` or `cargo-llvm-cov` for coverage reports
- Aim for high coverage but focus on meaningful tests

**Performance:**
- Unit tests run in parallel by default
- Use `cargo test -- --test-threads=1` for serial execution
- Integration tests in separate files run in parallel

This comprehensive testing approach ensures your Rust code is reliable, maintainable, and well-documented!

```rust
// ============================================================================
// RUST TESTING STRATEGIES - COMPREHENSIVE GUIDE
// ============================================================================

// ============================================================================
// 1. UNIT TESTS
// ============================================================================
// Unit tests are placed in the same file as the code they test, inside a 
// `tests` module with the `#[cfg(test)]` attribute.

pub struct Calculator;

impl Calculator {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    fn private_helper(x: i32) -> i32 {
        x * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Calculator::add(2, 3), 5);
    }

    #[test]
    fn test_divide_success() {
        assert_eq!(Calculator::divide(10, 2), Ok(5));
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(Calculator::divide(10, 0).is_err());
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_should_panic() {
        assert_eq!(1, 2);
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn expensive_test() {
        // Expensive computation here
    }

    // Can test private functions in unit tests
    #[test]
    fn test_private_helper() {
        assert_eq!(Calculator::private_helper(5), 10);
    }
}

// ============================================================================
// 2. INTEGRATION TESTS
// ============================================================================
// Integration tests go in the `tests/` directory at the project root.
// They test your crate as an external user would, using only public APIs.

// File: tests/integration_test.rs
// (This would be in a separate file in the tests/ directory)

/*
// Example integration test file structure:
// tests/
//   ├── integration_test.rs
//   ├── common/
//   │   └── mod.rs  // Shared test utilities
//   └── another_test.rs

use my_crate::Calculator;

#[test]
fn test_calculator_from_outside() {
    assert_eq!(Calculator::add(1, 1), 2);
}

// Cannot access private functions here
// #[test]
// fn cannot_test_private() {
//     Calculator::private_helper(5); // This would fail to compile
// }
*/

// ============================================================================
// 3. DOC TESTS
// ============================================================================
// Tests written in documentation comments are compiled and run.
// They serve as both documentation and tests.

/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use rust_testing::Calculator;
///
/// let result = Calculator::add(2, 2);
/// assert_eq!(result, 4);
/// ```
///
/// You can also show error cases:
///
/// ```
/// use rust_testing::Calculator;
///
/// let result = Calculator::divide(10, 2);
/// assert_eq!(result, Ok(5));
/// ```
///
/// Doc tests can be marked to demonstrate failures:
///
/// ```should_panic
/// use rust_testing::Calculator;
/// 
/// // This will panic
/// Calculator::divide(10, 0).unwrap();
/// ```
///
/// Or marked as compile_fail for code that shouldn't compile:
///
/// ```compile_fail
/// use rust_testing::Calculator;
/// 
/// // This won't compile (wrong type)
/// Calculator::add("two", "three");
/// ```
pub struct DocumentedCalculator;

// ============================================================================
// 4. PROPERTY-BASED TESTING (PROPTEST)
// ============================================================================
// Property-based testing generates random inputs to verify properties
// Add to Cargo.toml: proptest = "1.0"

#[cfg(test)]
mod proptest_examples {
    use super::*;
    
    // Uncomment if you have proptest in your Cargo.toml:
    /*
    use proptest::prelude::*;

    proptest! {
        // Test that addition is commutative
        #[test]
        fn test_add_commutative(a: i32, b: i32) {
            prop_assert_eq!(Calculator::add(a, b), Calculator::add(b, a));
        }

        // Test that adding zero is identity
        #[test]
        fn test_add_identity(a: i32) {
            prop_assert_eq!(Calculator::add(a, 0), a);
        }

        // Test division with non-zero divisor always succeeds
        #[test]
        fn test_divide_non_zero(a: i32, b in 1..=i32::MAX) {
            prop_assert!(Calculator::divide(a, b).is_ok());
        }

        // Test with constrained ranges
        #[test]
        fn test_with_range(x in 0..100, y in 1..50) {
            let result = Calculator::divide(x, y);
            prop_assert!(result.is_ok());
            prop_assert!(result.unwrap() <= x);
        }
    }

    // Custom strategies for complex types
    proptest! {
        #[test]
        fn test_with_custom_strategy(
            values in prop::collection::vec(1..100i32, 0..10)
        ) {
            // Test that all generated values are in range
            for val in values {
                prop_assert!(val >= 1 && val < 100);
            }
        }
    }
    */
}

// ============================================================================
// 5. MOCKING STRATEGIES
// ============================================================================

// Strategy 1: Trait-based mocking (manual)
pub trait DataStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

pub struct RealDataStore {
    // Real implementation with database, etc.
}

impl DataStore for RealDataStore {
    fn get(&self, key: &str) -> Option<String> {
        // Real database call
        None
    }

    fn set(&mut self, key: &str, value: String) {
        // Real database write
    }
}

// Mock implementation for testing
#[cfg(test)]
pub struct MockDataStore {
    data: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl MockDataStore {
    pub fn new() -> Self {
        MockDataStore {
            data: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
impl DataStore for MockDataStore {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}

pub struct Service<T: DataStore> {
    store: T,
}

impl<T: DataStore> Service<T> {
    pub fn new(store: T) -> Self {
        Service { store }
    }

    pub fn process(&mut self, key: &str) -> Option<String> {
        self.store.get(key).map(|v| v.to_uppercase())
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[test]
    fn test_service_with_mock() {
        let mut mock_store = MockDataStore::new();
        mock_store.set("test", "hello".to_string());
        
        let mut service = Service::new(mock_store);
        let result = service.process("test");
        
        assert_eq!(result, Some("HELLO".to_string()));
    }
}

// Strategy 2: Using mockall crate (add to Cargo.toml: mockall = "0.12")
#[cfg(test)]
mod mockall_examples {
    /*
    use mockall::{automock, predicate::*};

    #[automock]
    trait Database {
        fn fetch_user(&self, id: u32) -> Option<String>;
        fn save_user(&mut self, id: u32, name: String) -> Result<(), String>;
    }

    struct UserService<D: Database> {
        db: D,
    }

    impl<D: Database> UserService<D> {
        fn get_user_uppercase(&self, id: u32) -> Option<String> {
            self.db.fetch_user(id).map(|name| name.to_uppercase())
        }
    }

    #[test]
    fn test_with_mockall() {
        let mut mock = MockDatabase::new();
        
        // Set expectations
        mock.expect_fetch_user()
            .with(eq(1))
            .times(1)
            .returning(|_| Some("alice".to_string()));
        
        let service = UserService { db: mock };
        let result = service.get_user_uppercase(1);
        
        assert_eq!(result, Some("ALICE".to_string()));
    }

    #[test]
    #[should_panic(expected = "MockDatabase::fetch_user: No matching expectation found")]
    fn test_expectation_not_met() {
        let mut mock = MockDatabase::new();
        
        mock.expect_fetch_user()
            .with(eq(1))
            .times(1)
            .returning(|_| Some("bob".to_string()));
        
        let service = UserService { db: mock };
        // Calling with wrong ID will panic
        service.get_user_uppercase(2);
    }
    */
}

// ============================================================================
// ADVANCED TESTING PATTERNS
// ============================================================================

// Test fixtures with setup/teardown
#[cfg(test)]
mod fixture_tests {
    struct TestFixture {
        data: Vec<i32>,
    }

    impl TestFixture {
        fn setup() -> Self {
            TestFixture {
                data: vec![1, 2, 3, 4, 5],
            }
        }
    }

    #[test]
    fn test_with_fixture() {
        let fixture = TestFixture::setup();
        assert_eq!(fixture.data.len(), 5);
    }
}

// Parameterized tests
#[cfg(test)]
mod parameterized_tests {
    use super::*;

    #[test]
    fn test_multiple_cases() {
        let test_cases = vec![
            (2, 3, 5),
            (0, 0, 0),
            (-1, 1, 0),
            (100, 200, 300),
        ];

        for (a, b, expected) in test_cases {
            assert_eq!(
                Calculator::add(a, b),
                expected,
                "Failed for inputs: {}, {}",
                a,
                b
            );
        }
    }
}

// Benchmarking (requires nightly and benchmark feature)
// Add to Cargo.toml: [profile.bench] and use `cargo bench`
/*
#![feature(test)]
extern crate test;

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_add(b: &mut Bencher) {
        b.iter(|| Calculator::add(2, 2));
    }
}
*/

fn main() {
    println!("Run tests with: cargo test");
    println!("Run with output: cargo test -- --nocapture");
    println!("Run specific test: cargo test test_add");
    println!("Run ignored tests: cargo test -- --ignored");
    println!("Run benchmarks: cargo bench (requires nightly)");
}
```