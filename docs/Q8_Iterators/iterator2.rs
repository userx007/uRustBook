use std::ops::Index;

// ============================================
// APPROACH 1: Minimal Iterator (simplest)
// ============================================
struct SimpleVector<T> {
    data: Vec<T>,
}

impl<T> SimpleVector<T> {
    fn new() -> Self {
        SimpleVector { data: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    // Just expose the underlying Vec iterator
    fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
}

// ============================================
// APPROACH 2: Custom Iterator (basic)
// ============================================
struct MyVector<T> {
    data: Vec<T>,
}

// Simple iterator struct - just index tracking
struct MyVectorIter<'a, T> {
    data: &'a [T],
    index: usize,
}

impl<T> MyVector<T> {
    fn new() -> Self {
        MyVector { data: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    fn iter(&self) -> MyVectorIter<T> {
        MyVectorIter {
            data: &self.data,
            index: 0,
        }
    }
}

// Only implement the required method
impl<'a, T> Iterator for MyVectorIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// ============================================
// APPROACH 3: Using impl Trait (modern Rust)
// ============================================
struct ModernVector<T> {
    data: Vec<T>,
}

impl<T> ModernVector<T> {
    fn new() -> Self {
        ModernVector { data: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    // Return "some type that implements Iterator"
    // No need to name the iterator type!
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    // Can also do custom logic with iterator adapters
    fn iter_even_indices(&self) -> impl Iterator<Item = &T> {
        self.data.iter().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, v)| v)
    }
}

// ============================================
// APPROACH 4: Full-featured (what I showed before)
// ============================================
struct FullVector<T> {
    data: Vec<T>,
}

struct FullVectorIter<'a, T> {
    data: &'a Vec<T>,
    index: usize,
}

impl<T> FullVector<T> {
    fn new() -> Self {
        FullVector { data: Vec::new() }
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
    }

    fn iter(&self) -> FullVectorIter<T> {
        FullVectorIter {
            data: &self.data,
            index: 0,
        }
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
}

impl<'a, T> Iterator for FullVectorIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Optional: implement ExactSizeIterator
impl<'a, T> ExactSizeIterator for FullVectorIter<'a, T> {
    fn len(&self) -> usize {
        self.data.len() - self.index
    }
}

// Optional: implement DoubleEndedIterator
impl<'a, T> DoubleEndedIterator for FullVectorIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            Some(&self.data[self.data.len() - 1 - self.index])
        } else {
            None
        }
    }
}

// Optional: implement IntoIterator for for loops
impl<'a, T> IntoIterator for &'a FullVector<T> {
    type Item = &'a T;
    type IntoIter = FullVectorIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn main() {
    println!("=== APPROACH 1: Simple (reuse Vec iterator) ===");
    let mut sv = SimpleVector::new();
    sv.push(1);
    sv.push(2);
    sv.push(3);

    for val in sv.iter() {
        print!("{} ", val);
    }
    println!("\n");

    println!("=== APPROACH 2: Basic custom iterator ===");
    let mut mv = MyVector::new();
    mv.push(10);
    mv.push(20);
    mv.push(30);

    for val in mv.iter() {
        print!("{} ", val);
    }
    println!("\n");

    println!("=== APPROACH 3: Modern impl Trait ===");
    let mut modv = ModernVector::new();
    modv.push(100);
    modv.push(200);
    modv.push(300);
    modv.push(400);

    print!("All: ");
    for val in modv.iter() {
        print!("{} ", val);
    }
    println!();

    print!("Even indices: ");
    for val in modv.iter_even_indices() {
        print!("{} ", val);
    }
    println!("\n");

    println!("=== APPROACH 4: Full-featured ===");
    let mut fv = FullVector::new();
    fv.push(5);
    fv.push(10);
    fv.push(15);

    print!("Forward: ");
    for val in &fv {  // Works because we implemented IntoIterator
        print!("{} ", val);
    }
    println!();

    // ExactSizeIterator allows .len()
    let mut iter = fv.iter();
    println!("Iterator length: {}", iter.len());
    iter.next();
    println!("After next(), length: {}", iter.len());
}