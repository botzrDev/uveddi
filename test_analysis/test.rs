// Test file with various anti-patterns
struct GodObject {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: HashMap<String, i32>,
    field5: bool,
    field6: f64,
    field7: Option<String>,
    field8: Result<i32, String>,
    field9: Vec<Vec<i32>>,
    field10: String,
}

impl GodObject {
    fn method1(&self) {}
    fn method2(&self) {}
    fn method3(&self) {}
    fn method4(&self) {}
    fn method5(&self) {}
    fn method6(&self) {}
    fn method7(&self) {}
    fn method8(&self) {}
    fn method9(&self) {}
    fn method10(&self) {}
}

// Dead code
fn unused_function() {
    println!("Never called");
}

// Magic values
fn calculate_price(quantity: i32) -> f64 {
    quantity as f64 * 19.99  // Magic number
}

const TIMEOUT: i32 = 5000;  // Another magic number

use std::collections::HashMap;
