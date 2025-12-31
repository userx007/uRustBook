# Channels and Message Passing in Rust

Channels are one of Rust's primary tools for **safe concurrent communication** between threads. They embody the philosophy: *"Do not communicate by sharing memory; instead, share memory by communicating."* This approach, inspired by CSP (Communicating Sequential Processes), helps prevent data races and makes concurrent code more predictable.

## Core Concept

A channel has two halves:
- **Sender (Transmitter)**: Sends messages into the channel
- **Receiver**: Receives messages from the channel

The ownership system ensures messages are moved between threads safely.

---

## 1. **Standard Library `mpsc` Channels**

`mpsc` stands for **Multiple Producer, Single Consumer**. Multiple threads can send messages, but only one thread can receive them.

### Basic Example

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Create a channel
    let (tx, rx) = mpsc::channel();
    
    // Spawn a thread that sends messages
    thread::spawn(move || {
        let messages = vec!["Hello", "from", "another", "thread"];
        
        for msg in messages {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_millis(500));
        }
    });
    
    // Receive messages in the main thread
    for received in rx {
        println!("Got: {}", received);
    }
}
```

### Multiple Producers

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    // Clone the sender for multiple producers
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    
    thread::spawn(move || {
        tx1.send("Message from thread 1").unwrap();
    });
    
    thread::spawn(move || {
        tx2.send("Message from thread 2").unwrap();
    });
    
    // Drop the original sender
    drop(tx);
    
    // Receive all messages
    for msg in rx {
        println!("{}", msg);
    }
}
```

### Bounded vs Unbounded

The standard `mpsc::channel()` is **unbounded** - it can grow indefinitely. For bounded channels, use `sync_channel`:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // Bounded channel with capacity of 2
    let (tx, rx) = mpsc::sync_channel(2);
    
    let tx1 = tx.clone();
    thread::spawn(move || {
        tx1.send(1).unwrap();
        tx1.send(2).unwrap();
        println!("Sent 2 messages");
        
        // This will block until receiver consumes a message
        tx1.send(3).unwrap();
        println!("Sent 3rd message");
    });
    
    thread::sleep(std::time::Duration::from_secs(1));
    
    for msg in rx {
        println!("Received: {}", msg);
        thread::sleep(std::time::Duration::from_millis(500));
    }
}
```

---

## 2. **Crossbeam Channels**

Crossbeam provides more powerful channels with additional features:
- Multiple producers AND multiple consumers (mpmc)
- Better performance
- Select operations (like Go's select)
- Tick and timeout operations

### Basic Crossbeam Example

```rust
use crossbeam_channel::{unbounded, bounded};
use std::thread;

fn main() {
    // Unbounded channel
    let (tx, rx) = unbounded();
    
    thread::spawn(move || {
        tx.send("Hello from crossbeam").unwrap();
    });
    
    println!("{}", rx.recv().unwrap());
    
    // Bounded channel
    let (tx, rx) = bounded(5);
    tx.send(42).unwrap();
    assert_eq!(rx.recv().unwrap(), 42);
}
```

### Multiple Consumers (mpmc)

```rust
use crossbeam_channel::unbounded;
use std::thread;

fn main() {
    let (tx, rx) = unbounded();
    
    // Multiple consumers
    let rx1 = rx.clone();
    let rx2 = rx.clone();
    
    thread::spawn(move || {
        for msg in rx1 {
            println!("Consumer 1 got: {}", msg);
        }
    });
    
    thread::spawn(move || {
        for msg in rx2 {
            println!("Consumer 2 got: {}", msg);
        }
    });
    
    // Producer
    for i in 0..10 {
        tx.send(i).unwrap();
    }
    
    drop(tx); // Close the channel
    thread::sleep(std::time::Duration::from_secs(1));
}
```

### Select Operations

One of crossbeam's most powerful features - wait on multiple channels:

```rust
use crossbeam_channel::{unbounded, select};
use std::thread;
use std::time::Duration;

fn main() {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        tx1.send("fast").unwrap();
    });
    
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        tx2.send("slow").unwrap();
    });
    
    // Wait for whichever message arrives first
    select! {
        recv(rx1) -> msg => println!("Received from rx1: {:?}", msg),
        recv(rx2) -> msg => println!("Received from rx2: {:?}", msg),
    }
}
```

### Timeout and Tick

```rust
use crossbeam_channel::{unbounded, tick, after};
use std::time::Duration;

fn main() {
    let (tx, rx) = unbounded();
    let ticker = tick(Duration::from_millis(100));
    let timeout = after(Duration::from_secs(1));
    
    loop {
        select! {
            recv(rx) -> msg => {
                println!("Message: {:?}", msg);
                break;
            }
            recv(ticker) -> _ => {
                println!("Tick!");
            }
            recv(timeout) -> _ => {
                println!("Timeout - no message received");
                break;
            }
        }
    }
}
```

---

## 3. **CSP Patterns in Rust**

CSP (Communicating Sequential Processes) emphasizes using channels as the primary mechanism for sharing data.

### Worker Pool Pattern

```rust
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;

fn worker(id: usize, rx: Receiver<i32>, results: Sender<i32>) {
    for job in rx {
        println!("Worker {} processing job {}", id, job);
        let result = job * 2; // Simulate work
        results.send(result).unwrap();
    }
}

fn main() {
    let (job_tx, job_rx) = unbounded();
    let (result_tx, result_rx) = unbounded();
    
    // Spawn 3 workers
    for id in 0..3 {
        let rx = job_rx.clone();
        let tx = result_tx.clone();
        thread::spawn(move || worker(id, rx, tx));
    }
    
    drop(job_rx); // Drop the original
    drop(result_tx);
    
    // Send jobs
    for i in 0..9 {
        job_tx.send(i).unwrap();
    }
    drop(job_tx);
    
    // Collect results
    for result in result_rx {
        println!("Result: {}", result);
    }
}
```

### Pipeline Pattern

```rust
use crossbeam_channel::unbounded;
use std::thread;

fn main() {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    let (tx3, rx3) = unbounded();
    
    // Stage 1: Generate numbers
    thread::spawn(move || {
        for i in 1..=5 {
            tx1.send(i).unwrap();
        }
    });
    
    // Stage 2: Square numbers
    thread::spawn(move || {
        for num in rx1 {
            tx2.send(num * num).unwrap();
        }
    });
    
    // Stage 3: Add 1
    thread::spawn(move || {
        for num in rx2 {
            tx3.send(num + 1).unwrap();
        }
    });
    
    // Collect results
    for result in rx3.iter().take(5) {
        println!("{}", result); // Prints: 2, 5, 10, 17, 26
    }
}
```

### Fan-out/Fan-in Pattern

```rust
use crossbeam_channel::unbounded;
use std::thread;

fn main() {
    let (input_tx, input_rx) = unbounded();
    let (output_tx, output_rx) = unbounded();
    
    // Fan-out: Multiple workers process from same input
    for i in 0..3 {
        let rx = input_rx.clone();
        let tx = output_tx.clone();
        
        thread::spawn(move || {
            for msg in rx {
                let result = format!("Worker {} processed: {}", i, msg);
                tx.send(result).unwrap();
            }
        });
    }
    
    drop(input_rx);
    drop(output_tx);
    
    // Send work
    thread::spawn(move || {
        for i in 0..9 {
            input_tx.send(i).unwrap();
        }
    });
    
    // Fan-in: Collect from all workers
    for result in output_rx.iter().take(9) {
        println!("{}", result);
    }
}
```

---

## 4. **Bounded vs Unbounded Queues**

**Unbounded channels:**
- Can grow indefinitely
- Sending never blocks
- Risk of memory exhaustion if producer is faster than consumer

**Bounded channels:**
- Fixed capacity
- Sending blocks when full (backpressure)
- Better for flow control and preventing memory issues

**When to use bounded:**
- Prevent fast producers from overwhelming slow consumers
- Limit memory usage
- Implement backpressure mechanisms

**When to use unbounded:**
- Producer and consumer are well-balanced
- Messages are small and infrequent
- Need maximum throughput

---

## Key Takeaways

1. **Use channels for thread communication** instead of shared memory when possible
2. **Standard `mpsc`** is good for simple cases, **crossbeam** for advanced features
3. **Bounded channels** provide backpressure, **unbounded** offer simplicity
4. **CSP patterns** (pipelines, workers, fan-out/fan-in) create clean concurrent architectures
5. **Select operations** enable responsive multi-channel coordination

Channels in Rust provide a safe, ergonomic way to build concurrent systems that are both performant and free from data races!