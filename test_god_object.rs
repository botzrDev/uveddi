// Test file to trigger God Object detection
pub struct LargeStruct {
    field1: String,
    field2: i32,
    field3: bool,
    field4: Vec<String>,
    field5: HashMap<String, i32>,
    field6: Option<String>,
    field7: String,
    field8: i32,
    field9: bool,
}

impl LargeStruct {
    pub fn method1(&self) -> String { "method1".to_string() }
    pub fn method2(&self) -> i32 { 42 }
    pub fn method3(&self) -> bool { true }
    pub fn method4(&self) -> Vec<String> { vec![] }
    pub fn method5(&self) -> String { "method5".to_string() }
    pub fn method6(&self) -> i32 { 6 }
}
