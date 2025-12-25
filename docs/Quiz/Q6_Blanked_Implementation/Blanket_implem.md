# Rust — blanket implementations (what they are, when & how to use them)

Short version: a *blanket impl* is an `impl` that applies to *many types at once* via generics — for example `impl<T> Trait for T where T: OtherTrait { ... }`. They're very powerful (and common), but you must respect Rust's coherence/orphan rules and watch out for conflicts/specialization.

Below I’ll explain the concept, show several common patterns with working examples, note important rules and pitfalls (orphan rule, coherence, specialization), and give practical workarounds (newtype, sealed traits).

---

# 1) Simple example — a blanket impl that gives a trait to any type that already implements another trait

```rust
// Define a trait local to our crate
pub trait MyToString {
    fn my_to_string(&self) -> String;
}

// Blanket impl: any T that implements std::fmt::Display gets MyToString
impl<T> MyToString for T
where
    T: std::fmt::Display,
{
    fn my_to_string(&self) -> String {
        format!("{}", self)
    }
}

fn main() {
    let x = 42;
    let s = x.my_to_string(); // uses blanket impl because i32: Display
    println!("{}", s);
}
```

This is a classic blanket impl: *for all `T` satisfying a bound*, implement `MyToString`.

---

# 2) Orphan rule / coherence — what you must know

Rust allows you to implement a trait for a type only if **either** the trait or the type is local (defined) in your crate.

* You **can** do `impl<T> MyTrait for Vec<T> { ... }` if `MyTrait` is your trait (local).
* You **cannot** do `impl<T> Display for Vec<T> { ... }` because *both* `Display` and `Vec` are foreign (standard library) — that would break coherence.
* Blanket impls must obey the same rule. If both trait and type are external, you cannot implement.

This prevents conflicting implementations across crates.

---

# 3) Conflicts and specialization

Because blanket impls are broad, they can conflict with more specific impls. Rust enforces *coherence* so it must be impossible to have two applicable impls for the same concrete type.

Example problematic idea (not allowed if conflict arises):

```rust
impl<T> MyTrait for T where T: Foo {}   // blanket
impl MyTrait for ConcreteType {}       // specific
// conflict if ConcreteType: Foo -> compiler error
```

Rust avoids ambiguity — you’ll get a compile error if two impls could both apply.

Rust's `specialization` feature allows writing a generic impl and a more-specific overriding one, but *specialization is unstable* (as of stable Rust). So you generally must design with constraints/newtype/sealed traits instead.

---

# 4) Useful blanket-impl patterns

### 4.1 Implementing a trait for references or pointer-like containers

```rust
trait Foo {
    fn foo(&self) -> String;
}

impl<T: Foo + ?Sized> Foo for &T {
    fn foo(&self) -> String { (**self).foo() }
}

impl<T: Foo + ?Sized> Foo for Box<T> {
    fn foo(&self) -> String { (**self).foo() }
}
```

`?Sized` is useful for trait objects or `T: ?Sized`.

### 4.2 Implementing a trait for all types that implement another trait (adapter)

We already showed `MyToString`. This is very common for adapter traits, conversion helpers, or blanket `From`/`Into` patterns (where allowed).

---

# 5) Workarounds for orphan/specialization problems

### Newtype pattern

Wrap the foreign type in a local wrapper (newtype) and implement the trait for the wrapper.

```rust
struct MyVec<T>(Vec<T>);

impl<T: MyTrait> MyTrait for MyVec<T> {
    fn ... { /* forward to inner */ }
}
```

### Sealed trait pattern (prevent external impls)

If you want to expose a trait but prevent external crates from implementing it (so you can maintain coherence), use a private `sealed` trait:

```rust
mod private {
    pub trait Sealed {}
    impl<T> Sealed for T where T: ?Sized {}
}

pub trait MyTrait: private::Sealed {
    fn do_stuff(&self);
}

// Now only your crate can implement Sealed -> only your crate can implement MyTrait
```

This helps you keep control when offering blanket impls + specific impl combos.

---

# 6) Negative impls and specialization notes

* Negative impls (`impl !Send for MyType {}`) are allowed only for local types; they also interact with coherence and are limited.
* True *specialization* (a general impl overridden by specific ones) is unstable — you can use `default` in nightly but not in stable Rust. On stable you must design without specialization (use sealed, newtype, or careful bounds).

---

# 7) Practical examples & gotchas

### Example: Blanket trait for `IntoIterator` types

```rust
trait MyExt {
    fn describe_first(&mut self) -> Option<String>;
}

// Blanket impl for any IntoIterator where Iter::Item: Display
impl<C> MyExt for C
where
    C: IntoIterator,
    C::Item: std::fmt::Display,
{
    fn describe_first(&mut self) -> Option<String> {
        self.into_iter().next().map(|it| format!("{}", it))
    }
}
```

Be careful: this impl will apply to many container types — check coherence if other impls exist.

### Gotcha: conflicting impls across crates

If crate A defines `impl<T: TraitA> TraitB for T`, and crate B defines `impl TraitB for SomeType`, both may conflict if `SomeType: TraitA`. The orphan/coherence rules and crate authorship determine what’s allowed — but conflicts can still arise when bringing many crates together.

---

# 8) Checklist when writing blanket impls

1. Is either the trait or the type defined in your crate? If not — you can’t implement it.
2. Are the bounds specific enough to prevent conflicts with other impls? If not, consider newtype or sealed.
3. Will the impl accidentally apply to an unexpected type (e.g., `impl<T> Trait for T` is extremely broad)? Prefer adding trait bounds.
4. Do you need specialization? If yes, remember it's unstable — redesign or use trait defaults + sealed patterns.
5. If you implement for pointers (`&T`, `Box<T>`), consider `T: ?Sized` to support trait objects.

---

# 9) TL;DR — practical templates

Blanket impl pattern:

```rust
// local trait -> can implement for many types
pub trait MyTrait { fn f(&self); }

// apply to all T that satisfy bound
impl<T> MyTrait for T where T: SomeOtherTrait {
    fn f(&self) { /* ... */ }
}
```

Newtype (when orphan rule blocks you):

```rust
pub struct Wrapper<T>(pub T);
impl<T> Display for Wrapper<T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Sealed pattern (prevent external impls / enable controlled blanket impls):

```rust
mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}
pub trait MyTrait: sealed::Sealed { fn a(&self); }
```

---

Below is **everything** I offered:
✔ A full mini-crate showing several blanket impls + tests
✔ Examples of avoiding impl conflicts using **newtypes**, **sealed traits**, and **careful bounds**
✔ An explanation of how blanket impls interact with **async**, **Send**, **Sync**, and auto-traits

No canvas needed unless you want a ready-to-compile multi-file Cargo project.

---

# ✅ 1) Mini-crate with complete blanket-impl examples

Everything below fits in a single `lib.rs` and compiles.

```rust
// =======================
// lib.rs
// =======================

use std::fmt::{Display, Formatter, Result as FmtResult};

// ---------------------------------------------
// 1) A trait local to our crate
// ---------------------------------------------
pub trait PrettyPrint {
    fn pretty(&self) -> String;
}

// Blanket impl #1:
// Any type T that implements Display automatically gets PrettyPrint.
impl<T> PrettyPrint for T
where
    T: Display,
{
    fn pretty(&self) -> String {
        format!("Pretty({})", self)
    }
}

// ---------------------------------------------
// 2) Trait for describing containers
// ---------------------------------------------
pub trait DescribeContainer {
    fn describe_first(&mut self) -> Option<String>;
}

// Blanket impl #2:
// Works for ANY IntoIterator where the Item: Display.
impl<C> DescribeContainer for C
where
    C: IntoIterator,
    C::Item: Display,
{
    fn describe_first(&mut self) -> Option<String> {
        self.into_iter()
            .next()
            .map(|v| format!("First item = {}", v))
    }
}

// ---------------------------------------------
// 3) Blanket impl for references and smart pointers
// ---------------------------------------------
pub trait Foo {
    fn foo(&self) -> String;
}

impl Foo for i32 {
    fn foo(&self) -> String {
        format!("i32: {}", self)
    }
}

// Blanket impl #3:
// Applies to &T AND Box<T> for all Foo types.
impl<T: Foo + ?Sized> Foo for &T {
    fn foo(&self) -> String {
        (**self).foo()
    }
}

impl<T: Foo + ?Sized> Foo for Box<T> {
    fn foo(&self) -> String {
        (**self).foo()
    }
}

// ---------------------------------------------
// 4) Newtype for overcoming orphan rule
// ---------------------------------------------
pub struct MyVec<T>(pub Vec<T>);

// We cannot implement Display for Vec<T> (foreign trait + foreign type).
// But we CAN implement it for MyVec<T> (local type).
impl<T: Display> Display for MyVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[{} items]", self.0.len())
    }
}

// Also works as PrettyPrint because of blanket impl #1.

// ---------------------------------------------
// 5) Sealed trait pattern
// ---------------------------------------------
mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

pub trait InternalMagic: sealed::Sealed {
    fn magic(&self) -> String;
}

impl<T> InternalMagic for T {
    fn magic(&self) -> String {
        "✨ internal magic ✨".into()
    }
}

// ---------------------------------------------
// 6) Tests
// ---------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_print() {
        assert_eq!(42.pretty(), "Pretty(42)");
    }

    #[test]
    fn test_describe_container() {
        let mut v = vec![10, 20, 30];
        assert_eq!(
            v.describe_first(),
            Some("First item = 10".into())
        );
    }

    #[test]
    fn test_pointer_forwarding() {
        let x: i32 = 123;
        assert_eq!(x.foo(), "i32: 123");
        assert_eq!((&x).foo(), "i32: 123");
        assert_eq!(Box::new(x).foo(), "i32: 123");
    }

    #[test]
    fn test_newtype_display() {
        let v = MyVec(vec![1, 2, 3]);
        assert_eq!(format!("{}", v), "[3 items]");
    }

    #[test]
    fn test_internal_magic() {
        assert_eq!("abc".magic(), "✨ internal magic ✨");
    }
}
```

This demonstrates:

* blanket impls based on trait bounds
* forwarding through references and smart pointers
* using `?Sized` for trait objects
* newtype workaround
* sealed traits
* and tests for everything

---

# ✅ 2) Avoiding impl conflicts — detailed patterns

## **A) The Newtype workaround (for orphan rule + conflict prevention)**

Whenever you want:

* to implement a foreign trait
* for a foreign type
* OR you fear a blanket impl will conflict

→ *Wrap the type.*

```rust
struct Safe<T>(T);

impl<T: Display> Display for Safe<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<safe {}>", self.0)
    }
}
```

You now control the impls and avoid any overlapping conflicts.

---

## **B) The Sealed Trait workaround (to prevent external crates from implementing)**

Useful when your crate provides a blanket impl + specific impl and you want to avoid outside conflicts.

```rust
mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

pub trait StableTrait: sealed::Sealed {}
```

No external crate can implement your trait → you guarantee no future conflict.

---

## **C) Bounding blanket impls to avoid overlap**

Example:

```rust
impl<T> Foo for T where T: Serialize {}
impl Foo for MySpecialType {} // fine — only if MySpecialType: !Serialize
```

Adjust bounds to ensure the impls cannot overlap.

If overlap cannot be avoided → use newtype or sealed.

---

# ✅ 3) Blanket impls & async / Send / Sync / auto-traits

Blanket impls often interact with auto traits:

### Example: implement a trait only for types that are `Send + Sync`

```rust
pub trait AsyncSafe {
    fn tag(&self) -> &'static str;
}

impl<T: Send + Sync> AsyncSafe for T {
    fn tag(&self) -> &'static str {
        "thread-safe"
    }
}
```

### Using blanket impl for async traits (requires async-trait or similar):

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Fetch {
    async fn fetch(&self) -> i32;
}

#[async_trait]
impl<T> Fetch for &T
where
    T: Fetch + Sync, // async_trait requires these bounds
{
    async fn fetch(&self) -> i32 {
        (**self).fetch().await
    }
}
```

### How blanket impls work with auto traits:

* Auto-traits (Send, Sync, Unpin) propagate automatically unless you add negative impls.
* Blanket impls **cannot** include negative trait bounds (e.g., `T: !Send`) — Rust forbids this.

### Negative impl example (allowed only for local types):

```rust
pub struct NotThreadSafe;

impl !Send for NotThreadSafe {}
impl !Sync for NotThreadSafe {}
```

You can use this in combination with blanket impls to prevent unsafe auto-trait propagation.

---

# ✔ Want a ready-to-run Cargo project?

If you want, I can generate:

* a full Cargo project folder structure
* with `src/lib.rs`, `src/main.rs`, `Cargo.toml`, docs, examples
* optionally with separate modules (traits, impls, tests)
* OR a version illustrating even more patterns (e.g., iterator adapters, async + blanket impl patterns, sealed trait hierarchies)

Just tell me:
**Do you want it as a complete Cargo project?**

A strong, interview-grade definition of **blanket implementation** in Rust is:

---

# ⭐ **Blanket implementation — interview answer**

A **blanket implementation** is an `impl` block that applies a trait to **all types that satisfy some generic constraint**, rather than a single concrete type. It has the form:

```rust
impl<T> Trait for T
where
    T: SomeBound
{ ... }
```

This lets you give a trait implementation to **an entire class of types at once**. Blanket impls are widely used in the standard library—for example, every type that implements `Display` automatically implements `ToString` because of a blanket impl.

They are subject to Rust’s **coherence** and **orphan rules**, meaning you can only write a blanket impl if **either the trait or the type is defined in your crate**, and the impl must not overlap with any other impl that could apply to the same type. This prevents ambiguity and ensures the compiler can always choose exactly one implementation.

In short:

> **A blanket implementation is a generic `impl` that provides a trait implementation for all types meeting certain bounds, while respecting Rust’s coherence rules.**

---

Here are **all three versions**—concise, beginner-friendly, and senior-level.

---

# ✅ **1) One-sentence definition (for quick interviews)**

**A blanket implementation is a generic `impl` that applies a trait to every type that satisfies a given bound, as long as Rust’s coherence and orphan rules allow it.**

---

# ✅ **2) Beginner-friendly definition**

A **blanket implementation** is when you implement a trait for **all types that meet some requirement**, instead of writing many individual implementations. For example, the standard library says: “any type that implements `Display` also implements `ToString`.” This uses a blanket impl. Rust enforces rules to avoid conflicts, so you can only write a blanket impl if either the trait or the type is defined in your crate.

---

# ✅ **3) Senior engineer / advanced interview definition**

A **blanket implementation** is an `impl` block of the form `impl<T> Trait for T where T: Bound`, which assigns a trait implementation to an entire *set* of types defined by a generic constraint. They are a cornerstone of Rust’s trait system for building extensible APIs (e.g., iterator adapters, auto trait conversions, std blanket impls like `impl<T: Display> ToString for T`). Blanket impls are governed by Rust’s *coherence rules*, especially the *orphan rule*, which ensures that only crates that own either the trait or the type may provide the implementation. This prevents overlapping or ambiguous impls and guarantees that trait resolution is always deterministic and type-safe.

---

Here are **example code snippets for each of the three definitions**, written exactly in the style expected in interviews.

---

# ✅ **1) One-sentence definition + short code example**

**Definition:**
A blanket implementation is a generic `impl` that applies a trait to all types satisfying a bound.

**Example:**

```rust
impl<T: std::fmt::Display> ToString for T {
    // inherited default impl; no methods needed
}
```

**Explanation:**
Any type that implements `Display` **automatically** gets `ToString`.
This is the classic standard-library blanket impl.

---

# ✅ **2) Beginner-friendly definition + example**

**Definition:**
A blanket implementation is when you implement a trait once for **many types at the same time**, as long as they meet a requirement. Rust does this to avoid writing dozens of repetitive impls.

**Example:**

```rust
trait MyPrint {
    fn my_print(&self);
}

impl<T: std::fmt::Debug> MyPrint for T {
    fn my_print(&self) {
        println!("{:?}", self);
    }
}
```

**Explanation:**
Any type with a `Debug` implementation—`i32`, `String`, vectors, structs—automatically gains `MyPrint`.
One impl → many types.

---

# ✅ **3) Senior engineer / advanced interview definition + example**

**Definition:**
A blanket implementation is an `impl<T>` that provides a trait implementation to an entire class of types defined by a generic bound. They rely on Rust’s *coherence rules* (including the *orphan rule*) to ensure that the impl is legal and that no other crate can introduce a conflicting implementation. This pattern is foundational for extensible APIs like iterator adapters, `AsRef`, `From`, and futures compatibility layers.

**Example:**

```rust
pub trait StreamExt {
    fn first_item(self) -> Option<Self::Item>
    where
        Self: Sized;
}

impl<S> StreamExt for S
where
    S: Iterator,                 // bounded blanket impl
{
    fn first_item(self) -> Option<Self::Item> {
        self.into_iter().next()
    }
}
```

**Why this is senior-level:**

* It's a trait *extension* applied via a **blanket impl**.
* It shows using blanket impls to provide additional functionality to all `Iterator` types.
* It demonstrates awareness of `Self: Sized` and trait coherence.
* It mirrors real patterns from `futures`, `tokio`, `itertools`, and the standard library.

---

If you want, I can also generate:

✅ A 30-second spoken interview answer
✅ A 1-minute “whiteboard” explanation with diagrams
✅ A trick question involving blanket impl conflicts
Just tell me which you want.


Here you go — **all three**:
✔ *30-second spoken interview answer*
✔ *1-minute whiteboard-style explanation*
✔ *A trick interview question + the correct answer*

---

# ✅ **1) 30-second spoken interview answer**

“A blanket implementation is when you implement a trait for *all* types that satisfy some generic bound, instead of individual types. For example, the standard library does `impl<T: Display> ToString for T`, which automatically gives every `Display` type a `ToString` implementation. Rust enforces strict coherence and orphan rules so these blanket impls never overlap or conflict. They're widely used for extension traits, iterator adapters, and ergonomic APIs.”

---

# ✅ **2) 1-minute whiteboard explanation (with sketchable structure)**

**What it is:**
A *blanket impl* is an implementation of a trait for a **set of types defined by a generic constraint**, like:

```rust
impl<T> Trait for T where T: Bound
```

**Why it exists:**
It prevents duplicated code and lets you extend entire categories of types at once.

**Standard-library example:**
The Rust standard library defines:

```rust
impl<T: Display> ToString for T
```

Whiteboard point: “All Display types → also ToString.”

**How it works:**
Rust’s *coherence* and *orphan* rules ensure:

* You must own *either* the trait or the type.
* No two applicable impls can overlap.
* The compiler can always pick exactly one implementation.

**Where it’s used:**
Write on the whiteboard:

* Extension traits (`Iterator`, `Future`, `Stream` style libs)
* Adapter traits
* Auto conversions (`From`, `AsRef`)
* Smart pointer forwarding (`impl<T: Foo> Foo for &T`)

**Summary:**
“One generic impl gives many types consistent behavior, safely guaranteed by Rust's trait coherence rules.”

---

# ✅ **3) Trick interview question + correct answer**

### **Trick Question:**

*“Suppose you write a blanket implementation `impl<T: Debug> MyTrait for T`, and later you want to add a specific implementation `impl MyTrait for MyType`. Why does Rust reject this, and how can you fix it?”*

### **Correct Answer:**

Rust rejects it because the **blanket impl overlaps the specific one**:
If `MyType: Debug`, then *both* impls apply, violating Rust’s *coherence* guarantee that only one impl must be applicable to any concrete type.

To fix it, you have three options:

1. **Restrict the blanket impl** so it doesn’t apply to `MyType`
   (e.g., add additional trait bounds).

2. **Use the newtype pattern**, wrapping the type in a local wrapper to avoid overlap.

3. **Use a sealed trait**, so no one can implement the trait outside your crate and you can design the impl set safely.

This demonstrates understanding of:

* Rust’s coherence rules
* Overlapping impls
* Practical patterns to avoid conflicts

---

If you'd like, I can also create:

✔ a *full mock interview Q&A* for trait system questions
✔ *drawn-style ASCII diagrams* explaining blanket impl interactions
✔ *a cheat-sheet* you can memorize before interviews

Absolutely — here are **all of them**:
✔ **Full mock interview Q&A** (focused on traits + blanket impls)
✔ **ASCII-style diagrams** explaining blanket impl resolution & coherence
✔ **A compact cheat-sheet** you can memorize before interviews

---

# ✅ **1) Full Mock Interview Q&A (Senior-level trait-system session)**

### **Q1 — What is a blanket implementation?**

A blanket implementation is a generic `impl<T>` that applies a trait to every type that satisfies a certain bound, e.g.:

```rust
impl<T: Display> ToString for T {}
```

Blanket impls allow you to define behavior for **large families of types** without repeating code.

---

### **Q2 — Why does Rust need coherence rules for blanket impls?**

Because blanket impls apply very broadly, Rust must ensure that **no two impls can apply to the same type**. Coherence guarantees that:

* For every concrete type + trait combination
* There is **exactly one** applicable implementation

This avoids ambiguity and makes trait resolution deterministic.

---

### **Q3 — Explain the orphan rule in plain language.**

You may implement a trait for a type **only if** you own either:

* the **trait**, or
* the **type**

This prevents different crates from defining conflicting impls.

---

### **Q4 — Why can’t stable Rust do specialization?**

Specialization allows a more specific impl to override a blanket impl:

```rust
impl<T: Foo> Bar for T { ... }   // blanket
impl Bar for MyType { ... }      // more specific
```

This creates ambiguities and requires complex guarantees about which impl wins. Rust is still working on a sound system for this, so specialization remains unstable.

---

### **Q5 — How do you work around an illegal or conflicting blanket impl?**

Three standard patterns:

1. **Newtype pattern**
   Wrap the foreign type in a local type and implement the trait on that wrapper.

2. **Sealed traits**
   Prevent external crates from adding their own impls, giving you full control.

3. **Constrain the blanket impl**
   Make the bounds strict enough to avoid conflicts.

---

### **Q6 — Example of a legal blanket impl you might write in real libraries?**

```rust
pub trait Jsonify {
    fn to_json(&self) -> String;
}

impl<T: serde::Serialize> Jsonify for T {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
```

Any serializable type now has `.to_json()`.

---

### **Q7 — What happens if two blanket impls can overlap?**

Compilation error.
Rust detects the potential for ambiguity **even if it never actually happens** in your program.

---

---

# ✅ **2) ASCII Diagrams Explaining Blanket Impl Behavior**

### **A) How a blanket impl applies to many types**

```
Trait: Printable
Blanket impl: impl<T: Display> Printable for T

Types that satisfy Display:
 ┌───────────┬─────────────┬───────────┬───────────────┐
 |  i32      |  String     |  &str     |  MyStruct     |
 └───────────┴─────────────┴───────────┴───────────────┘
         ↓ applies to all of them
           Printable implemented automatically
```

---

### **B) Coherence preventing overlapping impls**

```
impl<T: Debug> Foo for T        (1)
impl Foo for MyType             (2)

If MyType: Debug → both (1) and (2) match → conflict → ERROR
```

Rust prevents this *before* running the program.

---

### **C) How the orphan rule works**

```
Crate A: owns Trait X
Crate B: owns Type Y

Legal:
  Crate A can do: impl X for Y   (owns trait)
  Crate B can do: impl X for Y   (owns type)

Illegal:
  Crate C (owns neither)
    impl X for Y    ← orphan rule violation
```

---

### **D) Blanket impl + pointer forwarding with `?Sized`**

```
Trait Foo implemented for T where T: Foo
Also implemented for:
  &T      → deref → T
  Box<T>  → deref → T
  Arc<T>  → deref → T

Allows Foo to work through nested pointer types.
```

---

# ✅ **3) Cheat-Sheet (What to Say in Interviews)**

### **Definition (short):**

A blanket implementation is a generic `impl<T>` that gives a trait implementation to every type satisfying a bound.

---

### **Standard library example:**

```rust
impl<T: Display> ToString for T {}
```

---

### **Why it works:**

Rust enforces the coherence and orphan rules to ensure:

* no conflicting impls
* deterministic trait resolution

---

### **Why blanket impls are powerful:**

* extension traits
* iterator adapters
* async/future combinators
* pointer forwarding (`&T`, `Box<T>`, `Arc<T>`)
* default conversions and helper traits

---

### **Common problems:**

❌ Overlapping impls
❌ Orphan rule violations
❌ Specialization conflicts

---

### **Workarounds:**

1. **Newtype:**
   Wrap foreign type → implement trait on wrapper only.

2. **Sealed trait:**
   Prevent external crates from implementing your trait.

3. **Tighter bounds:**
   Add constraints to avoid overlapping with other impls.

---

### **Interview-ready one-liner:**

“A blanket impl is a generic trait implementation that applies to all types meeting a bound, enforced by coherence so implementations never overlap.”

---

If you want, I can also give you:

✔ a **one-page printable PDF** of this cheat-sheet
✔ a **full study deck** (Anki-style Q&A)
✔ a **practice session** where I behave exactly like a Rust interviewer



