
// Test code for TUI integration testing
pub struct TestStruct {
    pub field1: String,
    pub field2: i32,
}

impl TestStruct {
    pub fn new() -> Self {
        Self {
            field1: "test".to_string(),
            field2: 42,
        }
    }
    
    pub fn unused_method(&self) {
        // This method is never called - should be detected as dead code
        println!("This method is unused");
    }
}

pub fn main() {
    let _instance = TestStruct::new();
    println!("Hello from test project");
}
