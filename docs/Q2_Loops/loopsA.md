Here are **50 fully working, real-world Rust loop examples**, grouped by purpose.

---

# ✅ **50 Real-World Rust Loop Examples**

---

# 1–10: Basic Counting & Iteration

---

### **1. Count up**

```rust
for i in 0..10 {
    println!("{i}");
}
```

### **2. Count down**

```rust
for i in (1..=10).rev() {
    println!("{i}");
}
```

### **3. Sum numbers**

```rust
let sum: i32 = (1..=100).sum();
```

### **4. Product of numbers**

```rust
let product: i32 = (1..=5).product();
```

### **5. Find max in array**

```rust
let arr = [4, 2, 9, 1];
let mut max = arr[0];

for &x in &arr {
    if x > max {
        max = x;
    }
}
```

### **6. Basic while**

```rust
let mut n = 5;
while n > 0 {
    n -= 1;
}
```

### **7. Infinite loop with break**

```rust
loop {
    println!("tick");
    break;
}
```

### **8. Loop with returned value**

```rust
let result = loop {
    break 42;
};
```

### **9. Loop with continue**

```rust
for i in 0..10 {
    if i % 2 == 0 {
        continue;
    }
    println!("{i}");
}
```

### **10. Iterating a string**

```rust
for c in "hello".chars() {
    println!("{c}");
}
```

---

# 11–20: Vectors, Iterators, Collections

---

### **11. Iterate vector**

```rust
let v = vec![10, 20, 30];
for val in &v {
    println!("{val}");
}
```

### **12. Mutate vector**

```rust
let mut v = vec![1, 2, 3];
for x in &mut v {
    *x += 10;
}
```

### **13. Consume vector**

```rust
for val in vec![1, 2, 3] {
    println!("{val}");
}
```

### **14. Enumerate (index + value)**

```rust
for (i, v) in v.iter().enumerate() {
    println!("{i} → {v}");
}
```

### **15. Filtered iteration**

```rust
for x in (0..20).filter(|x| x % 3 == 0) {
    println!("{x}");
}
```

### **16. Map transformation**

```rust
let doubled: Vec<_> = (0..5).map(|x| x * 2).collect();
```

### **17. Reading values until None**

```rust
let mut opt = Some(0);

while let Some(v) = opt {
    if v == 3 { opt = None; }
    else { opt = Some(v + 1); }
}
```

### **18. Loop over HashMap**

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("a", 1);
map.insert("b", 2);

for (key, val) in &map {
    println!("{key}: {val}");
}
```

### **19. Loop until iterator is empty**

```rust
let mut iter = (1..5);
while let Some(x) = iter.next() {
    println!("{x}");
}
```

### **20. Nested loops**

```rust
for i in 0..3 {
    for j in 0..3 {
        println!("{i},{j}");
    }
}
```

---

# 21–30: Algorithms & Data Processing

---

### **21. Bubble sort outer loop**

```rust
let mut arr = [5, 2, 3, 1];

for i in 0..arr.len() {
    for j in 0..arr.len() - 1 {
        if arr[j] > arr[j + 1] {
            arr.swap(j, j + 1);
        }
    }
}
```

### **22. Deduplicate sorted list**

```rust
let mut last = None;
for x in vec![1,1,2,3,3,3] {
    if Some(x) != last {
        println!("{x}");
    }
    last = Some(x);
}
```

### **23. Accumulate average**

```rust
let scores = [50, 70, 90];
let mut total = 0;
let mut count = 0;

for s in scores {
    total += s;
    count += 1;
}

let avg = total as f32 / count as f32;
```

### **24. Find first match**

```rust
let nums = [2, 4, 7, 8];
let idx = nums.iter().position(|&x| x % 2 == 1);
```

### **25. Reverse vector manually**

```rust
let mut v = vec![1,2,3,4];
let mut reversed = vec![];

for x in v.into_iter().rev() {
    reversed.push(x);
}
```

### **26. Count words**

```rust
let text = "hello world from rust";
let mut count = 0;

for _ in text.split_whitespace() {
    count += 1;
}
```

### **27. Fibonacci (loop)**

```rust
let (mut a, mut b) = (0, 1);

for _ in 0..10 {
    let next = a + b;
    a = b;
    b = next;
}
```

### **28. Searching for substring**

```rust
let s = "hello world";
for (i, _) in s.match_indices("o") {
    println!("{i}");
}
```

### **29. Counting frequencies**

```rust
use std::collections::HashMap;

let mut freq = HashMap::new();

for c in "abracadabra".chars() {
    *freq.entry(c).or_insert(0) += 1;
}
```

### **30. Processing log lines**

```rust
for line in log_text.lines() {
    if line.contains("ERROR") {
        println!("Found: {line}");
    }
}
```

---

# 31–40: IO, Files, Networking

---

### **31. Read file line-by-line**

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

let file = File::open("data.txt")?;
for line in BufReader::new(file).lines() {
    println!("{}", line?);
}
```

### **32. Read stdin until EOF**

```rust
use std::io::{self, BufRead};

for line in io::stdin().lock().lines() {
    println!("{}", line?);
}
```

### **33. Polling loop**

```rust
loop {
    if check_status() {
        break;
    }
}
```

### **34. Retry with exponential backoff**

```rust
let mut delay = 1;

loop {
    if try_request() {
        break;
    }

    std::thread::sleep(std::time::Duration::from_secs(delay));
    delay *= 2;
}
```

### **35. Receive messages from a channel**

```rust
while let Ok(msg) = rx.recv() {
    println!("message: {msg}");
}
```

### **36. Timer loop**

```rust
for _ in 0..5 {
    println!("tick");
    std::thread::sleep(std::time::Duration::from_secs(1));
}
```

### **37. Network listener accept loop**

```rust
use std::net::TcpListener;
let listener = TcpListener::bind("127.0.0.1:8000")?;

for stream in listener.incoming() {
    handle(stream?);
}
```

### **38. Read bytes until zero**

```rust
for &byte in buffer {
    if byte == 0 { break; }
    println!("{byte}");
}
```

### **39. Scan directory entries**

```rust
for entry in std::fs::read_dir(".")? {
    println!("{:?}", entry?);
}
```

### **40. JSON array iteration**

```rust
let json = serde_json::json!([1,2,3]);
for v in json.as_array().unwrap() {
    println!("{v}");
}
```

---

# 41–50: Advanced Patterns & State Machines

---

### **41. Labelled loop breaking outer loop**

```rust
'outer: for i in 0..3 {
    for j in 0..3 {
        if j == 1 {
            break 'outer;
        }
    }
}
```

### **42. State machine**

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

### **43. Simple event loop**

```rust
loop {
    match get_event() {
        Event::Quit => break,
        Event::Input(s) => println!("Input: {s}"),
    }
}
```

### **44. Circular buffer index**

```rust
let mut idx = 0;
for _ in 0..20 {
    idx = (idx + 1) % 5;
}
```

### **45. Drain vector**

```rust
let mut v = vec![1,2,3];
for x in v.drain(..) {
    println!("{x}");
}
```

### **46. Zipping two iterators**

```rust
for (a, b) in (1..5).zip(['a', 'b', 'c', 'd']) {
    println!("{a} {b}");
}
```

### **47. Group consecutive identical items**

```rust
let data = [1,1,2,3,3,3];
let mut prev = None;

for &x in &data {
    if prev != Some(x) {
        println!("group starts: {x}");
    }
    prev = Some(x);
}
```

### **48. Sliding window**

```rust
let arr = [1, 2, 3, 4, 5];

for win in arr.windows(3) {
    println!("{win:?}");
}
```

### **49. Chunk processing**

```rust
for chunk in arr.chunks(2) {
    println!("{chunk:?}");
}
```

### **50. Async stream loop (Tokio)**

```rust
use tokio_stream::StreamExt;

let mut stream = tokio_stream::iter(vec![1,2,3]);

while let Some(value) = stream.next().await {
    println!("{value}");
}
```

---

