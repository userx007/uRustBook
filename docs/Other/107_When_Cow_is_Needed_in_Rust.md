# Comprehensive Guide to `Cow<T>` in Rust

`Cow<T>` (Clone on Write) is a smart pointer in Rust that provides conditional ownership. It can hold either borrowed data or owned data, cloning only when mutation is necessary.

## What is `Cow<T>`?

```rust
pub enum Cow<'a, B> 
where
    B: 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

The type `B` is typically `str` (giving `Cow<'a, str>`) or `[T]` (giving `Cow<'a, [T]>`).

## Core Use Cases

### 1. **Conditional String Modification**

When you might need to modify a string, but often don't:

```rust
fn process_name(name: &str) -> Cow<str> {
    if name.contains("  ") {
        // Need to modify - clone and return owned
        Cow::Owned(name.replace("  ", " "))
    } else {
        // No modification needed - return borrowed
        Cow::Borrowed(name)
    }
}
```

**Why Cow?** Avoids unnecessary allocations when the string doesn't need modification.

### 2. **Escaping Special Characters**

HTML/URL encoding, JSON escaping, or similar transformations:

```rust
fn escape_html(input: &str) -> Cow<str> {
    if input.contains(&['<', '>', '&'][..]) {
        Cow::Owned(
            input
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        )
    } else {
        Cow::Borrowed(input)
    }
}
```

**Why Cow?** Most strings don't need escaping, so you avoid allocation in the common case.

### 3. **Case Normalization**

Converting to lowercase/uppercase only when needed:

```rust
fn normalize_key(key: &str) -> Cow<str> {
    if key.chars().all(|c| c.is_lowercase()) {
        Cow::Borrowed(key)
    } else {
        Cow::Owned(key.to_lowercase())
    }
}
```

### 4. **Path Manipulation**

Working with file paths that may or may not need modification:

```rust
use std::path::Path;

fn ensure_extension(path: &Path, ext: &str) -> Cow<Path> {
    if path.extension().and_then(|s| s.to_str()) == Some(ext) {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(path.with_extension(ext))
    }
}
```

### 5. **Validation with Sanitization**

Accepting input that usually passes validation:

```rust
fn sanitize_username(name: &str) -> Cow<str> {
    let invalid_chars = name.chars()
        .any(|c| !c.is_alphanumeric() && c != '_');
    
    if invalid_chars {
        Cow::Owned(
            name.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        )
    } else {
        Cow::Borrowed(name)
    }
}
```

### 6. **Deserialalization/Configuration**

When default values might be used instead of allocated strings:

```rust
struct Config<'a> {
    host: Cow<'a, str>,
    port: u16,
}

impl<'a> Config<'a> {
    fn new(host: Option<&'a str>) -> Self {
        Config {
            host: host.map(Cow::Borrowed)
                     .unwrap_or(Cow::Borrowed("localhost")),
            port: 8080,
        }
    }
}
```

### 7. **Lazy String Building**

When you might append to a string, but often don't:

```rust
fn add_suffix_if_needed(s: &str, needs_suffix: bool) -> Cow<str> {
    if needs_suffix {
        Cow::Owned(format!("{}_processed", s))
    } else {
        Cow::Borrowed(s)
    }
}
```

### 8. **Slice Operations**

Working with slices that may need extension:

```rust
fn ensure_min_length(data: &[u8], min_len: usize) -> Cow<[u8]> {
    if data.len() >= min_len {
        Cow::Borrowed(data)
    } else {
        let mut owned = data.to_vec();
        owned.resize(min_len, 0);
        Cow::Owned(owned)
    }
}
```

### 9. **API Design: Flexible Input**

Accepting either owned or borrowed data from callers:

```rust
fn process_data(data: Cow<str>) {
    // Can work with either borrowed or owned data
    println!("Processing: {}", data);
}

// Callers can pass either:
process_data(Cow::Borrowed("borrowed"));
process_data(Cow::Owned(String::from("owned")));
```

### 10. **Trim Operations**

Removing whitespace only when present:

```rust
fn trim_if_needed(s: &str) -> Cow<str> {
    let trimmed = s.trim();
    if trimmed.len() == s.len() {
        Cow::Borrowed(s)
    } else {
        Cow::Borrowed(trimmed)  // Still borrowed, just different slice
    }
}
```

### 11. **Prefix/Suffix Addition**

Adding markers only under certain conditions:

```rust
fn maybe_add_protocol(url: &str) -> Cow<str> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Cow::Borrowed(url)
    } else {
        Cow::Owned(format!("https://{}", url))
    }
}
```

### 12. **Template Expansion**

When templates often don't have variables:

```rust
fn expand_template(template: &str, vars: &HashMap<String, String>) -> Cow<str> {
    if !template.contains("${") {
        return Cow::Borrowed(template);
    }
    
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
    }
    Cow::Owned(result)
}
```

### 13. **Error Messages with Context**

Building error messages that may or may not need additional context:

```rust
fn format_error(base_msg: &str, context: Option<&str>) -> Cow<str> {
    match context {
        Some(ctx) => Cow::Owned(format!("{}: {}", base_msg, ctx)),
        None => Cow::Borrowed(base_msg),
    }
}
```

## When NOT to Use `Cow<T>`

### 1. **Always Modifying**
If you always clone/modify, just return `String` or `Vec<T>`.

### 2. **Simple Borrows**
If you never own the data, just use `&str` or `&[T]`.

### 3. **Always Owning**
If you always create new data, use `String` or `Vec<T>` directly.

### 4. **Performance Critical Paths**
The enum check has a tiny cost; profile if it matters.

### 5. **Public APIs**
`Cow` can be confusing; consider if simpler types work.

## Key Benefits

1. **Zero-cost borrowing**: When no modification is needed, no allocation occurs
2. **Unified interface**: Single type handles both borrowed and owned cases
3. **Lazy cloning**: Clone only when mutation is required via `to_mut()`
4. **Composability**: Works well with iterators and functional patterns

## Common Patterns

### Converting to Owned

```rust
let cow: Cow<str> = Cow::Borrowed("hello");
let owned: String = cow.into_owned();
```

### Mutating (triggers clone if borrowed)

```rust
let mut cow: Cow<str> = Cow::Borrowed("hello");
cow.to_mut().push_str(" world");  // Now Cow::Owned
```

### Checking Variant

```rust
match cow {
    Cow::Borrowed(b) => println!("Borrowed: {}", b),
    Cow::Owned(o) => println!("Owned: {}", o),
}
```

`Cow<T>` shines when you have operations that conditionally need ownership, allowing you to optimize for the common case while still handling all scenarios correctly.