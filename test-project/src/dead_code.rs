// Dead code examples - functions and structs that are never called or used

// Active code that is used
pub fn active_function() -> i32 {
    42
}

pub fn another_active_function() -> String {
    active_function().to_string()
}

// Dead code that is never called
fn unused_private_function() -> bool {
    true
}

pub fn unused_public_function() -> Vec<String> {
    vec!["never".to_string(), "called".to_string()]
}

fn another_unused_function(_param: i32) -> i32 {
    _param * 2 + unused_helper_function()
}

fn unused_helper_function() -> i32 {
    999
}

// Dead structs and implementations
struct UnusedStruct {
    _field1: String,
    _field2: i32,
}

impl UnusedStruct {
    fn new(_field1: String, _field2: i32) -> Self {
        Self { _field1, _field2 }
    }
    
    fn unused_method(&self) -> String {
        format!("{}: {}", self._field1, self._field2)
    }
}

// Dead enums
enum UnusedEnum {
    _Variant1,
    _Variant2(String),
    _Variant3 { _field: i32 },
}

// Dead constants
const UNUSED_CONSTANT: i32 = 123;
const ANOTHER_UNUSED_CONSTANT: &str = "unused";

// Mixed usage - some functions used, others not
pub fn mixed_usage_entry() -> String {
    used_helper().to_string()
}

fn used_helper() -> i32 {
    42
}

fn unused_helper_in_mixed() -> i32 {
    99  // This helper is never called
}

// Function that uses active code
pub fn main_usage() {
    println!("Active function result: {}", active_function());
    println!("Another active function result: {}", another_active_function());
    println!("Mixed usage entry: {}", mixed_usage_entry());
}