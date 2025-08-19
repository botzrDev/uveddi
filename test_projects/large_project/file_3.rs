fn unused_function_3() {
    println!("Unused function 3");
}

struct LargeStruct3 {
    field1: i32, field2: String, field3: Vec<i32>,
    field4: Option<String>, field5: bool, field6: f64,
    field7: Vec<String>, field8: u32, field9: u64,
    field10: String, field11: f32, field12: char,
}

impl LargeStruct3 {
    fn method1(&self) -> i32 { self.field1 }
    fn method2(&self) -> String { self.field2.clone() }
    fn method3(&self) -> Vec<i32> { self.field3.clone() }
    fn method4(&self) -> Option<String> { self.field4.clone() }
    fn method5(&self) -> bool { self.field5 }
    fn method6(&self) -> f64 { self.field6 }
    fn method7(&self) -> Vec<String> { self.field7.clone() }
    fn method8(&self) -> u32 { self.field8 }
    fn method9(&self) -> u64 { self.field9 }
    fn method10(&self) -> String { self.field10.clone() }
    fn method11(&self) -> f32 { self.field11 }
    fn method12(&self) -> char { self.field12 }
}
