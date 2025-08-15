// God Object anti-pattern test
struct GodStruct {
    field1: String,
    field2: i32,
    field3: f64,
    field4: bool,
    field5: Vec<String>,
    field6: HashMap<String, i32>,
    field7: Option<String>,
    field8: Result<i32, String>,
    field9: Box<dyn std::any::Any>,
    field10: Arc<Mutex<String>>,
    field11: u32,
    field12: f32,
    field13: char,
    field14: [i32; 10],
    field15: std::collections::BTreeMap<String, i32>,
}

impl GodStruct {
    fn method1(&self) -> String { "method1".to_string() }
    fn method2(&self) -> i32 { self.field2 }
    fn method3(&self) -> f64 { self.field3 }
    fn method4(&self) -> bool { self.field4 }
    fn method5(&self) -> &Vec<String> { &self.field5 }
    fn method6(&self) -> &HashMap<String, i32> { &self.field6 }
    fn method7(&self) -> &Option<String> { &self.field7 }
    fn method8(&self) -> &Result<i32, String> { &self.field8 }
    fn method9(&self) -> &Box<dyn std::any::Any> { &self.field9 }
    fn method10(&self) -> &Arc<Mutex<String>> { &self.field10 }
    fn method11(&self) -> u32 { self.field11 }
    fn method12(&self) -> f32 { self.field12 }
    fn method13(&self) -> char { self.field13 }
    fn method14(&self) -> &[i32; 10] { &self.field14 }
    fn method15(&self) -> &std::collections::BTreeMap<String, i32> { &self.field15 }
    
    // Additional methods to make it clearly a God Object
    fn calculate_something(&self) -> i32 { 42 }
    fn parse_data(&self, data: &str) -> Vec<String> { vec![] }
    fn save_to_database(&self) -> bool { true }
    fn send_email(&self, to: &str) -> bool { true }
    fn log_message(&self, msg: &str) {}
    fn validate_input(&self, input: &str) -> bool { true }
    fn format_output(&self, data: &str) -> String { data.to_string() }
    fn handle_error(&self, error: &str) {}
    fn compress_data(&self, data: &[u8]) -> Vec<u8> { vec![] }
    fn encrypt_data(&self, data: &str) -> String { data.to_string() }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};