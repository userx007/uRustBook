struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max : u32) -> Counter {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < self.max {
            Some(self.count)
        } else {
            None
        }
    }
}
fn main() {
    let mut counter = Counter::new(35);
    while let Some(x) = counter.next() {
        println!("{}", x);
    }
}