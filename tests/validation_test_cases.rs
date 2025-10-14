// Validation Test Cases for Uveddi Anti-Pattern Detection
// Purpose: Intentional anti-patterns to validate 100% detection rate
// DO NOT REFACTOR - These are deliberately bad for testing purposes

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// TEST CASE 1: God Object Anti-Pattern
// Expected: Should be detected as god object (30+ methods, low cohesion)
pub struct MassiveGodObject {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: bool,
    field5: f64,
    field6: Option<String>,
    field7: Result<i32, String>,
    field8: Box<dyn std::error::Error>,
    field9: std::collections::HashMap<String, i32>,
    field10: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    field11: String,
    field12: i32,
    field13: bool,
    field14: f64,
    field15: String,
}

impl MassiveGodObject {
    pub fn new() -> Self { Self { field1: String::new(), field2: 0, field3: Vec::new(), field4: false, field5: 0.0, field6: None, field7: Ok(0), field8: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test")), field9: std::collections::HashMap::new(), field10: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())), field11: String::new(), field12: 0, field13: false, field14: 0.0, field15: String::new() } }
    pub fn method1(&self) -> i32 { 1 }
    pub fn method2(&self) -> i32 { 2 }
    pub fn method3(&self) -> i32 { 3 }
    pub fn method4(&self) -> i32 { 4 }
    pub fn method5(&self) -> i32 { 5 }
    pub fn method6(&self) -> i32 { 6 }
    pub fn method7(&self) -> i32 { 7 }
    pub fn method8(&self) -> i32 { 8 }
    pub fn method9(&self) -> i32 { 9 }
    pub fn method10(&self) -> i32 { 10 }
    pub fn method11(&self) -> i32 { 11 }
    pub fn method12(&self) -> i32 { 12 }
    pub fn method13(&self) -> i32 { 13 }
    pub fn method14(&self) -> i32 { 14 }
    pub fn method15(&self) -> i32 { 15 }
    pub fn method16(&self) -> i32 { 16 }
    pub fn method17(&self) -> i32 { 17 }
    pub fn method18(&self) -> i32 { 18 }
    pub fn method19(&self) -> i32 { 19 }
    pub fn method20(&self) -> i32 { 20 }
    pub fn method21(&self) -> i32 { 21 }
    pub fn method22(&self) -> i32 { 22 }
    pub fn method23(&self) -> i32 { 23 }
    pub fn method24(&self) -> i32 { 24 }
    pub fn method25(&self) -> i32 { 25 }
    pub fn method26(&self) -> i32 { 26 }
    pub fn method27(&self) -> i32 { 27 }
    pub fn method28(&self) -> i32 { 28 }
    pub fn method29(&self) -> i32 { 29 }
    pub fn method30(&self) -> i32 { 30 }
    pub fn method31(&self) -> i32 { 31 }
}

// TEST CASE 2: Long Method with High Complexity
// Expected: Critical severity (250+ LOC, complexity > 50, nesting > 15)
pub fn extremely_long_and_complex_method(input: i32) -> Result<String, String> {
    let mut result = String::new();

    // Start deep nesting
    if input > 0 {
        if input > 10 {
            if input > 20 {
                if input > 30 {
                    if input > 40 {
                        if input > 50 {
                            if input > 60 {
                                if input > 70 {
                                    if input > 80 {
                                        if input > 90 {
                                            result.push_str("Very deep");
                                        } else {
                                            result.push_str("Deep 9");
                                        }
                                    } else {
                                        result.push_str("Deep 8");
                                    }
                                } else {
                                    result.push_str("Deep 7");
                                }
                            } else {
                                result.push_str("Deep 6");
                            }
                        } else {
                            result.push_str("Deep 5");
                        }
                    } else {
                        result.push_str("Deep 4");
                    }
                } else {
                    result.push_str("Deep 3");
                }
            } else {
                result.push_str("Deep 2");
            }
        } else {
            result.push_str("Deep 1");
        }
    }

    // Add lots of match statements for complexity
    match input % 10 {
        0 => result.push_str("0"),
        1 => result.push_str("1"),
        2 => result.push_str("2"),
        3 => result.push_str("3"),
        4 => result.push_str("4"),
        5 => result.push_str("5"),
        6 => result.push_str("6"),
        7 => result.push_str("7"),
        8 => result.push_str("8"),
        9 => result.push_str("9"),
        _ => result.push_str("unknown"),
    }

    // More complexity
    for i in 0..input {
        if i % 2 == 0 {
            if i % 3 == 0 {
                if i % 5 == 0 {
                    result.push_str("fizzbuzz");
                } else {
                    result.push_str("fizz");
                }
            } else {
                if i % 5 == 0 {
                    result.push_str("buzz");
                } else {
                    result.push_str(&i.to_string());
                }
            }
        } else {
            result.push_str(&i.to_string());
        }
    }

    // Add more lines to reach 250+ LOC
    let mut counter = 0;
    while counter < 100 {
        counter += 1;
        if counter % 2 == 0 {
            result.push_str("even");
        } else {
            result.push_str("odd");
        }

        match counter % 3 {
            0 => result.push_str("div3"),
            1 => result.push_str("mod1"),
            2 => result.push_str("mod2"),
            _ => unreachable!(),
        }

        if counter > 50 {
            if counter > 60 {
                if counter > 70 {
                    if counter > 80 {
                        if counter > 90 {
                            result.push_str("high");
                        }
                    }
                }
            }
        }
    }

    // More nested loops
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                if i == j {
                    if j == k {
                        result.push_str("equal");
                    }
                }
            }
        }
    }

    // Add more decision points
    if input < 0 {
        return Err("Negative".to_string());
    } else if input == 0 {
        return Err("Zero".to_string());
    } else if input > 1000 {
        return Err("Too large".to_string());
    }

    // More complexity to increase cyclomatic complexity
    match input {
        1 => result.push_str("one"),
        2 => result.push_str("two"),
        3 => result.push_str("three"),
        4 => result.push_str("four"),
        5 => result.push_str("five"),
        6 => result.push_str("six"),
        7 => result.push_str("seven"),
        8 => result.push_str("eight"),
        9 => result.push_str("nine"),
        10 => result.push_str("ten"),
        11..=20 => result.push_str("teens"),
        21..=30 => result.push_str("twenties"),
        31..=40 => result.push_str("thirties"),
        41..=50 => result.push_str("forties"),
        51..=60 => result.push_str("fifties"),
        61..=70 => result.push_str("sixties"),
        71..=80 => result.push_str("seventies"),
        81..=90 => result.push_str("eighties"),
        91..=100 => result.push_str("nineties"),
        _ => result.push_str("other"),
    }

    // Add more lines
    let mut temp = Vec::new();
    for i in 0..50 {
        temp.push(i);
        if temp.len() > 10 {
            temp.sort();
            temp.reverse();
            temp.dedup();
        }
    }

    // More nested conditions
    if result.len() > 100 {
        if result.contains("fizz") {
            if result.contains("buzz") {
                if result.contains("even") {
                    if result.contains("odd") {
                        result = result.to_uppercase();
                    }
                }
            }
        }
    }

    Ok(result)
}

// TEST CASE 3: Exact Code Duplication
// Expected: Should detect duplicate code blocks
pub fn duplicate_calculation_block_1(x: i32, y: i32) -> i32 {
    let mut result = 0;
    result += x * 2;
    result += y * 3;
    result -= x / 2;
    result -= y / 3;
    if result > 100 {
        result = result % 100;
    }
    if result < 0 {
        result = result.abs();
    }
    result += 42;
    result *= 2;
    result
}

pub fn duplicate_calculation_block_2(a: i32, b: i32) -> i32 {
    let mut result = 0;
    result += a * 2;
    result += b * 3;
    result -= a / 2;
    result -= b / 3;
    if result > 100 {
        result = result % 100;
    }
    if result < 0 {
        result = result.abs();
    }
    result += 42;
    result *= 2;
    result
}

// TEST CASE 4: Dead Code (Never Called)
// Expected: Should be detected as unused
fn completely_unused_function() -> String {
    "This function is never called anywhere".to_string()
}

fn another_unused_helper(x: i32) -> i32 {
    x * 2 + 42
}

struct UnusedStruct {
    unused_field: String,
}

impl UnusedStruct {
    fn unused_method(&self) -> &str {
        &self.unused_field
    }
}

// TEST CASE 5: Tight Coupling
// Expected: Should detect tight coupling between these structs
pub struct TightlyCoupledA {
    pub direct_reference: TightlyCoupledB,
}

pub struct TightlyCoupledB {
    pub back_reference: Option<Box<TightlyCoupledA>>,
}

impl TightlyCoupledA {
    pub fn new() -> Self {
        Self {
            direct_reference: TightlyCoupledB::new(),
        }
    }

    pub fn call_b_directly(&self) {
        self.direct_reference.call_a_directly();
    }
}

impl TightlyCoupledB {
    pub fn new() -> Self {
        Self {
            back_reference: None,
        }
    }

    pub fn call_a_directly(&self) {
        if let Some(ref a) = self.back_reference {
            // Circular dependency
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_god_object() {
        let obj = MassiveGodObject::new();
        assert_eq!(obj.method1(), 1);
    }

    #[test]
    fn test_long_method() {
        let result = extremely_long_and_complex_method(50);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicates() {
        assert_eq!(
            duplicate_calculation_block_1(10, 20),
            duplicate_calculation_block_2(10, 20)
        );
    }
}
