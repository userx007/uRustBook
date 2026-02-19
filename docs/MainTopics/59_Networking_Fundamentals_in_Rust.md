# Networking Fundamentals in Rust

- **`std::net` basics** — synchronous `TcpListener`, `TcpStream`, and `UdpSocket` with a working echo server/client.
- **TCP & UDP** — connection handling, multicast, BufReader/BufWriter wrappers, and timeouts.
- **Socket options** — `set_nodelay`, `set_read_timeout`, and advanced control via the `socket2` crate.
- **TLS** — loading certificates with `rustls-pemfile`, building a `ServerConfig`, and wrapping streams with `tokio-rustls`.
- **Async Tokio networking** — task-per-connection model, async UDP, and graceful shutdown using `tokio::select!`.
- **Common patterns** — connection pooling (`deadpool`), heartbeats, backpressure with bounded `mpsc` channels, length-prefixed message framing, and quick-start examples for `reqwest` and `axum`.
- **Error handling** — distinguishing `EINTR`/`WouldBlock` from fatal errors, and custom error types with `thiserror`.
- **Testing** — ephemeral port `0` trick for parallel tests and in-memory mocking with `tokio-test`.

A decision table at the end maps common requirements to the right crate or approach.

Rust's networking story is one of its strongest suits: the standard library provides
synchronous TCP/UDP primitives, the ecosystem adds TLS, and **Tokio** (together with
`async`/`await`) makes it practical to handle tens of thousands of concurrent connections
without leaving the language's safety guarantees behind.

---

## Table of Contents

1. [The Standard Library Networking Model](#1-the-standard-library-networking-model)
2. [TCP — Transmission Control Protocol](#2-tcp--transmission-control-protocol)
3. [UDP — User Datagram Protocol](#3-udp--user-datagram-protocol)
4. [Socket Options and Low-Level Control](#4-socket-options-and-low-level-control)
5. [TLS Integration](#5-tls-integration)
6. [Async Networking with Tokio](#6-async-networking-with-tokio)
7. [Common Networking Patterns](#7-common-networking-patterns)
8. [Error Handling in Network Code](#8-error-handling-in-network-code)
9. [Testing Network Code](#9-testing-network-code)
10. [Summary and Key Takeaways](#10-summary-and-key-takeaways)

---

## 1. The Standard Library Networking Model

The `std::net` module gives you blocking, synchronous networking. It is the right
starting point when you need simplicity, a CLI tool, or a scripting-style program where
a single thread-per-connection model is acceptable.

```rust
use std::net::{TcpListener, TcpStream, UdpSocket};
```

Key types:

| Type | Purpose |
|------|---------|
| `TcpListener` | Accepts incoming TCP connections |
| `TcpStream` | A connected TCP socket (read + write) |
| `UdpSocket` | A connectionless UDP socket |
| `SocketAddr` | An IP address + port pair |
| `IpAddr` | An IPv4 or IPv6 address |

All I/O operations on these types block the calling thread. If you need concurrency
you must either spawn OS threads or switch to the async model described later.

---

## 2. TCP — Transmission Control Protocol

TCP guarantees ordered, reliable delivery. It is the foundation of HTTP, SSH, databases,
and most internet protocols.

### 2.1 Synchronous TCP Server

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    println!("New connection from {peer}");

    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("{peer} disconnected");
                break;
            }
            Ok(_) => {
                print!("[{peer}] received: {line}");
                // Echo back in upper-case
                let response = line.trim().to_uppercase() + "\n";
                if writer.write_all(response.as_bytes()).is_err() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Read error from {peer}: {e}");
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    // Bind to all interfaces on port 8080
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // Spawn a new thread per connection (simple but limited)
                thread::spawn(|| handle_client(s));
            }
            Err(e) => eprintln!("Accept error: {e}"),
        }
    }
    Ok(())
}
```

### 2.2 Synchronous TCP Client

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to {}", stream.peer_addr()?);

    // Set read/write timeouts to avoid blocking forever
    use std::time::Duration;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let message = "hello, rust networking\n";
    stream.write_all(message.as_bytes())?;

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("Server replied: {}", response.trim());

    Ok(())
}
```

### 2.3 `TcpStream` as `Read` + `Write`

`TcpStream` implements both `std::io::Read` and `std::io::Write`, so you can wrap it in
any of the standard adapters (`BufReader`, `BufWriter`, `Lines`, etc.).

```rust
use std::io::{BufWriter, Write};
use std::net::TcpStream;

fn send_http_request(host: &str) -> std::io::Result<()> {
    let stream = TcpStream::connect((host, 80))?;
    let mut writer = BufWriter::new(&stream);

    write!(writer, "GET / HTTP/1.0\r\nHost: {host}\r\n\r\n")?;
    writer.flush()?; // BufWriter buffers — always flush!
    Ok(())
}
```

---

## 3. UDP — User Datagram Protocol

UDP sacrifices ordering and reliability in exchange for lower latency and no connection
overhead. It is used by DNS, DHCP, video streaming, online games, and QUIC.

### 3.1 UDP Echo Server

```rust
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;
    println!("UDP server listening on {}", socket.local_addr()?);

    let mut buf = [0u8; 1024];
    loop {
        // recv_from returns (bytes_received, sender_address)
        let (len, src) = socket.recv_from(&mut buf)?;
        let msg = std::str::from_utf8(&buf[..len]).unwrap_or("<invalid utf8>");
        println!("Received {len} bytes from {src}: {msg}");

        // Echo back
        socket.send_to(&buf[..len], src)?;
    }
}
```

### 3.2 UDP Client

```rust
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    // Binding to port 0 lets the OS pick an ephemeral port
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // connect() doesn't send any packets; it just sets the default destination
    socket.connect("127.0.0.1:9000")?;

    let msg = b"ping";
    socket.send(msg)?;

    let mut buf = [0u8; 1024];
    let len = socket.recv(&mut buf)?;
    println!("Got reply: {}", std::str::from_utf8(&buf[..len]).unwrap());

    Ok(())
}
```

### 3.3 Multicast UDP

```rust
use std::net::{Ipv4Addr, UdpSocket};

fn join_multicast_group() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5000")?;
    let multicast_addr = Ipv4Addr::new(239, 0, 0, 1);
    let interface    = Ipv4Addr::UNSPECIFIED;

    socket.join_multicast_v4(&multicast_addr, &interface)?;
    println!("Joined multicast group {multicast_addr}");

    let mut buf = [0u8; 4096];
    let (len, src) = socket.recv_from(&mut buf)?;
    println!("Multicast message from {src}: {} bytes", len);

    Ok(())
}
```

---

## 4. Socket Options and Low-Level Control

```rust
use std::net::TcpListener;
use std::time::Duration;

fn configure_listener() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;

    // Disable Nagle's algorithm for lower latency (at the cost of more small packets)
    // Note: set_nodelay is on TcpStream, not TcpListener; apply after accept()

    for stream in listener.incoming().take(1) {
        let stream = stream?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        // Keepalive can be set via the `socket2` crate for full control
        println!("Configured stream: {:?}", stream.local_addr()?);
    }
    Ok(())
}
```

For advanced socket options (e.g. `SO_REUSEPORT`, `TCP_KEEPIDLE`) the community
recommends the [`socket2`](https://crates.io/crates/socket2) crate, which gives raw
access to OS socket options while remaining cross-platform.

```toml
# Cargo.toml
[dependencies]
socket2 = "0.5"
```

```rust
use socket2::{Domain, Protocol, Socket, TcpKeepalive, Type};
use std::time::Duration;

fn socket_with_keepalive() -> std::io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;   // Linux/macOS only

    let keepalive = TcpKeepalive::new()
        .with_time(Duration::from_secs(60))
        .with_interval(Duration::from_secs(10));
    socket.set_tcp_keepalive(&keepalive)?;

    socket.bind(&"0.0.0.0:8080".parse::<std::net::SocketAddr>().unwrap().into())?;
    socket.listen(128)?;

    // Convert to std::net::TcpListener
    let _listener: std::net::TcpListener = socket.into();
    Ok(())
}
```

---

## 5. TLS Integration

Raw TCP sends data in plaintext. TLS (Transport Layer Security) adds encryption,
authentication, and integrity. The two most popular crates are:

| Crate | Backend | Notes |
|-------|---------|-------|
| `rustls` | Pure Rust (no OpenSSL) | Preferred for new projects |
| `native-tls` | System TLS (OpenSSL/SChannel/SecTransport) | Maximum compatibility |
| `tokio-rustls` | `rustls` + Tokio | Async TLS for Tokio apps |

### 5.1 TLS Server with `rustls`

```toml
[dependencies]
rustls        = "0.23"
rustls-pemfile = "2"
```

```rust
use rustls::ServerConfig;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

fn load_server_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    let cert_file = File::open(cert_path).expect("Cannot open cert file");
    let key_file  = File::open(key_path).expect("Cannot open key file");

    let certs: Vec<_> = certs(&mut BufReader::new(cert_file))
        .map(|r| r.unwrap())
        .collect();

    let key = private_key(&mut BufReader::new(key_file))
        .unwrap()
        .expect("No private key found");

    Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Bad certificate or key"),
    )
}
```

### 5.2 Async TLS with `tokio-rustls`

```toml
[dependencies]
tokio       = { version = "1", features = ["full"] }
tokio-rustls = "0.26"
rustls       = "0.23"
rustls-pemfile = "2"
```

```rust
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Assume load_server_config() from the previous example
    let config   = load_server_config("cert.pem", "key.pem");
    let acceptor = TlsAcceptor::from(config);
    let listener = TcpListener::bind("0.0.0.0:8443").await?;
    println!("TLS server on :8443");

    loop {
        let (stream, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    println!("TLS handshake OK from {addr}");
                    tls_stream.write_all(b"Hello over TLS!\n").await.ok();
                }
                Err(e) => eprintln!("TLS error from {addr}: {e}"),
            }
        });
    }
}
```

---

## 6. Async Networking with Tokio

Synchronous networking creates one OS thread per connection. At thousands of concurrent
connections that model hits memory and scheduling limits. **Tokio** solves this with an
async runtime: a small thread pool drives a large number of lightweight _tasks_ via
cooperative scheduling.

### 6.1 Tokio Dependencies

```toml
[dependencies]
tokio  = { version = "1", features = ["full"] }
```

### 6.2 Async TCP Echo Server

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle(stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        println!("[{addr}] {line}");
        let reply = format!("{}\n", line.to_uppercase());
        if write_half.write_all(reply.as_bytes()).await.is_err() {
            break;
        }
    }
    println!("{addr} disconnected");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Async TCP server on {}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        // Each connection runs as an independent async task
        tokio::spawn(handle(stream));
    }
}
```

### 6.3 Async TCP Client

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    write_half.write_all(b"hello async world\n").await?;

    if let Some(response) = lines.next_line().await? {
        println!("Server: {response}");
    }

    Ok(())
}
```

### 6.4 Async UDP with Tokio

```rust
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000").await?;
    let mut buf = vec![0u8; 1024];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let msg = std::str::from_utf8(&buf[..len]).unwrap_or("?");
        println!("UDP {src}: {msg}");
        socket.send_to(&buf[..len], src).await?;
    }
}
```

### 6.5 Graceful Shutdown with `tokio::select!`

```rust
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        tokio::select! {
            // Accept a new connection
            result = listener.accept() => {
                let (stream, addr) = result?;
                println!("Accepted {addr}");
                tokio::spawn(async move {
                    // ... handle connection ...
                    drop(stream);
                });
            }
            // Or respond to Ctrl-C
            _ = signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
        }
    }
    Ok(())
}
```

---

## 7. Common Networking Patterns

### 7.1 Connection Pooling

Opening a new TCP connection for every request is expensive (three-way handshake, TLS
handshake). Connection pools reuse established connections.

```toml
[dependencies]
deadpool = "0.12"
# For HTTP the `reqwest` or `hyper` client pools connections automatically.
```

For databases or raw TCP services, [`deadpool`](https://crates.io/crates/deadpool)
provides a generic async pool:

```rust
use deadpool::managed::{Manager, Metrics, Pool, RecycleResult};
use tokio::net::TcpStream;

struct TcpManager {
    addr: String,
}

impl Manager for TcpManager {
    type Type  = TcpStream;
    type Error = std::io::Error;

    async fn create(&self) -> Result<TcpStream, std::io::Error> {
        TcpStream::connect(&self.addr).await
    }

    async fn recycle(
        &self,
        _conn: &mut TcpStream,
        _metrics: &Metrics,
    ) -> RecycleResult<std::io::Error> {
        Ok(()) // add health-check logic here
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = TcpManager { addr: "127.0.0.1:8080".into() };
    let pool    = Pool::builder(manager).max_size(16).build()?;

    let mut conn = pool.get().await?;
    // use `conn` as a TcpStream ...
    Ok(())
}
```

### 7.2 Heartbeats / Keep-Alive Pings

Long-lived connections can be silently dropped by NAT devices or firewalls. Send a
periodic heartbeat to detect dead connections early.

```rust
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time;

async fn with_heartbeat(mut stream: TcpStream) {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        if stream.write_all(b"\x00").await.is_err() {
            eprintln!("Heartbeat failed — connection lost");
            break;
        }
    }
}
```

### 7.3 Backpressure with Bounded Channels

When producers are faster than consumers, unbounded channels grow without limit.
Use bounded channels to apply backpressure.

```rust
use tokio::sync::mpsc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Buffer at most 32 connections before back-pressuring the accept loop
    let (tx, mut rx) = mpsc::channel(32);
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    // Accept task
    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            // If the channel is full, this await will yield until space is available
            tx.send(stream).await.unwrap();
        }
    });

    // Worker task
    while let Some(stream) = rx.recv().await {
        tokio::spawn(async move {
            // handle stream
            drop(stream);
        });
    }
    Ok(())
}
```

### 7.4 Framing — Length-Prefixed Messages

Raw TCP is a byte stream with no inherent message boundaries. A common framing scheme
prepends a 4-byte big-endian length before each message.

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Send a framed message: [u32 length][payload]
async fn send_frame(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(payload).await?;
    Ok(())
}

/// Receive a framed message
async fn recv_frame(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(payload)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    send_frame(&mut stream, b"Hello, framing!").await?;
    let reply = recv_frame(&mut stream).await?;
    println!("Reply: {}", String::from_utf8_lossy(&reply));
    Ok(())
}
```

### 7.5 HTTP Client with `reqwest`

For HTTP/HTTPS you rarely write raw TCP code. The `reqwest` crate wraps Tokio and
handles connection pooling, redirects, gzip, and more.

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio   = { version = "1",    features = ["full"] }
serde   = { version = "1",    features = ["derive"] }
```

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let todo: Todo = reqwest::get("https://jsonplaceholder.typicode.com/todos/1")
        .await?
        .json()
        .await?;

    println!("{todo:#?}");
    Ok(())
}
```

### 7.6 Simple HTTP Server with `axum`

`axum` is a popular web framework built on top of Tokio and Hyper.

```toml
[dependencies]
axum  = "0.7"
tokio = { version = "1", features = ["full"] }
```

```rust
use axum::{routing::get, Router};

async fn hello() -> &'static str {
    "Hello from axum!\n"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

---

## 8. Error Handling in Network Code

Network operations fail often and in expected ways: connections are refused, timeouts
fire, peers disconnect mid-transfer. Good network code distinguishes _fatal_ errors from
_recoverable_ ones.

```rust
use std::io::{self, ErrorKind};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

async fn read_resilient(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                // Peer closed the connection cleanly
                return Err(io::Error::new(ErrorKind::ConnectionReset, "peer closed"));
            }
            Ok(n) => return Ok(buf[..n].to_vec()),

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                // Retry on EINTR
                continue;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // Shouldn't happen with async, but safe to handle
                tokio::task::yield_now().await;
                continue;
            }
            Err(e) => return Err(e), // Fatal
        }
    }
}
```

For production services the [`anyhow`](https://crates.io/crates/anyhow) or
[`thiserror`](https://crates.io/crates/thiserror) crates provide ergonomic error
propagation:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("connection refused to {addr}")]
    Refused { addr: String },

    #[error("timed out after {secs}s")]
    Timeout { secs: u64 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 9. Testing Network Code

### 9.1 Ephemeral Ports

Binding to port `0` asks the OS for a free ephemeral port — ideal for parallel tests
with no port conflicts.

```rust
#[tokio::test]
async fn test_echo_server() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{TcpListener, TcpStream};

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 64];
        let n = stream.read(&mut buf).await.unwrap();
        stream.write_all(&buf[..n]).await.unwrap();
    });

    // Connect client
    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"ping").await.unwrap();
    let mut resp = vec![0u8; 4];
    client.read_exact(&mut resp).await.unwrap();
    assert_eq!(&resp, b"ping");
}
```

### 9.2 Mocking with `tokio_test`

For unit tests that don't need a real socket, `tokio-test` offers an in-memory
`io::Mock`:

```toml
[dev-dependencies]
tokio-test = "0.4"
```

```rust
#[tokio::test]
async fn test_frame_reader() {
    use tokio::io::AsyncReadExt;

    // Build a mock stream containing a 4-byte length header + payload
    let payload = b"hello";
    let mut data = (payload.len() as u32).to_be_bytes().to_vec();
    data.extend_from_slice(payload);

    let mock = tokio_test::io::Builder::new().read(&data).build();
    let mut stream = tokio_test::io::Builder::new().read(&data).build();

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.unwrap();
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await.unwrap();
    assert_eq!(&body, b"hello");
}
```

---

## 10. Summary and Key Takeaways

Rust's networking ecosystem spans from bare-metal synchronous sockets to fully async,
TLS-secured services with minimal boilerplate. Here is a condensed decision guide:

| Requirement | Recommended approach |
|-------------|----------------------|
| Simple CLI tool, < 100 connections | `std::net` + `thread::spawn` |
| High-concurrency server | `tokio::net` + `tokio::spawn` |
| Encrypted transport | `tokio-rustls` (pure Rust) or `native-tls` |
| HTTP client | `reqwest` |
| HTTP server / REST API | `axum` or `warp` |
| Custom binary protocol | Length-prefixed framing + `tokio_util::codec` |
| Fine-grained socket options | `socket2` crate |
| Connection pooling | `deadpool` or driver-specific pool (e.g., `sqlx`) |

**Core principles to remember:**

- TCP is a byte _stream_ — always add framing to delimit messages.
- UDP is a datagram protocol — each `send` produces exactly one `recv`, but packets can be lost or reordered.
- Prefer `tokio::net` over `std::net` in any async context; mixing blocking I/O with an async runtime stalls the executor thread.
- Always set read/write timeouts (or use `tokio::time::timeout`) to avoid connections that hang indefinitely.
- Use `SO_REUSEPORT` on Linux when running multiple worker processes sharing the same listening port.
- Never skip TLS in production — `rustls` adds negligible overhead and eliminates an entire class of attacks.

---

_This document is part of the Rust deep-dive series. See also:_  
_58 — Concurrency Patterns · 60 — HTTP & REST with Axum_