## `Option<T>` — Methods

---

## `take()` — Extract + Clear In One Move

```rust
let mut a = Some(42);
let b = a.take();
// b = Some(42),  a = None
```

---

## `replace()` — Swap In a New Value

```rust
let mut a = Some(42);
let b = a.replace(99);
// b = Some(42)  ← old value returned
// a = Some(99)  ← new value left behind
```

Like `take()` but instead of leaving `None`, it **plants a new value**. Atomic swap.

---

## `get_or_insert()` — Fill If Empty

```rust
let mut a: Option<i32> = None;
let b = a.get_or_insert(42);
// b = &mut 42  ← reference to the value
// a = Some(42) ← was None, now filled

let mut c = Some(99);
let d = c.get_or_insert(42);
// d = &mut 99  ← already had a value, untouched
// c = Some(99)
```

Only writes if the option is `None`. Returns a mutable reference to the inner value.

---

## `get_or_insert_with()` — Fill If Empty (Lazy)

```rust
let mut a: Option<String> = None;
let b = a.get_or_insert_with(|| String::from("hello"));
// closure only runs if a == None — avoids unnecessary allocation
```

Same as `get_or_insert()` but the value is computed from a closure — useful when construction is expensive.

---

## `get_or_insert_default()` — Fill With Default

```rust
let mut a: Option<i32> = None;
let b = a.get_or_insert_default();
// b = &mut 0   ← i32::default() = 0
// a = Some(0)
```

Requires `T: Default`. Shorthand for `get_or_insert_with(T::default)`.

---

## `as_ref()` / `as_mut()` — Borrow Without Consuming

```rust
let a = Some(String::from("hello"));

let b = a.as_ref();   // Option<&String> — borrow inner value
// a still alive, not moved

let mut c = Some(42);
let d = c.as_mut();   // Option<&mut i32> — mutable borrow
if let Some(v) = d {
    *v = 99;
}
// c = Some(99)
```

Doesn't modify the option itself — just lets you peek/modify inside without consuming ownership.

---

## Summary Table

```
┌────────────────────────┬──────────────────┬──────────────────┬─────────────────────┐
│ Method                 │ Before           │ After            │ Returns             │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ take()                 │ Some(x)          │ None             │ Some(x)             │
│                        │ None             │ None             │ None                │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ replace(y)             │ Some(x)          │ Some(y)          │ Some(x)             │
│                        │ None             │ Some(y)          │ None                │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert(y)       │ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(y)          │ &mut y              │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert_with(f)  │ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(f())        │ &mut f()            │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert_default()│ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(T::default) │ &mut T::default     │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ as_ref()               │ Some(x)          │ Some(x)          │ Option<&T>          │
│ as_mut()               │ Some(x)          │ Some(x)          │ Option<&mut T>      │
└────────────────────────┴──────────────────┴──────────────────┴─────────────────────┘
```

All of these operate **in-place** via `&mut self` — that's what groups them together. They mutate or inspect the `Option` without requiring you to consume or clone it.