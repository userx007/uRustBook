In Rust, **declarative macros**, often written using `macro_rules!`, are a way to write **code that writes code** (metaprogramming) in a pattern-based style. They allow you to define reusable code patterns that get expanded at compile time. Unlike **procedural macros**, which are more like functions that manipulate the Rust syntax tree, declarative macros work by **matching patterns** and replacing them with code.

Let’s break it down:

---

### **Syntax**

```rust
macro_rules! macro_name {
    ( pattern1 ) => { replacement1 };
    ( pattern2 ) => { replacement2 };
}
```

* **`macro_name`**: The name of your macro.
* **`pattern`**: What the macro matches when called.
* **`replacement`**: The code that replaces the macro invocation.

---

### **Example 1: Simple Macro**

```rust
macro_rules! say_hello {
    () => {
        println!("Hello, world!");
    };
}

fn main() {
    say_hello!(); // expands to println!("Hello, world!");
}
```

Here, the macro has no arguments. When you call `say_hello!()`, the compiler replaces it with `println!("Hello, world!");`.

---

### **Example 2: Macro with Arguments**

```rust
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("Function {:?} called", stringify!($func_name));
        }
    };
}

create_function!(foo);
create_function!(bar);

fn main() {
    foo(); // prints: Function "foo" called
    bar(); // prints: Function "bar" called
}
```

* `$func_name:ident` matches an identifier.
* `stringify!($func_name)` converts the identifier into a string literal.

---

### **Example 3: Repeating Patterns**

```rust
macro_rules! print_all {
    ( $( $x:expr ),* ) => {
        $( println!("{}", $x); )*
    };
}

fn main() {
    print_all!(1, 2, 3, 4);
    // expands to:
    // println!("{}", 1);
    // println!("{}", 2);
    // println!("{}", 3);
    // println!("{}", 4);
}
```

* `$( ... ),*` matches **zero or more** comma-separated expressions.
* The `$()*` repeats the code for each matched value.

---

### ✅ **Key Points**

1. **Pattern-based**: Declarative macros match the structure of the code.
2. **Compile-time expansion**: The macro is expanded before the code is compiled.
3. **Flexible syntax**: You can match identifiers, expressions, types, blocks, and more.
4. **Repetition**: You can handle lists or sequences of items using `$(...)*` or `$(...)+`.

---

In short: `macro_rules!` lets you define **templated, pattern-matching code expansions**, which can reduce boilerplate and make your Rust code more expressive—but they can get tricky for complex logic, which is where **procedural macros** shine.

---
Absolutely! Let's create a **progressive Rust `macro_rules!` tutorial**, starting from simple macros and gradually introducing more complex features like pattern matching, repetition, and nested macros. I’ll make sure each step **builds on the previous one**, so you can follow the learning curve naturally.

---

# **Rust Declarative Macros Tutorial (Advanced)**

---

## **Step 1: Basic Macro – No Arguments**

A simple macro just prints a message.

```rust
macro_rules! hello {
    () => {
        println!("Hello, Rust macros!");
    };
}

fn main() {
    hello!(); // expands to println!("Hello, Rust macros!");
}
```

✅ Key concept: A macro can be **invoked like a function** but is replaced **at compile time**.

---

## **Step 2: Macro with Arguments**

Macros can take arguments like functions:

```rust
macro_rules! greet {
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

fn main() {
    greet!("Alice"); // prints: Hello, Alice!
}
```

* `$name:expr` matches any Rust expression.
* Expressions can be variables, literals, or even function calls.

---

## **Step 3: Matching Identifiers**

You can match identifiers (`$ident`) and inject them in code:

```rust
macro_rules! create_fn {
    ($func_name:ident) => {
        fn $func_name() {
            println!("Function {:?} called", stringify!($func_name));
        }
    };
}

create_fn!(foo);
create_fn!(bar);

fn main() {
    foo(); // Function "foo" called
    bar(); // Function "bar" called
}
```

* `ident` = identifier (variable or function name).
* `stringify!` converts code into a string at compile time.

---

## **Step 4: Multiple Patterns**

Macros can have **different branches** like a `match` statement:

```rust
macro_rules! math_op {
    (add, $a:expr, $b:expr) => {
        println!("{} + {} = {}", $a, $b, $a + $b);
    };
    (sub, $a:expr, $b:expr) => {
        println!("{} - {} = {}", $a, $b, $a - $b);
    };
}

fn main() {
    math_op!(add, 10, 5); // 10 + 5 = 15
    math_op!(sub, 10, 5); // 10 - 5 = 5
}
```

* Macros can **branch based on the first token**.
* Each branch has its own pattern and replacement.

---

## **Step 5: Repeating Patterns**

You can repeat elements using `$(...)*` or `$(...)+`.

```rust
macro_rules! print_values {
    ( $( $x:expr ),* ) => {
        $(
            println!("{}", $x);
        )*
    };
}

fn main() {
    print_values!(1, 2, 3, 4, 5);
    // expands to:
    // println!("{}", 1);
    // println!("{}", 2);
    // println!("{}", 3);
    // println!("{}", 4);
    // println!("{}", 5);
}
```

* `*` = zero or more
* `+` = one or more

---

## **Step 6: Nested Macros / Recursion**

Macros can **call themselves** recursively for advanced pattern handling:

```rust
macro_rules! sum {
    ($x:expr) => ($x);
    ($x:expr, $($rest:expr),+) => ($x + sum!($($rest),+));
}

fn main() {
    let result = sum!(1, 2, 3, 4); // 10
    println!("{}", result);
}
```

* Base case: single expression.
* Recursive case: first expression + sum of the rest.

This is like functional programming **recursion** but evaluated at **compile time**.

---

## **Step 7: Complex Example – Struct Builder**

We can build macros to generate **boilerplate code**, like structs with getters:

```rust
macro_rules! make_struct {
    ($name:ident { $( $field:ident : $t:ty ),* }) => {
        struct $name {
            $( $field: $t ),*
        }

        impl $name {
            $(
                fn $field(&self) -> &$t {
                    &self.$field
                }
            )*
        }
    };
}

make_struct!(Person {
    name: String,
    age: u32
});

fn main() {
    let person = Person { name: "Alice".to_string(), age: 30 };
    println!("{} is {} years old", person.name(), person.age());
}
```

* Matches an identifier (`$name`) and **repeats** fields.
* Automatically generates **getter methods** for all fields.

---

## ✅ **Tips for Mastering `macro_rules!`**

1. Start simple, then use repetitions (`*`/`+`) and recursive macros.
2. Use `:ident`, `:expr`, `:ty`, `:path` to match **different kinds of tokens**.
3. Use multiple patterns to handle **different cases**.
4. Remember: macros **expand at compile time**, so debugging can be tricky—`cargo expand` helps!
5. Keep readability in mind; overusing macros can make code hard to follow.

---

If you want, I can make a **visual “cheat sheet diagram”** showing **how patterns in `macro_rules!` match and expand**, including recursion and repetition. This really helps solidify the concept.

Do you want me to make that diagram?
