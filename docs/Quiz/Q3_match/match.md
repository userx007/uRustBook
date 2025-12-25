# Matching in Rust

✅ These cover the main practical uses of `match` in Rust:

* Simple value matching
* Multiple patterns
* Ranges
* Tuples, arrays, slices
* Structs & enums
* Guards & nested patterns
* References & borrowing
* Using `match` as an expression

---

## 1. **Matching simple values**

```rust
let number = 2;

match number {
    1 => println!("One"),
    2 => println!("Two"),
    3 => println!("Three"),
    _ => println!("Something else"), // catch-all
}
```

* `_` is the **catch-all pattern**.
* Only one arm executes.

---

## 2. **Matching multiple patterns**

```rust
let letter = 'b';

match letter {
    'a' | 'e' | 'i' | 'o' | 'u' => println!("Vowel"),
    _ => println!("Consonant"),
}
```

* `|` allows **OR patterns** in one arm.

---

## 3. **Matching ranges**

```rust
let age = 25;

match age {
    0..=12 => println!("Child"),
    13..=19 => println!("Teen"),
    20..=64 => println!("Adult"),
    _ => println!("Senior"),
}
```

* `..=` is an **inclusive range**.
* `..` is **exclusive range**.

---

## 4. **Destructuring tuples**

```rust
let pair = (0, -2);

match pair {
    (0, y) => println!("x is 0, y is {}", y),
    (x, 0) => println!("y is 0, x is {}", x),
    _ => println!("Neither is 0"),
}
```

* You can destructure and bind parts of the tuple.

---

## 5. **Destructuring arrays/slices**

```rust
let numbers = [1, 2, 3];

match numbers {
    [1, second, _] => println!("Starts with 1, second is {}", second),
    [.., last] => println!("Last element is {}", last),
    _ => println!("Other array"),
}
```

* `[a, b, c]` matches exact size.
* `[.., last]` matches any length but captures last element.

---

## 6. **Destructuring structs**

```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 0, y: 7 };

match p {
    Point { x: 0, y } => println!("x is 0, y is {}", y),
    Point { x, y: 0 } => println!("y is 0, x is {}", x),
    Point { x, y } => println!("x = {}, y = {}", x, y),
}
```

* You can destructure structs and bind their fields.

---

## 7. **Using `match` with enums**

```rust
enum Direction { Up, Down, Left, Right }

let dir = Direction::Left;

match dir {
    Direction::Up => println!("Going up!"),
    Direction::Down => println!("Going down!"),
    Direction::Left | Direction::Right => println!("Going sideways"),
}
```

* Very common for enum handling.

---

## 8. **Matching with guards**

```rust
let x = 5;

match x {
    n if n % 2 == 0 => println!("Even"),
    _ => println!("Odd"),
}
```

* `if` adds **extra condition** (guard) to a pattern.

---

## 9. **Ignoring values**

```rust
let some_tuple = (1, 2, 3);

match some_tuple {
    (_, y, _) => println!("Middle value is {}", y),
}
```

* `_` ignores values you don’t care about.

---

## 10. **Nested patterns**

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

let msg = Message::Move { x: 3, y: 7 };

match msg {
    Message::Move { x, y: 0 } => println!("Move horizontally by {}", x),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    _ => println!("Other message"),
}
```

* You can nest destructuring inside enums and structs.

---

## 11. **Matching references and borrowing**

```rust
let x = 5;
let y = &x;

match y {
    &val => println!("Got a copy: {}", val),
}

// Using ref to borrow
match x {
    ref r => println!("Got a reference: {}", r),
}
```

* Patterns can handle **ownership, borrowing, and references**.

---

## 12. **Using `match` in expressions**

```rust
let number = 3;
let text = match number {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "many",
};

println!("Number is {}", text);
```

* `match` can **return a value**, not just execute code.

---

