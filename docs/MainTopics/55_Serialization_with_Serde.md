# Serde reference document

- **Core concepts** — the `Serialize`/`Deserialize` traits and how they decouple your types from any format
- **Derive macros** — structs, enums, and all four enum tagging strategies (`external`, `internal`, `adjacent`, `untagged`)
- **Attribute system** — `rename_all`, `skip_serializing_if`, `default`, `flatten`, `alias`, `with`, and more with practical examples for each
- **Three major formats** — JSON (including streaming large files), TOML (config files), and Bincode (compact binary), each with full working code
- **Custom serializers** — using `serialize_with`/`deserialize_with`, implementing the traits manually via the `Visitor` pattern, and handling remote types with `#[serde(remote)]`
- **Schema evolution** — strategies for backward/forward compatibility: optional fields, `alias`, version enums, and `deny_unknown_fields`
- **Dynamic / untyped data** — `serde_json::Value`, JSON Pointer navigation, and partially typed structs
- **Performance tips** — zero-copy deserialization with lifetime-bound `&'de str`, streaming, and format selection guidance
- **Common pitfalls** — floating-point precision, bincode fragility, `untagged` enum cost, and recursive types
- **Quick-reference cheat-sheet** at the end for day-to-day use

# 55. Serialization with Serde

Serde (short for **Ser**ialize and **De**serialize) is Rust's de facto standard framework for converting data structures into formats like JSON, TOML, or binary, and back again. It is zero-overhead by design: the serialization logic is generated at compile time through Rust's powerful derive macro system, leaving no runtime reflection overhead.

---

## Table of Contents

1. [Core Concepts](#1-core-concepts)
2. [Serde Derive Macros](#2-serde-derive-macros)
3. [Field and Container Attributes](#3-field-and-container-attributes)
4. [Data Formats](#4-data-formats)
   - [JSON](#41-json-serde_json)
   - [TOML](#42-toml-toml-crate)
   - [Bincode](#43-binary-bincode)
5. [Custom Serializers and Deserializers](#5-custom-serializers-and-deserializers)
6. [Schema Evolution and Versioning](#6-schema-evolution-and-versioning)
7. [Working with Untyped / Dynamic Data](#7-working-with-untyped--dynamic-data)
8. [Performance Considerations](#8-performance-considerations)
9. [Common Pitfalls](#9-common-pitfalls)
10. [Quick Reference Cheat-Sheet](#10-quick-reference-cheat-sheet)

---

## 1. Core Concepts

Serde is built around two central traits:

```rust
pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}
```

The **data model** Serde operates on is a finite set of 29 types (booleans, integers, strings, sequences, maps, structs, enums, etc.). Any format that can express this data model can be used with any type that implements `Serialize`/`Deserialize`. This decoupling is what makes Serde so flexible.

```
Your Types  ←→  Serde Data Model  ←→  Data Formats
(Serialize/Deserialize)           (JSON, TOML, bincode…)
```

### Adding Serde to Your Project

```toml
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
bincode = "1"
```

---

## 2. Serde Derive Macros

The most common way to make a type serializable is to derive both traits automatically.

### Basic Struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

fn main() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("{}", json);
    // {
    //   "id": 1,
    //   "name": "Alice",
    //   "email": "alice@example.com",
    //   "active": true
    // }

    // Deserialize from JSON
    let back: User = serde_json::from_str(&json).unwrap();
    println!("{:?}", back);
}
```

### Enums

Serde supports all four standard enum representations:

```rust
use serde::{Deserialize, Serialize};

// --- External tagging (default) ---
// { "Cat": { "name": "Mittens" } }
#[derive(Serialize, Deserialize, Debug)]
enum Animal {
    Cat { name: String },
    Dog { name: String, breed: String },
    Fish,
}

// --- Internal tagging ---
// { "type": "Cat", "name": "Mittens" }
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum AnimalInternal {
    Cat { name: String },
    Dog { name: String, breed: String },
}

// --- Adjacent tagging ---
// { "t": "Cat", "c": { "name": "Mittens" } }
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "c")]
enum AnimalAdjacent {
    Cat { name: String },
    Dog { name: String, breed: String },
}

// --- Untagged ---
// tries to match the shape without any discriminator field
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum AnimalUntagged {
    Cat { name: String },
    Dog { name: String, breed: String },
}
```

---

## 3. Field and Container Attributes

Serde provides a rich set of attributes to control serialization behaviour without writing any manual code.

### `rename` — Change Field Names

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(rename = "max_connections")]
    max_conn: u32,

    #[serde(rename(serialize = "serverHost", deserialize = "server_host"))]
    host: String,
}
```

### `rename_all` — Case Conversion

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]  // snake_case → camelCase
struct ApiResponse {
    user_id: u64,       // serializes as "userId"
    first_name: String, // serializes as "firstName"
    last_name: String,  // serializes as "lastName"
}
```

Available strategies: `"lowercase"`, `"UPPERCASE"`, `"PascalCase"`, `"camelCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.

### `skip`, `skip_serializing`, `skip_deserializing`

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Session {
    user_id: u64,
    token: String,

    #[serde(skip)]                          // never serialized or deserialized
    internal_cache: Vec<String>,

    #[serde(skip_serializing)]              // only deserialized (read from input)
    legacy_field: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    optional_bio: Option<String>,           // omitted from output when None
}
```

### `default` — Provide Fallback Values

```rust
fn default_port() -> u16 { 8080 }

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    host: String,

    #[serde(default = "default_port")]
    port: u16,

    #[serde(default)]   // uses Default::default() — false for bool, 0 for ints, etc.
    tls_enabled: bool,
}
```

### `flatten` — Inline Nested Structs

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Pagination {
    page: u32,
    per_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListUsersRequest {
    search: String,
    #[serde(flatten)]
    pagination: Pagination,   // page & per_page appear at the top level
}
// Serializes as: { "search": "Alice", "page": 1, "per_page": 20 }
```

### `alias` — Accept Multiple Names on Deserialize

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Point {
    #[serde(alias = "x", alias = "lon", alias = "longitude")]
    x: f64,
    #[serde(alias = "y", alias = "lat", alias = "latitude")]
    y: f64,
}
```

---

## 4. Data Formats

### 4.1 JSON (`serde_json`)

The most widely used format. `serde_json` supports both typed and untyped access.

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    tags: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Serialize to string ---
    let product = Product {
        id: 42,
        name: "Widget".to_string(),
        price: 9.99,
        tags: vec!["sale".to_string(), "new".to_string()],
    };
    let json_str = serde_json::to_string(&product)?;
    let pretty   = serde_json::to_string_pretty(&product)?;

    // --- Serialize to bytes (for network/file I/O) ---
    let bytes = serde_json::to_vec(&product)?;

    // --- Deserialize ---
    let p2: Product = serde_json::from_str(&json_str)?;
    let p3: Product = serde_json::from_slice(&bytes)?;

    // --- Write directly to a writer (e.g. file) ---
    let file = std::fs::File::create("product.json")?;
    serde_json::to_writer_pretty(file, &product)?;

    // --- json! macro for ad-hoc values ---
    let val: Value = json!({
        "name": "Gadget",
        "price": 19.99,
        "in_stock": true
    });
    println!("{}", val["name"]); // "Gadget"

    Ok(())
}
```

#### Streaming Large JSON Files

```rust
use serde_json::Deserializer;
use std::fs::File;
use std::io::BufReader;

fn stream_json_array(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file   = File::open(path)?;
    let reader = BufReader::new(file);

    // Parse a top-level array element by element — avoids loading everything into RAM
    let stream = Deserializer::from_reader(reader).into_iter::<serde_json::Value>();
    for value in stream {
        println!("{:?}", value?);
    }
    Ok(())
}
```

---

### 4.2 TOML (`toml` crate)

TOML is Rust's preferred configuration format. The API mirrors `serde_json`.

```toml
# config.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://localhost/mydb"
pool_size = 10
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseConfig {
    url: String,
    pool_size: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read config file
    let content = std::fs::read_to_string("config.toml")?;
    let config: AppConfig = toml::from_str(&content)?;
    println!("{:#?}", config);

    // Serialize back to TOML string
    let toml_str = toml::to_string_pretty(&config)?;
    println!("{}", toml_str);

    Ok(())
}
```

---

### 4.3 Binary (`bincode`)

Bincode produces a compact binary representation — ideal for inter-process communication, caching, or game state.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GameState {
    score: u64,
    level: u32,
    player_pos: (f32, f32),
    inventory: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = GameState {
        score: 12_500,
        level: 7,
        player_pos: (10.5, 42.0),
        inventory: vec!["sword".to_string(), "potion".to_string()],
    };

    // Serialize to bytes — typically 60-80% smaller than JSON
    let encoded: Vec<u8> = bincode::serialize(&state)?;
    println!("Binary size: {} bytes", encoded.len());

    // Deserialize
    let decoded: GameState = bincode::deserialize(&encoded)?;
    assert_eq!(state, decoded);

    // Write to / read from a file
    let file = std::fs::File::create("state.bin")?;
    bincode::serialize_into(file, &state)?;

    let file = std::fs::File::open("state.bin")?;
    let loaded: GameState = bincode::deserialize_from(file)?;

    Ok(())
}
```

> **Note:** Bincode encodes no field names or type tags, so both sides must agree on the exact struct layout. This makes it unsuitable for long-term storage without versioning (see §6).

---

## 5. Custom Serializers and Deserializers

When the derive macros don't provide enough control, you can implement the traits manually or use the helper attributes `serialize_with` / `deserialize_with`.

### 5.1 `serialize_with` / `deserialize_with`

The lightest-weight customisation — attach a specific function to a single field.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod ts_seconds {
    use super::*;

    pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i64(dt.timestamp())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        let ts = i64::deserialize(d)?;
        DateTime::from_timestamp(ts, 0)
            .ok_or_else(|| serde::de::Error::custom("invalid unix timestamp"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    name: String,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
}
```

### 5.2 Implementing `Serialize` Manually

```rust
use serde::ser::{Serialize, SerializeStruct, Serializer};

struct Color(u8, u8, u8);

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize as a hex string: "#FF8000"
        let hex = format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2);
        serializer.serialize_str(&hex)
    }
}
```

### 5.3 Implementing `Deserialize` Manually

```rust
use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

struct Color(u8, u8, u8);

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex color string like #RRGGBB")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Color, E> {
        if value.len() != 7 || !value.starts_with('#') {
            return Err(E::custom(format!("invalid color: {}", value)));
        }
        let r = u8::from_str_radix(&value[1..3], 16).map_err(E::custom)?;
        let g = u8::from_str_radix(&value[3..5], 16).map_err(E::custom)?;
        let b = u8::from_str_radix(&value[5..7], 16).map_err(E::custom)?;
        Ok(Color(r, g, b))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        deserializer.deserialize_str(ColorVisitor)
    }
}
```

### 5.4 Remote Types (`remote`)

When you want to derive for a type you don't own:

```rust
// Define a "mirror" of the external type
#[derive(Serialize, Deserialize)]
#[serde(remote = "std::time::Duration")]
struct DurationDef {
    secs: u64,
    nanos: u32,
}

// Use it on a field
#[derive(Serialize, Deserialize)]
struct Task {
    name: String,
    #[serde(with = "DurationDef")]
    timeout: std::time::Duration,
}
```

---

## 6. Schema Evolution and Versioning

Real-world applications evolve: fields are added, renamed, or removed. Serde provides several strategies for backward and forward compatibility.

### 6.1 Adding Optional Fields (Backward Compatible)

```rust
#[derive(Serialize, Deserialize, Debug)]
struct UserV2 {
    id: u64,
    name: String,
    email: String,
    // New in V2 — old JSON without this field still parses fine
    #[serde(default)]
    role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
}
```

### 6.2 Renaming While Keeping Compatibility

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Profile {
    // Writes "username", but also accepts "login" from old data
    #[serde(rename = "username", alias = "login")]
    username: String,
}
```

### 6.3 Explicit Version Enum

The most robust approach for long-lived formats:

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum SaveFile {
    #[serde(rename = "1")]
    V1(SaveV1),
    #[serde(rename = "2")]
    V2(SaveV2),
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveV1 {
    player_name: String,
    score: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SaveV2 {
    player_name: String,
    score: u64,       // widened type
    achievements: Vec<String>,  // new field
}

impl From<SaveV1> for SaveV2 {
    fn from(v1: SaveV1) -> Self {
        SaveV2 {
            player_name: v1.player_name,
            score: v1.score as u64,
            achievements: vec![],
        }
    }
}

fn load_save(json: &str) -> SaveV2 {
    match serde_json::from_str::<SaveFile>(json).unwrap() {
        SaveFile::V1(v1) => v1.into(),
        SaveFile::V2(v2) => v2,
    }
}
```

### 6.4 Ignoring Unknown Fields

By default, unknown fields cause a deserialization error. Use `deny_unknown_fields` or allow them silently:

```rust
// Default: unknown fields are silently ignored (forward compatible)
#[derive(Deserialize)]
struct Request {
    action: String,
    payload: String,
}

// Strict: reject any extra fields (useful for strict API validation)
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictRequest {
    action: String,
    payload: String,
}

// Capture unknown fields into a map for debugging
use std::collections::HashMap;
#[derive(Deserialize)]
struct FlexibleRequest {
    action: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
```

---

## 7. Working with Untyped / Dynamic Data

Sometimes the shape of data is unknown at compile time. `serde_json::Value` represents any JSON value.

```rust
use serde_json::{json, Value};

fn process_dynamic(data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let v: Value = serde_json::from_str(data)?;

    // Navigate with indexing
    let name = v["user"]["name"].as_str().unwrap_or("unknown");
    let items = v["items"].as_array().map(|a| a.len()).unwrap_or(0);

    println!("User: {}, Items: {}", name, items);

    // Pointer syntax (JSON Pointer, RFC 6901)
    if let Some(city) = v.pointer("/user/address/city") {
        println!("City: {}", city);
    }

    Ok(())
}

// Partially typed: mix known and dynamic parts
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Event {
    event_type: String,
    timestamp: u64,
    #[serde(flatten)]
    payload: Value,   // rest of the fields captured dynamically
}
```

---

## 8. Performance Considerations

| Concern | Recommendation |
|---|---|
| Avoid heap allocations | Deserialize into `&'de str` instead of `String` where possible |
| Large JSON files | Use streaming via `serde_json::Deserializer::into_iter` |
| High-throughput binary | Prefer `bincode` or `rmp-serde` (MessagePack) over JSON |
| Repeated serialization of same type | Cache the serialized form; `serde_json::to_value()` + re-serialize is slower |
| Compile times | Feature-gate format crates; use `serde(bound = "...")` to tighten generic bounds |

```rust
// Zero-copy deserialization: borrows from the input buffer
#[derive(Deserialize, Debug)]
struct LogEntry<'a> {
    level: &'a str,    // borrows from source string, no allocation
    message: &'a str,
    timestamp: u64,
}

fn parse_log(raw: &str) -> LogEntry<'_> {
    serde_json::from_str(raw).unwrap()
}
```

---

## 9. Common Pitfalls

**Floating-point precision in JSON.** JSON numbers have no fixed precision. Use `Decimal` types or string representation for financial data.

```rust
#[derive(Serialize, Deserialize)]
struct Price {
    #[serde(serialize_with = "serialize_decimal", deserialize_with = "deserialize_decimal")]
    amount: rust_decimal::Decimal,
    currency: String,
}
```

**Bincode layout fragility.** Adding or reordering fields in a bincode-serialized struct silently corrupts old data. Always version binary formats.

**`untagged` enum performance.** Serde must try each variant until one matches. For hot paths, prefer tagged enums.

**Recursive types.** A type containing `Box<Self>` works fine with derive, but deeply nested structures can overflow the stack during deserialization. Consider `serde_json`'s `from_reader` with a custom stack limit.

**`#[serde(default)]` on enums.** The attribute is valid on struct fields and containers; on enum variants it has different semantics — the variant is chosen when the tag is missing.

---

## 10. Quick Reference Cheat-Sheet

```rust
// Most common attribute patterns

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]           // field name convention
#[serde(deny_unknown_fields)]               // strict parsing
#[serde(default)]                           // all fields get Default::default() if missing
struct MyStruct {

    #[serde(rename = "user_id")]            // override field name
    id: u64,

    #[serde(skip)]                          // exclude from both
    internal: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    bio: Option<String>,

    #[serde(default = "Vec::new")]          // custom default
    tags: Vec<String>,

    #[serde(alias = "old_name")]            // accept old key during deserialization
    new_name: String,

    #[serde(flatten)]                       // inline nested struct
    meta: Metadata,

    #[serde(with = "my_module")]            // custom ser/de module
    special: MyType,
}

// Enum representation
#[serde(tag = "type")]                      // internal tag
#[serde(tag = "t", content = "c")]          // adjacent tag
#[serde(untagged)]                          // no tag — match by shape

// Format functions
serde_json::to_string(&v)?                  // compact JSON string
serde_json::to_string_pretty(&v)?           // pretty-printed JSON
serde_json::from_str::<T>(s)?              // JSON string → T
toml::to_string(&v)?                        // TOML string
toml::from_str::<T>(s)?                    // TOML string → T
bincode::serialize(&v)?                     // binary Vec<u8>
bincode::deserialize::<T>(&bytes)?         // binary bytes → T
```

---

## Summary

Serde is one of Rust's most mature and powerful libraries. Its key strengths are:

- **Zero-cost abstractions** — all dispatch resolved at compile time.
- **Format agnosticism** — swap JSON for TOML or bincode with a one-line change.
- **Composability** — derive macros handle the 95% case; the `Serialize`/`Deserialize` traits cover the rest.
- **Schema evolution** — `default`, `alias`, `flatten`, and versioned enums give fine-grained control over backward compatibility.

Mastering the attribute system and understanding when to write custom implementations gives you complete control over how your Rust types are represented in any serialized format.