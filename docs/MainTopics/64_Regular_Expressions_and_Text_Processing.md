# Regular Expressions and Text Processing in Rust

- **`regex` crate** — basic matching, compiled patterns with `once_cell::Lazy`, named/unnamed capture groups, replacements, and splitting
- **Unicode handling** — default Unicode-awareness, `\p{}` category classes, and byte-level opt-out
- **`RegexSet`** — matching multiple patterns in a single pass
- **`nom`** — parser combinator examples for key-value pairs and lists
- **`pest`** — PEG grammar file + derive macro approach for language-like grammars
- **Tool selection guide** — a comparison table to help pick the right approach
- **Performance tips** — avoiding recompilation, anchoring patterns, preferring `find` over `captures`
- **Standard library alternatives** — showing when no crate is needed at all


Rust does not include regex support in its standard library by design — the language prefers lean
dependencies. Instead, the ecosystem provides powerful, well-audited crates for every text-processing
need, from simple pattern matching to full parser combinators.

---

## 1. The `regex` Crate

Add it to your project:

```toml
[dependencies]
regex = "1"
```

### 1.1 Basic Matching

```rust
use regex::Regex;

fn main() {
    let re = Regex::new(r"\b\d{4}\b").unwrap();
    let text = "Call us at 2024 or visit in 1999.";

    for mat in re.find_iter(text) {
        println!("Found: {} at {}..{}", mat.as_str(), mat.start(), mat.end());
    }
}
// Found: 2024 at 10..14
// Found: 1999 at 27..31
```

### 1.2 Compiled Patterns — `Regex::new` vs `lazy_static` / `once_cell`

Compiling a regex is expensive. Always compile **once** and reuse:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// Compiled exactly once, on first use
static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap()
});

fn is_email(s: &str) -> bool {
    EMAIL_RE.is_match(s)
}

fn main() {
    println!("{}", is_email("user@example.com")); // true
    println!("{}", is_email("not-an-email"));     // false
}
```

> **Why `once_cell`?** It replaces the older `lazy_static!` macro with a cleaner API and is likely
> to enter `std` in a future edition.

### 1.3 Capture Groups

Named and unnamed capture groups are both supported:

```rust
use regex::Regex;

fn main() {
    let re = Regex::new(
        r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})"
    ).unwrap();

    let date = "Release date: 2024-03-15";

    if let Some(caps) = re.captures(date) {
        println!("Year:  {}", &caps["year"]);
        println!("Month: {}", &caps["month"]);
        println!("Day:   {}", &caps["day"]);

        // Also accessible by index
        println!("Full match: {}", caps.get(0).unwrap().as_str());
    }
}
```

Iterating over all captures in a string:

```rust
use regex::Regex;

fn main() {
    let re = Regex::new(r"(\w+)=(\w+)").unwrap();
    let config = "host=localhost port=5432 user=admin";

    for caps in re.captures_iter(config) {
        println!("{} -> {}", &caps[1], &caps[2]);
    }
}
// host -> localhost
// port -> 5432
// user -> admin
```

### 1.4 Replacement

```rust
use regex::Regex;

fn main() {
    let re = Regex::new(r"\b(\w+)\s+\1\b").unwrap(); // doubled words
    let result = re.replace_all("the the quick brown fox fox", "$1");
    println!("{}", result); // "the quick brown fox"
}
```

### 1.5 Splitting

```rust
use regex::Regex;

fn main() {
    let re = Regex::new(r"[,;\s]+").unwrap();
    let parts: Vec<&str> = re.split("one, two;  three four").collect();
    println!("{:?}", parts); // ["one", "two", "three", "four"]
}
```

---

## 2. Unicode Handling

The `regex` crate is **Unicode-aware by default**.

### 2.1 Unicode Character Classes

```rust
use regex::Regex;

fn main() {
    // \w matches Unicode letters, not just ASCII
    let re = Regex::new(r"^\w+$").unwrap();
    println!("{}", re.is_match("héllo"));  // true
    println!("{}", re.is_match("日本語")); // true
    println!("{}", re.is_match("hi!"));    // false
}
```

### 2.2 Unicode Categories with `\p{}`

```rust
use regex::Regex;

fn main() {
    // Match any Unicode uppercase letter
    let re = Regex::new(r"\p{Lu}+").unwrap();
    let text = "Hello WORLD, Ünïcödé";

    for mat in re.find_iter(text) {
        println!("{}", mat.as_str());
    }
    // H, WORLD, Ü
}
```

### 2.3 Disabling Unicode for Byte-Level Matching

When you need raw-byte performance (e.g., binary data):

```rust
use regex::bytes::Regex;

fn main() {
    let re = Regex::new(r"(?-u)\xFF\xFE").unwrap(); // BOM
    let data: &[u8] = b"\xFF\xFEhello";
    println!("{}", re.is_match(data)); // true
}
```

---

## 3. The `RegexSet` — Matching Multiple Patterns Efficiently

```rust
use regex::RegexSet;

fn main() {
    let set = RegexSet::new(&[
        r"^\d{5}$",          // US ZIP
        r"^\d{5}-\d{4}$",   // ZIP+4
        r"^[A-Z]\d[A-Z] \d[A-Z]\d$", // Canadian postal code
    ]).unwrap();

    let codes = ["90210", "10001-2345", "K1A 0B1", "invalid"];
    for code in &codes {
        let matches: Vec<_> = set.matches(code).into_iter().collect();
        println!("{}: patterns {:?}", code, matches);
    }
}
```

---

## 4. Parser Combinators: `nom`

`nom` is a zero-copy, streaming parser combinator library ideal for structured binary or text
formats where regex falls short.

```toml
[dependencies]
nom = "7"
```

### 4.1 Parsing a Simple Key-Value Pair

```rust
use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0},
    sequence::{separated_pair, delimited},
    IResult,
};

fn is_key_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_kv(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        take_while1(is_key_char),
        delimited(space0, char('='), space0),
        take_while1(|c: char| c != '\n' && c != ';'),
    )(input)
}

fn main() {
    let (rest, (key, val)) = parse_kv("timeout = 30").unwrap();
    println!("key={}, val={}, rest={:?}", key, val, rest);
    // key=timeout, val=30, rest=""
}
```

### 4.2 Parsing a List of Numbers

```rust
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space0},
    combinator::map_res,
    multi::separated_list1,
    IResult,
};

fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn parse_list(input: &str) -> IResult<&str, Vec<u32>> {
    separated_list1(
        delimited(space0, tag(","), space0),
        parse_u32,
    )(input)
}

use nom::sequence::delimited;

fn main() {
    let (_, nums) = parse_list("1, 2, 3, 42").unwrap();
    println!("{:?}", nums); // [1, 2, 3, 42]
}
```

---

## 5. PEG Parsers: `pest`

`pest` uses a clean **PEG (Parsing Expression Grammar)** defined in a separate `.pest` file,
generating a Rust parser at compile time via a derive macro.

```toml
[dependencies]
pest = "2"
pest_derive = "2"
```

**`src/grammar.pest`:**
```pest
WHITESPACE = _{ " " | "\t" }

integer = { ASCII_DIGIT+ }
ident   = { ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

expr    = { ident ~ "(" ~ integer ~ ")" }
```

**`src/main.rs`:**
```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ExprParser;

fn main() {
    let pairs = ExprParser::parse(Rule::expr, "foo(42)")
        .expect("parse error");

    for pair in pairs {
        for inner in pair.into_inner() {
            println!("Rule: {:?}  Text: {}", inner.as_rule(), inner.as_str());
        }
    }
}
// Rule: ident    Text: foo
// Rule: integer  Text: 42
```

---

## 6. Choosing the Right Tool

| Scenario | Best Choice |
|---|---|
| Simple pattern matching / search | `regex` |
| Validate well-known formats (email, IP) | `regex` with a `Lazy<Regex>` |
| Multiple patterns simultaneously | `regex::RegexSet` |
| Binary protocols, custom file formats | `nom` |
| Language-like grammars, DSLs | `pest` |
| Simple splitting / replacing | `str::split`, `str::replace` (no crate!) |

---

## 7. Performance Tips

**Avoid recompiling regexes in loops.** This is the single most common mistake:

```rust
// BAD — compiles the regex on every call
fn find_numbers_bad(text: &str) -> bool {
    Regex::new(r"\d+").unwrap().is_match(text)
}

// GOOD — compile once
static NUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").unwrap());

fn find_numbers_good(text: &str) -> bool {
    NUM_RE.is_match(text)
}
```

**Anchor patterns** to reduce backtracking:

```rust
// Unanchored — scans the whole string looking for a match
let re = Regex::new(r"\d+").unwrap();

// Anchored — asserts position, much faster on mismatches
let re = Regex::new(r"^\d+$").unwrap();
```

**Prefer `find` over `captures`** when you don't need groups — it skips the capture overhead.

---

## 8. Standard Library Alternatives

Many common tasks don't need regex at all:

```rust
fn main() {
    let s = "Hello, World!";

    // Check prefix / suffix
    println!("{}", s.starts_with("Hello")); // true
    println!("{}", s.ends_with("!"));       // true

    // Split on a pattern
    let words: Vec<&str> = s.split(|c: char| !c.is_alphabetic()).collect();
    println!("{:?}", words); // ["Hello", "World", ""]

    // Find a substring
    if let Some(pos) = s.find("World") {
        println!("Found at {}", pos); // 7
    }

    // Replace
    let new = s.replace("World", "Rust");
    println!("{}", new); // Hello, Rust!

    // Trim
    let padded = "  trim me  ";
    println!("{}", padded.trim()); // "trim me"
}
```

---

## Summary

Rust's text processing ecosystem is composable and performance-focused. The `regex` crate covers most
pattern-matching needs with full Unicode support and a safe, linear-time engine. When you need
structured parsing beyond what patterns can express, `nom`'s combinator style offers zero-copy
streaming, while `pest` provides a grammar-file approach that keeps parsing rules readable and
maintainable. And for the simplest cases, Rust's standard `str` methods are often all you need.