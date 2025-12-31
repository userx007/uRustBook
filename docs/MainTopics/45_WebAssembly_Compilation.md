# WebAssembly (WASM) Compilation in Rust

## Overview

WebAssembly (WASM) is a binary instruction format that allows code written in languages like Rust to run in web browsers at near-native speed. Rust has excellent support for WASM compilation, making it a popular choice for high-performance web applications.

## 1. Setting Up for WASM Development

### Installing Required Tools

```bash
# Install wasm-pack (all-in-one tool for building and publishing WASM)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add the WASM target to rustc
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (optional, but useful)
cargo install wasm-bindgen-cli
```

## 2. Basic WASM Compilation

### Simple Rust Library

**Cargo.toml:**
```toml
[package]
name = "hello-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Dynamic system library for WASM

[dependencies]
wasm-bindgen = "0.2"
```

**src/lib.rs:**
```rust
use wasm_bindgen::prelude::*;

// Export a simple function to JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Add two numbers
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Building the Project

```bash
# Build with wasm-pack (recommended)
wasm-pack build --target web

# Or manually with cargo
cargo build --target wasm32-unknown-unknown --release
```

## 3. Using wasm-bindgen

`wasm-bindgen` bridges Rust and JavaScript, handling type conversions and memory management.

### Working with JavaScript Types

```rust
use wasm_bindgen::prelude::*;

// Import JavaScript functions
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

// Export complex types
#[wasm_bindgen]
pub struct Person {
    name: String,
    age: u32,
}

#[wasm_bindgen]
impl Person {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, age: u32) -> Person {
        Person { name, age }
    }
    
    // Getter
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    
    // Method
    pub fn greet(&self) {
        log(&format!("{} is {} years old", self.name, self.age));
    }
    
    pub fn birthday(&mut self) {
        self.age += 1;
    }
}

// Return Result to JavaScript
#[wasm_bindgen]
pub fn divide(a: f64, b: f64) -> Result<f64, JsValue> {
    if b == 0.0 {
        Err(JsValue::from_str("Division by zero"))
    } else {
        Ok(a / b)
    }
}
```

### Working with JavaScript Objects and Arrays

```rust
use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};

#[wasm_bindgen]
pub fn process_array(arr: &Array) -> Array {
    let result = Array::new();
    
    for i in 0..arr.length() {
        let val = arr.get(i);
        // Process each element
        result.push(&val);
    }
    
    result
}

#[wasm_bindgen]
pub fn create_object() -> Object {
    let obj = Object::new();
    
    Reflect::set(
        &obj,
        &JsValue::from_str("name"),
        &JsValue::from_str("Rust"),
    ).unwrap();
    
    Reflect::set(
        &obj,
        &JsValue::from_str("version"),
        &JsValue::from_f64(1.0),
    ).unwrap();
    
    obj
}
```

## 4. Interacting with the DOM

```rust
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Get the window and document
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    
    // Create and append an element
    let body = document.body().expect("no body");
    let paragraph = document.create_element("p")?;
    paragraph.set_text_content(Some("Hello from Rust!"));
    body.append_child(&paragraph)?;
    
    Ok(())
}

#[wasm_bindgen]
pub fn update_content(element_id: &str, text: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let element = document
        .get_element_by_id(element_id)
        .ok_or_else(|| JsValue::from_str("Element not found"))?;
    
    element.set_text_content(Some(text));
    Ok(())
}
```

**Cargo.toml addition:**
```toml
[dependencies.web-sys]
version = "0.3"
features = [
    "Document",
    "Element",
    "HtmlElement",
    "Node",
    "Window",
]
```

## 5. Using WASM in JavaScript

### Loading and Using WASM Module

**index.html:**
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>WASM Example</title>
</head>
<body>
    <div id="content"></div>
    <script type="module">
        import init, { greet, Person, divide } from './pkg/hello_wasm.js';
        
        async function run() {
            // Initialize the WASM module
            await init();
            
            // Call exported functions
            const greeting = greet("World");
            console.log(greeting);
            
            // Use exported classes
            const person = new Person("Alice", 30);
            person.greet();
            person.birthday();
            
            // Handle Results
            try {
                const result = divide(10, 2);
                console.log("Result:", result);
            } catch (err) {
                console.error("Error:", err);
            }
        }
        
        run();
    </script>
</body>
</html>
```

## 6. Optimizing WASM Binaries

### Size Optimization

**Cargo.toml:**
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Enable Link Time Optimization
codegen-units = 1      # Reduce parallel code generation
panic = "abort"        # Disable unwinding
strip = true           # Strip symbols
```

### Additional Optimization Tools

```bash
# Install wasm-opt from binaryen
# On macOS:
brew install binaryen

# On Linux:
apt-get install binaryen

# Optimize WASM binary
wasm-opt -Oz -o output_optimized.wasm input.wasm

# Build with wasm-pack in release mode
wasm-pack build --release --target web
```

### Using wee_alloc (Smaller Allocator)

```rust
// Use a smaller allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

**Cargo.toml:**
```toml
[dependencies]
wee_alloc = "0.4.5"
```

## 7. Advanced Example: Canvas Graphics

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct Canvas {
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl Canvas {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Canvas, JsValue> {
        let document = web_sys::window()
            .unwrap()
            .document()
            .unwrap();
        
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;
        
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        Ok(Canvas {
            context,
            width: canvas.width(),
            height: canvas.height(),
        })
    }
    
    pub fn draw_circle(&self, x: f64, y: f64, radius: f64, color: &str) {
        self.context.begin_path();
        self.context
            .arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        self.context.set_fill_style(&JsValue::from_str(color));
        self.context.fill();
    }
    
    pub fn clear(&self) {
        self.context.clear_rect(
            0.0,
            0.0,
            self.width as f64,
            self.height as f64,
        );
    }
}
```

## 8. Performance Tips

### Minimize String Conversions

```rust
use wasm_bindgen::prelude::*;

// Less efficient - creates String
#[wasm_bindgen]
pub fn process_text(text: String) -> String {
    text.to_uppercase()
}

// More efficient - uses &str when possible
#[wasm_bindgen]
pub fn process_text_efficient(text: &str) -> String {
    text.to_uppercase()
}
```

### Use Typed Arrays for Bulk Data

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_numbers(data: &[f64]) -> Vec<f64> {
    data.iter().map(|x| x * 2.0).collect()
}

// Or work directly with JavaScript typed arrays
use js_sys::Float64Array;

#[wasm_bindgen]
pub fn process_typed_array(arr: &Float64Array) -> Float64Array {
    let len = arr.length() as usize;
    let mut vec = vec![0.0; len];
    arr.copy_to(&mut vec);
    
    for val in &mut vec {
        *val *= 2.0;
    }
    
    Float64Array::from(&vec[..])
}
```

## 9. Debugging WASM

```rust
// Enable console_error_panic_hook for better error messages
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
```

**Cargo.toml:**
```toml
[dependencies]
console_error_panic_hook = { version = "0.1", optional = true }

[features]
default = ["console_error_panic_hook"]
```

## Key Takeaways

1. **Use wasm-pack** for streamlined builds and packaging
2. **wasm-bindgen** handles JavaScript interop seamlessly
3. **Optimize for size** with appropriate compiler flags and tools
4. **Use typed arrays** for efficient data transfer between Rust and JavaScript
5. **web-sys** provides access to Web APIs
6. **Profile and measure** to identify bottlenecks
7. **Consider tradeoffs** between binary size and runtime performance

WebAssembly with Rust enables high-performance web applications while maintaining Rust's safety guarantees and excellent tooling support.