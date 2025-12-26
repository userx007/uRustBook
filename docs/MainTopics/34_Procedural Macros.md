# Rust Procedural Macros 

**Key Topics Covered:**

1. **Three Types of Procedural Macros** - Complete examples of derive, attribute, and function-like macros
2. **Derive Macros** - A full Builder pattern implementation showing struct parsing and code generation
3. **Attribute Macros** - A routing system example demonstrating function decoration
4. **Function-like Macros** - An SQL validator showing custom syntax handling
5. **TokenStream** - Explanation of both `proc_macro` and `proc_macro2` token streams
6. **syn Crate** - Detailed parsing examples and common types (DeriveInput, ItemFn, etc.)
7. **quote Crate** - Template syntax, interpolation, and repetition patterns

**Practical Examples Include:**
- Builder pattern generator
- Route attribute macro
- SQL query validator
- Custom Debug trait implementation

The guide also covers best practices like error handling with proper spans, testing strategies, and debugging with `cargo expand`. Each example is production-ready and demonstrates real-world patterns used by popular crates like `serde`, `tokio`, and others.

# Rust Procedural Macros: Complete Guide

## Introduction

Procedural macros in Rust are a powerful metaprogramming feature that allows you to write code that generates other code at compile time. Unlike declarative macros (`macro_rules!`), procedural macros operate on the abstract syntax tree (AST) of Rust code, giving you fine-grained control over code generation.

## Types of Procedural Macros

There are three types of procedural macros in Rust:

1. **Derive macros** - Automatically implement traits
2. **Attribute macros** - Define custom attributes
3. **Function-like macros** - Look like function calls but operate at compile time

## Setting Up a Procedural Macro Crate

Procedural macros must be defined in a separate crate with `proc-macro = true` in `Cargo.toml`:

```toml
[lib]
proc-macro = true

[dependencies]
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
```

## 1. Derive Macros

Derive macros automatically implement traits for structs and enums using the `#[derive(...)]` attribute.

### Basic Example

```rust
// In your proc-macro crate (lib.rs)
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());
    
    // Extract fields from the struct
    let fields = if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            panic!("Builder only works with named fields");
        }
    } else {
        panic!("Builder only works with structs");
    };
    
    // Generate builder fields (all Optional)
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: Option<#ty> }
    });
    
    // Generate setter methods
    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, #name: #ty) -> Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    
    // Generate build method
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });
    
    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_fields),*
                }
            }
        }
        
        pub struct #builder_name {
            #(#builder_fields),*
        }
        
        impl #builder_name {
            #(#setters)*
            
            pub fn build(self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields),*
                })
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

### Usage

```rust
// In your application crate
#[derive(Builder)]
struct User {
    name: String,
    age: u32,
    email: String,
}

fn main() {
    let user = User::builder()
        .name("Alice".to_string())
        .age(30)
        .email("alice@example.com".to_string())
        .build()
        .unwrap();
    
    println!("{} is {} years old", user.name, user.age);
}
```

## 2. Attribute Macros

Attribute macros define custom attributes that can be applied to items like functions, structs, or modules.

### Example: Route Attribute

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_sig = &input_fn.sig;
    
    let route_name = syn::Ident::new(
        &format!("ROUTE_{}", fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    
    let expanded = quote! {
        #input_fn
        
        #[doc(hidden)]
        pub const #route_name: &str = #path;
        
        inventory::submit! {
            Route {
                path: #path,
                handler: #fn_name,
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

### Usage

```rust
#[route("/api/users")]
fn get_users() -> String {
    "List of users".to_string()
}

#[route("/api/users/{id}")]
fn get_user(id: u32) -> String {
    format!("User {}", id)
}
```

## 3. Function-like Macros

Function-like macros are invoked like regular macros but have the full power of procedural macros.

### Example: SQL Query Builder

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, LitStr, Token};

struct SqlQuery {
    query: LitStr,
}

impl Parse for SqlQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let query: LitStr = input.parse()?;
        Ok(SqlQuery { query })
    }
}

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let SqlQuery { query } = parse_macro_input!(input as SqlQuery);
    let query_str = query.value();
    
    // Basic SQL validation (in reality, you'd do more)
    let query_upper = query_str.to_uppercase();
    if !query_upper.starts_with("SELECT") 
        && !query_upper.starts_with("INSERT")
        && !query_upper.starts_with("UPDATE")
        && !query_upper.starts_with("DELETE") {
        panic!("Invalid SQL query");
    }
    
    let expanded = quote! {
        {
            const QUERY: &str = #query;
            // Compile-time validation passed
            QUERY
        }
    };
    
    TokenStream::from(expanded)
}
```

### Usage

```rust
fn main() {
    let query = sql!("SELECT * FROM users WHERE age > 18");
    println!("Query: {}", query);
    
    // This would fail at compile time:
    // let bad_query = sql!("INVALID SQL");
}
```

## TokenStream Manipulation

`TokenStream` is the fundamental type for procedural macros. It represents a stream of tokens that make up Rust code.

### Understanding TokenStream

```rust
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

// proc_macro::TokenStream - used at macro boundaries
// proc_macro2::TokenStream - used internally with syn/quote

#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Convert to proc_macro2::TokenStream for easier manipulation
    let input2: TokenStream2 = input.into();
    
    // Do something with it
    let output2: TokenStream2 = quote! {
        println!("Generated code!");
    };
    
    // Convert back to proc_macro::TokenStream
    output2.into()
}
```

## The `syn` Crate

`syn` is a parsing library for Rust syntax. It converts `TokenStream` into structured data (AST).

### Common syn Types

```rust
use syn::{
    DeriveInput,    // Entire item with #[derive]
    ItemFn,         // Function item
    ItemStruct,     // Struct definition
    Expr,           // Expression
    Type,           // Type annotation
    Ident,          // Identifier
    LitStr,         // String literal
};

// Parsing examples
use syn::parse_macro_input;

#[proc_macro_derive(MyDerive)]
pub fn my_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Access the name
    let name = &input.ident;
    
    // Access generics
    let generics = &input.generics;
    
    // Match on the data kind
    match &input.data {
        syn::Data::Struct(s) => {
            // Handle struct
            for field in &s.fields {
                let field_name = &field.ident;
                let field_type = &field.ty;
                // ...
            }
        }
        syn::Data::Enum(e) => {
            // Handle enum
            for variant in &e.variants {
                let variant_name = &variant.ident;
                // ...
            }
        }
        syn::Data::Union(_) => panic!("Unions not supported"),
    }
    
    TokenStream::new()
}
```

## The `quote` Crate

`quote` provides a convenient way to generate Rust code using a template syntax.

### Basic quote Usage

```rust
use quote::quote;

let name = syn::Ident::new("MyStruct", proc_macro2::Span::call_site());
let field = syn::Ident::new("field", proc_macro2::Span::call_site());

// Interpolate with #
let output = quote! {
    struct #name {
        #field: i32,
    }
};

// Repeat with #(...)*
let fields = vec![
    syn::Ident::new("x", proc_macro2::Span::call_site()),
    syn::Ident::new("y", proc_macro2::Span::call_site()),
];

let output = quote! {
    struct Point {
        #(#fields: f64),*
    }
};
// Expands to: struct Point { x: f64, y: f64 }

// Repeat with separator
let output = quote! {
    (#(#fields),*)
};
// Expands to: (x, y)
```

### Advanced quote Patterns

```rust
use quote::{quote, format_ident};

// Create identifiers dynamically
let getter_name = format_ident!("get_{}", field_name);

let output = quote! {
    pub fn #getter_name(&self) -> &String {
        &self.#field_name
    }
};

// Conditional generation
let optional = true;
let output = quote! {
    struct MyStruct {
        field: i32,
        #(
            #[cfg(feature = "optional")]
            optional_field: String,
        )*
    }
};
```

## Complete Example: Debug Derive

Here's a complete example implementing a custom `Debug` trait:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(MyDebug)]
pub fn derive_my_debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let fields_debug = match &input.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let field_prints = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let name_str = name.as_ref().unwrap().to_string();
                        quote! {
                            .field(#name_str, &self.#name)
                        }
                    });
                    
                    quote! {
                        f.debug_struct(stringify!(#name))
                            #(#field_prints)*
                            .finish()
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_prints = (0..fields.unnamed.len()).map(|i| {
                        let index = syn::Index::from(i);
                        quote! {
                            .field(&self.#index)
                        }
                    });
                    
                    quote! {
                        f.debug_tuple(stringify!(#name))
                            #(#field_prints)*
                            .finish()
                    }
                }
                Fields::Unit => {
                    quote! {
                        write!(f, stringify!(#name))
                    }
                }
            }
        }
        _ => panic!("MyDebug only works on structs"),
    };
    
    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #fields_debug
            }
        }
    };
    
    TokenStream::from(expanded)
}
```

## Best Practices

1. **Error Handling**: Use `syn::Error` for user-friendly compile errors
```rust
return syn::Error::new(field.span(), "Expected a string type")
    .to_compile_error()
    .into();
```

2. **Span Preservation**: Maintain proper spans for better error messages
```rust
let ident = syn::Ident::new("name", field.span());
```

3. **Testing**: Test your macros with `trybuild` for compile-fail tests

4. **Documentation**: Document generated code and macro behavior

5. **Hygiene**: Use `proc_macro2::Span::call_site()` for generated identifiers

## Common Use Cases

- **Serialization/Deserialization**: `serde_derive`
- **Builder Patterns**: Automatically generate builders
- **ORM Models**: Generate database queries
- **Error Handling**: Generate `From` implementations
- **Testing**: Generate test cases
- **FFI Bindings**: Generate foreign function interfaces

## Debugging Tips

Use `cargo expand` to see the generated code:
```bash
cargo install cargo-expand
cargo expand --lib my_macro
```

This guide covers the essentials of procedural macros in Rust. Mastering these concepts allows you to write powerful code generation tools that enhance Rust's already expressive type system.