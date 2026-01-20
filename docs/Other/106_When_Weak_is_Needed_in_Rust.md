# When `Weak<T>` is Needed in Rust

## Overview

`Weak<T>` provides a **non-owning reference** to data managed by `Rc<T>` or `Arc<T>`. It doesn't keep the data alive and must be upgraded to a strong reference before use. The primary purpose is **breaking reference cycles** and **optional caching**.

## 1. **Breaking Reference Cycles (Parent-Child Relationships)**

The fundamental use case: preventing memory leaks in bidirectional relationships.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,      // Weak prevents cycle
    children: RefCell<Vec<Rc<Node>>>,  // Strong ownership of children
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }
    
    fn add_child(parent: &Rc<Node>, child_value: i32) -> Rc<Node> {
        let child = Node::new(child_value);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
    
    fn get_parent(&self) -> Option<Rc<Node>> {
        self.parent.borrow().upgrade()  // Convert Weak to Rc
    }
}

fn main() {
    let parent = Node::new(1);
    let child = Node::add_child(&parent, 2);
    
    if let Some(p) = child.get_parent() {
        println!("Child's parent value: {}", p.value);
    }
    
    // When parent is dropped, child's Weak reference becomes invalid
    drop(parent);
    assert!(child.get_parent().is_none());
}
```

## 2. **Tree Structures with Parent Pointers**

Trees where children need optional access to their parent.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TreeNode {
    value: String,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(value: String) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }
    
    fn insert_child(parent: &Rc<TreeNode>, value: String) -> Rc<TreeNode> {
        let child = TreeNode::new(value);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
    
    fn get_path_to_root(&self) -> Vec<String> {
        let mut path = vec![self.value.clone()];
        let mut current = self.parent.borrow().upgrade();
        
        while let Some(node) = current {
            path.push(node.value.clone());
            current = node.parent.borrow().upgrade();
        }
        
        path.reverse();
        path
    }
}

fn main() {
    let root = TreeNode::new("root".to_string());
    let child1 = TreeNode::insert_child(&root, "child1".to_string());
    let grandchild = TreeNode::insert_child(&child1, "grandchild".to_string());
    
    println!("Path: {:?}", grandchild.get_path_to_root());
    // Output: ["root", "child1", "grandchild"]
}
```

## 3. **Doubly-Linked Lists**

Preventing cycles in bidirectional linked structures.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct ListNode {
    value: i32,
    next: RefCell<Option<Rc<ListNode>>>,
    prev: RefCell<Weak<ListNode>>,  // Weak to prevent cycle
}

impl ListNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(ListNode {
            value,
            next: RefCell::new(None),
            prev: RefCell::new(Weak::new()),
        })
    }
    
    fn append(node: &Rc<ListNode>, value: i32) -> Rc<ListNode> {
        let new_node = ListNode::new(value);
        *new_node.prev.borrow_mut() = Rc::downgrade(node);
        *node.next.borrow_mut() = Some(Rc::clone(&new_node));
        new_node
    }
    
    fn traverse_forward(&self) {
        print!("{}", self.value);
        if let Some(ref next) = *self.next.borrow() {
            print!(" -> ");
            next.traverse_forward();
        } else {
            println!();
        }
    }
    
    fn traverse_backward(&self) {
        print!("{}", self.value);
        if let Some(prev) = self.prev.borrow().upgrade() {
            print!(" <- ");
            prev.traverse_backward();
        } else {
            println!();
        }
    }
}

fn main() {
    let node1 = ListNode::new(1);
    let node2 = ListNode::append(&node1, 2);
    let node3 = ListNode::append(&node2, 3);
    
    println!("Forward:");
    node1.traverse_forward();
    
    println!("Backward:");
    node3.traverse_backward();
}
```

## 4. **Observer Pattern (Avoiding Memory Leaks)**

Observers that shouldn't keep the subject alive.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn update(&self, message: &str);
}

struct ConcreteObserver {
    id: String,
}

impl Observer for ConcreteObserver {
    fn update(&self, message: &str) {
        println!("Observer {} received: {}", self.id, message);
    }
}

struct Subject {
    // Weak references don't keep observers alive
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: RefCell::new(vec![]),
        }
    }
    
    fn attach(&self, observer: &Rc<dyn Observer>) {
        self.observers.borrow_mut().push(Rc::downgrade(observer));
    }
    
    fn notify(&self, message: &str) {
        // Clean up dead observers and notify live ones
        self.observers.borrow_mut().retain(|weak_obs| {
            if let Some(observer) = weak_obs.upgrade() {
                observer.update(message);
                true  // Keep this observer
            } else {
                false  // Remove dead observer
            }
        });
    }
}

fn main() {
    let subject = Subject::new();
    
    {
        let observer1 = Rc::new(ConcreteObserver {
            id: "Observer1".to_string(),
        });
        let observer2 = Rc::new(ConcreteObserver {
            id: "Observer2".to_string(),
        });
        
        subject.attach(&observer1);
        subject.attach(&observer2);
        
        subject.notify("First message");
        // Both observers receive the message
    }
    // observer1 and observer2 dropped here
    
    subject.notify("Second message");
    // No observers left, none receive message
}
```

## 5. **Caching Without Preventing Cleanup**

Caches that hold references but allow items to be freed when unused elsewhere.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

struct ExpensiveResource {
    id: String,
    data: Vec<u8>,
}

impl ExpensiveResource {
    fn new(id: String) -> Self {
        println!("Creating expensive resource: {}", id);
        ExpensiveResource {
            id,
            data: vec![0; 1_000_000],
        }
    }
}

struct Cache {
    // Weak references allow resources to be freed
    resources: RefCell<HashMap<String, Weak<ExpensiveResource>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            resources: RefCell::new(HashMap::new()),
        }
    }
    
    fn get_or_create(&self, id: &str) -> Rc<ExpensiveResource> {
        // Try to upgrade existing weak reference
        if let Some(weak) = self.resources.borrow().get(id) {
            if let Some(resource) = weak.upgrade() {
                println!("Cache hit: {}", id);
                return resource;
            }
        }
        
        // Create new resource
        println!("Cache miss: {}", id);
        let resource = Rc::new(ExpensiveResource::new(id.to_string()));
        self.resources
            .borrow_mut()
            .insert(id.to_string(), Rc::downgrade(&resource));
        resource
    }
    
    fn cleanup_dead_entries(&self) {
        self.resources.borrow_mut().retain(|_, weak| {
            weak.upgrade().is_some()
        });
    }
}

fn main() {
    let cache = Cache::new();
    
    {
        let res1 = cache.get_or_create("resource1");
        let res2 = cache.get_or_create("resource1");  // Cache hit
        
        println!("Strong count: {}", Rc::strong_count(&res1));  // 2
    }
    // res1 and res2 dropped, resource can be freed
    
    cache.cleanup_dead_entries();
    
    let res3 = cache.get_or_create("resource1");  // Cache miss, recreated
}
```

## 6. **Multi-threaded Observer Pattern (Arc + Weak)**

Thread-safe observers using `Arc` and `Weak`.

```rust
use std::sync::{Arc, Weak, Mutex};
use std::thread;

trait Observer: Send + Sync {
    fn notify(&self, event: &str);
}

struct EventManager {
    observers: Mutex<Vec<Weak<dyn Observer>>>,
}

impl EventManager {
    fn new() -> Self {
        EventManager {
            observers: Mutex::new(vec![]),
        }
    }
    
    fn subscribe(&self, observer: &Arc<dyn Observer>) {
        self.observers.lock().unwrap().push(Arc::downgrade(observer));
    }
    
    fn emit(&self, event: &str) {
        let mut observers = self.observers.lock().unwrap();
        observers.retain(|weak| {
            if let Some(observer) = weak.upgrade() {
                observer.notify(event);
                true
            } else {
                false  // Remove dead observer
            }
        });
    }
}

struct Logger {
    name: String,
}

impl Observer for Logger {
    fn notify(&self, event: &str) {
        println!("{} received: {}", self.name, event);
    }
}

fn main() {
    let manager = Arc::new(EventManager::new());
    
    {
        let logger1 = Arc::new(Logger {
            name: "Logger1".to_string(),
        });
        
        manager.subscribe(&logger1);
        
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            manager_clone.emit("Event 1");
        });
        
        handle.join().unwrap();
    }
    // logger1 dropped
    
    manager.emit("Event 2");  // No observers left
}
```

## 7. **Self-Referential Structures (Advanced)**

Structures that need to reference themselves without creating cycles.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Graph {
    nodes: Vec<Rc<GraphNode>>,
}

struct GraphNode {
    id: usize,
    // Weak references to avoid cycles
    neighbors: RefCell<Vec<Weak<GraphNode>>>,
}

impl Graph {
    fn new() -> Self {
        Graph { nodes: vec![] }
    }
    
    fn add_node(&mut self, id: usize) -> Rc<GraphNode> {
        let node = Rc::new(GraphNode {
            id,
            neighbors: RefCell::new(vec![]),
        });
        self.nodes.push(Rc::clone(&node));
        node
    }
    
    fn add_edge(&self, from: &Rc<GraphNode>, to: &Rc<GraphNode>) {
        from.neighbors.borrow_mut().push(Rc::downgrade(to));
    }
    
    fn print_neighbors(&self, node: &GraphNode) {
        print!("Node {} neighbors: ", node.id);
        for weak_neighbor in node.neighbors.borrow().iter() {
            if let Some(neighbor) = weak_neighbor.upgrade() {
                print!("{} ", neighbor.id);
            }
        }
        println!();
    }
}

fn main() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(1);
    let node2 = graph.add_node(2);
    let node3 = graph.add_node(3);
    
    graph.add_edge(&node1, &node2);
    graph.add_edge(&node1, &node3);
    graph.add_edge(&node2, &node1);  // Cycle, but Weak prevents leak
    
    graph.print_neighbors(&node1);
    graph.print_neighbors(&node2);
}
```

## 8. **Temporary References to Shared State**

When you need to reference shared state but shouldn't keep it alive.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct GameState {
    score: i32,
    player_name: String,
}

struct UI {
    // UI doesn't own the game state, just references it
    game_state: Weak<RefCell<GameState>>,
}

impl UI {
    fn new(state: &Rc<RefCell<GameState>>) -> Self {
        UI {
            game_state: Rc::downgrade(state),
        }
    }
    
    fn render(&self) {
        if let Some(state) = self.game_state.upgrade() {
            let state = state.borrow();
            println!("Player: {}, Score: {}", state.player_name, state.score);
        } else {
            println!("Game state no longer available");
        }
    }
}

fn main() {
    let ui = {
        let game_state = Rc::new(RefCell::new(GameState {
            score: 100,
            player_name: "Player1".to_string(),
        }));
        
        let ui = UI::new(&game_state);
        ui.render();  // Works
        
        ui
    };  // game_state dropped here
    
    ui.render();  // Prints "Game state no longer available"
}
```

## 9. **Weak References in Data Structures for Garbage Collection**

Allowing automatic cleanup when strong references are gone.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Registry {
    items: RefCell<Vec<Weak<Item>>>,
}

struct Item {
    id: usize,
    data: String,
}

impl Registry {
    fn new() -> Self {
        Registry {
            items: RefCell::new(vec![]),
        }
    }
    
    fn register(&self, item: &Rc<Item>) {
        self.items.borrow_mut().push(Rc::downgrade(item));
    }
    
    fn list_active_items(&self) {
        let mut items = self.items.borrow_mut();
        
        // Remove dead items and list active ones
        items.retain(|weak_item| {
            if let Some(item) = weak_item.upgrade() {
                println!("Active item {}: {}", item.id, item.data);
                true
            } else {
                println!("Item was cleaned up");
                false
            }
        });
    }
    
    fn active_count(&self) -> usize {
        self.items
            .borrow()
            .iter()
            .filter(|weak| weak.upgrade().is_some())
            .count()
    }
}

fn main() {
    let registry = Registry::new();
    
    {
        let item1 = Rc::new(Item { id: 1, data: "First".to_string() });
        let item2 = Rc::new(Item { id: 2, data: "Second".to_string() });
        
        registry.register(&item1);
        registry.register(&item2);
        
        println!("Active count: {}", registry.active_count());  // 2
        registry.list_active_items();
    }
    // item1 and item2 dropped
    
    println!("\nAfter items dropped:");
    println!("Active count: {}", registry.active_count());  // 0
    registry.list_active_items();
}
```

## 10. **Plugin Systems**

Plugins that reference the main application without keeping it alive.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Plugin {
    fn execute(&self);
}

struct Application {
    name: String,
    plugins: RefCell<Vec<Box<dyn Plugin>>>,
}

struct MyPlugin {
    app: Weak<Application>,  // Doesn't keep app alive
    plugin_name: String,
}

impl Plugin for MyPlugin {
    fn execute(&self) {
        if let Some(app) = self.app.upgrade() {
            println!("Plugin '{}' running in '{}'", self.plugin_name, app.name);
        } else {
            println!("Plugin '{}': Application no longer available", self.plugin_name);
        }
    }
}

impl Application {
    fn new(name: String) -> Rc<Self> {
        Rc::new(Application {
            name,
            plugins: RefCell::new(vec![]),
        })
    }
    
    fn add_plugin(app: &Rc<Application>, plugin_name: String) {
        let plugin = MyPlugin {
            app: Rc::downgrade(app),
            plugin_name,
        };
        app.plugins.borrow_mut().push(Box::new(plugin));
    }
    
    fn run_plugins(&self) {
        for plugin in self.plugins.borrow().iter() {
            plugin.execute();
        }
    }
}

fn main() {
    let plugins = {
        let app = Application::new("MyApp".to_string());
        Application::add_plugin(&app, "Plugin1".to_string());
        Application::add_plugin(&app, "Plugin2".to_string());
        
        app.run_plugins();
        
        // Extract plugins before app is dropped
        app.plugins.borrow_mut().drain(..).collect::<Vec<_>>()
    };  // app dropped here
    
    println!("\nAfter app dropped:");
    for plugin in &plugins {
        plugin.execute();
    }
}
```

## When **NOT** to Use `Weak<T>`

❌ **You need guaranteed access** - `Weak` can fail to upgrade  
❌ **Single ownership is sufficient** - Use `Box<T>`  
❌ **No cycles exist** - Use `Rc<T>` or `Arc<T>` directly  
❌ **Performance-critical paths** - Upgrading has overhead  
❌ **Simple references work** - Use `&T` when possible  

## Important Operations

### Creating Weak References

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(42);
let weak: Weak<i32> = Rc::downgrade(&strong);

// Or create empty Weak
let empty: Weak<i32> = Weak::new();
```

### Upgrading Weak to Strong

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);

// Attempt upgrade
if let Some(upgraded) = weak.upgrade() {
    println!("Value: {}", upgraded);
} else {
    println!("Strong reference was dropped");
}
```

### Checking if Still Valid

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);

println!("Strong count: {}", weak.strong_count());  // 1
println!("Weak count: {}", weak.weak_count());      // 1

drop(strong);

println!("Strong count: {}", weak.strong_count());  // 0
assert!(weak.upgrade().is_none());
```

## Reference Counting Behavior

```rust
use std::rc::{Rc, Weak};

let strong1 = Rc::new(42);
println!("Strong: {}, Weak: {}", 
    Rc::strong_count(&strong1),  // 1
    Rc::weak_count(&strong1)     // 0
);

let weak1 = Rc::downgrade(&strong1);
println!("Strong: {}, Weak: {}", 
    Rc::strong_count(&strong1),  // 1
    Rc::weak_count(&strong1)     // 1
);

let strong2 = Rc::clone(&strong1);
println!("Strong: {}, Weak: {}", 
    Rc::strong_count(&strong1),  // 2
    Rc::weak_count(&strong1)     // 1
);

drop(strong1);
drop(strong2);
// Data is deallocated, but weak reference metadata remains
assert!(weak1.upgrade().is_none());
```

## Memory Behavior

```rust
use std::rc::{Rc, Weak};

{
    let strong = Rc::new(vec![1, 2, 3, 4, 5]);
    let weak = Rc::downgrade(&strong);
    
    // Both exist
    assert_eq!(Rc::strong_count(&strong), 1);
    assert!(weak.upgrade().is_some());
    
    drop(strong);
    
    // Data deallocated, but Weak metadata still exists
    // Memory for reference counts isn't freed until all Weak refs dropped
    assert!(weak.upgrade().is_none());
}
// Now all memory is freed
```

## Quick Decision Guide

```
Do you have bidirectional references? → YES: One side should use Weak
                                       ↓ NO
Do you have cycles (A→B→C→A)? → YES: Break cycle with Weak
                                ↓ NO
Is this a cache/registry? → YES: Consider Weak to allow cleanup
                           ↓ NO
Should references prevent cleanup? → NO: Use Weak
                                    ↓ YES
                                    Use Rc/Arc
```

## Performance Characteristics

- **Downgrade cost**: O(1) - just increments weak count
- **Upgrade cost**: O(1) - checks strong count and increments if non-zero
- **Memory overhead**: Shares allocation with `Rc`/`Arc` (no extra allocation)
- **Failed upgrade**: Returns `None` if strong count is 0

## Common Patterns

### Weak + Option for Optional Parent

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    parent: RefCell<Weak<Node>>,  // Optional parent
}
```

### Cleaning Up Dead Weak References

```rust
use std::rc::Weak;

fn cleanup_dead<T>(weak_refs: &mut Vec<Weak<T>>) {
    weak_refs.retain(|weak| weak.upgrade().is_some());
}
```

### Weak in Collections

```rust
use std::rc::{Rc, Weak};
use std::collections::HashMap;

struct Manager {
    items: HashMap<String, Weak<Item>>,
}
```

## Thread-Safe Version (Arc + Weak)

```rust
use std::sync::{Arc, Weak};

let strong = Arc::new(42);
let weak: Weak<i32> = Arc::downgrade(&strong);

// Same API as Rc/Weak, but thread-safe
```

**Key insight**: `Weak<T>` is for **non-owning references** that don't keep data alive. It's essential for breaking reference cycles and implementing caches/registries where items can be freed when no longer in use elsewhere. Always use `Weak` when you need to reference data but shouldn't prevent its cleanup.