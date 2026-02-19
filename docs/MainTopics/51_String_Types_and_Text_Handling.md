# String Types and Text Handling

## Topic Sections

**1. `String` vs `&str`** — The core ownership distinction. `String` is owned and heap-allocated; `&str` is a borrowed fat pointer (pointer + length). Includes the key idiom of accepting `&str` in function signatures for maximum caller flexibility.

**2. UTF-8, Characters, and Bytes** — Why you can't index a string with `s[i]`, and how to iterate by `chars()` vs `bytes()`. Covers safe slicing with byte indices and the `char_indices()` pattern for CJK and multi-byte characters.

**3. String Methods** — A practical tour of `contains`, `find`, `trim`, `split`, `replace`, `join`, case conversion, `format!`, and building strings efficiently with `String::with_capacity`.

**4. `OsStr` / `OsString`** — Platform-native strings that don't guarantee UTF-8, used when interfacing with environment variables, filenames, and OS APIs. Covers `to_str()` for fallible conversion and `to_string_lossy()` for the always-safe path.

**5. `Path` / `PathBuf`** — Thin wrappers over `OsStr`/`OsString` with path-aware methods (`file_name`, `extension`, `parent`, `push`, `join`, `set_extension`). Handles `/` vs `\` differences transparently.

**6. `Cow<str>`** — Clone-on-Write for avoiding allocation when input doesn't need modification. Shows the classic sanitizer pattern.

**7. Encoding and `from_utf8`** — Validated conversion from raw bytes, lossy conversion, `as_bytes()`, and a note on the `encoding_rs` crate for non-UTF-8 encodings.

**8. Advanced Slicing** — `char_indices`-based safe slicing, grapheme clusters via `unicode-segmentation` for emoji/combining characters.

**9. Practical Patterns** — Config line parsing, safe path building, and zero-allocation line iteration.

**10. Quick Reference Table** — All six types summarized by ownership, UTF-8 guarantee, and intended use case.

# 51. String Types and Text Handling in Rust

Rust takes a uniquely deliberate approach to text. Rather than offering one all-purpose string type, it provides a carefully layered family of types — each designed for a specific ownership, encoding, or platform context. Understanding when and why to reach for each one is essential to writing idiomatic, correct Rust.

---

## 1. The Core Duo: `String` vs `&str`

### `String` — Owned, Heap-Allocated, Growable

`String` is Rust's growable string type. It owns its data, stores it on the heap, and can be mutated freely.

```rust
fn main() {
    let mut s = String::from("Hello");
    s.push_str(", world!"); // mutate in place
    s.push('!');
    println!("{}", s); // Hello, world!!

    // String::new() for an empty string
    let mut greeting = String::new();
    greeting += "Hi there";

    // from a literal via .to_string()
    let owned: String = "Rust".to_string();
}
```

Internally, `String` is simply a `Vec<u8>` that is **guaranteed to contain valid UTF-8**. That guarantee is enforced at every API boundary.

### `&str` — Borrowed String Slice

`&str` is a **view** into a sequence of UTF-8 bytes. It is a fat pointer: a `(pointer, length)` pair. It does not own any data; it borrows from somewhere else — a string literal baked into the binary, a `String` on the heap, or any `[u8]` that is valid UTF-8.

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    // String literals are &'static str
    let literal: &'static str = "Alice";

    // Borrow a slice from a String
    let owned = String::from("Bob");
    let slice: &str = &owned;        // borrows the whole string
    let partial: &str = &owned[0..2]; // borrows just "Bo"

    greet(literal);
    greet(slice);
    greet("Carol"); // string literal works directly
}
```

### When to Use Which

| Situation | Use |
|---|---|
| Storing text that must outlive a scope | `String` |
| Accepting text from a caller (read-only) | `&str` |
| Building or modifying text | `String` |
| Function parameters (prefer generic) | `&str` |
| Returning text you just created | `String` |

A key idiom: accept `&str` in function signatures so callers can pass both `String` references and string literals without allocating.

```rust
// ✅ Flexible — accepts &str, String, &String, etc.
fn process(text: &str) -> usize {
    text.len()
}

fn main() {
    let owned = String::from("hello");
    process(&owned);      // &String coerces to &str
    process("literal");   // &'static str works directly
}
```

---

## 2. UTF-8, Characters, and Bytes

All Rust strings are **valid UTF-8**. This has practical consequences for indexing and iteration.

### You Cannot Index a String Directly

```rust
let s = String::from("café");
// let c = s[2]; // ❌ compile error — ambiguous: byte or char?
```

Instead, iterate explicitly:

```rust
let s = "café";

// Iterate over Unicode scalar values (char)
for c in s.chars() {
    print!("{} ", c); // c a f é
}

// Iterate over raw bytes
for b in s.bytes() {
    print!("{} ", b); // 99 97 102 195 169
}
```

Notice `café` has 4 characters but 5 bytes because `é` is encoded as two bytes (`0xC3 0xA9`) in UTF-8.

### Safe Slicing

Slicing is done by **byte index**, which must fall on a character boundary:

```rust
let s = "hello";
let slice = &s[1..4]; // "ell" — all ASCII, safe

let s2 = "café";
let e_acute = &s2[3..5]; // "é" — byte indices 3 and 5 are boundaries
// &s2[3..4] would panic at runtime — mid-char boundary
```

Use `char_indices()` for safe char-boundary-aware slicing:

```rust
let s = "日本語";
let mut indices = s.char_indices();
let (start, _) = indices.next().unwrap(); // 0
let (end, _) = indices.next().unwrap();   // 3 (each CJK char is 3 bytes)
println!("{}", &s[start..end]); // "日"
```

---

## 3. String Methods and Manipulation

`String` and `&str` share a rich set of methods via `Deref` coercion.

### Searching and Checking

```rust
let text = "  Hello, Rust World!  ";

println!("{}", text.contains("Rust"));         // true
println!("{}", text.starts_with("  Hello"));   // true
println!("{}", text.ends_with("!  "));         // true
println!("{:?}", text.find("Rust"));           // Some(9)
println!("{:?}", text.rfind('o'));             // Some(17)
```

### Trimming

```rust
let padded = "   hello   ";
println!("{:?}", padded.trim());         // "hello"
println!("{:?}", padded.trim_start());   // "hello   "
println!("{:?}", padded.trim_end());     // "   hello"

// Trim specific characters
let dotted = "...hello...";
println!("{}", dotted.trim_matches('.'));  // "hello"
```

### Splitting and Joining

```rust
let csv = "one,two,three";
let parts: Vec<&str> = csv.split(',').collect();
println!("{:?}", parts); // ["one", "two", "three"]

// Split by whitespace
let words: Vec<&str> = "  foo   bar  baz  ".split_whitespace().collect();
println!("{:?}", words); // ["foo", "bar", "baz"]

// Limit splits
let limited: Vec<&str> = "a:b:c:d".splitn(3, ':').collect();
println!("{:?}", limited); // ["a", "b", "c:d"]

// Join
let rejoined = parts.join(" | ");
println!("{}", rejoined); // one | two | three
```

### Replacing

```rust
let s = "foo bar foo baz foo";
println!("{}", s.replace("foo", "qux"));         // qux bar qux baz qux
println!("{}", s.replacen("foo", "qux", 2));     // qux bar qux baz foo
```

### Case

```rust
let s = "Hello, World!";
println!("{}", s.to_uppercase()); // HELLO, WORLD!
println!("{}", s.to_lowercase()); // hello, world!
```

### Building Strings Efficiently

```rust
// Use format! for composing strings
let name = "Alice";
let age = 30;
let bio = format!("{} is {} years old.", name, age);

// Use String::with_capacity to avoid reallocations
let mut buf = String::with_capacity(64);
for word in &["the", "quick", "brown", "fox"] {
    if !buf.is_empty() { buf.push(' '); }
    buf.push_str(word);
}
println!("{}", buf); // the quick brown fox

// Collect from an iterator
let result: String = vec!["a", "b", "c"]
    .into_iter()
    .collect::<Vec<_>>()
    .join("-");
println!("{}", result); // a-b-c
```

---

## 4. `OsStr` and `OsString` — Platform-Native Strings

The OS does not guarantee strings are UTF-8. Filenames on Linux are arbitrary byte sequences (minus `\0` and `/`); on Windows they are WTF-16. `OsStr` and `OsString` handle this reality.

```
OsStr   : &str  :: non-owning view, may not be valid UTF-8
OsString: String :: owned, heap-allocated OS string
```

```rust
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt; // Unix-only byte access

fn main() {
    // From Rust strings
    let os: OsString = OsString::from("hello.txt");
    let borrowed: &OsStr = &os;

    // Try to get a &str back — may fail on non-UTF-8 names
    match borrowed.to_str() {
        Some(s) => println!("Valid UTF-8: {}", s),
        None    => println!("Not valid UTF-8"),
    }

    // Always-safe lossy conversion (replaces invalid sequences with U+FFFD)
    let lossy: std::borrow::Cow<str> = borrowed.to_string_lossy();
    println!("{}", lossy);
}
```

You rarely construct `OsString` by hand; they surface when interacting with environment variables or directory entries:

```rust
use std::env;
for (key, value) in env::vars_os() {
    // key and value are OsString
    if let (Some(k), Some(v)) = (key.to_str(), value.to_str()) {
        println!("{} = {}", k, v);
    }
}
```

---

## 5. `Path` and `PathBuf` — Filesystem Paths

`Path` and `PathBuf` are thin wrappers around `OsStr`/`OsString` with path-aware methods. They automatically handle platform differences (`/` on Unix, `\` on Windows).

```
Path    : &OsStr  :: borrowed, non-owning path view
PathBuf : OsString :: owned, heap-allocated path
```

```rust
use std::path::{Path, PathBuf};

fn main() {
    // Borrowed path — like a &str for paths
    let p: &Path = Path::new("/usr/local/bin/rustc");

    println!("{:?}", p.file_name());       // Some("rustc")
    println!("{:?}", p.extension());       // None
    println!("{:?}", p.parent());          // Some("/usr/local/bin")
    println!("{:?}", p.components().count()); // 5
    println!("{}", p.display());           // /usr/local/bin/rustc

    // Owned path — like a String for paths
    let mut buf = PathBuf::from("/home/alice");
    buf.push("projects");        // /home/alice/projects
    buf.push("my_app");          // /home/alice/projects/my_app
    buf.set_extension("tar.gz"); // /home/alice/projects/my_app.tar.gz
    println!("{}", buf.display());

    // Joining paths
    let config_dir = Path::new("/etc");
    let config_file = config_dir.join("app").join("config.toml");
    println!("{}", config_file.display()); // /etc/app/config.toml
}
```

### Checking and Inspecting Paths

```rust
use std::path::Path;

let p = Path::new("src/main.rs");

println!("{:?}", p.file_stem());    // Some("main")
println!("{:?}", p.extension());    // Some("rs")
println!("{:?}", p.is_absolute());  // false
println!("{:?}", p.is_relative());  // true

// Runtime checks (hits the filesystem)
println!("{}", p.exists());         // true if the file exists
println!("{}", p.is_file());        // true if it's a regular file
println!("{}", p.is_dir());         // true if it's a directory
```

### Converting Between Path and String

```rust
use std::path::PathBuf;

let buf = PathBuf::from("/tmp/output.log");

// &str → PathBuf: always works
// PathBuf → String: may fail (non-UTF-8 paths)
match buf.to_str() {
    Some(s) => println!("Path as str: {}", s),
    None    => eprintln!("Path is not valid UTF-8"),
}

// Lossy — never fails, replaces bad bytes
let s = buf.to_string_lossy();
println!("{}", s);
```

---

## 6. The `Cow<str>` Pattern — Borrow or Own

`Cow<'a, str>` (Clone-on-Write) represents a value that is **either borrowed (`&'a str`) or owned (`String`)**. It avoids allocation when borrowing suffices.

```rust
use std::borrow::Cow;

fn sanitize(input: &str) -> Cow<str> {
    if input.contains('<') {
        // Must allocate a new String to replace content
        Cow::Owned(input.replace('<', "&lt;"))
    } else {
        // Safe to return a borrowed slice — no allocation
        Cow::Borrowed(input)
    }
}

fn main() {
    let clean   = sanitize("hello world");  // Borrowed — no allocation
    let escaped = sanitize("1 < 2");        // Owned — allocated once

    println!("{}", clean);   // hello world
    println!("{}", escaped); // 1 &lt; 2
}
```

`Cow<str>` shines in parsers, serializers, and any code where most inputs don't need modification.

---

## 7. String Encoding and `from_utf8`

Since `String` enforces UTF-8, converting raw bytes requires explicit validation:

```rust
fn main() {
    // Valid UTF-8 bytes
    let bytes = vec![72u8, 101, 108, 108, 111]; // "Hello"
    match String::from_utf8(bytes) {
        Ok(s)  => println!("{}", s),          // Hello
        Err(e) => eprintln!("Invalid: {}", e),
    }

    // Lossy — replaces invalid sequences with U+FFFD (replacement character)
    let bad_bytes = vec![0xFF, 0xFE, 72, 105];
    let s = String::from_utf8_lossy(&bad_bytes);
    println!("{}", s); // "Hi" with replacement chars before it

    // from_utf8 on a slice (no allocation)
    let raw: &[u8] = b"hello";
    let s: &str = std::str::from_utf8(raw).unwrap();
    println!("{}", s); // hello

    // Encode back to bytes
    let text = "Rust";
    let encoded: &[u8] = text.as_bytes();
    println!("{:?}", encoded); // [82, 117, 115, 116]
}
```

For non-UTF-8 encodings (Latin-1, Shift-JIS, etc.) use the `encoding_rs` crate:

```toml
# Cargo.toml
[dependencies]
encoding_rs = "0.8"
```

```rust
use encoding_rs::WINDOWS_1252;

let windows_bytes: &[u8] = &[0x48, 0xe9, 0x6c, 0x6c, 0x6f]; // "Héllo" in Windows-1252
let (cow, _encoding, had_errors) = WINDOWS_1252.decode(windows_bytes);
println!("{}", cow); // Héllo
```

---

## 8. Advanced Slicing — `char_indices` and `splitn`

```rust
fn first_n_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((idx, _)) => &s[..idx],
        None           => s,  // fewer than n chars — return whole string
    }
}

fn main() {
    println!("{}", first_n_chars("Hello, world!", 5)); // Hello
    println!("{}", first_n_chars("日本語テスト", 3));    // 日本語
}
```

### Grapheme Clusters

Some characters are composed of multiple Unicode scalar values (e.g. flag emojis, accented characters with combining marks). For true "user-perceived characters", use the `unicode-segmentation` crate:

```toml
[dependencies]
unicode-segmentation = "1"
```

```rust
use unicode_segmentation::UnicodeSegmentation;

let s = "Héllo"; // é may be e + combining acute (2 scalars, 1 grapheme)
println!("chars:     {}", s.chars().count());      // may be 6
println!("graphemes: {}", s.graphemes(true).count()); // 5
```

---

## 9. Practical Patterns

### Pattern: Parse a Config Line

```rust
fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');
    let key = parts.next()?.trim();
    let val = parts.next()?.trim();
    Some((key, val))
}

fn main() {
    let line = "  host = localhost  ";
    if let Some((k, v)) = parse_kv(line) {
        println!("key={:?} val={:?}", k, v); // key="host" val="localhost"
    }
}
```

### Pattern: Build a Path Safely

```rust
use std::path::PathBuf;

fn user_config_path(username: &str, app: &str) -> PathBuf {
    let mut p = PathBuf::from("/home");
    p.push(username);
    p.push(".config");
    p.push(app);
    p.push("settings.toml");
    p
}

fn main() {
    let path = user_config_path("alice", "myapp");
    println!("{}", path.display());
    // /home/alice/.config/myapp/settings.toml
}
```

### Pattern: Iterate Lines Without Allocation

```rust
let text = "line one\nline two\nline three";
for (i, line) in text.lines().enumerate() {
    println!("{}: {}", i + 1, line);
}
// 1: line one
// 2: line two
// 3: line three
```

`.lines()` yields `&str` slices — no allocation per line.

---

## 10. Quick Reference Table

| Type | Owned? | UTF-8 Guaranteed? | Use Case |
|---|---|---|---|
| `String` | Yes | Yes | Owning, building, mutating text |
| `&str` | No | Yes | Borrowing text (parameters, slices) |
| `OsString` | Yes | No | Owned OS-native strings |
| `OsStr` | No | No | Borrowed OS-native strings |
| `PathBuf` | Yes | No | Owned filesystem paths |
| `Path` | No | No | Borrowed filesystem paths |
| `Cow<str>` | Either | Yes | Avoid allocation when possible |

---

## Summary

Rust's string ecosystem is designed around two principles: **ownership clarity** and **encoding correctness**. Every type has a clear role. `String`/`&str` are your everyday text types with a UTF-8 guarantee. `OsString`/`OsStr` handle the messy reality of operating system APIs where UTF-8 is not guaranteed. `PathBuf`/`Path` add platform-aware path semantics on top of that. `Cow<str>` bridges the gap when you want to avoid allocation in the common case. Mastering these types — and the conversions between them — eliminates an entire class of text-handling bugs that plague programs written in languages with a single, unchecked string type.