# Ownership Rules

Rust enforces three fundamental ownership rules at compile time:

1. **Each value has a single owner** (a variable that holds it)
2. **There can only be one owner at a time**
3. **When the owner goes out of scope, the value is dropped** (memory is freed)

---

# Why Trait Bounds?

Without trait bounds, generic types can't do anything useful because Rust doesn't know what operations they support. 
Trait bounds tell the compiler what methods and behaviors a type must implement.
The `T: Trait` syntax means "T must implement Trait." This allows you to call trait methods on `T`.


```rust

fn function<T: Trait>(param: T) { }  

fn function<T: Trait1 + Trait2 + Trait3>(param: T) { }

fn function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Debug + PartialEq,
{ }

```

You can provide different implementations based on what traits a type implements:

```rust

impl<T: Display> MyStruct<T> { }               // All T with Display

impl<T: Display + PartialOrd> MyStruct<T> { }  // Additional methods for comparable types

```

Constrain not just the generic type, but its associated types:

```rust
where
    T: Iterator,
    T::Item: Display,  // The items produced must be displayable


           ┌─────────────────────────────┐
           │     SpiTransfer (trait)     │
           │─────────────────────────────│
           │  transfer(...)              │
           │  write(...)                 │
           │  read(...)                  │
           └───────────────▲─────────────┘
                           │
           (T must implement this trait)
                           │
           ┌───────────────┴─────────────┐
           │  AdvancedLoopbackTester<T>  │
           │─────────────────────────────│
           │  T : SpiTransfer            │
           └─────────────────────────────┘

                    Compile Time
     ┌──────────────┐         ┌──────────────┐
     │ HardwareSpi  │         │   MockSpi    │
     └───────▲──────┘         └───────▲──────┘
             │ implements             │ implements
             │                        │
     ┌───────┴────────────────────────┴───────┐
     │           SpiTransfer trait            │
     └────────────────────────────────────────┘

     ┌────────────────────────────────────────┐
     │ AdvancedLoopbackTester<HardwareSpi>    │
     └────────────────────────────────────────┘

     ┌────────────────────────────────────────┐
     │ AdvancedLoopbackTester<MockSpi>        │
     └────────────────────────────────────────┘

_______________________________________________

struct AdvancedLoopbackTester<T: SpiTransfer>{
	spi: T,
}
     ┌───────────────────────────────────────┐
     │   AdvancedLoopbackTester<HardwareSpi> │
     │───────────────────────────────────────│
     │  spi ───────────────┐                 │
     │                     ▼                 │
     │              HardwareSpi              │
     └───────────────────────────────────────┘
"The tester owns the SPI device."
_______________________________________________

struct AdvancedLoopbackTester<'a, T: SpiTransfer>{
    spi: &'a mut T,
}

	┌───────────────────────────────────────┐
	│ AdvancedLoopbackTester<'a, T>         │
	│───────────────────────────────────────│
	│ spi ──&'a mut T────────────────────┐  │
	└────────────────────────────────────│──┘  
	                                     ▼
	                              ┌─────────────┐
	                              │ HardwareSpi │
	                              └─────────────┘
"The tester temporarily borrow the SPI device, doesn’t own it."

_______________________________________________
```


                ┌──────────────────────────┐
                │          Fn              │
                │──────────────────────────│
                │ call(&self)              │
                │ • no mutation            │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │         FnMut            │
                │──────────────────────────│
                │ call(&mut self)          │
                │ • may mutate captures    │
                │ • callable many times    │
                └─────────────▲────────────┘
                              │
                ┌─────────────┴────────────┐
                │        FnOnce            │
                │──────────────────────────│
                │ call_once(self)          │
                │ • may consume captures   │
                │ • callable at least once │
                └──────────────────────────┘
