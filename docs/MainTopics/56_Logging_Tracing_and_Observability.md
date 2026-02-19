# Logging, Tracing, and Observability in Rust

**10 sections covering:**

- **`log` crate** — the classic façade, macros, `env_logger`, and its limitations
- **`tracing` fundamentals** — the Span / Event / Subscriber / Layer mental model
- **Spans in depth** — RAII guards, `in_scope`, nesting, and how parent/child relationships form the trace tree
- **Events & structured fields** — `%` (Display) vs `?` (Debug) sigils, keeping messages static and data in fields
- **Subscribers** — `fmt`, JSON mode, with full configuration options
- **Async instrumentation** — the `#[instrument]` macro with `skip`, `fields`, `err`, `ret`, and manual `.instrument()` for futures
- **Layer composition** — stacking filter + format layers via `registry()`, plus dynamic runtime reload
- **Platform integrations** — full OpenTelemetry/OTLP setup (Jaeger, Tempo, Honeycomb, Datadog), bridging `log` → `tracing` via `tracing-log`, and `tokio-console`
- **Best practices** — structured fields over string interpolation, always flush before exit, environment-based config


Rust's ecosystem provides a rich, layered approach to observability. At the foundation sits the `log` crate — a lightweight façade. Above it lives the `tracing` crate — a structured, async-aware, context-rich evolution. Together with subscribers, exporters, and third-party platforms, they form a complete observability stack covering logs, metrics, and distributed traces.

---

## Table of Contents

1. [The `log` Crate — The Classic Façade](#1-the-log-crate)
2. [The `tracing` Crate — Structured Observability](#2-the-tracing-crate)
3. [Spans — Context and Causality](#3-spans)
4. [Events — Structured Log Records](#4-events)
5. [Subscribers — Consuming Telemetry](#5-subscribers)
6. [Structured Logging with Fields](#6-structured-logging-with-fields)
7. [Instrumenting Async Code](#7-instrumenting-async-code)
8. [Composing Layers with `tracing-subscriber`](#8-composing-layers)
9. [Integrating with Observability Platforms](#9-integrating-with-observability-platforms)
10. [Best Practices](#10-best-practices)

---

## 1. The `log` Crate

The `log` crate defines a universal logging façade. Libraries emit log records; applications choose the concrete backend at startup.

### Cargo.toml

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"   # one of many possible backends
```

### Macros: `error!`, `warn!`, `info!`, `debug!`, `trace!`

```rust
use log::{error, warn, info, debug, trace};

fn process_order(order_id: u64, amount: f64) {
    info!("Processing order id={} amount={:.2}", order_id, amount);

    if amount > 10_000.0 {
        warn!("Large order detected: id={} amount={:.2}", order_id, amount);
    }

    match do_payment(amount) {
        Ok(_)  => debug!("Payment succeeded for order {}", order_id),
        Err(e) => error!("Payment failed for order {}: {}", order_id, e),
    }
}

fn do_payment(_: f64) -> Result<(), String> { Ok(()) }

fn main() {
    env_logger::init(); // reads RUST_LOG env var
    process_order(42, 250.0);
}
```

Run with:

```bash
RUST_LOG=debug cargo run
```

### Setting the Maximum Log Level

`env_logger` respects `RUST_LOG` with fine-grained module filtering:

```bash
RUST_LOG=my_app=debug,hyper=warn cargo run
```

### Limitations of `log`

The `log` crate records individual unstructured text messages. It has no notion of *context* (which request triggered this log?), no nesting, and no native support for async tasks. This is where `tracing` shines.

---

## 2. The `tracing` Crate

`tracing` was built to address the shortcomings of `log` in modern async, multi-threaded services. Its core concepts are:

| Concept      | Description |
|--------------|-------------|
| **Span**     | A period of time with a name, structured fields, and optional parent |
| **Event**    | A point-in-time record (equivalent to a log line), emitted inside a span |
| **Subscriber** | Collects spans and events; the backend |
| **Layer**    | A composable piece of a subscriber pipeline |

### Cargo.toml

```toml
[dependencies]
tracing        = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

---

## 3. Spans

A span represents a unit of work — an HTTP request, a database query, a function call. Spans can be nested to form a *trace tree*.

```rust
use tracing::{span, Level, info};

fn authenticate(user_id: u64) -> bool {
    // Create a span with a structured field
    let span = span!(Level::INFO, "authenticate", user_id = user_id);
    let _guard = span.enter(); // span is active for this scope

    info!("Checking credentials");
    // ... auth logic ...
    true
}

fn handle_request(user_id: u64) {
    let span = span!(Level::INFO, "handle_request", user_id = user_id);
    let _guard = span.enter();

    info!("Request received");
    let ok = authenticate(user_id); // child span is automatically parented here
    info!(authenticated = ok, "Auth complete");
}

fn main() {
    tracing_subscriber::fmt::init();
    handle_request(99);
}
```

Sample output:

```
INFO handle_request{user_id=99}: my_app: Request received
INFO handle_request{user_id=99}:authenticate{user_id=99}: my_app: Checking credentials
INFO handle_request{user_id=99}: my_app: Auth complete authenticated=true
```

Notice how every event automatically carries the span context.

### Span Lifecycle

```rust
use tracing::{span, Level};

fn demo() {
    let span = span!(Level::DEBUG, "my_span", key = "value");

    // Method 1: RAII guard — exits when guard is dropped
    {
        let _guard = span.enter();
        // work inside span
    } // span exits here

    // Method 2: explicit enter/exit
    let guard = span.enter();
    drop(guard); // explicitly exit

    // Method 3: in_scope closure
    span.in_scope(|| {
        tracing::info!("inside the span");
    });
}
```

---

## 4. Events

Events are the `tracing` equivalent of log lines. They always occur *within* the current span context.

```rust
use tracing::{event, Level};

fn process_item(id: u32, value: f64) {
    // Key=value structured fields
    event!(Level::INFO, item_id = id, item_value = value, "Processing item");

    if value < 0.0 {
        event!(
            Level::WARN,
            item_id = id,
            item_value = value,
            "Negative value encountered"
        );
    }
}
```

### Convenience Macros

```rust
use tracing::{trace, debug, info, warn, error};

fn example(x: i32) {
    trace!(x, "entered example");
    debug!(x, "processing");
    info!("normal operation");
    warn!(x, "unusual input");
    error!(x, "something went wrong");
}
```

---

## 5. Subscribers

A `Subscriber` is the backend that receives all span and event data. The `tracing-subscriber` crate provides the standard implementation.

### Simple Formatted Subscriber

```rust
fn main() {
    // Reads RUST_LOG; pretty human-readable output
    tracing_subscriber::fmt::init();
    tracing::info!("Application started");
}
```

### Configured Subscriber

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)          // show module path
        .with_thread_ids(true)      // include thread IDs
        .with_file(true)            // include source file
        .with_line_number(true)     // include line number
        .with_level(true)           // show log level
        .compact()                  // single-line format
        .init();

    tracing::info!("Ready");
}
```

### JSON Subscriber (for log aggregators)

```toml
tracing-subscriber = { version = "0.3", features = ["json"] }
```

```rust
use tracing_subscriber::fmt;

fn main() {
    fmt()
        .json()                     // emit newline-delimited JSON
        .with_current_span(true)    // include span context in each record
        .init();

    tracing::info!(service = "payments", "Service started");
}
```

Output:

```json
{"timestamp":"2024-11-01T10:00:00Z","level":"INFO","target":"my_app","service":"payments","message":"Service started"}
```

---

## 6. Structured Logging with Fields

Structured logging means attaching machine-readable key-value pairs to records instead of embedding data into message strings.

```rust
use tracing::info;

#[derive(Debug)]
struct Order {
    id: u64,
    customer_id: u64,
    total: f64,
}

fn fulfill_order(order: &Order) {
    // Fields are recorded as structured key=value pairs
    info!(
        order.id = order.id,
        order.customer_id = order.customer_id,
        order.total = order.total,
        "Fulfilling order"
    );
}
```

### Recording Errors as Fields

```rust
use tracing::{error, warn};

fn connect(url: &str) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused"))
}

fn init_db(url: &str) {
    match connect(url) {
        Ok(_)  => {},
        Err(e) => error!(error = %e, url, "Database connection failed"),
        //                      ^ Display format for the error
    }
}
```

The `%` sigil uses `Display`; `?` uses `Debug`:

```rust
tracing::info!(value = ?some_struct, "Debug format");
tracing::info!(value = %some_struct, "Display format");
```

---

## 7. Instrumenting Async Code

`tracing` is designed to work with async runtimes. The key challenge is that async tasks can suspend and resume on different threads, which breaks the classic thread-local span stack. `tracing` solves this with `Instrument`.

### The `#[instrument]` Macro

```toml
tracing = { version = "0.1", features = ["attributes"] }
```

```rust
use tracing::instrument;

#[instrument]             // auto-creates a span named after the function
async fn fetch_user(user_id: u64) -> Result<String, String> {
    tracing::info!("Fetching user from database");
    // async work...
    Ok(format!("User {}", user_id))
}

#[instrument(skip(password), fields(username))]
async fn login(username: String, password: String) {
    // `password` is excluded from the span; `username` is included
    tracing::info!("Login attempt");
}

#[instrument(name = "db.query", level = "debug", err)]
async fn run_query(sql: &str) -> Result<Vec<String>, String> {
    // `err` automatically records errors as span events
    Err("syntax error".into())
}
```

### Manual `.instrument()` for Futures

```rust
use tracing::{info_span, Instrument};

async fn background_task() {
    let span = info_span!("background_task", task_id = 7);
    async move {
        tracing::info!("Task running");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        tracing::info!("Task done");
    }
    .instrument(span)
    .await;
}
```

### Full Async Example with Tokio

```rust
use tracing::{info, instrument};
use tracing_subscriber::fmt;

#[instrument(fields(order_id))]
async fn process_payment(order_id: u64, amount: f64) -> Result<(), String> {
    info!(amount, "Starting payment");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    info!("Payment complete");
    Ok(())
}

#[tokio::main]
async fn main() {
    fmt::init();

    tokio::join!(
        process_payment(1, 49.99),
        process_payment(2, 199.00),
    );
}
```

Even though both tasks may interleave on the same thread, each event correctly carries its own span context.

---

## 8. Composing Layers

`tracing-subscriber` uses a *layer* system: each layer processes telemetry independently and they are stacked together. This allows separating concerns such as filtering, formatting, and exporting.

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

fn init_tracing() {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .compact();

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)   // apply filtering first
        .with(fmt_layer)      // then format and print
        .init();
}

fn main() {
    init_tracing();
    tracing::info!("Layered subscriber ready");
}
```

### Dynamic Filtering with `reload`

```rust
use tracing_subscriber::{reload, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    let filter = EnvFilter::new("info");
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initial level: info");

    // Dynamically change the filter at runtime (e.g., via an admin endpoint)
    reload_handle
        .modify(|f| *f = EnvFilter::new("debug"))
        .unwrap();

    tracing::debug!("Now debug messages appear too");
}
```

---

## 9. Integrating with Observability Platforms

### OpenTelemetry (OTLP — Jaeger, Tempo, Honeycomb, Datadog…)

OpenTelemetry is the industry standard for portable distributed tracing and metrics. The `tracing-opentelemetry` crate bridges the two worlds.

```toml
[dependencies]
tracing                  = "0.1"
tracing-subscriber       = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry    = "0.25"
opentelemetry            = "0.24"
opentelemetry_sdk        = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-otlp       = { version = "0.17", features = ["tonic"] }
```

```rust
use opentelemetry::global;
use opentelemetry_sdk::runtime;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

async fn init_telemetry() {
    // Build an OTLP exporter sending to a local collector
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to install tracer");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())  // local console output
        .with(otel_layer)                        // remote OTLP export
        .init();
}

#[tracing::instrument]
async fn handle_http_request(path: &str) {
    tracing::info!(path, "Handling request");
    // child spans automatically propagate trace context
}

#[tokio::main]
async fn main() {
    init_telemetry().await;
    handle_http_request("/api/orders").await;
    global::shutdown_tracer_provider(); // flush remaining spans before exit
}
```

### Bridging `log` → `tracing`

Libraries that still use the classic `log` crate can have their output routed through `tracing`:

```toml
tracing-log = "0.2"
```

```rust
fn main() {
    tracing_subscriber::fmt::init();

    // Route log records into the tracing subscriber
    tracing_log::LogTracer::init().unwrap();

    // Now `log::info!` calls appear as tracing events
    log::info!("This comes from a log-based library");
}
```

### tokio-console — Live Async Task Inspection

`tokio-console` is a terminal UI for inspecting live Tokio tasks.

```toml
console-subscriber = "0.3"
```

```rust
fn main() {
    console_subscriber::init(); // replaces the normal tracing subscriber
    // launch `tokio-console` in another terminal to connect
}
```

---

## 10. Best Practices

### Use `tracing` for new projects

Prefer `tracing` over `log` for all new code. It is a strict superset — libraries using `log` still work via the bridge crate.

### Structured fields over string interpolation

```rust
// Prefer this — fields are queryable in log aggregators:
tracing::info!(user_id = 42, action = "login", "User action");

// Over this — data is buried in the message string:
tracing::info!("User 42 performed login");
```

### Keep message strings static

Structured fields should carry the dynamic values; keep the message string constant so aggregators can group records by message:

```rust
// Good — aggregator can count "Payment processed" records and filter by amount
tracing::info!(amount = 99.0, currency = "EUR", "Payment processed");

// Avoid — every record has a unique message string
tracing::info!("Payment of 99.0 EUR processed");
```

### Use `#[instrument]` on public API boundaries

Instrument at function boundaries rather than sprinkling manual spans everywhere. This gives clean, automatic traces of call depth.

```rust
#[tracing::instrument(skip(ctx), err, ret)]
pub async fn create_invoice(ctx: &AppContext, customer_id: u64) -> Result<u64, AppError> {
    // `err` records the error as a span event if Err is returned
    // `ret` records the return value
    Ok(42)
}
```

### Flush before exit

Async exporters buffer data. Always call the shutdown function before your process exits to avoid losing the last spans:

```rust
opentelemetry::global::shutdown_tracer_provider();
```

### Environment-based configuration

Use `RUST_LOG` (or `OTEL_*` env vars for OpenTelemetry) for runtime control rather than hardcoding levels:

```bash
# For development
RUST_LOG=my_app=debug,sqlx=warn cargo run

# For production with JSON logs
RUST_LOG=info cargo run
```

---

## Summary

| Need | Tool |
|------|------|
| Simple logging façade in a library | `log` crate |
| Structured, contextual observability | `tracing` crate |
| Human-readable console output | `tracing-subscriber::fmt` |
| JSON logs for aggregators (ELK, Loki) | `tracing-subscriber` JSON feature |
| Distributed tracing (Jaeger, Tempo) | `tracing-opentelemetry` + OTLP |
| Bridge `log` libraries into `tracing` | `tracing-log` |
| Live async task inspection | `tokio-console` |
| Dynamic log level changes at runtime | `tracing-subscriber::reload` |

Rust's observability story is composable by design: each piece — emission, filtering, formatting, and export — is a separate, swappable crate, and they all connect through the common `tracing` subscriber interface.