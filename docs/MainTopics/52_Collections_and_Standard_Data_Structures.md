# Collections & Standard Data Structures

> **Vec · HashMap · BTreeMap · HashSet · VecDeque · BinaryHeap · LinkedList**


# Collections & Standard Data Structures

**Sections covered:**
- **Vec\<T\>** — the default workhorse; creation, indexing, capacity management, retain/dedup/sort
- **HashMap\<K, V\>** — O(1) lookups, the idiomatic `entry()` API, word-count pattern
- **BTreeMap\<K, V\>** — sorted keys, range queries (`Bound::Excluded`/`Included`), first/last key
- **HashSet\<T\>** — set algebra (union, intersection, difference, symmetric difference), subset/superset
- **VecDeque\<T\>** — ring-buffer double-ended queue, rotate, sliding-window algorithm example
- **BinaryHeap\<T\>** — max-heap & min-heap via `Reverse<T>`, full Dijkstra's algorithm example
- **LinkedList\<T\>** — when it's actually justified (O(1) append), and why to avoid it otherwise

**Also includes:**
- A side-by-side **performance comparison table** (Big-O for all operations)
- A **decision tree** for picking the right collection
- Advanced patterns: nested collections, drain/consume, BTreeMap as a sorted multimap
- Custom hashers with the `ahash` crate for speed-critical paths
- A **summary cheat sheet** table for quick reference

---

## 1. Overview

Rust's standard library provides a rich set of collection types in the `std::collections` module. Each is designed for a specific memory layout, access pattern, and performance trade-off. Unlike many languages that default to one "universal" collection, Rust encourages choosing the right tool precisely to achieve predictable performance and clear intent.

> **Why Does Collection Choice Matter?**
> Rust has no garbage collector — every allocation is visible. Choosing the wrong collection (e.g. a `LinkedList` when a `Vec` would do) can cause unnecessary heap fragmentation, cache misses, and harder borrow-checker interactions. Rust's type system makes the trade-offs explicit and measurable.

---

## 2. `Vec<T>` — The Workhorse

`Vec<T>` is a heap-allocated, contiguous, growable array. It is the most commonly used collection in Rust and should be your **default** unless you have a specific reason to use something else.

| Property | Detail |
|---|---|
| Backing store | Single contiguous heap allocation |
| Access | O(1) random access by index |
| Push / Pop (end) | Amortized O(1) |
| Insert / Remove (middle) | O(n) — elements must shift |
| Memory overhead | 3 words: pointer, length, capacity |

### 2.1 Creating a Vec

```rust
// Macro literal
let v: Vec<i32> = vec![1, 2, 3, 4, 5];

// Pre-allocated with capacity to avoid reallocations
let mut v: Vec<String> = Vec::with_capacity(100);

// Collect from an iterator
let squares: Vec<u64> = (1..=10).map(|x| x * x).collect();

// Fill with a repeated value
let zeros = vec![0u8; 256];
```

### 2.2 Common Operations

```rust
let mut v = vec![10, 20, 30];

// Push and pop
v.push(40);               // [10, 20, 30, 40]
let last = v.pop();       // Some(40) — removes & returns last

// Indexing (panics if out of bounds)
let x = v[0];             // 10

// Safe get — returns Option
let maybe = v.get(5);     // None  (no panic)

// Slicing
let slice = &v[1..3];     // &[20, 30]

// Iteration
for item in &v { println!("{}", item); }
for item in &mut v { *item *= 2; }

// Retain only matching elements
v.retain(|&x| x > 10);

// Dedup consecutive duplicates
let mut d = vec![1, 1, 2, 3, 3, 3, 4];
d.dedup();  // [1, 2, 3, 4]

// Sort and sort_by
v.sort();
v.sort_by(|a, b| b.cmp(a));  // descending
```

### 2.3 Capacity Management

A `Vec` grows by doubling its capacity when full, keeping `push` amortized O(1). You can observe and control this explicitly.

```rust
let mut v: Vec<i32> = Vec::new();
println!("{} / {}", v.len(), v.capacity()); // 0 / 0

v.push(1);
println!("{} / {}", v.len(), v.capacity()); // 1 / 4  (platform-dependent)

// Avoid reallocations when size is known
v.reserve(50);   // ensure at least 50 more slots

// Shrink wasted memory
v.shrink_to_fit();

// Truncate without freeing
v.truncate(2);   // keep only first 2 elements
```

---

## 3. `HashMap<K, V>` — Fast Key-Value Lookups

`HashMap<K, V>` stores key-value pairs in a hash table using the **SipHash 1-3** algorithm by default (DoS-resistant). Lookup, insert, and delete are all average O(1). Keys must implement the `Eq` and `Hash` traits.

| Property | Detail |
|---|---|
| Backing store | Hash table with open addressing |
| Lookup / Insert / Delete | Average O(1), worst-case O(n) |
| Ordering | No ordering guarantee — iteration is random |
| Key requirement | `Eq + Hash` traits |
| Default hasher | SipHash 1-3 (secure against hash flooding) |
| Faster hasher option | `ahash`, `fnv` via external crates |

### 3.1 Creating and Populating

```rust
use std::collections::HashMap;

// Empty map
let mut scores: HashMap<String, u32> = HashMap::new();

// Insert
scores.insert(String::from("Alice"), 100);
scores.insert(String::from("Bob"),   85);

// Collect from an iterator of tuples
let pairs = vec![("x", 1), ("y", 2), ("z", 3)];
let map: HashMap<&str, i32> = pairs.into_iter().collect();
```

### 3.2 Reading and Updating

```rust
// Lookup — returns Option<&V>
if let Some(score) = scores.get("Alice") {
    println!("Alice scored {}", score);
}

// contains_key
if scores.contains_key("Bob") { /* ... */ }

// Update: overwrite existing key
scores.insert(String::from("Alice"), 120);

// Insert only if not present
scores.entry(String::from("Charlie")).or_insert(90);

// Modify in-place via entry API
let counter = scores
    .entry(String::from("Dave"))
    .or_insert(0);
*counter += 10;   // Dave now has 10

// Remove
scores.remove("Bob");
```

### 3.3 The Entry API — The Idiomatic Pattern

The `entry()` API avoids double-lookups (one to check existence, one to insert) by returning a mutable handle into the map's slot.

```rust
// Classic word-count idiom
let text = "hello world hello rust world hello";
let mut word_count: HashMap<&str, u32> = HashMap::new();

for word in text.split_whitespace() {
    let count = word_count.entry(word).or_insert(0);
    *count += 1;
}
// { "hello": 3, "world": 2, "rust": 1 }

// Insert computed default
word_count
    .entry("rust")
    .or_insert_with(|| expensive_computation());

// Modify only if key already exists
word_count.entry("hello").and_modify(|v| *v *= 10);
```

---

## 4. `BTreeMap<K, V>` — Sorted Key-Value Store

`BTreeMap<K, V>` is an ordered map backed by a B-Tree. Keys are always iterated in **sorted order**, making it the right choice when you need ordered output, range queries, or the minimum/maximum key at any time.

| Property | Detail |
|---|---|
| Backing store | B-Tree (self-balancing ordered tree) |
| Lookup / Insert / Delete | O(log n) |
| Ordering | Keys always sorted (`Ord` trait required) |
| Key requirement | `Ord` trait (implies `Eq + PartialOrd`) |
| Range query | O(log n + k) where k = result count |
| Memory overhead | Higher than `HashMap` (tree node pointers) |

### 4.1 Creating and Using BTreeMap

```rust
use std::collections::BTreeMap;

let mut scores: BTreeMap<String, u32> = BTreeMap::new();
scores.insert(String::from("Charlie"), 70);
scores.insert(String::from("Alice"),   100);
scores.insert(String::from("Bob"),     85);

// Iteration is ALWAYS sorted by key
for (name, score) in &scores {
    println!("{}: {}", name, score);
}
// Alice: 100
// Bob: 85
// Charlie: 70
```

### 4.2 Range Queries

```rust
use std::ops::Bound::*;

let mut map: BTreeMap<i32, &str> = BTreeMap::new();
for i in 0..10 { map.insert(i, "val"); }

// Iterate over keys 3..=7
for (k, v) in map.range(3..=7) {
    println!("{}: {}", k, v);
}

// Custom bounds
for (k, _) in map.range((Excluded(&2), Included(&8))) {
    print!("{} ", k); // 3 4 5 6 7 8
}

// First / last key
println!("{:?}", map.first_key_value()); // Some((0, "val"))
println!("{:?}", map.last_key_value());  // Some((9, "val"))
```

---

## 5. `HashSet<T>` — Fast Membership Testing

`HashSet<T>` is essentially a `HashMap<T, ()>` — it stores **unique values** with O(1) average insert, remove, and `contains`. It also provides full set-algebra operations.

### 5.1 Core Operations

```rust
use std::collections::HashSet;

let mut set: HashSet<i32> = HashSet::new();
set.insert(1);
set.insert(2);
set.insert(2); // duplicate — silently ignored

println!("{}", set.contains(&1)); // true
println!("{}", set.len());        // 2

set.remove(&1);

// Collect a Vec into a deduplicated HashSet
let items = vec![1, 2, 3, 2, 1, 4];
let unique: HashSet<_> = items.into_iter().collect();
```

### 5.2 Set Algebra

```rust
let a: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
let b: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();

// Union: elements in a OR b
let union: HashSet<_> = a.union(&b).collect();
// {1, 2, 3, 4, 5, 6}

// Intersection: elements in BOTH a AND b
let inter: HashSet<_> = a.intersection(&b).collect();
// {3, 4}

// Difference: in a but NOT in b
let diff: HashSet<_> = a.difference(&b).collect();
// {1, 2}

// Symmetric difference: in one but not both
let sym: HashSet<_> = a.symmetric_difference(&b).collect();
// {1, 2, 5, 6}

// Subset and superset checks
let c: HashSet<i32> = [3, 4].iter().cloned().collect();
println!("{}", c.is_subset(&a));   // true
println!("{}", a.is_superset(&c)); // true
```

---

## 6. `VecDeque<T>` — Double-Ended Queue

`VecDeque<T>` is a **ring-buffer** double-ended queue. It provides O(1) push and pop from both ends, making it ideal for FIFO queues, sliding-window algorithms, and breadth-first search.

| Property | Detail |
|---|---|
| Backing store | Ring buffer (circular contiguous memory) |
| Push/Pop front | O(1) amortized |
| Push/Pop back | O(1) amortized |
| Random access | O(1) — but not always contiguous in memory |
| Convert to Vec | O(1) if already contiguous, else O(n) |

### 6.1 Usage

```rust
use std::collections::VecDeque;

let mut dq: VecDeque<i32> = VecDeque::new();

dq.push_back(10);   // [10]
dq.push_back(20);   // [10, 20]
dq.push_front(5);   // [5, 10, 20]
dq.push_front(1);   // [1, 5, 10, 20]

println!("{:?}", dq.pop_front()); // Some(1)
println!("{:?}", dq.pop_back());  // Some(20)
// dq is now [5, 10]

// Rotate — efficient for ring-buffer logic
dq.rotate_left(1);   // [10, 5]
dq.rotate_right(1);  // [5, 10]
```

### 6.2 Sliding Window Maximum with VecDeque

```rust
fn max_sliding_window(nums: &[i32], k: usize) -> Vec<i32> {
    let mut dq: VecDeque<usize> = VecDeque::new(); // stores indices
    let mut result = Vec::new();

    for i in 0..nums.len() {
        // Remove indices outside the window
        while dq.front().map_or(false, |&f| f + k <= i) {
            dq.pop_front();
        }
        // Maintain decreasing order
        while dq.back().map_or(false, |&b| nums[b] <= nums[i]) {
            dq.pop_back();
        }
        dq.push_back(i);
        if i + 1 >= k {
            result.push(nums[*dq.front().unwrap()]);
        }
    }
    result
}
```

---

## 7. `BinaryHeap<T>` — Priority Queue

`BinaryHeap<T>` is a **max-heap** — the greatest element is always at the top. It provides O(1) peek at the maximum and O(log n) insert and pop. This is the standard priority-queue structure in Rust.

| Property | Detail |
|---|---|
| Peek max | O(1) |
| Push | O(log n) |
| Pop max | O(log n) |
| Ordering | Max-heap: largest value at top |
| For min-heap | Wrap values in `std::cmp::Reverse<T>` |
| Key requirement | `Ord` trait |

### 7.1 Max-Heap and Min-Heap Examples

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

// ── Max-heap (default) ──
let mut max_heap: BinaryHeap<i32> = BinaryHeap::new();
max_heap.push(3);
max_heap.push(1);
max_heap.push(4);
max_heap.push(2);

println!("{:?}", max_heap.peek()); // Some(4)
while let Some(top) = max_heap.pop() {
    print!("{} ", top); // 4 3 2 1  (sorted descending)
}

// ── Min-heap using Reverse ──
let mut min_heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
min_heap.push(Reverse(3));
min_heap.push(Reverse(1));
min_heap.push(Reverse(4));

while let Some(Reverse(top)) = min_heap.pop() {
    print!("{} ", top); // 1 3 4  (sorted ascending)
}
```

### 7.2 Dijkstra's Shortest Path with BinaryHeap

```rust
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

type Graph = HashMap<usize, Vec<(usize, u64)>>; // node -> [(neighbor, cost)]

fn dijkstra(graph: &Graph, start: usize) -> HashMap<usize, u64> {
    let mut dist: HashMap<usize, u64> = HashMap::new();
    // Min-heap: (cost, node)
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();

    heap.push(Reverse((0, start)));
    dist.insert(start, 0);

    while let Some(Reverse((cost, node))) = heap.pop() {
        if cost > *dist.get(&node).unwrap_or(&u64::MAX) { continue; }
        if let Some(neighbors) = graph.get(&node) {
            for &(nbr, weight) in neighbors {
                let next = cost + weight;
                if next < *dist.get(&nbr).unwrap_or(&u64::MAX) {
                    dist.insert(nbr, next);
                    heap.push(Reverse((next, nbr)));
                }
            }
        }
    }
    dist
}
```

---

## 8. `LinkedList<T>` — Rarely the Right Choice

`LinkedList<T>` is a doubly-linked list where each node is a separate heap allocation. Despite its theoretical O(1) front/back operations, real-world performance is usually **worse than `VecDeque`** due to pointer chasing and poor cache locality. The Rust documentation itself recommends avoiding it in most cases.

> **When to Actually Use LinkedList**
> `LinkedList` shines in one niche: O(1) concatenation of two lists (`append`) without copying elements. If you need to frequently merge two large lists, `LinkedList` is justified. Otherwise, prefer `VecDeque` for queue-like patterns or `Vec` for everything else.

```rust
use std::collections::LinkedList;

let mut list: LinkedList<i32> = LinkedList::new();
list.push_back(1);
list.push_back(2);
list.push_front(0);
// [0, 1, 2]

// O(1) append — unique advantage over Vec and VecDeque
let mut other: LinkedList<i32> = LinkedList::from([3, 4, 5]);
list.append(&mut other); // [0, 1, 2, 3, 4, 5], other is now empty

// Front and back
println!("{:?}", list.front()); // Some(0)
println!("{:?}", list.back());  // Some(5)

list.pop_front(); // removes 0
list.pop_back();  // removes 5
```

---

## 9. Side-by-Side Performance Comparison

| Collection | Access | Push/Pop End | Push/Pop Front | Search | Ordered? | Memory |
|---|---|---|---|---|---|---|
| `Vec<T>` | O(1) | O(1)* | O(n) | O(n) | No | Low |
| `VecDeque<T>` | O(1) | O(1)* | O(1)* | O(n) | No | Low |
| `LinkedList<T>` | O(n) | O(1) | O(1) | O(n) | No | High |
| `HashMap<K,V>` | O(1)* | O(1)* | N/A | O(1)* | No | Medium |
| `BTreeMap<K,V>` | O(log n) | O(log n) | N/A | O(log n) | Yes | Medium |
| `HashSet<T>` | N/A | O(1)* | N/A | O(1)* | No | Medium |
| `BinaryHeap<T>` | O(1) peek | O(log n) | N/A | O(n) | Partial | Low |

*\* Amortized. All complexities are average-case unless stated otherwise.*

---

## 10. Choosing the Right Collection

### 10.1 Decision Tree

```
Do you need key-value pairs?
  Yes -> Do you need sorted keys or range queries?
            Yes -> BTreeMap<K, V>
            No  -> HashMap<K, V>  (faster average case)
  No  -> Do you need unique membership only?
            Yes -> HashSet<T>  (or BTreeSet<T> if ordered)
            No  -> Do you need fast operations at both ends?
                      Yes -> VecDeque<T>  (queue / deque)
                      No  -> Do you need max/min priority?
                                Yes -> BinaryHeap<T>
                                No  -> Vec<T>  (default)
```

### 10.2 Practical Guidelines

- **Default to `Vec<T>`.** It is cache-friendly, simple, and covers 80%+ of use cases.
- **Use `HashMap<K, V>`** when you need O(1) key lookups and don't care about order.
- **Upgrade to `BTreeMap<K, V>`** only when sorted iteration, min/max access, or range queries matter.
- **Use `VecDeque<T>` over `LinkedList<T>`** for FIFO queues — same O(1) front ops but contiguous memory.
- **Use `BinaryHeap<T>`** for priority queues, scheduling, and Dijkstra-style algorithms.
- **Wrap in `Reverse<T>`** to turn a max-heap into a min-heap without a custom comparator.
- **`LinkedList<T>` is justified only** for O(1) list concatenation (`append`). Otherwise avoid it.

### 10.3 Custom Hashers — When SipHash is Too Slow

The default SipHash hasher is DoS-resistant but has overhead. If you control all inputs (no user data), you can use faster hashers via the `BuildHasher` trait:

```rust
// In Cargo.toml: ahash = "0.8"
use std::collections::HashMap;
use ahash::AHasher;
use std::hash::BuildHasherDefault;

type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<AHasher>>;

let mut map: FastMap<i32, i32> = FastMap::default();
map.insert(1, 100);

// Or more conveniently with ahash's own API:
let mut fast: ahash::AHashMap<String, i32> = ahash::AHashMap::new();
fast.insert(String::from("key"), 42);
```

---

## 11. Advanced Patterns

### 11.1 Nested Collections

```rust
use std::collections::HashMap;

// Map of name -> list of scores
let mut student_scores: HashMap<String, Vec<u32>> = HashMap::new();

student_scores
    .entry(String::from("Alice"))
    .or_insert_with(Vec::new)
    .push(95);

student_scores
    .entry(String::from("Alice"))
    .or_insert_with(Vec::new)
    .push(88);

// student_scores["Alice"] = [95, 88]
```

### 11.2 Draining and Consuming Collections

```rust
let mut v = vec![1, 2, 3, 4, 5];

// drain() removes a range, yielding an iterator of the removed elements
let removed: Vec<i32> = v.drain(1..3).collect();
// removed = [2, 3],  v = [1, 4, 5]

// into_iter() consumes the Vec
let doubled: Vec<i32> = v.into_iter().map(|x| x * 2).collect();
// [2, 8, 10]

// HashMap::drain()
use std::collections::HashMap;
let mut map: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
for (k, v) in map.drain() {
    println!("{}: {}", k, v);
}
// map is now empty, but still valid
```

### 11.3 BTreeMap for a Sorted Multimap

```rust
use std::collections::BTreeMap;

// Group words by their first character — sorted by letter
let words = ["banana", "apple", "avocado", "cherry", "blueberry"];
let mut grouped: BTreeMap<char, Vec<&str>> = BTreeMap::new();

for word in &words {
    grouped
        .entry(word.chars().next().unwrap())
        .or_insert_with(Vec::new)
        .push(word);
}

for (letter, group) in &grouped {
    println!("'{}': {:?}", letter, group);
}
// 'a': ["apple", "avocado"]
// 'b': ["banana", "blueberry"]
// 'c': ["cherry"]
```

---

## 12. Summary Cheat Sheet

| You want... | Use |
|---|---|
| A resizable array, default storage | `Vec<T>` |
| O(1) push/pop from both ends (queue/deque) | `VecDeque<T>` |
| Fast key-value lookup, unordered | `HashMap<K, V>` |
| Key-value lookup WITH sorted order/ranges | `BTreeMap<K, V>` |
| Unique membership test, unordered | `HashSet<T>` |
| Unique membership test, sorted | `BTreeSet<T>` |
| Always pop the maximum element | `BinaryHeap<T>` |
| Always pop the minimum element | `BinaryHeap<Reverse<T>>` |
| O(1) concatenation of two lists | `LinkedList<T>` |
| Faster hashing (trusted inputs only) | `HashMap` + `ahash` crate |

---

> **The Golden Rule:** Start with `Vec<T>` or `HashMap<K, V>`. Profile before switching. The performance difference between collections only matters at scale — and the right data structure chosen early prevents expensive refactors later.
