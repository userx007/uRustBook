// ============================================
// TRAIT OBJECTS AND DYNAMIC DISPATCH IN RUST
// ============================================

// 1. BASIC TRAIT DEFINITION
// First, let's define a trait that we'll use throughout
trait Animal {
    fn make_sound(&self) -> String;
    fn name(&self) -> String;
}

struct Dog {
    name: String,
}

struct Cat {
    name: String,
}

impl Animal for Dog {
    fn make_sound(&self) -> String {
        "Woof!".to_string()
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Animal for Cat {
    fn make_sound(&self) -> String {
        "Meow!".to_string()
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}

// ============================================
// 2. STATIC DISPATCH (Without Trait Objects)
// ============================================
// Using generics - the type is known at compile time
fn animal_sound_static<T: Animal>(animal: &T) {
    println!("{} says: {}", animal.name(), animal.make_sound());
}

// ============================================
// 3. DYNAMIC DISPATCH (With Trait Objects)
// ============================================
// Using trait objects - the type is determined at runtime
// The 'dyn' keyword indicates dynamic dispatch
fn animal_sound_dynamic(animal: &dyn Animal) {
    println!("{} says: {}", animal.name(), animal.make_sound());
}

// ============================================
// 4. STORING DIFFERENT TYPES IN COLLECTIONS
// ============================================
// This is where trait objects really shine!
fn demonstrate_heterogeneous_collection() {
    println!("\n--- Heterogeneous Collection ---");
    
    // We can store different types that implement the same trait
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { name: "Buddy".to_string() }),
        Box::new(Cat { name: "Whiskers".to_string() }),
        Box::new(Dog { name: "Max".to_string() }),
        Box::new(Cat { name: "Felix".to_string() }),
    ];
    
    // Iterate through different types as if they were the same
    for animal in &animals {
        println!("{} says: {}", animal.name(), animal.make_sound());
    }
}

// ============================================
// 5. TRAIT OBJECT SYNTAX VARIATIONS
// ============================================
// Different ways to use trait objects:

// Box<dyn Trait> - owned trait object
fn process_owned(animal: Box<dyn Animal>) {
    println!("Processing owned: {}", animal.name());
}

// &dyn Trait - borrowed trait object
fn process_borrowed(animal: &dyn Animal) {
    println!("Processing borrowed: {}", animal.name());
}

// &mut dyn Trait - mutable borrowed trait object
trait Trainable {
    fn train(&mut self);
    fn skill_level(&self) -> u32;
}

struct Pet {
    name: String,
    skill: u32,
}

impl Trainable for Pet {
    fn train(&mut self) {
        self.skill += 1;
    }
    
    fn skill_level(&self) -> u32 {
        self.skill
    }
}

fn train_animal(animal: &mut dyn Trainable) {
    animal.train();
    println!("Skill level now: {}", animal.skill_level());
}

// ============================================
// 6. OBJECT SAFETY RULES
// ============================================
// Not all traits can be made into trait objects
// A trait is "object-safe" if:
// 1. It doesn't return Self
// 2. It has no generic type parameters

// This trait is NOT object-safe (returns Self)
trait Cloneable {
    fn clone_self(&self) -> Self;
}

// This trait IS object-safe
trait Drawable {
    fn draw(&self);
}

struct Circle;
struct Rectangle;

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing a circle");
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing a rectangle");
    }
}

// ============================================
// 7. PRACTICAL EXAMPLE: PLUGIN SYSTEM
// ============================================
trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self);
}

struct LoggerPlugin;
struct MetricsPlugin;

impl Plugin for LoggerPlugin {
    fn name(&self) -> &str {
        "Logger"
    }
    
    fn execute(&self) {
        println!("[Logger] Logging data...");
    }
}

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str {
        "Metrics"
    }
    
    fn execute(&self) {
        println!("[Metrics] Collecting metrics...");
    }
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Vec::new(),
        }
    }
    
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    fn run_all(&self) {
        for plugin in &self.plugins {
            println!("Running plugin: {}", plugin.name());
            plugin.execute();
        }
    }
}

// ============================================
// 8. PERFORMANCE CONSIDERATIONS
// ============================================
// Dynamic dispatch has a small runtime cost due to vtable lookup
// but provides flexibility to work with different types at runtime

fn demonstrate_vtable_concept() {
    println!("\n--- VTable Concept ---");
    
    // When we create a trait object, Rust creates a "fat pointer":
    // 1. Pointer to the data
    // 2. Pointer to the vtable (virtual method table)
    
    let dog = Dog { name: "Rover".to_string() };
    let trait_obj: &dyn Animal = &dog;
    
    // At runtime, Rust uses the vtable to find the correct method
    println!("Size of Dog: {}", std::mem::size_of::<Dog>());
    println!("Size of &Dog: {}", std::mem::size_of::<&Dog>());
    println!("Size of &dyn Animal: {}", std::mem::size_of::<&dyn Animal>());
    // Notice: &dyn Animal is twice the size (two pointers!)
}

// ============================================
// MAIN FUNCTION - RUNNING EXAMPLES
// ============================================
fn main() {
    println!("=== TRAIT OBJECTS AND DYNAMIC DISPATCH ===\n");
    
    let dog = Dog { name: "Buddy".to_string() };
    let cat = Cat { name: "Whiskers".to_string() };
    
    // Static dispatch
    println!("--- Static Dispatch ---");
    animal_sound_static(&dog);
    animal_sound_static(&cat);
    
    // Dynamic dispatch
    println!("\n--- Dynamic Dispatch ---");
    animal_sound_dynamic(&dog);
    animal_sound_dynamic(&cat);
    
    // Heterogeneous collection
    demonstrate_heterogeneous_collection();
    
    // Mutable trait objects
    println!("\n--- Mutable Trait Objects ---");
    let mut pet = Pet { name: "Fido".to_string(), skill: 0 };
    train_animal(&mut pet);
    train_animal(&mut pet);
    
    // Drawable shapes
    println!("\n--- Drawable Objects ---");
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle),
        Box::new(Rectangle),
        Box::new(Circle),
    ];
    
    for shape in &shapes {
        shape.draw();
    }
    
    // Plugin system
    println!("\n--- Plugin System ---");
    let mut manager = PluginManager::new();
    manager.register(Box::new(LoggerPlugin));
    manager.register(Box::new(MetricsPlugin));
    manager.run_all();
    
    // VTable demonstration
    demonstrate_vtable_concept();
}

// ============================================
// KEY TAKEAWAYS:
// ============================================
// 1. Static Dispatch (generics): Type known at compile time, monomorphization
// 2. Dynamic Dispatch (trait objects): Type determined at runtime via vtable
// 3. Use `dyn Trait` to create trait objects
// 4. Trait objects enable heterogeneous collections
// 5. Trait objects have a small runtime cost but provide flexibility
// 6. Not all traits can be trait objects (object safety rules)
// 7. Common patterns: Box<dyn Trait>, &dyn Trait, &mut dyn Trait