# Module System and Visibility

**Structure of the document:**

1. **What Is the Module System** — the crate/module tree mental model
2. **Declaring Modules** — inline vs file-based, the modern single-file style vs `mod.rs`
3. **Visibility Modifiers** — full breakdown of `pub`, `pub(crate)`, `pub(super)`, and `pub(in path)` with concrete examples for each
4. **`use` Declarations** — basic imports, `as` aliasing, glob imports, and nested path syntax
5. **Re-exports (`pub use`)** — flattening deep hierarchies, the prelude pattern, re-exporting dependencies
6. **Paths: Absolute vs Relative** — `crate::`, `super::`, `self::` explained side by side
7. **Crate Structure Best Practices** — canonical library layout, separating public API from internals, workspace crates
8. **Worked Example** — a self-contained mini-library wiring together every concept
9. **Quick-Reference Cheat-Sheet** — all syntax at a glance

The key insight threaded throughout is the **"default-private, opt-in visibility"** philosophy — design your internal machinery freely with `pub(crate)`, then sculpt your public API surface deliberately with `pub use`.

# 53. The Module System and Visibility

> **Topics covered:** Module hierarchy · `pub` / `pub(crate)` / `pub(super)` · `use` declarations · re-exports · crate structure best practices

---

## 1. What Is the Module System?

Rust's module system is the language's primary tool for **organising code** and **controlling visibility**. It answers two questions for every item (function, struct, trait, constant, …):

1. *Where* does this item live in the codebase?
2. *Who* is allowed to see and use it?

Every Rust program or library is itself a **crate**. Inside a crate, code is split into a tree of **modules**. The root of that tree is always called the **crate root** — `src/main.rs` for a binary crate, `src/lib.rs` for a library crate.

```
crate (src/lib.rs)
 ├── module: network
 │    ├── module: http
 │    └── module: tcp
 └── module: storage
      └── module: cache
```

---

## 2. Declaring Modules

### 2.1 Inline modules

```rust
// src/lib.rs

mod network {
    pub fn connect() {
        println!("Connecting…");
    }

    mod internal {          // private sub-module
        pub fn handshake() {
            println!("Handshaking…");
        }
    }

    pub fn establish() {
        internal::handshake(); // OK – same parent
    }
}

pub fn run() {
    network::connect();    // OK – connect() is pub
    // network::internal::handshake(); // ERROR – internal is private
}
```

### 2.2 File-based modules

For larger codebases, each module typically lives in its own file.

```
src/
  lib.rs
  network.rs          ← mod network;
  network/
    http.rs           ← mod http;
    tcp.rs            ← mod tcp;
  storage.rs          ← mod storage;
  storage/
    cache.rs          ← mod cache;
```

```rust
// src/lib.rs
pub mod network;   // loads src/network.rs
pub mod storage;   // loads src/storage.rs
```

```rust
// src/network.rs
pub mod http;      // loads src/network/http.rs
pub mod tcp;       // loads src/network/tcp.rs

pub fn connect() { /* … */ }
```

> **Old-style (`mod.rs`):** `src/network/mod.rs` is still valid but the newer single-file style (`src/network.rs`) is preferred in modern Rust projects.

---

## 3. Visibility Modifiers

By default, **everything in Rust is private** — visible only within the module where it is defined (and its descendants).

| Modifier | Visible to |
|---|---|
| *(none)* | The current module and its descendants only |
| `pub` | Anyone who can name the item |
| `pub(crate)` | Any code inside the same crate |
| `pub(super)` | The parent module only |
| `pub(in path)` | Any module at the given path |

### 3.1 `pub` — fully public

```rust
pub struct Config {
    pub host: String,       // readable and writable by anyone
    pub port: u16,
    password: String,       // private field – only accessible inside this module
}

impl Config {
    pub fn new(host: &str, port: u16, password: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            password: password.to_string(),
        }
    }

    pub fn is_secure(&self) -> bool {
        !self.password.is_empty()
    }
}
```

Even though `Config` is `pub`, the `password` field remains private. External code can call `cfg.is_secure()` but cannot read `cfg.password` directly.

### 3.2 `pub(crate)` — crate-internal API

Use this when something must be shared across modules in your crate but must **not** be exposed to downstream users of the library.

```rust
// src/storage/cache.rs

pub(crate) struct CacheEntry {
    pub(crate) key: String,
    pub(crate) value: Vec<u8>,
    expires_at: std::time::Instant,  // completely private
}

impl CacheEntry {
    pub(crate) fn new(key: &str, value: Vec<u8>, ttl_secs: u64) -> Self {
        Self {
            key: key.to_string(),
            value,
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(ttl_secs),
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        std::time::Instant::now() > self.expires_at
    }
}
```

Code in `src/network/http.rs` can freely use `CacheEntry`, but no consumer of your library ever sees it.

### 3.3 `pub(super)` — visible to parent only

This is ideal for helper code that belongs to a sub-module but needs to surface one level up.

```rust
// src/network/http.rs

mod parser {
    pub(super) fn parse_headers(raw: &str) -> Vec<(String, String)> {
        // … detailed parsing logic hidden from the rest of the crate
        vec![]
    }
}

pub fn handle_request(raw: &str) {
    let headers = parser::parse_headers(raw); // OK – http is super of parser
    println!("Got {} headers", headers.len());
}
```

```rust
// src/network/tcp.rs  (a sibling of http)
// parser::parse_headers(…)  // ERROR – pub(super) limits it to http only
```

### 3.4 `pub(in path)` — targeted visibility

A more surgical option when `pub(super)` isn't specific enough:

```rust
mod outer {
    mod inner {
        pub(in crate::outer) fn secret() -> &'static str {
            "only outer can call me"
        }
    }

    pub fn demo() {
        println!("{}", inner::secret()); // OK
    }
}
```

---

## 4. The `use` Declaration

`use` brings a path into scope so you don't have to repeat it.

### 4.1 Basic `use`

```rust
use std::collections::HashMap;
use std::io::{self, Read, Write}; // self brings io itself, plus Read and Write

fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        *map.entry(word).or_insert(0) += 1;
    }
    map
}
```

### 4.2 Aliasing with `as`

```rust
use std::fmt::Result as FmtResult;
use std::io::Result as IoResult;

fn fmt_something() -> FmtResult { Ok(()) }
fn read_something() -> IoResult<Vec<u8>> { Ok(vec![]) }
```

### 4.3 Glob imports (`*`)

```rust
// Only recommended for preludes, test modules, or clearly-scoped namespaces
use std::io::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;   // common pattern to bring parent items into test scope
    // …
}
```

### 4.4 Nested paths for concise imports

```rust
// Instead of:
use std::cmp::Ordering;
use std::cmp::min;

// Write:
use std::cmp::{Ordering, min};

// Or with self for the parent too:
use std::io::{self, BufReader, BufWriter};
```

---

## 5. Re-exports (`pub use`)

`pub use` re-exports an item, making it accessible at the re-exporting module's path. This is the cornerstone of designing a clean **public API** that hides internal structure.

### 5.1 Flattening a deep hierarchy

```rust
// src/lib.rs  – internal structure is an implementation detail
mod network;
mod storage;

// Public API surface – callers never need to know about submodules
pub use network::http::Client;
pub use network::http::Response;
pub use storage::cache::Cache;
```

External users can now write:

```rust
use my_crate::Client;    // instead of  my_crate::network::http::Client
use my_crate::Cache;     // instead of  my_crate::storage::cache::Cache
```

### 5.2 The prelude pattern

Many crates publish a `prelude` module that re-exports the most commonly needed types, allowing users to do a single wildcard import.

```rust
// src/prelude.rs
pub use crate::error::{Error, Result};
pub use crate::config::Config;
pub use crate::client::Client;
pub use crate::traits::{Fetch, Parse};
```

```rust
// user code
use my_crate::prelude::*;
```

### 5.3 Selective re-export from a dependency

```rust
// Expose serde's Serialize and Deserialize as if they were your own,
// so users of your crate don't have to add serde to their own dependencies.
pub use serde::{Deserialize, Serialize};
```

---

## 6. Paths: Absolute vs. Relative

```rust
mod audio {
    pub mod codec {
        pub fn encode() {}
    }

    pub fn process() {
        // relative path (from current module)
        codec::encode();

        // absolute path (always starts from crate root)
        crate::audio::codec::encode();
    }
}

// From outside the module:
fn main() {
    audio::codec::encode();           // relative from crate root
    crate::audio::codec::encode();    // absolute
}
```

Use `super::` to step up one level:

```rust
mod parent {
    pub fn helper() {}

    mod child {
        pub fn do_work() {
            super::helper();  // refers to parent::helper
        }
    }
}
```

---

## 7. Crate Structure Best Practices

### 7.1 Typical library layout

```
my_crate/
  Cargo.toml
  src/
    lib.rs          ← crate root; defines public API via re-exports
    error.rs        ← unified error type
    config.rs       ← configuration structs
    client.rs       ← main user-facing struct
    prelude.rs      ← convenience re-exports
    internal/       ← implementation details; pub(crate) items
      mod.rs  (or internal.rs)
      parser.rs
      pool.rs
```

### 7.2 Separate public API from internal machinery

```rust
// src/lib.rs

mod internal;           // private – never part of public docs
pub mod error;
pub mod config;

mod client;             // module is private …
pub use client::Client; // … but the type is re-exported publicly

pub mod prelude;
```

### 7.3 Use `pub(crate)` aggressively for cross-module helpers

```rust
// src/internal/parser.rs
pub(crate) fn tokenise(input: &str) -> Vec<Token> { /* … */ }

// src/client.rs
use crate::internal::parser::tokenise;
```

This keeps your public-facing surface lean while still sharing code freely inside the crate.

### 7.4 Document visibility decisions

```rust
/// Internal connection pool. Not part of the public API.
///
/// Exposed only within the crate so that [`Client`] and [`Session`]
/// can share pool instances.
pub(crate) struct Pool { /* … */ }
```

### 7.5 Workspace crates for large projects

For very large projects, split code into multiple crates inside a Cargo workspace:

```
my_workspace/
  Cargo.toml            ← [workspace] manifest
  crates/
    core/               ← shared types, traits
    client/             ← public-facing library, depends on core
    server/             ← binary, depends on core
    macros/             ← proc-macro crate
```

Each crate has its own visibility boundary. Items must be `pub` to cross crate borders even within a workspace.

---

## 8. Putting It All Together — A Worked Example

Below is a self-contained mini-library that demonstrates every concept covered above.

```rust
// ── src/lib.rs ────────────────────────────────────────────────────────────────

mod auth;
mod db;

// Flat public API: callers only need one import path
pub use auth::Credentials;
pub use db::connection::Connection;
pub use db::query::Query;

pub mod prelude {
    pub use super::{Connection, Credentials, Query};
}
```

```rust
// ── src/auth.rs ───────────────────────────────────────────────────────────────

pub struct Credentials {
    pub username: String,
    password_hash: String,   // never exposed
}

impl Credentials {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password_hash: hash(password),
        }
    }

    pub(crate) fn hash(&self) -> &str {
        &self.password_hash
    }
}

fn hash(password: &str) -> String {
    // simplified – use argon2 / bcrypt in production
    format!("{:x}", password.len())
}
```

```rust
// ── src/db/mod.rs (or src/db.rs + src/db/) ───────────────────────────────────

pub mod connection;
pub mod query;

mod pool;          // implementation detail – pub(crate) items inside
```

```rust
// ── src/db/pool.rs ────────────────────────────────────────────────────────────

pub(crate) struct Pool {
    max_size: usize,
}

impl Pool {
    pub(crate) fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    pub(crate) fn acquire(&self) -> Option<RawConn> {
        // … omitted
        None
    }
}

pub(crate) struct RawConn;  // raw socket handle – stays inside the crate
```

```rust
// ── src/db/connection.rs ──────────────────────────────────────────────────────

use super::pool::Pool;   // super = db module

pub struct Connection {
    pool: Pool,
}

impl Connection {
    pub fn new(max_connections: usize) -> Self {
        Self {
            pool: Pool::new(max_connections),
        }
    }

    pub fn ping(&self) -> bool {
        self.pool.acquire().is_some()
    }
}
```

```rust
// ── src/db/query.rs ───────────────────────────────────────────────────────────

use crate::auth::Credentials;   // absolute path from crate root

pub struct Query {
    sql: String,
}

impl Query {
    pub fn new(sql: &str) -> Self {
        Self { sql: sql.to_string() }
    }

    pub fn authenticated(sql: &str, creds: &Credentials) -> Self {
        // pub(crate) method on Credentials accessible here
        println!("Running as {} (hash {})", creds.username, creds.hash());
        Self::new(sql)
    }
}
```

External usage of the library is clean and flat:

```rust
use my_lib::prelude::*;

fn main() {
    let conn  = Connection::new(10);
    let creds = Credentials::new("alice", "s3cr3t");
    let query = Query::authenticated("SELECT 1", &creds);

    println!("Connected: {}", conn.ping());
}
```

---

## 9. Quick-Reference Cheat-Sheet

```rust
mod foo { }                      // declare inline module
mod foo;                         // load from foo.rs or foo/mod.rs

pub mod foo { }                  // public module
pub(crate) mod foo { }           // crate-internal module
pub(super) mod foo { }           // visible to parent only

pub fn f() { }                   // public function
pub(crate) fn f() { }            // crate-internal function
fn f() { }                       // private function

use std::collections::HashMap;   // bring into scope
use std::io::{self, Read};        // multiple items
use some::Type as Alias;         // alias
use some::module::*;             // glob (use sparingly)

pub use some::inner::Type;       // re-export (flattens API)

crate::module::item              // absolute path
super::sibling_fn()              // one level up
self::local_fn()                 // current module (rarely needed)
```

---

## Summary

The module system is the backbone of Rust's **encapsulation model**. The key takeaways are:

- **Default-private** — nothing is accidentally visible. You must opt in with a visibility modifier.
- **`pub(crate)` is your workhorse** for sharing implementation details across a crate without leaking them.
- **`pub use` lets you design the API independently** of the file/folder structure.
- **Keep the public surface small** — expose types and functions, not modules, wherever possible.
- **Workspaces** scale the same principles to multi-crate projects.

Mastering these tools lets you write libraries that are simultaneously flexible internally and impossible to misuse externally.