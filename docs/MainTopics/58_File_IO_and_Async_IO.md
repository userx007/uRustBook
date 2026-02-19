# File I/O and Async I/O in Rust

**What's inside:**

- **Sync File I/O** — `File::open`, `File::create`, `OpenOptions`, `Seek`, and the `std::fs` convenience functions
- **BufReader / BufWriter** — how buffering reduces syscalls, custom buffer sizes, explicit flushing, and the silent-drop pitfall
- **Path handling** — `Path` vs `PathBuf`, metadata, directory operations and iteration
- **Memory-mapped files** — read-only and mutable maps via `memmap2`, with a concrete counter example and a use-case comparison table
- **Tokio async I/O** — async file read/write, seeking, and convenience functions mirroring `std::fs`
- **`AsyncRead` / `AsyncWrite` traits** — generic async code, `io::split` for bidirectional streams, and a custom `AsyncRead` implementation
- **Async buffering** — `tokio::io::BufReader` line iteration and `BufWriter` flushing
- **Sync vs Async decision guide** — comparison table and the `spawn_blocking` escape hatch
- **Common pitfalls** — forgetting to flush, accidental truncation, blocking inside async tasks, unsafe mmap invariants, and `EINTR` handling


Rust provides a rich, layered I/O system. At the base sits the synchronous standard library (`std::fs`, `std::io`), and on top of that, async runtimes like **Tokio** offer non-blocking file and network I/O. Understanding both layers — and knowing when to use each — is essential for writing performant, correct Rust programs.

---

## Table of Contents

1. [Synchronous File I/O](#1-synchronous-file-io)
2. [BufReader and BufWriter](#2-bufreader-and-bufwriter)
3. [Working with Paths](#3-working-with-paths)
4. [File Metadata and Directory Operations](#4-file-metadata-and-directory-operations)
5. [Memory-Mapped Files](#5-memory-mapped-files)
6. [Async I/O with Tokio](#6-async-io-with-tokio)
7. [Tokio I/O Traits: AsyncRead / AsyncWrite](#7-tokio-io-traits-asyncread--asyncwrite)
8. [Async BufReader and BufWriter](#8-async-bufreader-and-bufwriter)
9. [Sync vs Async: When to Use Which](#9-sync-vs-async-when-to-use-which)
10. [Common Pitfalls](#10-common-pitfalls)

---

## 1. Synchronous File I/O

The entry point for file operations is `std::fs::File`, which implements both `Read` and `Write`.

### Opening and Reading a File

```rust
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut file = File::open("hello.txt")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("File contents:\n{}", contents);
    Ok(())
}
```

### Writing to a File

```rust
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut file = File::create("output.txt")?;
    file.write_all(b"Hello, Rust!\n")?;
    Ok(())
}
```

### OpenOptions — Fine-Grained Control

`File::open` is read-only, `File::create` truncates. Use `OpenOptions` for more control:

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)       // don't truncate, add to end
        .create(true)       // create if it doesn't exist
        .open("log.txt")?;

    writeln!(file, "New log entry")?;
    Ok(())
}
```

### Convenience Functions

`std::fs` exposes helpers that open, read/write, and close in one call:

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    // Read entire file into a String
    let text = fs::read_to_string("hello.txt")?;

    // Read entire file into Vec<u8>
    let bytes = fs::read("image.png")?;

    // Write a slice to a file (creates or truncates)
    fs::write("out.txt", b"data")?;

    println!("Read {} bytes", bytes.len());
    Ok(())
}
```

### Seeking Within a File

`File` implements `Seek`, enabling random access:

```rust
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

fn main() -> io::Result<()> {
    let mut file = File::open("data.bin")?;

    // Jump to byte 100 from the start
    file.seek(SeekFrom::Start(100))?;

    let mut buf = [0u8; 16];
    file.read_exact(&mut buf)?;

    println!("16 bytes at offset 100: {:?}", buf);
    Ok(())
}
```

---

## 2. BufReader and BufWriter

Raw `File` I/O makes a syscall for **every** `read` or `write`. For small, frequent operations this is expensive. `BufReader` and `BufWriter` wrap any `Read`/`Write` implementor with an in-memory buffer, batching syscalls automatically.

### BufReader — Buffered Reading

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let file = File::open("lines.txt")?;
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        println!("{:>4}: {}", i + 1, line);
    }
    Ok(())
}
```

`BufReader::new` uses a default 8 KB buffer. You can customise it:

```rust
let reader = BufReader::with_capacity(64 * 1024, file); // 64 KB buffer
```

### BufWriter — Buffered Writing

```rust
use std::fs::File;
use std::io::{self, BufWriter, Write};

fn main() -> io::Result<()> {
    let file = File::create("output.txt")?;
    let mut writer = BufWriter::new(file);

    for i in 0..10_000 {
        writeln!(writer, "line {}", i)?;
    }
    // BufWriter flushes on drop, but it swallows errors then.
    // Explicit flush lets you handle the error:
    writer.flush()?;
    Ok(())
}
```

> **Important:** `BufWriter` flushes its buffer when dropped, but any I/O error during that implicit flush is **silently ignored**. Always call `flush()` explicitly when correctness matters.

### Reading Word by Word with `split`

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let reader = BufReader::new(File::open("words.txt")?);

    // split on space character (0x20)
    for word in reader.split(b' ') {
        let word = word?;
        println!("{}", String::from_utf8_lossy(&word));
    }
    Ok(())
}
```

### Choosing Buffer Size

| Scenario | Recommended buffer |
|---|---|
| Line-by-line text parsing | Default 8 KB |
| Large binary file transfers | 64 KB – 1 MB |
| Network streams (loopback) | 16–32 KB |
| Network streams (WAN) | Match MTU or TCP window |

---

## 3. Working with Paths

Rust separates **path manipulation** (`std::path`) from **filesystem operations** (`std::fs`).

```rust
use std::path::{Path, PathBuf};

fn main() {
    // PathBuf is an owned, mutable path
    let mut path = PathBuf::from("/home/user");
    path.push("documents");
    path.push("report.pdf");

    println!("Full path : {}", path.display());
    println!("File name : {:?}", path.file_name());
    println!("Extension : {:?}", path.extension());
    println!("Parent    : {:?}", path.parent());

    // Path is a borrowed slice (like &str vs String)
    let p: &Path = path.as_path();
    println!("Exists? {}", p.exists());
}
```

---

## 4. File Metadata and Directory Operations

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    // Metadata
    let meta = fs::metadata("hello.txt")?;
    println!("Size    : {} bytes", meta.len());
    println!("Is file : {}", meta.is_file());
    println!("Is dir  : {}", meta.is_dir());

    // Create directories (like mkdir -p)
    fs::create_dir_all("data/cache/tmp")?;

    // Rename / move
    fs::rename("old_name.txt", "new_name.txt")?;

    // Copy
    let bytes_copied = fs::copy("src.txt", "dst.txt")?;
    println!("Copied {} bytes", bytes_copied);

    // Remove
    fs::remove_file("dst.txt")?;
    fs::remove_dir_all("data/cache")?;

    Ok(())
}
```

### Iterating a Directory

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path  = entry.path();
        let meta  = entry.metadata()?;
        println!("{:<40} {:>10} bytes", path.display(), meta.len());
    }
    Ok(())
}
```

---

## 5. Memory-Mapped Files

For very large files — or when you need random access without loading the file into memory — a **memory map** lets the OS manage paging automatically. The [`memmap2`](https://crates.io/crates/memmap2) crate is the standard choice.

```toml
# Cargo.toml
[dependencies]
memmap2 = "0.9"
```

### Read-Only Memory Map

```rust
use memmap2::Mmap;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let file = File::open("large_dataset.bin")?;

    // SAFETY: the file must not be modified externally while mapped.
    let mmap = unsafe { Mmap::map(&file)? };

    println!("Mapped {} bytes", mmap.len());

    // Access the file as a plain byte slice — no explicit read() calls.
    let first_16 = &mmap[..16.min(mmap.len())];
    println!("First bytes: {:?}", first_16);

    Ok(())
}
```

### Mutable Memory Map

```rust
use memmap2::MmapMut;
use std::fs::OpenOptions;

fn main() -> std::io::Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("counter.bin")?;

    let mut mmap = unsafe { MmapMut::map_mut(&file)? };

    // Interpret the first 8 bytes as a little-endian u64 counter
    let counter = u64::from_le_bytes(mmap[..8].try_into().unwrap());
    let new_val = counter + 1;
    mmap[..8].copy_from_slice(&new_val.to_le_bytes());

    // Flush OS page cache → disk
    mmap.flush()?;
    println!("Counter updated to {}", new_val);
    Ok(())
}
```

### When to Use Memory Maps

| ✅ Good fit | ❌ Poor fit |
|---|---|
| Random access into huge files | Small files (overhead not worth it) |
| Parsing large binary/structured data | Files that grow frequently |
| Inter-process shared memory | Highly concurrent writes (no built-in sync) |
| Search indexes, databases | Files on network filesystems (NFS) |

---

## 6. Async I/O with Tokio

Synchronous I/O blocks the calling thread while waiting for the OS. In an async context (especially a server handling thousands of connections), blocking threads kills concurrency. Tokio provides a non-blocking I/O layer built on `io_uring` (Linux) or equivalent platform APIs.

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### Async File Reading

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::open("hello.txt").await?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    println!("{}", contents);
    Ok(())
}
```

### Async File Writing

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("output.txt").await?;
    file.write_all(b"async hello!\n").await?;
    file.flush().await?;
    Ok(())
}
```

### Async Convenience Functions

Tokio mirrors `std::fs` with async equivalents:

```rust
use tokio::fs;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let text  = fs::read_to_string("notes.txt").await?;
    let bytes = fs::read("image.png").await?;
    fs::write("out.txt", b"data").await?;
    fs::remove_file("out.txt").await?;
    println!("Read {} chars, {} bytes", text.len(), bytes.len());
    Ok(())
}
```

### Async Seeking

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut file = File::open("data.bin").await?;
    file.seek(SeekFrom::Start(512)).await?;

    let mut buf = vec![0u8; 128];
    file.read_exact(&mut buf).await?;
    println!("Read {} bytes from offset 512", buf.len());
    Ok(())
}
```

---

## 7. Tokio I/O Traits: AsyncRead / AsyncWrite

Tokio's async I/O revolves around two core traits from `tokio::io`:

- **`AsyncRead`** — provides `poll_read`, extended by the `AsyncReadExt` helper trait
- **`AsyncWrite`** — provides `poll_write` / `poll_flush` / `poll_shutdown`, extended by `AsyncWriteExt`

These traits enable **generic async I/O code** that works with files, TCP sockets, TLS streams, in-memory buffers, and more.

### Generic Async Copy

```rust
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};

async fn copy_data<R, W>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    io::copy(reader, writer).await
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut source = tokio::fs::File::open("input.txt").await?;
    let mut dest   = tokio::fs::File::create("copy.txt").await?;

    let bytes = copy_data(&mut source, &mut dest).await?;
    dest.flush().await?;
    println!("Copied {} bytes", bytes);
    Ok(())
}
```

### Using `tokio::io::split` for Bidirectional I/O

When a single type implements both `AsyncRead` and `AsyncWrite` (e.g., a TCP stream), you can split it into independent halves:

```rust
use tokio::net::TcpStream;
use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (mut reader, mut writer) = io::split(stream);

    // writer and reader can now be used independently,
    // even passed to different tasks.
    writer.write_all(b"ping\n").await?;

    let mut buf = vec![0u8; 64];
    let n = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await?;
    println!("Got: {}", String::from_utf8_lossy(&buf[..n]));
    Ok(())
}
```

### Implementing `AsyncRead` on a Custom Type

```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

/// A simple source that yields a repeating byte pattern
struct PatternReader {
    pattern: Vec<u8>,
    pos: usize,
    total_limit: usize,
    emitted: usize,
}

impl PatternReader {
    fn new(pattern: Vec<u8>, limit: usize) -> Self {
        Self { pattern, pos: 0, total_limit: limit, emitted: 0 }
    }
}

impl AsyncRead for PatternReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.emitted >= self.total_limit {
            return Poll::Ready(Ok(())); // EOF
        }
        let remaining = self.total_limit - self.emitted;
        let to_fill   = buf.remaining().min(remaining).min(self.pattern.len());

        for i in 0..to_fill {
            buf.put_slice(&[self.pattern[self.pos]]);
            self.pos = (self.pos + 1) % self.pattern.len();
        }
        self.emitted += to_fill;
        Poll::Ready(Ok(()))
    }
}
```

---

## 8. Async BufReader and BufWriter

Just as in sync code, buffering async I/O dramatically reduces system call overhead.

### Async BufReader — Line-by-Line

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    let file   = File::open("lines.txt").await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
    }
    Ok(())
}
```

### Async BufWriter

```rust
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt, BufWriter};

#[tokio::main]
async fn main() -> io::Result<()> {
    let file   = File::create("output.txt").await?;
    let mut writer = BufWriter::new(file);

    for i in 0..10_000u32 {
        let line = format!("line {}\n", i);
        writer.write_all(line.as_bytes()).await?;
    }
    writer.flush().await?;
    Ok(())
}
```

### Async `read_until` and `read_line`

```rust
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let file   = File::open("data.csv").await?;
    let mut reader = BufReader::new(file);

    let mut line = String::new();
    // Read exactly one line (including the '\n')
    let bytes_read = reader.read_line(&mut line).await?;
    println!("Header ({} bytes): {}", bytes_read, line.trim());
    Ok(())
}
```

---

## 9. Sync vs Async: When to Use Which

| Criterion | Synchronous (`std::fs`) | Asynchronous (Tokio) |
|---|---|---|
| **Simplicity** | ✅ Straightforward, no runtime | ❌ Requires async runtime |
| **CLI / scripts** | ✅ Ideal | ❌ Overkill |
| **High-concurrency servers** | ❌ Blocks threads | ✅ Scales to thousands of tasks |
| **One-shot file processing** | ✅ Fine | ❌ Adds complexity for no gain |
| **Interleaving I/O + network** | ❌ Each blocks independently | ✅ Multiplex easily |
| **Startup / config reading** | ✅ Simple and fast | ❌ Unnecessary |
| **Streaming large uploads/downloads** | ⚠️ Needs threads | ✅ Backpressure-aware |

### The `spawn_blocking` Escape Hatch

When you must call a synchronous, potentially blocking API from an async context, Tokio provides `spawn_blocking` to run it on a dedicated thread pool without blocking the async executor:

```rust
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run a blocking operation off the async executor
    let result = task::spawn_blocking(|| {
        // Expensive or blocking sync work here
        std::fs::read_to_string("big_file.txt")
    })
    .await??;

    println!("Read {} chars", result.len());
    Ok(())
}
```

> **Rule of thumb:** In an async program, **never** call blocking I/O directly. Use `spawn_blocking` or Tokio's async I/O wrappers.

---

## 10. Common Pitfalls

### 1. Forgetting to Flush BufWriter

```rust
// ❌ BufWriter dropped silently — flush error lost
{
    let mut w = BufWriter::new(File::create("out.txt")?);
    w.write_all(b"data")?;
} // implicit flush here, error swallowed!

// ✅ Explicit flush
let mut w = BufWriter::new(File::create("out.txt")?);
w.write_all(b"data")?;
w.flush()?; // handle the error properly
```

### 2. Opening a File for Writing When You Meant Append

```rust
// ❌ File::create truncates to zero — existing data is lost!
let file = File::create("log.txt")?;

// ✅ Use OpenOptions to append
let file = OpenOptions::new().append(true).create(true).open("log.txt")?;
```

### 3. Calling Blocking I/O Inside an Async Task

```rust
// ❌ Blocks the Tokio thread — hurts all other tasks on the runtime
async fn bad() {
    let _ = std::fs::read_to_string("file.txt"); // blocking!
}

// ✅ Use tokio::fs or spawn_blocking
async fn good() -> tokio::io::Result<String> {
    tokio::fs::read_to_string("file.txt").await
}
```

### 4. Holding a Memory Map While the File Is Externally Modified

```rust
// Undefined behaviour if another process truncates the file while mapped.
// Always ensure exclusive access or use MAP_PRIVATE semantics.
let mmap = unsafe { Mmap::map(&file)? }; // SAFETY comment must document invariants
```

### 5. Not Handling `ErrorKind::Interrupted`

Low-level `read` can return `EINTR`. `std::io::Read::read_to_end` handles this, but manual `read` loops must retry:

```rust
use std::io::{self, Read};

fn read_all(reader: &mut impl Read, buf: &mut Vec<u8>) -> io::Result<()> {
    loop {
        match reader.read_to_end(buf) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}
```

---

## Summary

Rust's I/O story is **layered and composable**:

- `std::fs::File` + `std::io::{Read, Write, Seek}` — raw synchronous primitives
- `BufReader` / `BufWriter` — batch syscalls with an in-memory buffer
- `memmap2` — map large files directly into the process address space
- `tokio::fs` + `AsyncRead` / `AsyncWrite` — non-blocking async I/O for concurrent programs
- `tokio::io::BufReader` / `BufWriter` — async-aware buffering
- `spawn_blocking` — bridge between sync and async worlds

Choose the sync API for scripts, CLIs, and startup logic. Reach for async when building servers, proxies, or any program that interleaves I/O from many sources at once.