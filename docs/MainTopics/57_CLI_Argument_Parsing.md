# CLI Argument Parsing in Rust

- **`std::env::args`** — the raw baseline before reaching for a library
- **`clap` Derive API** — the recommended approach, mapping CLI args directly to Rust structs with auto-generated help text
- **`clap` Builder API** — for runtime-dynamic CLI construction
- **Arguments, Options, Flags & multi-value args** — with count-based verbosity (`-vvv`)
- **Subcommands** — modelled as enums with exhaustive `match`, including nested subcommands
- **Environment variable integration** — `#[arg(env = "VAR")]` for twelve-factor app patterns, with `dotenvy` for `.env` file support
- **Validation & custom parsers** — value ranges, custom `FromStr` types, and early error reporting
- **`argh`** — the lightweight Google alternative with its own derive API and subcommand support
- **Ergonomic CLI patterns** — exit codes, `anyhow` error handling, progress bars, shell completion generation, and configuration layering
- **`clap` vs `argh` comparison table** — to help you choose the right tool for the job

# 57. CLI Argument Parsing in Rust

> Using `clap` and `argh`, subcommands, environment variable integration, and building ergonomic CLI tools.

---

## Table of Contents

1. [Introduction](#introduction)
2. [The Standard Library: `std::env::args`](#the-standard-library-stdenvargs)
3. [The `clap` Crate](#the-clap-crate)
   - [Derive API (Recommended)](#derive-api-recommended)
   - [Builder API](#builder-api)
4. [Arguments, Options, and Flags](#arguments-options-and-flags)
5. [Subcommands](#subcommands)
6. [Environment Variable Integration](#environment-variable-integration)
7. [Validation and Custom Parsers](#validation-and-custom-parsers)
8. [The `argh` Crate](#the-argh-crate)
9. [Building Ergonomic CLI Tools](#building-ergonomic-cli-tools)
10. [Comparison: `clap` vs `argh`](#comparison-clap-vs-argh)
11. [Summary](#summary)

---

## Introduction

Command-Line Interface (CLI) tools are a staple of systems programming, and Rust's ecosystem provides excellent libraries for parsing arguments ergonomically and safely. Rather than manually parsing `std::env::args()`, crates like **`clap`** and **`argh`** let you declare your CLI's structure using Rust types and derive macros, generating all parsing, validation, and help text automatically.

---

## The Standard Library: `std::env::args`

Before reaching for a library, it's worth understanding what Rust gives you out of the box.

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // args[0] is always the program name
    if args.len() < 2 {
        eprintln!("Usage: {} <name>", args[0]);
        std::process::exit(1);
    }

    println!("Hello, {}!", args[1]);
}
```

```
$ cargo run -- Alice
Hello, Alice!
```

This approach works for trivial tools but quickly becomes unwieldy. There is no support for flags, optional arguments, type coercion, or auto-generated help text. That is where `clap` and `argh` shine.

---

## The `clap` Crate

`clap` (Command Line Argument Parser) is the most widely used CLI parsing library in the Rust ecosystem. It supports both a **Derive API** (declarative, struct-based) and a **Builder API** (programmatic).

Add it to `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

---

### Derive API (Recommended)

The Derive API lets you describe your entire CLI as a plain Rust struct annotated with `#[derive(Parser)]`. `clap` generates all parsing logic at compile time.

```rust
use clap::Parser;

/// A simple file search tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The pattern to search for
    pattern: String,

    /// The file path to search in
    path: std::path::PathBuf,

    /// Show line numbers alongside matches
    #[arg(short, long)]
    line_numbers: bool,

    /// Maximum number of results to return
    #[arg(short, long, default_value_t = 100)]
    max_results: usize,
}

fn main() {
    let args = Args::parse();

    println!("Searching for '{}' in '{}'", args.pattern, args.path.display());
    if args.line_numbers {
        println!("Line numbers: enabled");
    }
    println!("Max results: {}", args.max_results);
}
```

Running with `--help` produces well-formatted documentation automatically:

```
$ cargo run -- --help

A simple file search tool

Usage: mysearch [OPTIONS] <PATTERN> <PATH>

Arguments:
  <PATTERN>  The pattern to search for
  <PATH>     The file path to search in

Options:
  -l, --line-numbers          Show line numbers alongside matches
  -m, --max-results <MAX_RESULTS>  Maximum number of results to return [default: 100]
  -h, --help                  Print help
  -V, --version               Print version
```

Key observations:
- **Positional arguments** are plain fields without `#[arg(short, long)]`.
- **Flags** (booleans) use `#[arg(short, long)]` and default to `false`.
- **Options** (flags with values) use `#[arg(short, long)]` on non-boolean types.
- Doc comments (`///`) become the help text automatically.

---

### Builder API

The Builder API constructs the CLI programmatically at runtime. It is more verbose but useful when the CLI structure is not known at compile time.

```rust
use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("myapp")
        .version("1.0")
        .author("Jane Doe")
        .about("Demonstrates the Builder API")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enables verbose output"),
        )
        .get_matches();

    if let Some(config) = matches.get_one::<String>("config") {
        println!("Using config: {}", config);
    }

    if matches.get_flag("verbose") {
        println!("Verbose mode enabled");
    }
}
```

---

## Arguments, Options, and Flags

Understanding the three core concepts is essential:

| Concept | Description | Example |
|---|---|---|
| **Positional Argument** | Required, identified by position | `myapp <file>` |
| **Option** | Named, takes a value | `myapp --output out.txt` |
| **Flag** | Named, boolean switch | `myapp --verbose` |

### Optional and Multiple Values

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Input files (can specify multiple)
    #[arg(required = true, num_args = 1..)]
    files: Vec<String>,

    /// Output directory (optional)
    #[arg(short, long)]
    output: Option<String>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    println!("Files: {:?}", args.files);
    println!("Output: {:?}", args.output);
    println!("Verbosity: {}", args.verbose);
}
```

```
$ cargo run -- file1.txt file2.txt -vvv --output ./dist
Files: ["file1.txt", "file2.txt"]
Output: Some("./dist")
Verbosity: 3
```

---

## Subcommands

Subcommands allow a single binary to expose multiple distinct operations — similar to how `git commit`, `git push`, and `git clone` are all part of `git`. In `clap`, subcommands are modelled as an `enum` derived with `Subcommand`.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "A package manager example")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install a package
    Install {
        /// Name of the package to install
        package: String,

        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Remove a package
    Remove {
        /// Name of the package to remove
        package: String,

        /// Also remove configuration files
        #[arg(long)]
        purge: bool,
    },

    /// List installed packages
    List {
        /// Filter by name prefix
        #[arg(short, long)]
        filter: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("[verbose mode]");
    }

    match cli.command {
        Commands::Install { package, version } => {
            match version {
                Some(v) => println!("Installing {}@{}", package, v),
                None    => println!("Installing {} (latest)", package),
            }
        }
        Commands::Remove { package, purge } => {
            if purge {
                println!("Removing {} and its config files", package);
            } else {
                println!("Removing {}", package);
            }
        }
        Commands::List { filter } => {
            match filter {
                Some(f) => println!("Listing packages matching '{}'", f),
                None    => println!("Listing all packages"),
            }
        }
    }
}
```

```
$ cargo run -- install serde --version 1.0
Installing serde@1.0

$ cargo run -- remove serde --purge
Removing serde and its config files

$ cargo run -- list --filter ser
Listing packages matching 'ser'

$ cargo run -- --help
A package manager example

Usage: pkgman [OPTIONS] <COMMAND>

Commands:
  install  Install a package
  remove   Remove a package
  list     List installed packages
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose output
  -h, --help     Print help
  -V, --version  Print version
```

### Nested Subcommands

Subcommands can be nested by placing another `#[command(subcommand)]` inside an enum variant's associated struct.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: TopLevel,
}

#[derive(Subcommand, Debug)]
enum TopLevel {
    /// Remote operations
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },
}

#[derive(Subcommand, Debug)]
enum RemoteAction {
    /// Add a new remote
    Add { name: String, url: String },
    /// Remove a remote
    Remove { name: String },
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
}
```

```
$ cargo run -- remote add origin https://example.com/repo.git
```

---

## Environment Variable Integration

It is common for CLI tools to accept configuration from both command-line arguments **and** environment variables, with the command line taking precedence. `clap` supports this natively with the `env` attribute.

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Database connection tool")]
struct Args {
    /// Database host
    #[arg(long, env = "DB_HOST", default_value = "localhost")]
    host: String,

    /// Database port
    #[arg(long, env = "DB_PORT", default_value_t = 5432)]
    port: u16,

    /// Database name
    #[arg(long, env = "DB_NAME")]
    database: String,

    /// Database password (hidden from help)
    #[arg(long, env = "DB_PASSWORD", hide_env_values = true)]
    password: String,
}

fn main() {
    let args = Args::parse();

    println!(
        "Connecting to {}:{}/{} ...",
        args.host, args.port, args.database
    );
}
```

With this setup, you can configure the tool via environment variables:

```bash
# Using environment variables
export DB_HOST=db.production.com
export DB_PORT=5433
export DB_NAME=myapp
export DB_PASSWORD=supersecret
cargo run

# Or override on the command line (takes precedence)
cargo run -- --host localhost --database devdb --password localpass
```

The `hide_env_values = true` attribute prevents sensitive environment variable values (like passwords) from appearing in help output.

### Using a `.env` File with `dotenvy`

In practice, environment variables are often loaded from a `.env` file using the `dotenvy` crate:

```toml
[dependencies]
clap    = { version = "4", features = ["derive", "env"] }
dotenvy = "0.15"
```

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "API_KEY")]
    api_key: String,
}

fn main() {
    // Load .env file before parsing args
    dotenvy::dotenv().ok();

    let args = Args::parse();
    println!("Using API key: {}...", &args.api_key[..4]);
}
```

---

## Validation and Custom Parsers

`clap` can enforce value ranges and call custom parsing functions at argument parse time, providing early, clear error messages.

### Value Ranges

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Compression level (1-9)
    #[arg(short, long, default_value_t = 6, value_parser = clap::value_parser!(u8).range(1..=9))]
    level: u8,
}

fn main() {
    let args = Args::parse();
    println!("Compression level: {}", args.level);
}
```

```
$ cargo run -- --level 10
error: invalid value '10' for '--level <LEVEL>': 10 is not in 1..=9
```

### Custom Parsing Functions

```rust
use clap::Parser;
use std::net::IpAddr;

fn parse_ip(s: &str) -> Result<IpAddr, String> {
    s.parse::<IpAddr>().map_err(|e| format!("Invalid IP address '{}': {}", s, e))
}

#[derive(Parser, Debug)]
struct Args {
    /// Server IP address to bind to
    #[arg(long, default_value = "127.0.0.1", value_parser = parse_ip)]
    bind: IpAddr,
}

fn main() {
    let args = Args::parse();
    println!("Binding to: {}", args.bind);
}
```

### Custom Types with `FromStr`

Any type implementing `std::str::FromStr` can be used directly as an argument type:

```rust
use clap::Parser;
use std::str::FromStr;

#[derive(Debug)]
enum LogLevel { Error, Warn, Info, Debug, Trace }

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn"  => Ok(LogLevel::Warn),
            "info"  => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            other   => Err(format!("Unknown log level: '{}'", other)),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: LogLevel,
}

fn main() {
    let args = Args::parse();
    println!("Log level: {:?}", args.log_level);
}
```

---

## The `argh` Crate

`argh` is a lightweight alternative to `clap` developed by Google's Fuchsia team. It is significantly smaller in binary size and compile time, making it a good fit for embedded or size-sensitive applications. Its API is also derive-based.

```toml
[dependencies]
argh = "0.1"
```

```rust
use argh::FromArgs;

/// Top-level CLI tool
#[derive(FromArgs, Debug)]
struct Args {
    /// the name to greet
    #[argh(positional)]
    name: String,

    /// how many times to repeat the greeting
    #[argh(option, short = 'n', default = "1")]
    count: u32,

    /// use uppercase output
    #[argh(switch, short = 'u')]
    uppercase: bool,
}

fn main() {
    let args: Args = argh::from_env();

    for _ in 0..args.count {
        let greeting = format!("Hello, {}!", args.name);
        if args.uppercase {
            println!("{}", greeting.to_uppercase());
        } else {
            println!("{}", greeting);
        }
    }
}
```

```
$ cargo run -- Alice -n 3 -u
HELLO, ALICE!
HELLO, ALICE!
HELLO, ALICE!
```

### Subcommands with `argh`

```rust
use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// A deployment tool
struct Cli {
    #[argh(subcommand)]
    command: SubCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Deploy(DeployArgs),
    Rollback(RollbackArgs),
}

/// Deploy the application
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "deploy")]
struct DeployArgs {
    /// environment to deploy to
    #[argh(option, short = 'e')]
    env: String,
}

/// Roll back the last deployment
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "rollback")]
struct RollbackArgs {
    /// number of versions to roll back
    #[argh(option, short = 'n', default = "1")]
    steps: u32,
}

fn main() {
    let cli: Cli = argh::from_env();

    match cli.command {
        SubCommands::Deploy(args)   => println!("Deploying to '{}'", args.env),
        SubCommands::Rollback(args) => println!("Rolling back {} step(s)", args.steps),
    }
}
```

---

## Building Ergonomic CLI Tools

Beyond parsing arguments, a truly ergonomic CLI tool follows several best practices.

### 1. Exit Codes

Always use meaningful exit codes. The convention is `0` for success and non-zero for errors.

```rust
use clap::Parser;
use std::process;

#[derive(Parser)]
struct Args {
    file: String,
}

fn run(args: Args) -> Result<(), String> {
    if !std::path::Path::new(&args.file).exists() {
        return Err(format!("File not found: {}", args.file));
    }
    println!("Processing '{}'", args.file);
    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
```

### 2. Structured Error Handling with `anyhow`

Combining `clap` with `anyhow` is a common, ergonomic pattern:

```toml
[dependencies]
clap   = { version = "4", features = ["derive"] }
anyhow = "1"
```

```rust
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Path to the config file
    config: String,
}

fn run(args: Args) -> Result<()> {
    let content = std::fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config file: {}", args.config))?;

    println!("Config has {} bytes", content.len());
    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(err) = run(args) {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
```

### 3. Progress and Coloured Output

Use `indicatif` for progress bars and `colored` or `owo-colors` for terminal colours:

```toml
[dependencies]
clap       = { version = "4", features = ["derive"] }
indicatif  = "0.17"
colored    = "2"
```

```rust
use clap::Parser;
use colored::Colorize;
use indicatif::ProgressBar;

#[derive(Parser)]
struct Args {
    /// Number of items to process
    #[arg(default_value_t = 10)]
    count: u64,
}

fn main() {
    let args = Args::parse();

    println!("{}", "Starting processing...".green().bold());

    let bar = ProgressBar::new(args.count);
    for _ in 0..args.count {
        std::thread::sleep(std::time::Duration::from_millis(100));
        bar.inc(1);
    }
    bar.finish_with_message("done");

    println!("{}", "All items processed successfully!".cyan());
}
```

### 4. Shell Completions

`clap` can generate shell completion scripts at runtime:

```rust
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

#[derive(Parser)]
#[command(name = "myapp")]
struct Args {
    /// Generate shell completions for the given shell
    #[arg(long, value_enum)]
    completions: Option<Shell>,

    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(shell) = args.completions {
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "myapp", &mut std::io::stdout());
        return;
    }

    println!("Running normally with file: {:?}", args.file);
}
```

```
$ myapp --completions bash >> ~/.bash_completion
$ myapp --completions zsh  >> ~/.zsh_completions/_myapp
```

### 5. Configuration Layering Pattern

A common real-world pattern layers configuration from multiple sources:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Config file path
    #[arg(short, long, env = "APP_CONFIG", default_value = "config.toml")]
    config: String,

    /// Override: database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Override: listening port
    #[arg(short, long, env = "PORT", default_value_t = 8080)]
    port: u16,
}

#[derive(Debug)]
struct AppConfig {
    database_url: String,
    port: u16,
}

impl AppConfig {
    fn from_args(args: Args) -> Self {
        // Priority: CLI flag > environment variable > config file > hardcoded default
        // clap already handles the first two levels via `env =`.
        // Here we'd load `args.config` and merge. Simplified for brevity:
        AppConfig {
            database_url: args.database_url
                .unwrap_or_else(|| "postgres://localhost/myapp".to_string()),
            port: args.port,
        }
    }
}

fn main() {
    let args = Args::parse();
    let config = AppConfig::from_args(args);
    println!("Config: {:?}", config);
}
```

---

## Comparison: `clap` vs `argh`

| Feature | `clap` | `argh` |
|---|---|---|
| **Approach** | Derive + Builder APIs | Derive only |
| **Binary size impact** | Medium–Large | Very small |
| **Compile time** | Slower | Faster |
| **Feature richness** | Extensive | Minimal |
| **Environment variables** | Built-in (`env =`) | Not built-in |
| **Value validation** | Built-in ranges + custom parsers | Manual |
| **Shell completions** | Yes (via `clap_complete`) | No |
| **Help formatting** | Highly customisable | Simple, fixed |
| **Best for** | Full-featured user-facing CLI tools | Embedded / size-critical tools |

**Rule of thumb:** Use `clap` for applications where developer experience and feature richness matter. Use `argh` when binary size and compile time are paramount constraints.

---

## Summary

Rust's CLI parsing ecosystem is mature and expressive. The key takeaways are:

- **`std::env::args`** is suitable only for the simplest scripts; prefer a library for anything real.
- **`clap` with the Derive API** is the gold standard for most CLI applications — it maps your argument structure directly to Rust types, auto-generates help text, and handles subcommands, environment variables, and validation elegantly.
- **Subcommands** are modelled as Rust enums with `#[derive(Subcommand)]`, giving you exhaustive matching and compile-time safety.
- **Environment variable integration** via `#[arg(env = "VAR")]` enables twelve-factor app patterns with no extra effort.
- **`argh`** is an excellent lightweight alternative when binary size and compile speed matter more than feature richness.
- Combining `clap` with crates like `anyhow`, `indicatif`, and `colored` gives you a complete toolkit for building professional, ergonomic CLI tools.