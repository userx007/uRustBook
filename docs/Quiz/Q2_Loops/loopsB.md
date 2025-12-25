# ✅ **50 Advanced Rust Loop Examples**


* Iterator combinators (`map`, `filter`, `flat_map`, `zip`, `fold`, `take`, `skip`)
* Functional FP-style loops
* Async / Tokio / `StreamExt` loops
* FSM & state-machine loops
* Nested / labelled loops
* Sliding windows, chunks, grouping

---

## **1–10: Advanced Iterators & Functional Style**

### 1. Map + filter

```rust
let doubled_evens: Vec<_> = (0..10)
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .collect();
```

### 2. Flatten nested iterator

```rust
let v = vec![vec![1,2], vec![3,4]];
let flat: Vec<_> = v.into_iter().flatten().collect();
```

### 3. Chain iterators

```rust
let iter = (1..3).chain(10..12);
for x in iter {
    println!("{x}");
}
```

### 4. Peekable iterator

```rust
let mut iter = (1..5).peekable();
while let Some(&x) = iter.peek() {
    println!("{x}");
    iter.next();
}
```

### 5. Take / skip

```rust
for x in (1..100).skip(10).take(5) {
    println!("{x}");
}
```

### 6. Cycle iterator

```rust
for x in [1,2,3].iter().cycle().take(10) {
    println!("{x}");
}
```

### 7. Inspect side effects

```rust
let sum: i32 = (1..5).inspect(|x| println!("x = {x}")).sum();
```

### 8. Enumerate + map

```rust
let v: Vec<_> = (10..15).enumerate().map(|(i,x)| i + x).collect();
```

### 9. Windows of an array

```rust
let arr = [1,2,3,4,5];
for win in arr.windows(3) {
    println!("{win:?}");
}
```

### 10. Chunks iterator

```rust
for chunk in arr.chunks(2) {
    println!("{chunk:?}");
}
```

---

## **11–20: Advanced Vector & Collection Loops**

### 11. Mutable iterator chain

```rust
let mut v = vec![1,2,3];
v.iter_mut().for_each(|x| *x += 10);
```

### 12. Filter_map

```rust
let vals = vec![Some(1), None, Some(3)];
let numbers: Vec<_> = vals.into_iter().filter_map(|x| x).collect();
```

### 13. Collect into HashMap

```rust
let keys = ["a","b","c"];
let values = [1,2,3];
let map: std::collections::HashMap<_,_> = keys.iter().cloned().zip(values.iter().cloned()).collect();
```

### 14. Group by consecutive values

```rust
let data = [1,1,2,2,3];
let mut prev = None;
for &x in &data {
    if prev != Some(x) { println!("group starts: {x}"); }
    prev = Some(x);
}
```

### 15. Sum of squares

```rust
let sum_squares: i32 = (1..=5).map(|x| x*x).sum();
```

### 16. Find first matching

```rust
let first_even = (1..10).find(|x| x % 2 == 0);
```

### 17. Partition by predicate

```rust
let (even, odd): (Vec<_>, Vec<_>) = (1..10).partition(|x| x % 2 == 0);
```

### 18. Any / All

```rust
let all_positive = (1..5).all(|x| x > 0);
let any_even = (1..5).any(|x| x % 2 == 0);
```

### 19. Zip multiple iterators

```rust
let a = [1,2,3];
let b = [4,5,6];
for (x,y) in a.iter().zip(b.iter()) {
    println!("{x} + {y} = {}", x+y);
}
```

### 20. Reduce / fold

```rust
let sum = (1..=5).fold(0, |acc,x| acc+x);
```

---

## **21–30: Async / Tokio / Concurrency Loops**

### 21. Async loop with delay

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    for i in 0..5 {
        println!("{i}");
        sleep(Duration::from_secs(1)).await;
    }
}
```

### 22. Async while-let stream

```rust
use tokio_stream::StreamExt;

let mut stream = tokio_stream::iter(vec![1,2,3]);
while let Some(val) = stream.next().await {
    println!("{val}");
}
```

### 23. Async tasks spawn loop

```rust
use tokio::task;

for i in 0..5 {
    task::spawn(async move {
        println!("Task {i}");
    });
}
```

### 24. Join all async tasks

```rust
use futures::future::join_all;

let tasks: Vec<_> = (0..5).map(|i| async move { i*i }).collect();
let results = join_all(tasks).await;
```

### 25. Periodic polling

```rust
loop {
    check_status().await;
    sleep(Duration::from_secs(5)).await;
}
```

### 26. Async channel receiver

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(5);
while let Some(msg) = rx.recv().await {
    println!("Received: {msg}");
}
```

### 27. Timeout loop

```rust
use tokio::time::{timeout, Duration};

loop {
    if let Ok(_) = timeout(Duration::from_secs(1), async_task()).await {
        break;
    }
}
```

### 28. Select over streams

```rust
use tokio::select;

loop {
    select! {
        Some(msg) = stream1.next() => println!("{msg}"),
        Some(msg) = stream2.next() => println!("{msg}"),
    }
}
```

### 29. Async retry with exponential backoff

```rust
let mut delay = 1;
loop {
    if try_request().await { break; }
    sleep(Duration::from_secs(delay)).await;
    delay *= 2;
}
```

### 30. Concurrent download tasks

```rust
let urls = ["url1","url2"];
let tasks: Vec<_> = urls.iter().map(|url| download(url)).collect();
join_all(tasks).await;
```

---

## **31–40: State Machines & Patterns**

### 31. Enum-based FSM

```rust
enum State { Start, Work, Done }
let mut state = State::Start;
loop {
    state = match state {
        State::Start => State::Work,
        State::Work => State::Done,
        State::Done => break,
    };
}
```

### 32. Event loop with match

```rust
loop {
    match get_event() {
        Event::Quit => break,
        Event::Input(s) => println!("{s}"),
    }
}
```

### 33. Retry until success

```rust
loop {
    if try_operation() { break; }
}
```

### 34. Sliding window in iterator

```rust
let arr = [1,2,3,4,5];
for win in arr.windows(3) {
    println!("{win:?}");
}
```

### 35. Chunked processing

```rust
for chunk in arr.chunks(2) {
    println!("{chunk:?}");
}
```

### 36. Labelled loop to break outer

```rust
'outer: for i in 0..3 {
    for j in 0..3 {
        if j==1 { break 'outer; }
    }
}
```

### 37. Circular buffer index

```rust
let mut idx = 0;
for _ in 0..10 {
    idx = (idx+1)%5;
}
```

### 38. Drain vector

```rust
let mut v = vec![1,2,3];
for x in v.drain(..) { println!("{x}"); }
```

### 39. Zip multiple iterators

```rust
for (a,b) in (1..4).zip([10,20,30]) {
    println!("{a} {b}");
}
```

### 40. Group consecutive duplicates

```rust
let arr = [1,1,2,2,3];
let mut prev = None;
for &x in &arr {
    if prev != Some(x) { println!("new group: {x}"); }
    prev = Some(x);
}
```

---

## **41–50: Misc Advanced Patterns**

### 41. Infinite iterator

```rust
for x in (0..).take(10) { println!("{x}"); }
```

### 42. Iterator folding into struct

```rust
struct Stats { sum:i32, count:i32 }
let stats = (1..6).fold(Stats{sum:0,count:0}, |mut s,x| { s.sum+=x; s.count+=1; s});
```

### 43. Functional factorial

```rust
let fact: u64 = (1..=5).product();
```

### 44. Map + filter + sum

```rust
let result: i32 = (0..10).filter(|x| x%2==0).map(|x| x*x).sum();
```

### 45. Flatten nested vectors and map

```rust
let nested = vec![vec![1,2], vec![3,4]];
let flat: Vec<_> = nested.into_iter().flat_map(|v| v.into_iter()).map(|x| x*2).collect();
```

### 46. Iterator combinators for string parsing

```rust
let nums: Vec<i32> = "1,2,3".split(',').map(|s| s.parse().unwrap()).collect();
```

### 47. Functional sliding sum

```rust
let arr = [1,2,3,4];
let sums: Vec<_> = arr.windows(2).map(|w| w[0]+w[1]).collect();
```

### 48. Recursive functional loop (Fibonacci)

```rust
fn fib(n:u32)->u32 { if n<2 {n} else {fib(n-1)+fib(n-2)} }
for i in 0..10 { println!("{}", fib(i)); }
```

### 49. Lazy iterator chain

```rust
let iter = (0..).filter(|x| x%2==0).map(|x| x*x).take(5);
for x in iter { println!("{x}"); }
```

### 50. Async iterator with timeout

```rust
use tokio::time::{timeout, Duration};
while let Ok(Some(val)) = timeout(Duration::from_secs(1), async_iter.next()).await {
    println!("{val}");
}
```

---

