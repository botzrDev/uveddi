// Simple test code with some intentional issues
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    
    // Unused variable (dead code)
    let unused_var = 42;
    
    println!("Map size: {}", map.len());
}

// Large function that might be flagged
fn large_function() {
    let mut result = 0;
    for i in 0..100 {
        result += i;
        if i % 10 == 0 {
            println!("Processing: {}", i);
        }
    }
    println!("Result: {}", result);
}
