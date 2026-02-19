# Type Conversions: `From`, `Into`, `TryFrom`, `TryInto`, `AsRef`, `Borrow`

- **`From` / `Into`** — Always implement `From`, never `Into` directly; the blanket impl gives you `Into` for free. Heavily used with `?` for error conversion.
- **`TryFrom` / `TryInto`** — The fallible equivalents, returning `Result`. Same pattern: implement `TryFrom`, get `TryInto` automatically.
- **`AsRef` / `AsMut`** — Cheap reference-to-reference conversions for generic function params (e.g., accepting both `&str` and `String`). No semantic guarantees required.
- **`Borrow`** — Like `AsRef` but with the strict contract that `Hash`/`Eq`/`Ord` must be identical between the owned and borrowed forms. This is what makes `HashMap::get("key")` work when keys are `String`.

Key pitfalls highlighted: the orphan rule, the identity-`From` blanket impl conflict, why implementing `Into` directly breaks `From`, and the `Borrow` contract violation (wrong hash results, not a compile error).


Rust's standard library provides a rich set of conversion traits that form the backbone of ergonomic, type-safe APIs. Understanding when and how to use each trait — and the subtle pitfalls around coherence and blanket implementations — is essential for writing idiomatic Rust.

---

## Overview of the Conversion Trait Family

| Trait | Infallible? | Owned/Borrowed | Direction |
|---|---|---|---|
| `From<T>` | Yes | Owned | `T → Self` |
| `Into<T>` | Yes | Owned | `Self → T` |
| `TryFrom<T>` | No (`Result`) | Owned | `T → Self` |
| `TryInto<T>` | No (`Result`) | Owned | `Self → T` |
| `AsRef<T>` | Yes | Borrowed | `&Self → &T` |
| `AsMut<T>` | Yes | Borrowed | `&mut Self → &mut T` |
| `Borrow<T>` | Yes | Borrowed | `&Self → &T` (with stronger guarantees) |

---

## 1. `From` and `Into`

### `From<T>`

`From<T>` converts a value of type `T` into `Self`. It is the **primary** trait to implement — `Into` is derived automatically.

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

fn main() {
    let boiling = Celsius(100.0);
    let f = Fahrenheit::from(boiling);
    println!("{:.1}°F", f.0); // 212.0°F
}
```

### The Blanket `Into` Implementation

The standard library provides:

```rust
impl<T, U> Into<U> for T where U: From<T> { ... }
```

This means you **never** implement `Into` directly. Implement `From`, and you get `Into` for free.

```rust
// Works because From<Celsius> for Fahrenheit is defined above
let f: Fahrenheit = Celsius(0.0).into();
```

### `From` for Error Handling

`From` is deeply integrated with the `?` operator. When you use `?`, Rust calls `From::from` to convert the error type:

```rust
use std::num::ParseIntError;
use std::fmt;

#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    TooBig,
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn parse_and_validate(s: &str) -> Result<u8, AppError> {
    let n: u32 = s.parse()?; // ParseIntError auto-converted via From
    if n > 255 {
        return Err(AppError::TooBig);
    }
    Ok(n as u8)
}
```

### Using `Into` in Function Signatures

Prefer `Into<T>` in function parameters when you want to accept multiple types ergonomically:

```rust
fn greet(name: impl Into<String>) {
    let name = name.into();
    println!("Hello, {}!", name);
}

greet("Alice");            // &str → String
greet(String::from("Bob")); // String → String (no-op)
```

> **Idiom:** Accept `impl Into<T>` in APIs; implement `From<T>` in type definitions.

---

## 2. `TryFrom` and `TryInto`

When a conversion can fail, use `TryFrom<T>` / `TryInto<T>`. They return `Result<Self, Self::Error>`.

```rust
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
struct EvenNumber(i32);

impl TryFrom<i32> for EvenNumber {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value % 2 == 0 {
            Ok(EvenNumber(value))
        } else {
            Err(format!("{} is not even", value))
        }
    }
}

fn main() {
    let ok = EvenNumber::try_from(4);
    assert_eq!(ok, Ok(EvenNumber(4)));

    let err = EvenNumber::try_from(3);
    assert!(err.is_err());

    // TryInto is auto-derived just like Into
    let n: Result<EvenNumber, _> = 8_i32.try_into();
    assert_eq!(n, Ok(EvenNumber(8)));
}
```

### Common Standard Library Uses

```rust
use std::convert::TryFrom;

// i64 → i32 may overflow
let big: i64 = 300;
let small = i32::try_from(big); // Ok(300)

let too_big: i64 = i64::MAX;
let overflow = i32::try_from(too_big); // Err(TryFromIntError)
```

---

## 3. `AsRef<T>` and `AsMut<T>`

`AsRef<T>` provides a **cheap reference-to-reference conversion**. It is used heavily for generic functions that can accept multiple reference-like types.

```rust
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
```

### Using `AsRef` in Generic APIs

```rust
fn print_length<S: AsRef<str>>(s: S) {
    println!("Length: {}", s.as_ref().len());
}

print_length("hello");                  // &str
print_length(String::from("world"));    // String
print_length(std::borrow::Cow::from("cow")); // Cow<str>
```

### Implementing `AsRef`

```rust
struct FilePath(String);

impl AsRef<str> for FilePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<std::path::Path> for FilePath {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(&self.0)
    }
}
```

### `AsMut<T>`

The mutable counterpart:

```rust
fn zero_first<T: AsMut<[u8]>>(mut data: T) {
    let slice = data.as_mut();
    if !slice.is_empty() {
        slice[0] = 0;
    }
}

let mut buf = vec![1u8, 2, 3];
zero_first(&mut buf);
assert_eq!(buf[0], 0);
```

---

## 4. `Borrow<T>`

`Borrow<T>` is similar to `AsRef<T>` but carries **stronger semantic guarantees**: the borrowed value must have the same `Hash`, `Eq`, and `Ord` as the owned type. This is critical for collections.

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

### Why `Borrow` Exists: `HashMap` Lookups

`HashMap<K, V>` uses `Borrow` to allow lookups with a different type than the stored key:

```rust
use std::collections::HashMap;

let mut map: HashMap<String, i32> = HashMap::new();
map.insert("hello".to_string(), 42);

// We can look up with &str even though keys are String
// because String: Borrow<str>
let val = map.get("hello"); // works!
assert_eq!(val, Some(&42));
```

This works because `String` implements `Borrow<str>`, and the `Eq`/`Hash` of `String` and `str` are identical.

### Implementing `Borrow` Correctly

```rust
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Eq, PartialEq, Hash, Debug)]
struct UserId(u64);

// Borrow to &u64 is safe because Hash/Eq are consistent
impl Borrow<u64> for UserId {
    fn borrow(&self) -> &u64 {
        &self.0
    }
}

// Now you can look up by &u64 in HashMap<UserId, ...>
```

> **Critical Rule:** If `x: K` and you implement `Borrow<Q>` for `K`, then `x.borrow()` must produce the same hash and equality as `x`. Violating this is **undefined behavior** in practice (wrong results, not memory unsafety).

### `Borrow` vs `AsRef`

| | `Borrow<T>` | `AsRef<T>` |
|---|---|---|
| Semantic guarantee | `Hash`/`Eq`/`Ord` must match | None required |
| Primary use case | Collection lookups | Generic function params |
| Blanket impls | `T: Borrow<T>`, `&T: Borrow<T>` | `T: AsRef<T>`, `&T: AsRef<T>` |

```rust
// Both accept &str, String, Box<str>, etc.
fn with_asref<S: AsRef<str>>(s: S) { /* cheap, no hash guarantee */ }
fn with_borrow<S: Borrow<str>>(s: S) { /* suitable for map keys */ }
```

---

## 5. Coherence and Pitfalls

### The Orphan Rule

You can only implement a trait for a type if **either the trait or the type** is defined in your crate. This prevents conflicting implementations across crates.

```rust
// DOES NOT COMPILE — both Vec and From are foreign
// impl From<Vec<u8>> for Vec<u16> { ... } // ERROR
```

### Conflicting Blanket Implementations

The stdlib blanket `impl<T> From<T> for T` (identity conversion) means you **cannot** implement `From<MyType>` for `MyType` in any non-trivial way yourself — it's already covered.

```rust
// This compiles fine — converting FROM another type
impl From<u32> for MyId {
    fn from(n: u32) -> Self { MyId(n) }
}

// This would conflict with the blanket impl<T> From<T> for T
// impl From<MyId> for MyId { ... } // ERROR: conflicting
```

### `Into` Pitfall: Don't Implement Directly

```rust
// BAD: implementing Into directly
impl Into<String> for MyType {
    fn into(self) -> String { self.0 }
}
// Now From<MyType> for String is NOT available!
// String::from(my_value) won't work.

// GOOD: implement From instead
impl From<MyType> for String {
    fn from(val: MyType) -> String { val.0 }
}
// Both String::from(x) AND x.into() now work.
```

### `AsRef` Reflexivity Pitfall

The blanket `impl<T: ?Sized> AsRef<T> for &T` means that `&&str` implements `AsRef<str>` transitively, but this can sometimes cause ambiguity in complex generic bounds. Keep bounds simple.

---

## 6. Putting It Together: An Ergonomic API

```rust
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

#[derive(Debug)]
pub struct InvalidEmail;

impl TryFrom<String> for Email {
    type Error = InvalidEmail;
    fn try_from(s: String) -> Result<Self, InvalidEmail> {
        if s.contains('@') {
            Ok(Email(s))
        } else {
            Err(InvalidEmail)
        }
    }
}

impl TryFrom<&str> for Email {
    type Error = InvalidEmail;
    fn try_from(s: &str) -> Result<Self, InvalidEmail> {
        Email::try_from(s.to_string())
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Email {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// Ergonomic API: accept anything convertible to Email
pub fn send_email(to: impl TryInto<Email>) -> Result<(), Box<dyn std::error::Error>> {
    let email = to.try_into().map_err(|_| "invalid email")?;
    println!("Sending to: {}", email.as_ref());
    Ok(())
}

fn main() {
    send_email("user@example.com").unwrap();
    send_email(String::from("admin@example.com")).unwrap();

    // HashMap lookup by &str even though keys are Email
    use std::collections::HashMap;
    let mut db: HashMap<Email, &str> = HashMap::new();
    db.insert(Email::try_from("a@b.com").unwrap(), "Alice");
    let found = db.get("a@b.com"); // uses Borrow<str>
    assert_eq!(found, Some(&"Alice"));
}
```

---

## 7. Quick Reference: Which Trait to Use?

**Implement `From<T>`** when conversion is infallible and you own the type being converted into.

**Use `impl Into<T>` in parameters** when you want callers to pass multiple types that can become `T` — especially `&str` + `String`, or `i32` + `u8`, etc.

**Implement `TryFrom<T>`** when conversion can fail (validation, overflow, parsing). You get `TryInto` free.

**Implement `AsRef<T>`** when you want to expose your type as a reference to some inner slice, str, path, or other unsized type for use in generic functions.

**Implement `Borrow<T>`** only when the borrowed representation has identical `Hash`/`Eq`/`Ord` semantics — primarily for use as `HashMap`/`HashSet`/`BTreeMap` keys.

---

## Summary

Rust's conversion traits build a layered hierarchy of expressiveness:

- `From`/`Into` for cheap, infallible owned conversions
- `TryFrom`/`TryInto` for fallible owned conversions
- `AsRef`/`AsMut` for cheap, flexible borrowed conversions
- `Borrow`/`BorrowMut` for borrowed conversions with hash/equality contracts

The golden rules: **implement `From`, not `Into`**; **implement `Borrow` only when the semantic contract holds**; and always respect the orphan rule to avoid coherence conflicts. Following these patterns yields APIs that are both ergonomic for callers and correct for the compiler.