# Reflection and Runtime Type Information in Rust

**Core mechanisms** — `TypeId` for unique runtime type identity and `dyn Any` for type-erased values with safe downcasting (`downcast_ref`, `downcast_mut`, `downcast`).

**Practical patterns** — heterogeneous collections, a type-keyed registry, and a full plugin system example showing how `TypeId` + `Any` work together.

**Limitations** — the `'static` constraint, no field/method introspection in std, no stable cross-build IDs, no reflection-based serialization out of the box.

**Compile-time alternatives** — generics, trait objects, enums, derive macros, `const` generics, and nightly specialization — covering how Rust shifts most "reflection" work to compile time.

A summary table at the end gives a quick at-a-glance reference of what is and isn't available in Rust's reflection model.


## Introduction

Reflection is the ability of a program to examine and potentially modify its own structure and behavior at runtime. Many languages (Java, C#, Python) offer rich reflection systems. Rust deliberately takes a **minimal, zero-cost-abstractions** approach: it provides just enough runtime type information to be useful, while pushing most type-level work to **compile time**.

This document covers:
- `TypeId` — unique runtime type identifiers
- The `Any` trait — type-erased values with safe downcasting
- Downcasting patterns
- Limitations of Rust's reflection model
- Compile-time alternatives (`std::any::type_name`, generics, trait objects, macros, `const` generics)

---

## 1. `TypeId`

`TypeId` lives in `std::any` and is an opaque token that uniquely identifies a type at runtime. It supports equality and hashing, but intentionally reveals no other information about the type.

```rust
use std::any::TypeId;

fn main() {
    let id_i32   = TypeId::of::<i32>();
    let id_u32   = TypeId::of::<u32>();
    let id_i32_2 = TypeId::of::<i32>();

    println!("i32 == i32 : {}", id_i32 == id_i32_2); // true
    println!("i32 == u32 : {}", id_i32 == id_u32);   // false
}
```

### Key Properties
- Requires the type to be `'static` (no non-`'static` lifetime parameters).
- Stable within a single compilation, but **not** guaranteed to be stable across different compilations or versions.
- Useful for type dispatch in generic or heterogeneous contexts.

```rust
use std::any::TypeId;

fn is_string<T: 'static>(_val: &T) -> bool {
    TypeId::of::<T>() == TypeId::of::<String>()
}

fn main() {
    println!("{}", is_string(&String::from("hello"))); // true
    println!("{}", is_string(&42i32));                  // false
}
```

---

## 2. The `Any` Trait

`std::any::Any` is a trait automatically implemented for every `'static` type. It exposes a single method, `type_id()`, but its real power comes from the extension methods on `dyn Any`:

| Method | Description |
|---|---|
| `.is::<T>()` | Returns `true` if the boxed value is of type `T` |
| `.downcast_ref::<T>()` | Returns `Option<&T>` |
| `.downcast_mut::<T>()` | Returns `Option<&mut T>` |
| `.downcast::<T>()` | (on `Box<dyn Any>`) Returns `Result<Box<T>, Box<dyn Any>>` |

```rust
use std::any::Any;

fn print_if_string(val: &dyn Any) {
    if let Some(s) = val.downcast_ref::<String>() {
        println!("It's a String: {s}");
    } else {
        println!("Not a String");
    }
}

fn main() {
    let s = String::from("Hello, Any!");
    let n = 42i32;

    print_if_string(&s); // It's a String: Hello, Any!
    print_if_string(&n); // Not a String
}
```

### Storing Heterogeneous Values

`Box<dyn Any>` lets you store values of different types in a single collection:

```rust
use std::any::Any;

fn main() {
    let mut bag: Vec<Box<dyn Any>> = Vec::new();

    bag.push(Box::new(42i32));
    bag.push(Box::new("hello"));
    bag.push(Box::new(3.14f64));
    bag.push(Box::new(vec![1u8, 2, 3]));

    for item in &bag {
        if let Some(n) = item.downcast_ref::<i32>() {
            println!("i32: {n}");
        } else if let Some(s) = item.downcast_ref::<&str>() {
            println!("&str: {s}");
        } else if let Some(f) = item.downcast_ref::<f64>() {
            println!("f64: {f}");
        } else {
            println!("unknown type");
        }
    }
}
```

---

## 3. Downcasting in Detail

### `downcast_ref` and `downcast_mut`

These are safe, checked casts. They return `None` instead of panicking or causing undefined behavior.

```rust
use std::any::Any;

struct Config {
    debug: bool,
}

fn toggle_debug(val: &mut dyn Any) {
    if let Some(cfg) = val.downcast_mut::<Config>() {
        cfg.debug = !cfg.debug;
        println!("Debug is now: {}", cfg.debug);
    } else {
        println!("Not a Config");
    }
}

fn main() {
    let mut cfg = Config { debug: false };
    toggle_debug(&mut cfg); // Debug is now: true
}
```

### Consuming Downcast with `Box<dyn Any>`

```rust
use std::any::Any;

fn extract_string(val: Box<dyn Any>) -> Option<String> {
    val.downcast::<String>().ok().map(|b| *b)
}

fn main() {
    let boxed: Box<dyn Any> = Box::new(String::from("owned value"));
    match extract_string(boxed) {
        Some(s) => println!("Got: {s}"),
        None    => println!("Not a String"),
    }
}
```

### Building a Simple Type-Erased Registry

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
struct Registry {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Registry {
    fn insert<T: 'static>(&mut self, val: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(val));
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }
}

fn main() {
    let mut reg = Registry::default();
    reg.insert(42u32);
    reg.insert("global config");
    reg.insert(vec![1.0f64, 2.0, 3.0]);

    println!("{:?}", reg.get::<u32>());       // Some(42)
    println!("{:?}", reg.get::<&str>());      // Some("global config")
    println!("{:?}", reg.get::<Vec<f64>>()); // Some([1.0, 2.0, 3.0])
}
```

---

## 4. `std::any::type_name`

For **debugging and diagnostics**, `type_name::<T>()` returns a human-readable string of a type's name. It is not stable across compilers or versions and should never be used for logic.

```rust
use std::any::type_name;

fn log_type<T>() {
    println!("Type is: {}", type_name::<T>());
}

fn main() {
    log_type::<Vec<String>>();    // alloc::vec::Vec<alloc::string::String>
    log_type::<i32>();            // i32
    log_type::<Option<f64>>();   // core::option::Option<f64>
}
```

---

## 5. Limitations of Rust's Reflection Model

Rust's reflection is intentionally limited compared to Java or C#. Understanding these constraints is essential.

### 5.1 No Field or Method Introspection

You cannot list the fields of a struct, enumerate methods of a trait object, or read/write fields by name at runtime.

```rust
// This is NOT possible in Rust (pseudo-code):
// let fields = MyStruct::fields();  // ❌ does not exist
// let value  = obj.get_field("name"); // ❌ does not exist
```

### 5.2 The `'static` Constraint

`Any` requires `'static`. Types with non-`'static` lifetime parameters cannot be used with `Any`.

```rust
use std::any::Any;

struct Wrapper<'a>(&'a str);

// This will NOT compile:
// fn use_any(w: &dyn Any) {}  // Wrapper<'a> does not implement Any
```

### 5.3 `TypeId` Is Not Stable Across Compilations

`TypeId` values are deterministic within a program run but should not be serialized and compared across different builds or versions.

### 5.4 No Dynamic Method Dispatch via Reflection

You cannot call a method by name at runtime the way you can in Python (`getattr(obj, method_name)()`). Rust trait objects provide *static* dynamic dispatch — the vtable is fixed at compile time.

### 5.5 No Automatic Serialization

There is no built-in way to serialize a `dyn Any` to JSON/binary without knowing the concrete type. Crates like `serde` solve this, but require `derive` macros at compile time.

### 5.6 Generic Types Are Monomorphized

Because Rust monomorphizes generics at compile time, there is no runtime representation of a generic type parameter. `TypeId::of::<Vec<i32>>()` and `TypeId::of::<Vec<String>>()` are different types.

---

## 6. Compile-Time Alternatives

Because Rust favors zero-cost abstractions, many things done with runtime reflection elsewhere can be done at compile time in Rust.

### 6.1 Generics and Trait Bounds

The most idiomatic Rust alternative — let the compiler specialize behavior per type.

```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct Article { title: String }
struct Tweet   { content: String }

impl Summary for Article {
    fn summarize(&self) -> String { format!("Article: {}", self.title) }
}

impl Summary for Tweet {
    fn summarize(&self) -> String { format!("Tweet: {}", self.content) }
}

fn print_summary<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}
```

### 6.2 Trait Objects for Runtime Polymorphism

When you genuinely need a heterogeneous collection, use `dyn Trait` rather than `dyn Any`:

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
struct Cat;

impl Animal for Dog { fn speak(&self) { println!("Woof!"); } }
impl Animal for Cat { fn speak(&self) { println!("Meow!"); } }

fn main() {
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
    for animal in &animals {
        animal.speak();
    }
}
```

### 6.3 Enums for Closed Type Sets

When the set of variants is known, an enum is simpler and faster than `dyn Any`:

```rust
enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

fn describe(v: &Value) {
    match v {
        Value::Int(n)   => println!("Integer: {n}"),
        Value::Float(f) => println!("Float: {f}"),
        Value::Text(s)  => println!("Text: {s}"),
        Value::Bool(b)  => println!("Bool: {b}"),
    }
}
```

### 6.4 Derive Macros for Compile-Time Code Generation

Procedural macros analyze types at compile time and generate code. This is how `serde`, `Debug`, `Clone`, etc. work — they are the Rust equivalent of reflection-based serialization frameworks.

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    println!("{p:?}"); // Point { x: 1.0, y: 2.0 }
}
```

### 6.5 `const` Generics and `const fn`

Compile-time computation over type-level constants:

```rust
fn array_sum<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()
}

fn main() {
    println!("{}", array_sum([1, 2, 3, 4, 5])); // 15
}
```

### 6.6 Specialization (Nightly)

The nightly `specialization` feature allows providing different implementations for more specific types — a limited, safe form of compile-time reflection:

```rust
// Nightly only — not in stable Rust as of 2025
#![feature(specialization)]

trait Stringify {
    fn stringify(&self) -> String;
}

impl<T: std::fmt::Debug> Stringify for T {
    default fn stringify(&self) -> String { format!("{:?}", self) }
}

impl Stringify for String {
    fn stringify(&self) -> String { self.clone() }
}
```

---

## 7. Third-Party Crates

The Rust ecosystem provides crates that extend reflection capabilities:

| Crate | Purpose |
|---|---|
| [`bevy_reflect`](https://docs.rs/bevy_reflect) | Full field-level reflection, dynamic access, type registry |
| [`typemap`](https://docs.rs/typemap_rev) | Type-keyed maps (like the Registry example above) |
| [`serde`](https://docs.rs/serde) | Compile-time serialization/deserialization via derive macros |
| [`erased-serde`](https://docs.rs/erased-serde) | Serialize `dyn Trait` objects |
| [`inventory`](https://docs.rs/inventory) | Collect items from across crates at startup (plugin systems) |

---

## 8. Complete Worked Example: Plugin System

Here is a practical use of `Any` + `TypeId` to build a simple runtime plugin registry:

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

trait Plugin: Any {
    fn name(&self) -> &str;
    fn run(&self);
    fn as_any(&self) -> &dyn Any;
}

struct LoggerPlugin { level: String }
struct MetricsPlugin { interval_ms: u64 }

impl Plugin for LoggerPlugin {
    fn name(&self) -> &str { "Logger" }
    fn run(&self) { println!("[Logger] level={}", self.level); }
    fn as_any(&self) -> &dyn Any { self }
}

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str { "Metrics" }
    fn run(&self) { println!("[Metrics] interval={}ms", self.interval_ms); }
    fn as_any(&self) -> &dyn Any { self }
}

struct PluginHost {
    plugins: HashMap<TypeId, Box<dyn Plugin>>,
}

impl PluginHost {
    fn new() -> Self { Self { plugins: HashMap::new() } }

    fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.insert(TypeId::of::<P>(), Box::new(plugin));
    }

    fn get<P: Plugin + 'static>(&self) -> Option<&P> {
        self.plugins
            .get(&TypeId::of::<P>())
            .and_then(|p| p.as_any().downcast_ref::<P>())
    }

    fn run_all(&self) {
        for plugin in self.plugins.values() {
            plugin.run();
        }
    }
}

fn main() {
    let mut host = PluginHost::new();
    host.register(LoggerPlugin  { level: "DEBUG".into() });
    host.register(MetricsPlugin { interval_ms: 500 });

    host.run_all();

    if let Some(logger) = host.get::<LoggerPlugin>() {
        println!("Logger level is: {}", logger.level);
    }
}
```

---

## 9. Summary

| Feature | Available in Rust | Notes |
|---|---|---|
| Runtime type identity (`TypeId`) | ✅ | `'static` types only |
| Type-erased values (`dyn Any`) | ✅ | `'static` types only |
| Safe downcasting | ✅ | Returns `Option`, never UB |
| Human-readable type name | ✅ (`type_name`) | Unstable string, debug only |
| Field/method introspection | ❌ (std) / ✅ (crates) | `bevy_reflect` etc. |
| Non-`'static` type reflection | ❌ | Fundamental limitation |
| Stable type IDs across builds | ❌ | Do not serialize `TypeId` |
| Compile-time alternatives | ✅✅ | Generics, traits, macros, enums |

Rust's philosophy is: **prefer compile-time correctness over runtime flexibility**. The `Any` + `TypeId` machinery covers the cases where runtime type erasure is genuinely needed (plugin systems, heterogeneous caches, event buses), while keeping the door open for zero-cost compile-time solutions in all other cases.