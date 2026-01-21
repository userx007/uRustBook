#### Destructuring details

```rust
use std::collections::HashMap;  // Import HashMap from standard library

// Create a new empty HashMap with type inference
// 'mut' is required because we'll be modifying the HashMap
// Type will be inferred as HashMap<String, i32> from first insertion
let mut ages = HashMap::new();

ages.insert("Alice".to_string(), 30);
ages.insert("Bob".to_string(), 25);
ages.insert("Charlie".to_string(), 35);

for (&name, &age) in &ages {
    println!("{}: {}", name, age);
}
```

- **`&age` works**: You can dereference `&i32` to get `i32` because `i32` implements `Copy`. The value is copied, not moved.

- **`&name` fails**: You cannot dereference `&String` to get `String` because `String` does NOT implement `Copy`. Dereferencing would try to move the `String` out of the HashMap, which Rust doesn't allow since the HashMap still owns it.

**What actually works:**

```rust
// Option 1: Borrow both (most common)
for (name, age) in &ages {
    // name: &String, age: &i32
    println!("{}: {}", name, age);
}

// Option 2: Dereference only age
for (name, &age) in &ages {
    // name: &String, age: i32 (copied)
    println!("{}: {}", name, age);
}

// Option 3: Clone the String if you need ownership
for (name, age) in &ages {
    let owned_name: String = name.clone();
    println!("{}: {}", owned_name, age);
}
```