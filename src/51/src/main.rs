
fn main() {

    println!("\n======= take() 1st ============\n");
    let mut x : Option<u32> = Some(42);
    let y = x.take();
    
    println!("y={:?}", y.unwrap_or(0));
    println!("x={:?}", x);
    

    println!("\n======= take() 2nd ============\n");

    let mut x1 : Option<u32> = None;
    let y1 = x1.take();
    
    println!("y1={:?}", y1.unwrap_or(0));
    println!("x1={:?}", x1);

    println!("\n======= replace() ============\n");    
    
    let mut r : Option<u32> = Some(44);
    println!("r={:?}", r.unwrap_or(0));
    
    r.replace(99);
    println!("r={:?}", r.unwrap_or(0));
    
    println!("\n======= get_or_insert() 1st ============\n");    
    
    let mut x3 : Option<u32> = None;
    let g3 = x3.get_or_insert(32);
    
    println!("g3={:?}", g3);
    println!("x3={:?}", x3.as_ref().unwrap_or(&0));    
    
    println!("\n======= get_or_insert() 2nd ============\n");        
    
    let mut x2 : Option<u32> = Some(77);
    let g2 = x2.get_or_insert(32);

    println!("g2={:?}", g2);
    println!("x2={:?}", x2.unwrap_or(0));    
    
    
    println!("\n======= get_or_insert_with() 1st ============\n");        

    let mut x4 : Option<String> = None;
    let g4 = x4.get_or_insert_with(|| "Hello".to_string());
    println!("g4={:?}", g4);

    println!("\n======= get_or_insert_with() 2nd ============\n");        

    let mut x5 : Option<String> = Some("World".to_string());
    let g5 = x5.get_or_insert_with(|| "Hello".to_string());
    println!("g5={:?}", g5);


    println!("\n======= get_or_insert_default() 1nd ============\n");        

    let mut x6 : Option<String> = Some("World".to_string());
    let g6 = x6.get_or_insert_default();
    println!("g6={:?}", g6);
    println!("x6={:?}", x6.as_ref().unwrap());

    println!("\n======= get_or_insert_default() 2nd ============\n");        

    let mut x7 : Option<String> = None;
    let g7 = x7.get_or_insert_default();
    println!("g7={:?}", g7);
    println!("x7={:?}", x7.as_ref().unwrap());
    
    println!("\n======= as_ref() ============\n");            
    
    let x8: Option<u32> = Some(99);
    let g8 = x8.as_ref();

    println!("g8={:?}", g8.unwrap());
    println!("x8={:?}", x8.unwrap());

    println!("\n======= as_mut() 1st ============\n");            
    
    let mut x9: Option<u32> = Some(99);
    
    if let Some(v) = x9.as_mut() {
        *v = 100;
    }
    
    println!("x9={:?}", x9.unwrap())


}


/*
┌────────────────────────┬──────────────────┬──────────────────┬─────────────────────┐
│ Method                 │ Before           │ After            │ Returns             │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ take()                 │ Some(x)          │ None             │ Some(x)             │
│                        │ None             │ None             │ None                │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ replace(y)             │ Some(x)          │ Some(y)          │ Some(x)             │
│                        │ None             │ Some(y)          │ None                │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert(y)       │ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(y)          │ &mut y              │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert_with(f)  │ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(f())        │ &mut f()            │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ get_or_insert_default()│ Some(x)          │ Some(x)          │ &mut x              │
│                        │ None             │ Some(T::default) │ &mut T::default     │
├────────────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│ as_ref()               │ Some(x)          │ Some(x)          │ Option<&T>          │
│ as_mut()               │ Some(x)          │ Some(x)          │ Option<&mut T>      │
└────────────────────────┴──────────────────┴──────────────────┴─────────────────────┘
*/