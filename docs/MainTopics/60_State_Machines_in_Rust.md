# State Machines in Rust

- **Enum-based machines** — simple, idiomatic, exhaustive matching for runtime-driven state
- **The Typestate Pattern** — using `PhantomData<S>` to encode state into the type system so illegal transitions become compile errors, not panics
- **Sealed traits** — restricting which types can serve as state markers
- **Real-world examples** — a TCP handshake lifecycle, a type-safe HTTP request builder, a network session with mandatory setup ordering, and an order fulfillment workflow
- **Error handling in transitions** — returning `(OldState, Error)` to enable retry-safe login flows
- **Combining both patterns** — typestate for compile-time phases, enums for dynamic runtime sub-states within a phase
- **Comparison table** and a **best practices summary** at the end

The central theme throughout is Rust's guiding principle: *make illegal states unrepresentable* — and how ownership, generics, and zero-cost phantom types let you enforce protocol correctness entirely at compile time.

> Modeling state machines with enums, the typestate pattern, transitions, and correctness guarantees.

---

## Table of Contents

1. [What Is a State Machine?](#1-what-is-a-state-machine)
2. [Modeling States with Enums](#2-modeling-states-with-enums)
3. [Enum-Based State Machine with Transitions](#3-enum-based-state-machine-with-transitions)
4. [The Typestate Pattern](#4-the-typestate-pattern)
5. [Typestate with Generic Structs](#5-typestate-with-generic-structs)
6. [Encoding Transitions as Methods](#6-encoding-transitions-as-methods)
7. [Illegal States Made Unrepresentable](#7-illegal-states-made-unrepresentable)
8. [Real-World Example: TCP Connection](#8-real-world-example-tcp-connection)
9. [Real-World Example: Builder Pattern as a State Machine](#9-real-world-example-builder-pattern-as-a-state-machine)
10. [State Machines with Associated Data](#10-state-machines-with-associated-data)
11. [Combining Enums and Typestate](#11-combining-enums-and-typestate)
12. [Error Handling in Transitions](#12-error-handling-in-transitions)
13. [Compile-Time vs Run-Time State Machines](#13-compile-time-vs-run-time-state-machines)
14. [Summary and Best Practices](#14-summary-and-best-practices)

---

## 1. What Is a State Machine?

A **finite state machine (FSM)** is a computational model consisting of:

- A **finite set of states**
- A set of **transitions** between states, triggered by events or conditions
- An **initial state**
- One or more **terminal (accepting) states**

State machines are everywhere: network protocols, UI workflows, parsers, game logic, embedded systems. The challenge in most languages is enforcing that transitions only happen in valid sequences. Rust gives us powerful tools to enforce these constraints **at compile time** using its type system.

```
[Locked] --insert_coin--> [Unlocked] --push--> [Locked]
                                      --insert_coin--> [Unlocked]
```

---

## 2. Modeling States with Enums

The simplest approach is to represent each state as a variant of an enum.

```rust
#[derive(Debug, PartialEq)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    fn next(&self) -> TrafficLight {
        match self {
            TrafficLight::Red    => TrafficLight::Green,
            TrafficLight::Green  => TrafficLight::Yellow,
            TrafficLight::Yellow => TrafficLight::Red,
        }
    }

    fn duration_secs(&self) -> u32 {
        match self {
            TrafficLight::Red    => 60,
            TrafficLight::Green  => 45,
            TrafficLight::Yellow => 5,
        }
    }
}

fn main() {
    let mut light = TrafficLight::Red;
    for _ in 0..6 {
        println!("{:?} - {}s", light, light.duration_secs());
        light = light.next();
    }
}
```

**Output:**
```
Red - 60s
Green - 45s
Yellow - 5s
Red - 60s
Green - 45s
Yellow - 5s
```

This is clean, idiomatic, and exhaustive — the compiler forces you to handle every variant in every `match`.

---

## 3. Enum-Based State Machine with Transitions

For more complex machines, enums can carry data and the transition logic can live in a central `transition` function.

```rust
#[derive(Debug)]
enum VendingMachine {
    Idle,
    HasMoney { amount: u32 },
    Dispensing { item: String },
    OutOfStock,
}

#[derive(Debug)]
enum Event {
    InsertCoin(u32),
    SelectItem(String, u32 /* cost */),
    Dispense,
    Restock,
}

impl VendingMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (VendingMachine::Idle, Event::InsertCoin(amount)) => {
                println!("Inserted {} cents.", amount);
                VendingMachine::HasMoney { amount }
            }
            (VendingMachine::HasMoney { amount }, Event::InsertCoin(more)) => {
                println!("Added {} cents. Total: {}", more, amount + more);
                VendingMachine::HasMoney { amount: amount + more }
            }
            (VendingMachine::HasMoney { amount }, Event::SelectItem(item, cost))
                if amount >= cost =>
            {
                println!("Selected '{}'. Dispensing...", item);
                VendingMachine::Dispensing { item }
            }
            (VendingMachine::HasMoney { amount }, Event::SelectItem(item, cost)) => {
                println!("Not enough money for '{}'. Need {}, have {}.", item, cost, amount);
                VendingMachine::HasMoney { amount }
            }
            (VendingMachine::Dispensing { item }, Event::Dispense) => {
                println!("Enjoy your {}!", item);
                VendingMachine::Idle
            }
            (VendingMachine::OutOfStock, Event::Restock) => {
                println!("Machine restocked.");
                VendingMachine::Idle
            }
            (state, event) => {
                println!("Invalid transition: {:?} + {:?}", state, event);
                state
            }
        }
    }
}

fn main() {
    let machine = VendingMachine::Idle;
    let machine = machine.transition(Event::InsertCoin(50));
    let machine = machine.transition(Event::InsertCoin(75));
    let machine = machine.transition(Event::SelectItem("Cola".into(), 100));
    let machine = machine.transition(Event::Dispense);
    println!("Final state: {:?}", machine);
}
```

> **Key observation:** `transition` takes ownership of `self` and returns a *new* state, making it impossible to use the old state after a transition. This aligns perfectly with Rust's ownership model.

---

## 4. The Typestate Pattern

The **typestate pattern** encodes the state of an object into its **type**, not its value. This means:

- Invalid transitions are **compile errors**, not runtime panics.
- Each state can expose **only the methods that are valid for that state**.
- The type checker enforces the protocol.

The idea: use a generic struct parameterized by a **phantom type** (or a zero-sized type) that represents the current state.

```rust
use std::marker::PhantomData;

// State markers — zero-sized types, never instantiated
struct Locked;
struct Unlocked;

// The turnstile parameterized by its state
struct Turnstile<State> {
    coins_collected: u32,
    _state: PhantomData<State>,
}

// Methods only available when Locked
impl Turnstile<Locked> {
    fn new() -> Self {
        Turnstile { coins_collected: 0, _state: PhantomData }
    }

    fn insert_coin(self) -> Turnstile<Unlocked> {
        println!("Coin inserted. Turnstile unlocked.");
        Turnstile {
            coins_collected: self.coins_collected + 1,
            _state: PhantomData,
        }
    }
}

// Methods only available when Unlocked
impl Turnstile<Unlocked> {
    fn push(self) -> Turnstile<Locked> {
        println!("Pushed through. Turnstile locked.");
        Turnstile {
            coins_collected: self.coins_collected,
            _state: PhantomData,
        }
    }

    fn insert_coin(self) -> Turnstile<Unlocked> {
        // Extra coin returned; stays unlocked
        println!("Already unlocked. Coin returned.");
        self
    }
}

fn main() {
    let t = Turnstile::<Locked>::new();
    // t.push(); // ERROR: method `push` not found for `Turnstile<Locked>`
    let t = t.insert_coin();   // Locked -> Unlocked
    let t = t.push();          // Unlocked -> Locked
    let t = t.insert_coin();   // Locked -> Unlocked
    let t = t.insert_coin();   // Unlocked -> Unlocked (coin returned)
    let t = t.push();          // Unlocked -> Locked
    println!("Total coins: {}", t.coins_collected);
}
```

The commented-out line `t.push()` would be a **compile-time error** — you genuinely cannot push a locked turnstile in this model.

---

## 5. Typestate with Generic Structs

Typestates become even more powerful when you use **sealed traits** to restrict which types can be used as state markers, preventing misuse.

```rust
// A sealed module to prevent external state types
mod sealed {
    pub trait DoorState {}
}

pub struct Open;
pub struct Closed;
pub struct Locked;

impl sealed::DoorState for Open {}
impl sealed::DoorState for Closed {}
impl sealed::DoorState for Locked {}

use std::marker::PhantomData;

pub struct Door<S: sealed::DoorState> {
    id: u32,
    _state: PhantomData<S>,
}

impl Door<Closed> {
    pub fn new(id: u32) -> Self {
        Door { id, _state: PhantomData }
    }

    pub fn open(self) -> Door<Open> {
        println!("Door {} opened.", self.id);
        Door { id: self.id, _state: PhantomData }
    }

    pub fn lock(self) -> Door<Locked> {
        println!("Door {} locked.", self.id);
        Door { id: self.id, _state: PhantomData }
    }
}

impl Door<Open> {
    pub fn close(self) -> Door<Closed> {
        println!("Door {} closed.", self.id);
        Door { id: self.id, _state: PhantomData }
    }
}

impl Door<Locked> {
    pub fn unlock(self) -> Door<Closed> {
        println!("Door {} unlocked.", self.id);
        Door { id: self.id, _state: PhantomData }
    }
}

fn main() {
    let door = Door::new(42);       // Door<Closed>
    let door = door.open();         // Door<Open>
    let door = door.close();        // Door<Closed>
    let door = door.lock();         // Door<Locked>
    let door = door.unlock();       // Door<Closed>
    let _door = door.open();        // Door<Open>

    // All of the following would be compile errors:
    // door.lock()    — can't lock an open door
    // door.unlock()  — can't unlock a closed (not locked) door
    // door.close()   — can't close an already-closed door
}
```

---

## 6. Encoding Transitions as Methods

A key advantage of the typestate pattern is that **transitions are just methods that consume `self` and return a new typed value**. The old state is moved out and can never be used again.

```rust
// Demonstrating that the old state is truly inaccessible
fn demonstrate_move_semantics() {
    let door = Door::<Closed>::new(1);
    let open_door = door.open();

    // door.lock(); // ERROR: use of moved value `door`
    // The compiler won't let you accidentally use the old state.
    
    let _ = open_door.close();
}
```

This is Rust's ownership model working in perfect harmony with the typestate pattern — **no extra runtime cost, no locks, no `Option` unwrapping**.

---

## 7. Illegal States Made Unrepresentable

The greatest power of typestate is the phrase: **"make illegal states unrepresentable."**

Consider a naive approach using runtime flags:

```rust
// BAD: Runtime approach — illegal states are possible
struct Connection {
    connected: bool,
    authenticated: bool,  // Meaningless if not connected!
    tls_established: bool, // Meaningless if not authenticated!
}
```

Nothing stops you from setting `authenticated = true` without being connected. With typestate:

```rust
use std::marker::PhantomData;

struct Disconnected;
struct Connected;
struct Authenticated;
struct TlsEstablished;

struct NetworkSession<S> {
    address: String,
    _state: PhantomData<S>,
}

impl NetworkSession<Disconnected> {
    fn new(address: &str) -> Self {
        NetworkSession { address: address.to_string(), _state: PhantomData }
    }

    fn connect(self) -> Result<NetworkSession<Connected>, String> {
        // Simulate connection attempt
        println!("Connecting to {}...", self.address);
        Ok(NetworkSession { address: self.address, _state: PhantomData })
    }
}

impl NetworkSession<Connected> {
    fn authenticate(self, _token: &str) -> Result<NetworkSession<Authenticated>, String> {
        println!("Authenticating...");
        Ok(NetworkSession { address: self.address, _state: PhantomData })
    }
}

impl NetworkSession<Authenticated> {
    fn establish_tls(self) -> Result<NetworkSession<TlsEstablished>, String> {
        println!("Establishing TLS...");
        Ok(NetworkSession { address: self.address, _state: PhantomData })
    }
}

impl NetworkSession<TlsEstablished> {
    fn send_data(&self, data: &str) {
        // Only reachable after the full setup sequence
        println!("Sending securely: {}", data);
    }

    fn disconnect(self) -> NetworkSession<Disconnected> {
        println!("Disconnecting.");
        NetworkSession { address: self.address, _state: PhantomData }
    }
}

fn main() -> Result<(), String> {
    let session = NetworkSession::new("192.168.1.1");
    let session = session.connect()?;
    let session = session.authenticate("secret-token")?;
    let session = session.establish_tls()?;

    session.send_data("Hello, secure world!");

    // session.send_data("...")  // Impossible without the full chain above
    Ok(())
}
```

`send_data` is **only available** on `NetworkSession<TlsEstablished>`. The only way to get there is through the correct sequence. The protocol is enforced by the type system.

---

## 8. Real-World Example: TCP Connection

A simplified model of the TCP handshake lifecycle:

```rust
use std::marker::PhantomData;

// State markers
struct SynSent;
struct SynReceived;
struct Established;
struct FinWait;
struct Closed;

struct TcpConnection<S> {
    local_port: u16,
    remote_port: u16,
    sequence_number: u32,
    _state: PhantomData<S>,
}

impl TcpConnection<Closed> {
    fn new(local_port: u16, remote_port: u16) -> Self {
        TcpConnection {
            local_port,
            remote_port,
            sequence_number: 0,
            _state: PhantomData,
        }
    }

    fn send_syn(self) -> TcpConnection<SynSent> {
        println!("[{}->{}] SYN sent", self.local_port, self.remote_port);
        TcpConnection {
            sequence_number: self.sequence_number + 1,
            ..self.into_state()
        }
    }
}

impl TcpConnection<SynSent> {
    fn receive_syn_ack(self) -> TcpConnection<Established> {
        println!("[{}->{}] SYN-ACK received, ACK sent — connection established",
            self.local_port, self.remote_port);
        TcpConnection {
            sequence_number: self.sequence_number + 1,
            ..self.into_state()
        }
    }
}

impl TcpConnection<Established> {
    fn send(&self, data: &str) {
        println!("[{}->{}] DATA: {}", self.local_port, self.remote_port, data);
    }

    fn close(self) -> TcpConnection<FinWait> {
        println!("[{}->{}] FIN sent", self.local_port, self.remote_port);
        TcpConnection {
            sequence_number: self.sequence_number + 1,
            ..self.into_state()
        }
    }
}

impl TcpConnection<FinWait> {
    fn receive_fin_ack(self) -> TcpConnection<Closed> {
        println!("[{}->{}] FIN-ACK received — connection closed",
            self.local_port, self.remote_port);
        self.into_state()
    }
}

// Helper: reinterpret state type without changing fields
impl<S> TcpConnection<S> {
    fn into_state<T>(self) -> TcpConnection<T> {
        TcpConnection {
            local_port: self.local_port,
            remote_port: self.remote_port,
            sequence_number: self.sequence_number,
            _state: PhantomData,
        }
    }
}

fn main() {
    let conn = TcpConnection::<Closed>::new(54321, 80);
    let conn = conn.send_syn();
    let conn = conn.receive_syn_ack();

    conn.send("GET / HTTP/1.1");
    conn.send("Host: example.com");

    let conn = conn.close();
    let _conn = conn.receive_fin_ack();
}
```

---

## 9. Real-World Example: Builder Pattern as a State Machine

The builder pattern can be seen as a state machine where fields are progressively filled and the final state is only reachable when all required fields have been provided.

```rust
use std::marker::PhantomData;

// Marker types for required fields
struct NoUrl;
struct HasUrl(String);
struct NoMethod;
struct HasMethod(String);

struct HttpRequestBuilder<U, M> {
    url: U,
    method: M,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl HttpRequestBuilder<NoUrl, NoMethod> {
    fn new() -> Self {
        HttpRequestBuilder {
            url: NoUrl,
            method: NoMethod,
            headers: vec![],
            body: None,
        }
    }
}

impl<M> HttpRequestBuilder<NoUrl, M> {
    fn url(self, url: &str) -> HttpRequestBuilder<HasUrl, M> {
        HttpRequestBuilder {
            url: HasUrl(url.to_string()),
            method: self.method,
            headers: self.headers,
            body: self.body,
        }
    }
}

impl<U> HttpRequestBuilder<U, NoMethod> {
    fn method(self, method: &str) -> HttpRequestBuilder<U, HasMethod> {
        HttpRequestBuilder {
            url: self.url,
            method: HasMethod(method.to_string()),
            headers: self.headers,
            body: self.body,
        }
    }
}

impl<U, M> HttpRequestBuilder<U, M> {
    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

// `build()` is ONLY available when both URL and method have been set
impl HttpRequestBuilder<HasUrl, HasMethod> {
    fn build(self) -> HttpRequest {
        HttpRequest {
            url: self.url.0,
            method: self.method.0,
            headers: self.headers,
            body: self.body,
        }
    }
}

#[derive(Debug)]
struct HttpRequest {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

fn main() {
    let request = HttpRequestBuilder::new()
        .url("https://api.example.com/data")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(r#"{"key": "value"}"#)
        .build(); // Only compiles because both url() and method() were called

    println!("{:#?}", request);

    // This would NOT compile:
    // HttpRequestBuilder::new().url("...").build()
    //   => ERROR: method `build` not found for HttpRequestBuilder<HasUrl, NoMethod>
}
```

---

## 10. State Machines with Associated Data

States often carry **different data** depending on the current state. Enums shine here because each variant can have its own fields.

```rust
#[derive(Debug)]
enum OrderStatus {
    Pending {
        cart_items: Vec<String>,
    },
    PaymentProcessing {
        cart_items: Vec<String>,
        transaction_id: String,
    },
    Shipped {
        tracking_number: String,
        estimated_delivery: String,
    },
    Delivered {
        delivered_at: String,
        signature: Option<String>,
    },
    Cancelled {
        reason: String,
        refund_issued: bool,
    },
}

impl OrderStatus {
    fn confirm_payment(self, transaction_id: String) -> Self {
        match self {
            OrderStatus::Pending { cart_items } => {
                println!("Payment processing: txn {}", transaction_id);
                OrderStatus::PaymentProcessing { cart_items, transaction_id }
            }
            other => {
                eprintln!("Cannot process payment in state: {:?}", other);
                other
            }
        }
    }

    fn ship(self, tracking: &str, eta: &str) -> Self {
        match self {
            OrderStatus::PaymentProcessing { .. } => {
                println!("Order shipped! Tracking: {}", tracking);
                OrderStatus::Shipped {
                    tracking_number: tracking.to_string(),
                    estimated_delivery: eta.to_string(),
                }
            }
            other => {
                eprintln!("Cannot ship order in state: {:?}", other);
                other
            }
        }
    }

    fn deliver(self, timestamp: &str, signature: Option<String>) -> Self {
        match self {
            OrderStatus::Shipped { .. } => {
                println!("Order delivered at {}", timestamp);
                OrderStatus::Delivered {
                    delivered_at: timestamp.to_string(),
                    signature,
                }
            }
            other => {
                eprintln!("Cannot deliver order in state: {:?}", other);
                other
            }
        }
    }
}

fn main() {
    let order = OrderStatus::Pending {
        cart_items: vec!["Book".into(), "Pen".into()],
    };

    let order = order.confirm_payment("TXN-987654".into());
    let order = order.ship("1Z999AA10123456784", "2024-12-20");
    let order = order.deliver("2024-12-19", Some("J. Smith".into()));

    println!("\nFinal: {:#?}", order);
}
```

---

## 11. Combining Enums and Typestate

These two approaches are complementary: use **typestate** when the state is known at compile time (e.g., protocol phases), and **enums** when state is dynamic (e.g., driven by runtime input).

```rust
use std::marker::PhantomData;

// Compile-time phase
struct Configuration;
struct Running;

// Runtime sub-state (driven by events)
#[derive(Debug, Clone)]
enum WorkerStatus {
    Idle,
    Processing { job_id: u64 },
    Error(String),
}

struct Worker<Phase> {
    name: String,
    status: WorkerStatus,
    _phase: PhantomData<Phase>,
}

impl Worker<Configuration> {
    fn new(name: &str) -> Self {
        Worker {
            name: name.to_string(),
            status: WorkerStatus::Idle,
            _phase: PhantomData,
        }
    }

    fn configure(self, _config: &str) -> Self {
        println!("Worker '{}' configured.", self.name);
        self
    }

    fn start(self) -> Worker<Running> {
        println!("Worker '{}' started.", self.name);
        Worker {
            name: self.name,
            status: WorkerStatus::Idle,
            _phase: PhantomData,
        }
    }
}

impl Worker<Running> {
    fn assign_job(&mut self, job_id: u64) {
        self.status = WorkerStatus::Processing { job_id };
        println!("Worker '{}' processing job {}.", self.name, job_id);
    }

    fn complete_job(&mut self) {
        self.status = WorkerStatus::Idle;
        println!("Worker '{}' completed job.", self.name);
    }

    fn report_error(&mut self, msg: &str) {
        self.status = WorkerStatus::Error(msg.to_string());
        println!("Worker '{}' error: {}", self.name, msg);
    }

    fn current_status(&self) -> &WorkerStatus {
        &self.status
    }
}

fn main() {
    let worker = Worker::<Configuration>::new("worker-1")
        .configure("threads=4,timeout=30");

    // worker.assign_job(1); // ERROR: method only available on Worker<Running>

    let mut worker = worker.start();
    worker.assign_job(42);
    worker.complete_job();
    worker.assign_job(99);
    worker.report_error("Disk full");

    println!("Status: {:?}", worker.current_status());
}
```

---

## 12. Error Handling in Transitions

State transitions often fail. Using `Result` in transitions allows for both safe error propagation and natural state rollback.

```rust
use std::marker::PhantomData;

#[derive(Debug)]
enum AuthError {
    InvalidCredentials,
    AccountLocked,
    NetworkError(String),
}

struct LoggedOut;
struct LoggedIn { user_id: u64 }

struct Session<S> {
    _state: PhantomData<S>,
}

impl Session<LoggedOut> {
    fn new() -> Self {
        Session { _state: PhantomData }
    }

    fn login(
        self,
        username: &str,
        password: &str,
    ) -> Result<Session<LoggedIn>, (Session<LoggedOut>, AuthError)> {
        // Simulate authentication
        if username == "admin" && password == "secret" {
            println!("Login successful.");
            Ok(Session { _state: PhantomData })
        } else if password.is_empty() {
            // On failure, return the original state so it can be retried
            Err((Session { _state: PhantomData }, AuthError::InvalidCredentials))
        } else {
            Err((Session { _state: PhantomData }, AuthError::AccountLocked))
        }
    }
}

impl Session<LoggedIn> {
    fn logout(self) -> Session<LoggedOut> {
        println!("Logged out.");
        Session { _state: PhantomData }
    }

    fn fetch_data(&self) -> &str {
        "sensitive user data"
    }
}

fn main() {
    let session = Session::<LoggedOut>::new();

    // First attempt: wrong password
    let session = match session.login("admin", "wrong") {
        Ok(s) => {
            println!("Data: {}", s.fetch_data());
            s.logout()
        }
        Err((session, err)) => {
            println!("Login failed: {:?}. Retrying...", err);
            session // recover the LoggedOut session
        }
    };

    // Second attempt: correct password
    let session = match session.login("admin", "secret") {
        Ok(s) => s,
        Err((_, err)) => {
            eprintln!("Fatal: {:?}", err);
            return;
        }
    };

    println!("Fetched: {}", session.fetch_data());
    let _ = session.logout();
}
```

> **Pattern:** On failure, returning `(OldState, Error)` allows the caller to **recover and retry**, which is common in retry-loop patterns for network protocols.

---

## 13. Compile-Time vs Run-Time State Machines

| Feature | Enum-Based (Runtime) | Typestate (Compile-Time) |
|---|---|---|
| State known at compile time | No | Yes |
| Invalid transitions | Runtime panic/error | Compile error |
| State carries different data | Easy (enum variants) | Harder (need generics) |
| Dynamic/data-driven transitions | Yes | No |
| Zero-cost abstraction | Yes | Yes |
| Code complexity | Lower | Higher |
| Best for | Parsers, protocols driven by input | APIs, protocol clients, builders |

**Rule of thumb:**
- If state is **driven by external input at runtime** → use enums.
- If state **represents a phase of an API or protocol** that the programmer controls → use typestate.

---

## 14. Summary and Best Practices

### Core Concepts

**Enums for state** — Idiomatic, exhaustive, great for runtime-driven machines. Each variant can hold relevant data for that state.

**Typestate pattern** — Zero-cost compile-time enforcement. Uses phantom types (`PhantomData<S>`) to parameterize structs by their state. Methods are only available in the correct state.

**Sealed traits** — Combine with typestate to prevent arbitrary types from being used as state markers.

**Transitions consume `self`** — Both patterns leverage Rust's ownership to make it impossible to use a state after transitioning away from it.

### Best Practices

- **Make illegal states unrepresentable** — If a combination of values makes no sense, don't allow it to exist at the type level.
- **Use ownership to enforce linearity** — Taking `self` by value in transitions prevents accidental reuse of stale state.
- **Return `Result<NewState, (OldState, Error)>`** when transitions can fail and retry is desirable.
- **Combine both patterns** when needed — typestate for phases, enums for dynamic sub-states within a phase.
- **Use `PhantomData<S>`** (not `PhantomData<fn() -> S>`) for simple state markers; the latter is needed in advanced variance scenarios.
- **Keep state markers zero-sized** — They incur zero runtime overhead.

### Quick Reference

```rust
// Enum state machine — runtime
enum State { A, B, C }
fn transition(s: State, event: Event) -> State { ... }

// Typestate — compile-time
use std::marker::PhantomData;
struct StateA; struct StateB;
struct Machine<S> { _s: PhantomData<S> }
impl Machine<StateA> {
    fn to_b(self) -> Machine<StateB> { Machine { _s: PhantomData } }
}
// Machine<StateA>::to_b() → Machine<StateB>
// Machine<StateB> has NO to_b() — compile error if you try
```

State machines in Rust embody the language's philosophy: **correctness without compromise on performance**. By leveraging the type system, you can model complex stateful systems where the compiler itself becomes your protocol enforcer.

---

*See also: [Builder Pattern](../BuilderPattern.md) | [Phantom Types](../PhantomTypes.md) | [Ownership & Borrowing](../Ownership.md)*