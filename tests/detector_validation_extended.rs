// Extended Validation Test Cases for Uveddi Detectors
// Purpose: Comprehensive testing of CodeDuplicationDetector and MagicValuesDetector
// DO NOT REFACTOR - These are deliberately designed to trigger specific detections

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use std::collections::HashMap;

// ============================================================================
// CODE DUPLICATION DETECTOR TESTS
// ============================================================================
// Requirements: min_tokens=50, min_lines=10, similarity_threshold=0.8

/// Test data structure for duplication tests
#[derive(Debug, Clone)]
pub struct ProcessedData {
    pub category_low: Vec<i32>,
    pub category_medium: Vec<i32>,
    pub category_high: Vec<i32>,
    pub total_count: usize,
    pub mean: f64,
    pub variance: f64,
    pub standard_deviation: f64,
    pub median: f64,
    pub mode: Option<i32>,
}

impl ProcessedData {
    pub fn new() -> Self {
        Self {
            category_low: Vec::new(),
            category_medium: Vec::new(),
            category_high: Vec::new(),
            total_count: 0,
            mean: 0.0,
            variance: 0.0,
            standard_deviation: 0.0,
            median: 0.0,
            mode: None,
        }
    }
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData,
    OutOfRange,
    InsufficientData,
    CalculationError,
}

// TEST CASE 1: Type-1 Clone (Exact Duplicate) - 80+ lines
// Expected: Should be detected as high-severity duplication
pub fn complex_data_processing_algorithm_v1(input_data: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut result = ProcessedData::new();

    // Phase 1: Input Validation (15 lines)
    if input_data.is_empty() {
        return Err(ProcessingError::InsufficientData);
    }

    for item in &input_data {
        if *item < 0 {
            return Err(ProcessingError::InvalidData);
        }
        if *item > 10000 {
            return Err(ProcessingError::OutOfRange);
        }
    }

    // Phase 2: Statistical Analysis (25 lines)
    let sum: i32 = input_data.iter().sum();
    let count = input_data.len();
    let mean = sum as f64 / count as f64;

    // Calculate variance
    let variance = input_data
        .iter()
        .map(|x| (*x as f64 - mean).powi(2))
        .sum::<f64>()
        / count as f64;

    let standard_deviation = variance.sqrt();

    // Calculate median
    let mut sorted_data = input_data.clone();
    sorted_data.sort();
    let median = if count % 2 == 0 {
        let mid = count / 2;
        (sorted_data[mid - 1] + sorted_data[mid]) as f64 / 2.0
    } else {
        sorted_data[count / 2] as f64
    };

    // Calculate mode (most frequent value)
    let mut frequency_map: HashMap<i32, usize> = HashMap::new();
    for item in &input_data {
        *frequency_map.entry(*item).or_insert(0) += 1;
    }

    let mode = frequency_map
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(value, _)| *value);

    // Phase 3: Data Categorization (20 lines)
    for item in &input_data {
        match item {
            0..=100 => {
                result.category_low.push(*item);
            }
            101..=500 => {
                result.category_medium.push(*item);
            }
            501..=10000 => {
                result.category_high.push(*item);
            }
            _ => {
                return Err(ProcessingError::OutOfRange);
            }
        }
    }

    // Phase 4: Category Processing (15 lines)
    result.category_low.sort();
    result.category_low.dedup();

    result.category_medium.sort();
    result.category_medium.dedup();

    result.category_high.sort();
    result.category_high.dedup();

    // Phase 5: Final Aggregation (10 lines)
    result.total_count = count;
    result.mean = mean;
    result.variance = variance;
    result.standard_deviation = standard_deviation;
    result.median = median;
    result.mode = mode;

    Ok(result)
}

// TEST CASE 2: Type-1 Clone (EXACT DUPLICATE of above)
// Expected: Should be detected with ~100% similarity
pub fn complex_data_processing_algorithm_v2(input_data: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut result = ProcessedData::new();

    // Phase 1: Input Validation (15 lines)
    if input_data.is_empty() {
        return Err(ProcessingError::InsufficientData);
    }

    for item in &input_data {
        if *item < 0 {
            return Err(ProcessingError::InvalidData);
        }
        if *item > 10000 {
            return Err(ProcessingError::OutOfRange);
        }
    }

    // Phase 2: Statistical Analysis (25 lines)
    let sum: i32 = input_data.iter().sum();
    let count = input_data.len();
    let mean = sum as f64 / count as f64;

    // Calculate variance
    let variance = input_data
        .iter()
        .map(|x| (*x as f64 - mean).powi(2))
        .sum::<f64>()
        / count as f64;

    let standard_deviation = variance.sqrt();

    // Calculate median
    let mut sorted_data = input_data.clone();
    sorted_data.sort();
    let median = if count % 2 == 0 {
        let mid = count / 2;
        (sorted_data[mid - 1] + sorted_data[mid]) as f64 / 2.0
    } else {
        sorted_data[count / 2] as f64
    };

    // Calculate mode (most frequent value)
    let mut frequency_map: HashMap<i32, usize> = HashMap::new();
    for item in &input_data {
        *frequency_map.entry(*item).or_insert(0) += 1;
    }

    let mode = frequency_map
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(value, _)| *value);

    // Phase 3: Data Categorization (20 lines)
    for item in &input_data {
        match item {
            0..=100 => {
                result.category_low.push(*item);
            }
            101..=500 => {
                result.category_medium.push(*item);
            }
            501..=10000 => {
                result.category_high.push(*item);
            }
            _ => {
                return Err(ProcessingError::OutOfRange);
            }
        }
    }

    // Phase 4: Category Processing (15 lines)
    result.category_low.sort();
    result.category_low.dedup();

    result.category_medium.sort();
    result.category_medium.dedup();

    result.category_high.sort();
    result.category_high.dedup();

    // Phase 5: Final Aggregation (10 lines)
    result.total_count = count;
    result.mean = mean;
    result.variance = variance;
    result.standard_deviation = standard_deviation;
    result.median = median;
    result.mode = mode;

    Ok(result)
}

// TEST CASE 3: Type-2 Clone (Renamed identifiers) - 60+ lines
// Expected: Should be detected with ~90-95% similarity (identifiers normalized)
pub fn analyze_numeric_dataset(numbers: Vec<i32>) -> Result<ProcessedData, ProcessingError> {
    let mut output = ProcessedData::new();

    // Validation phase
    if numbers.is_empty() {
        return Err(ProcessingError::InsufficientData);
    }

    for value in &numbers {
        if *value < 0 {
            return Err(ProcessingError::InvalidData);
        }
        if *value > 10000 {
            return Err(ProcessingError::OutOfRange);
        }
    }

    // Statistical computation
    let total: i32 = numbers.iter().sum();
    let num_elements = numbers.len();
    let average = total as f64 / num_elements as f64;

    let var = numbers
        .iter()
        .map(|n| (*n as f64 - average).powi(2))
        .sum::<f64>()
        / num_elements as f64;

    let std_dev = var.sqrt();

    // Median calculation
    let mut sorted_numbers = numbers.clone();
    sorted_numbers.sort();
    let mid_value = if num_elements % 2 == 0 {
        let middle = num_elements / 2;
        (sorted_numbers[middle - 1] + sorted_numbers[middle]) as f64 / 2.0
    } else {
        sorted_numbers[num_elements / 2] as f64
    };

    // Mode calculation
    let mut freq_table: HashMap<i32, usize> = HashMap::new();
    for val in &numbers {
        *freq_table.entry(*val).or_insert(0) += 1;
    }

    let most_common = freq_table
        .iter()
        .max_by_key(|(_, freq)| *freq)
        .map(|(val, _)| *val);

    // Categorize data
    for val in &numbers {
        match val {
            0..=100 => {
                output.category_low.push(*val);
            }
            101..=500 => {
                output.category_medium.push(*val);
            }
            501..=10000 => {
                output.category_high.push(*val);
            }
            _ => {
                return Err(ProcessingError::OutOfRange);
            }
        }
    }

    // Process categories
    output.category_low.sort();
    output.category_low.dedup();

    output.category_medium.sort();
    output.category_medium.dedup();

    output.category_high.sort();
    output.category_high.dedup();

    // Final results
    output.total_count = num_elements;
    output.mean = average;
    output.variance = var;
    output.standard_deviation = std_dev;
    output.median = mid_value;
    output.mode = most_common;

    Ok(output)
}

// ============================================================================
// MAGIC VALUES DETECTOR TESTS
// ============================================================================

#[derive(Debug)]
pub enum UserError {
    TooYoung,
    InsufficientBalance,
    Unauthorized,
    Timeout,
}

pub struct DatabaseConnection {
    timeout: u32,
    max_retries: u32,
    port: u16,
}

impl DatabaseConnection {
    pub fn new() -> Self {
        Self {
            timeout: 0,
            max_retries: 0,
            port: 0,
        }
    }

    pub fn set_timeout(&mut self, seconds: u32) {
        self.timeout = seconds;
    }

    pub fn connect(&mut self, host: &str, port: u16) -> Result<(), UserError> {
        self.port = port;
        Ok(())
    }
}

// TEST CASE 4: Magic Values - High Severity (Comparisons and Function Args)
// Expected: Should detect multiple magic values with HIGH severity
pub fn process_user_account_with_magic_values(age: i32, balance: f64, role: &str) -> Result<(), UserError> {
    // HIGH SEVERITY: Magic number in comparison
    if age < 18 {  // MAGIC: 18 (legal age threshold)
        return Err(UserError::TooYoung);
    }

    // HIGH SEVERITY: Magic number in comparison
    if balance < 100.50 {  // MAGIC: 100.50 (minimum balance)
        return Err(UserError::InsufficientBalance);
    }

    // HIGH SEVERITY: Magic string in comparison
    if role == "administrator" {  // MAGIC: "administrator" string
        println!("Admin access granted");
    }

    // HIGH SEVERITY: Magic string in comparison
    if role != "guest" {  // MAGIC: "guest" string
        println!("Registered user");
    }

    // HIGH SEVERITY: Magic number as function argument
    let mut db = DatabaseConnection::new();
    db.set_timeout(300);  // MAGIC: 300 seconds timeout

    // HIGH SEVERITY: Magic number as function argument
    db.connect("localhost", 5432)?;  // MAGIC: 5432 (PostgreSQL port)

    Ok(())
}

// TEST CASE 5: Magic Values - Medium Severity (Assignments and Returns)
// Expected: Should detect magic values with MEDIUM severity
pub fn configure_system_settings() -> (u32, f64, String) {
    // MEDIUM SEVERITY: Magic number in assignment
    let retry_count = 5;  // MAGIC: 5 retries

    // MEDIUM SEVERITY: Magic number in assignment
    let timeout_seconds = 3600;  // MAGIC: 3600 (1 hour in seconds)

    // MEDIUM SEVERITY: Magic number in assignment
    let cache_size = 500;  // MAGIC: 500 MB cache size

    // MEDIUM SEVERITY: Magic float in assignment
    let threshold = 0.75;  // MAGIC: 0.75 threshold

    // MEDIUM SEVERITY: Magic string in assignment
    let environment = "production";  // MAGIC: "production" string

    // MEDIUM SEVERITY: Magic values in return statement
    (retry_count, threshold, environment.to_string())
}

// TEST CASE 6: Magic Values - Should NOT Detect (Allowed/Excluded)
// Expected: Should NOT detect these values (const declarations, powers of 2, allowed values)
pub fn values_that_should_not_be_detected() -> i32 {
    // SHOULD NOT DETECT: Const declaration (excluded by default)
    const MAX_ATTEMPTS: i32 = 3;
    const API_VERSION: i32 = 42;
    const BUFFER_SIZE: usize = 4096;

    // SHOULD NOT DETECT: Powers of 2 (heuristic exclusion)
    let small_buffer = 1024;  // 2^10
    let large_buffer = 8192;  // 2^13
    let page_size = 4096;     // 2^12

    // SHOULD NOT DETECT: Allowed values (0, 1, -1, 2)
    let index = 0;
    let counter = 1;
    let flag = -1;
    let double = 2;

    // SHOULD NOT DETECT: Array index (0-99 range)
    let items = vec![10, 20, 30, 40, 50];
    let first = items[0];
    let second = items[1];
    let tenth = items[9];

    MAX_ATTEMPTS + small_buffer as i32 + index
}

// TEST CASE 7: Magic Values - Mixed Contexts
// Expected: Different severity levels based on context
pub fn mixed_magic_value_contexts(count: i32) -> Result<String, UserError> {
    // HIGH: Magic in comparison
    if count > 100 {  // MAGIC: 100 threshold
        return Err(UserError::Timeout);
    }

    // MEDIUM: Magic in assignment
    let max_items = 50;  // MAGIC: 50 max items

    // HIGH: Magic in function call
    process_batch(count, 25);  // MAGIC: 25 batch size

    // MEDIUM: Magic in field initialization
    let connection = DatabaseConnection {
        timeout: 60,      // MAGIC: 60 seconds
        max_retries: 3,   // Borderline - might be detected
        port: 8080,       // MAGIC: 8080 HTTP alternate port
    };

    Ok("Success".to_string())
}

fn process_batch(count: i32, batch_size: i32) {
    // Helper function for above test
}

// TEST CASE 8: Magic Strings - Various Contexts
// Expected: Detect non-trivial magic strings
pub fn magic_string_examples(mode: &str) -> String {
    // HIGH: Magic string in comparison
    if mode == "debug" {  // MAGIC: "debug"
        println!("Debug mode enabled");
    }

    // MEDIUM: Magic string in assignment
    let api_endpoint = "https://api.example.com/v1/users";  // MAGIC: URL

    // MEDIUM: Magic string in return
    match mode {
        "production" => "prod".to_string(),  // MAGIC: "production", "prod"
        "staging" => "stage".to_string(),    // MAGIC: "staging", "stage"
        _ => "development".to_string(),      // MAGIC: "development"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplication_exact_clone() {
        let data = vec![1, 2, 3, 4, 5];
        let result1 = complex_data_processing_algorithm_v1(data.clone());
        let result2 = complex_data_processing_algorithm_v2(data.clone());

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Both should produce identical results
        let r1 = result1.unwrap();
        let r2 = result2.unwrap();
        assert_eq!(r1.total_count, r2.total_count);
        assert_eq!(r1.mean, r2.mean);
    }

    #[test]
    fn test_duplication_renamed_clone() {
        let data = vec![1, 2, 3, 4, 5];
        let result = analyze_numeric_dataset(data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_magic_values_detection() {
        let result = process_user_account_with_magic_values(25, 200.0, "user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_allowed_values() {
        let result = values_that_should_not_be_detected();
        assert!(result > 0);
    }
}
