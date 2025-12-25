fn main() {
    // ----- [1] vector of numbers to string (separated by spaces) -----
    let x1 = vec![1, 2, 3, 4, 5];
    let r1 = x1
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", r1); // 1 2 3 4 5

    // ----- [1a] vector of strings to string (separated by spaces) -----
    let x2 = vec!["A", "B", "C", "D", "E"];
    let r2 = x2.join(" ");
    println!("{}", r2); // A B C D E

    // ----- [1b] range to string (separated by spaces) -----
    let r3 = (0..=10)
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", r3); // 0 1 2 3 4 5 6 7 8 9 10

    // ----- [1c] filtered range to string (separated by spaces) -----
    let r4 = (0..=10)
        .filter(|x| x % 2 == 0)
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", r4); // 0 2 4 6 8 10

    // ----- [1d] filtered and processed range to string (separated by spaces) -----
    let r5 = (0..=10)
        .filter(|x| x % 2 == 0)
        .map(|x| (x * 2).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", r5); // 0 4 8 12 16 20

    // ----- [2] flatten a vector of vectors -----
    let x6 = vec![vec![1, 2], vec![3, 4]];
    let r6 = x6.into_iter().flatten().collect::<Vec<_>>();
    println!("{:?}", r6); // [1, 2, 3, 4]

    // ----- [3] chain iterators -----
    let it = (1..3).chain(10..12);
    let r7 = it.map(|x| x.to_string()).collect::<Vec<_>>().join(" ");
    println!("{}", r7); // 1 2 10 11

    // ----- [4] peekable iterator -----
    // peek() lets you look at the next value without moving the iterator.
    // You can decide whether to consume it with next() based on some condition.
    // Without peekable(), you’d have to consume the item immediately and couldn’t “peek ahead” safely.

    let mut it = (0..22).peekable();
    while let Some(&x) = it.peek() {
        print!("{x} "); // 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
        it.next();
    }
    println!();

    // ----- [5] Take / skip -----
    for x in (1..100).skip(10).take(5) {
        print!("{} ", x); // 11 12 13 14 15
    }
    println!();

    // ----- [6] Cycle iterator -----
    for x in [1, 2, 3, 4, 5].iter().cycle().take(10) {
        print!("{x} "); // 1 2 3 4 5 1 2 3 4 5
    }
    println!();

    // ----- [7] Inspect side effects -----
    let sum: i32 = (1..5).inspect(|x| print!("x={x} ")).sum();
    println!(": Sum = {sum}"); // x=1 x=2 x=3 x=4 : Sum = 10

    // ----- [8] Enumerate + map -----
    let v: Vec<_> = (10..15).enumerate().map(|(i, x)| i + x).collect();
    println!("v = {:?}", v); // v = [10, 12, 14, 16, 18]

    // ----- [9] windows of an array -----
    let arr = [1, 2, 3, 4, 5];
    let w: Vec<_> = arr.windows(3).collect();
    println!("W= {w:?}"); // W= [[1, 2, 3], [2, 3, 4], [3, 4, 5]]

    // ----- [10] chunks iterator -----
    let c: Vec<_> = arr.chunks(2).collect();
    println!("c = {c:?}"); // c = [[1, 2], [3, 4], [5]]

    for chunk in arr.chunks(2) {
        print!("{chunk:?} "); // [1, 2] [3, 4] [5]
    }
    println!();

    println!("==========================================================");

    // ----- [11] Mutable iterator chain -----
    let mut v = vec![1, 2, 3, 4, 5];
    v.iter_mut().for_each(|x| *x += 10);
    println!("v = {:?}", v);

    // ----- [12] Filter_map (// filter_map: drop Nones, unwrap Somes) -----
    // filter_map lets you take each item in an iterator, run a closure that returns an Option<T>, and then:
    // - Keep the value if it’s Some(T) (using the inner T)
    // - Discard it if it's None
    let vals = vec![Some(1), None, Some(3)];
    let nums: Vec<_> = vals.into_iter().filter_map(|x| x).collect();
    println!("{nums:?}");

    let vals = vec!["10", "hi", "20"];
    let nums: Vec<i32> = vals.into_iter().filter_map(|x| x.parse().ok()).collect();
    println!("{nums:?}"); // [10, 20]

    // use map instead to add a default value for the invalid entries
    let vals = vec!["10", "hi", "20"];
    let nums: Vec<i32> = vals.into_iter().map(|x| x.parse().unwrap_or(0)).collect();
    println!("{nums:?}"); // [10, 0, 20]

    // ----- [13] Collect into a HashMap with zip -----
    let keys = ["a", "b", "c", "d"];
    let values = [1, 2, 3, 4];
    let map: std::collections::HashMap<_, _> =
        keys.iter().cloned().zip(values.iter().cloned()).collect();
    println!("{:?}", map); // {"b": 2, "a": 1, "c": 3, "d": 4}

    // ----- [14] Group by consecutive values -----
    // Version         x type  Need *x
    // for &x in &data   i32     no
    // for  x in &data  &i32     yes
    // for &x automatically copies/dereferences the integer.
    // for  x keeps the reference; you just have to dereference manually (*x).
    let data = [1, 1, 1, 1, 2, 2, 2, 3, 3];
    let mut prev = None;
    for &x in &data {
        if prev != Some(x) {
            println!("group starts {x}");
        }
        prev = Some(x)
    }

    // ----- [15] Sum of squares -----
    let sum: u32 = (1..=10).map(|x| x * x).sum();
    println!("sum = {}", sum); // sum = 385

    // ----- [16] Find first matching -----
    let first_even = (1..=10).find(|x| x % 3 == 0).unwrap_or(0);
    println!("first_even = {:?}", first_even); // first_even = 3

    // ----- [17] Partition by predicate -----
    let (even, odd): (Vec<_>, Vec<_>) = (0..=10).partition(|x| x % 2 == 0);
    println!("{:?}", (even, odd));

    // ----- [18a] All -----
    // return true if ALL elements fullfill the predicate, false otherwise
    let all_positive = (-10..10).all(|x| x > 0);
    println!("{:?}", all_positive); // false

    // ----- [18b] Any -----
    // return true if any of elements fullfill the predicate, false otherwise
    let has_even = (1..5).any(|x| x % 2 == 0);
    println!("{:?}", has_even); // true

    let has_even = [1, 3, 5, 7].iter().any(|x| x % 2 == 0);
    println!("{:?}", has_even); // false

    // ----- [19] Zip multiple iterators -----
    let a = [1, 2, 3, 4];
    let b = [11, 12, 13, 14];
    let c = a.iter().zip(b.iter());
    for (x, y) in c {
        print!("{:?} ", (x, y))
    }
    println!("");

    // ----- [20a] fold ----
    let sum = (1..=3).fold(0, |acc, x| acc + x);
    println!("sum={}", sum); // sum=6
    let product = (1..=5).fold(1, |prod, x| prod * x);
    println!("product={}", product); // product=120

    let words = ["Hello", "world", "from", "space"];
    let sentence = words.iter().fold(String::new(), |acc, x| acc + x + " ");
    println!("Sentence = {}", sentence); // Sentence = Hello world from space

    // ----- [20b] reduce ----
    // Similar to fold, but does not take an initial value.
    // The first element of the iterator becomes the initial accumulator.
    // Returns Option<T>, because the iterator might be empty.
    let sum = (1..3).reduce(|acc, x| acc + x);
    println!("sum = {:?}", sum.unwrap_or(0));

    let sum = (1..=0).reduce(|acc, x| acc + x);
    println!("sum = {:?}", sum);

    println!("==========================================================");

    // ----- [38] Drain vector ----
    // drain removes elements from the vector and returns them as an iterator.
    let mut v = vec![1, 2, 3, 4, 5, 6, 7];
    for x in v.drain(2..=4) {
        print!("{} ", x);
    }
    println!("\n{:?}", v);

    // ----- [41] Infinite iterator ----
    for x in (0..).take(10) {
        print!("{} ", x);
    }
    println!("");

    // ----- [42] Iterator folding into struc ----
    #[derive(Debug)]
    struct Stats {
        sum: i32,
        count: i32,
    }
    let stats = (1..6).fold(Stats { sum: 0, count: 0 }, |mut s, x| {
        s.sum += x;
        s.count += 1;
        s
    });
    println!("{:?}", stats);

    // ----- [43] Functional factorial ----
    let prod = (1..=10).product::<u64>();
    println!("prod = {:?}", prod); // prod = 3628800

    // ----- [44] Map + filter + sum ----
    let sum: u64 = (0..10)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .inspect(|x| println!("x={}", x))
        .sum();
    println!("sum = {:?}", sum); // sum = 120

    // ----- [45] Flatten nested vectors and map -----
    // flat_map(|v| v) <=> map(|v| v).flatten()

    let nested = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let flat: Vec<_> = nested
        .into_iter()
        .flat_map(|v| v.into_iter())
        .map(|x| x * 2)
        .collect();
    println!("flat = {:?}", flat); // flat = [2, 4, 6, 8, 10, 12]

    // Flatten Vec<Vec<Vec<T>>> into Vec<T>
    let nested = vec![vec![vec![1, 2], vec![3]], vec![vec![4], vec![5, 6]]];
    let flat: Vec<i32> = nested
        .into_iter()
        .flat_map(|v2| v2.into_iter()) // Vec<Vec<i32>> -> Vec<i32>
        .flat_map(|v1| v1.into_iter()) // Vec<i32> -> i32
        .collect();
    println!("flat = {:?}", flat); // flat = [1, 2, 3, 4, 5, 6]

    // ----- [46] Iterator combinators for string parsing
    let x: Vec<i32> = "1,2,3,g"
        .split(",")
        .map(|x| x.parse().unwrap_or(0))
        .collect();
    println!("x = {:?}", x); // x = [1, 2, 3]

    // ----- [47] Functional sliding sum -----
    let arr = [1, 2, 3, 4];
    let sums: Vec<_> = arr.windows(2).map(|w| w[0] + w[1]).collect();
    println!("sums = {:?}", sums); //

    let arr = [1, 2, 3, 4, 5, 6, 7];
    let sums: Vec<_> = arr.windows(3).map(|w| w[0] + w[1] + w[2]).collect();
    println!("sums = {:?}", sums); //

    // ----- [48] Recursive functional loop (Fibonacci) -----
    fn fib(n: u32) -> u32 {
        if n < 2 { n } else { fib(n - 2) + fib(n - 1) }
    }
    for i in 0..10 {
        print!("{} ", fib(i))
    }
    println!("");

    // ----- [50] Lazy iterator chain -----
    let iter = (0..).filter(|x| x % 2 == 0).map(|x| x*x).take(5);
    for i in iter {
        print!("{i} ");
    }
    println!("");

    // ----- [83] Circular buffer with iterators ------
    let mut idx = 0;
    for _ in 0..20 {
        idx = (idx + 1) % 5;
        print!("{idx} ");
    }
}
