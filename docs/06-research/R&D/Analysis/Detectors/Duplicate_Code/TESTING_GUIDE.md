# Duplicate Code Detector - Testing Guide

## Overview

This guide covers the comprehensive testing strategy for the Duplicate Code Detector, including test categories, implementation patterns, and best practices for ensuring robust detection capabilities.

## Test Suite Structure

The test suite is organized into several categories to ensure comprehensive coverage:

### Location
- **Main Test File**: `src/analysis/tests/universal/code_duplication_detection.rs`
- **Helper Modules**: `src/analysis/tests/universal/mod.rs`
- **Integration Tests**: `tests/` directory (for end-to-end scenarios)

### Test Categories

#### 1. Exact Duplication Tests
Tests for Type-1 clones (identical code except for whitespace and comments).

```rust
#[test]
fn test_exact_duplication() {
    let detector = create_test_detector();
    
    let duplicate_code = r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Sum: {}", result);
    result
}
"#;
    
    create_test_file(duplicate_code, "test_file1.rs").unwrap();
    create_test_file(duplicate_code, "test_file2.rs").unwrap();
    
    let files = vec!["test_file1.rs".to_string(), "test_file2.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    assert_eq!(clones.len(), 1);
    assert_eq!(clones[0].clone_type, CloneType::Type1);
    assert!(clones[0].similarity_score >= 0.95);
}
```

#### 2. Similar Code Tests
Tests for Type-2 clones (identical structure with different identifiers/literals).

```rust
#[test]
fn test_similar_code_different_variables() {
    let detector = create_test_detector();
    
    let code1 = r#"
fn process_data(input: i32, factor: i32) -> i32 {
    let temp = input * factor;
    println!("Processing: {}", temp);
    temp + 100
}
"#;
    
    let code2 = r#"
fn handle_values(x: i32, multiplier: i32) -> i32 {
    let result = x * multiplier;
    println!("Processing: {}", result);
    result + 100
}
"#;
    
    create_test_file(code1, "test_file1.rs").unwrap();
    create_test_file(code2, "test_file2.rs").unwrap();
    
    let files = vec!["test_file1.rs".to_string(), "test_file2.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    assert_eq!(clones.len(), 1);
    assert_eq!(clones[0].clone_type, CloneType::Type2);
    assert!(clones[0].similarity_score >= 0.7);
}
```

#### 3. Cross-File Duplication Tests
Tests for detecting clones across different files.

```rust
#[test]
fn test_cross_file_duplication() {
    let detector = create_test_detector();
    
    let shared_logic = r#"
fn validate_input(value: i32) -> bool {
    if value < 0 {
        return false;
    }
    if value > 1000 {
        return false;
    }
    true
}
"#;
    
    create_test_file(shared_logic, "module1.rs").unwrap();
    create_test_file(shared_logic, "module2.rs").unwrap();
    
    let files = vec!["module1.rs".to_string(), "module2.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    assert_eq!(clones.len(), 1);
    assert!(clones[0].is_cross_file());
}
```

#### 4. Language-Specific Tests
Tests for each supported programming language.

```rust
#[test]
fn test_python_duplication() {
    let detector = create_test_detector();
    
    let python_code1 = r#"
def calculate_average(numbers):
    total = sum(numbers)
    count = len(numbers)
    if count == 0:
        return 0
    return total / count
"#;
    
    let python_code2 = r#"
def compute_mean(values):
    total = sum(values)
    count = len(values)
    if count == 0:
        return 0
    return total / count
"#;
    
    create_test_file(python_code1, "test1.py").unwrap();
    create_test_file(python_code2, "test2.py").unwrap();
    
    let files = vec!["test1.py".to_string(), "test2.py".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    assert_eq!(clones.len(), 1);
    assert_eq!(clones[0].clone_type, CloneType::Type2);
}

#[test]
fn test_javascript_duplication() {
    let detector = create_test_detector();
    
    let js_code1 = r#"
function formatDate(date) {
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const day = date.getDate();
    return `${year}-${month}-${day}`;
}
"#;
    
    let js_code2 = r#"
function formatTimestamp(timestamp) {
    const year = timestamp.getFullYear();
    const month = timestamp.getMonth() + 1;
    const day = timestamp.getDate();
    return `${year}-${month}-${day}`;
}
"#;
    
    create_test_file(js_code1, "test1.js").unwrap();
    create_test_file(js_code2, "test2.js").unwrap();
    
    let files = vec!["test1.js".to_string(), "test2.js".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    assert_eq!(clones.len(), 1);
    assert_eq!(clones[0].clone_type, CloneType::Type2);
}
```

#### 5. Extract Method Opportunity Tests
Tests for identifying code that should be extracted into a common method.

```rust
#[test]
fn test_extract_method_opportunity() {
    let detector = create_test_detector();
    
    let code_with_duplication = r#"
fn process_user_data(user: &User) -> String {
    // Validation logic (duplicated)
    if user.name.is_empty() {
        return "Invalid name".to_string();
    }
    if user.email.is_empty() {
        return "Invalid email".to_string();
    }
    
    format!("User: {}", user.name)
}

fn process_admin_data(admin: &Admin) -> String {
    // Same validation logic (duplicated)
    if admin.name.is_empty() {
        return "Invalid name".to_string();
    }
    if admin.email.is_empty() {
        return "Invalid email".to_string();
    }
    
    format!("Admin: {}", admin.name)
}
"#;
    
    create_test_file(code_with_duplication, "user_module.rs").unwrap();
    
    let files = vec!["user_module.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    // Should detect the duplicated validation logic
    assert!(!clones.is_empty());
    let clone = &clones[0];
    assert!(clone.similarity_score >= 0.7);
}
```

#### 6. Threshold Tuning Tests
Tests for validating configuration parameter effects.

```rust
#[test]
fn test_similarity_threshold_tuning() {
    let similar_code1 = r#"
fn calculate_tax(amount: f64, rate: f64) -> f64 {
    let tax = amount * rate;
    println!("Tax calculated: {}", tax);
    tax
}
"#;
    
    let similar_code2 = r#"
fn compute_fee(value: f64, percentage: f64) -> f64 {
    let fee = value * percentage;
    println!("Fee computed: {}", fee);
    fee
}
"#;
    
    create_test_file(similar_code1, "test1.rs").unwrap();
    create_test_file(similar_code2, "test2.rs").unwrap();
    
    let files = vec!["test1.rs".to_string(), "test2.rs".to_string()];
    
    // Test with high threshold - should not detect as clone
    let high_threshold_config = CodeDuplicationConfig {
        similarity_threshold: 0.95,
        ..create_test_config()
    };
    let detector_high = CodeDuplicationDetector::new(high_threshold_config);
    let clones_high = detector_high.detect_duplicates(&files).unwrap();
    assert_eq!(clones_high.len(), 0);
    
    // Test with low threshold - should detect as clone
    let low_threshold_config = CodeDuplicationConfig {
        similarity_threshold: 0.6,
        ..create_test_config()
    };
    let detector_low = CodeDuplicationDetector::new(low_threshold_config);
    let clones_low = detector_low.detect_duplicates(&files).unwrap();
    assert_eq!(clones_low.len(), 1);
}

#[test]
fn test_min_tokens_threshold() {
    let small_function = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    
    create_test_file(small_function, "test1.rs").unwrap();
    create_test_file(small_function, "test2.rs").unwrap();
    
    let files = vec!["test1.rs".to_string(), "test2.rs".to_string()];
    
    // Test with high min_tokens - should not detect small function
    let high_min_config = CodeDuplicationConfig {
        min_tokens: 50,
        ..create_test_config()
    };
    let detector_high = CodeDuplicationDetector::new(high_min_config);
    let clones_high = detector_high.detect_duplicates(&files).unwrap();
    assert_eq!(clones_high.len(), 0);
    
    // Test with low min_tokens - should detect small function
    let low_min_config = CodeDuplicationConfig {
        min_tokens: 5,
        ..create_test_config()
    };
    let detector_low = CodeDuplicationDetector::new(low_min_config);
    let clones_low = detector_low.detect_duplicates(&files).unwrap();
    assert_eq!(clones_low.len(), 1);
}
```

#### 7. Negative Cases Tests
Tests to ensure the detector doesn't produce false positives.

```rust
#[test]
fn test_different_logic_no_duplication() {
    let detector = create_test_detector();
    
    let code1 = r#"
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}
"#;
    
    let code2 = r#"
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}
"#;
    
    create_test_file(code1, "test1.rs").unwrap();
    create_test_file(code2, "test2.rs").unwrap();
    
    let files = vec!["test1.rs".to_string(), "test2.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    // Should not detect as clones - different logic
    assert_eq!(clones.len(), 0);
}

#[test]
fn test_short_functions_no_duplication() {
    let detector = create_test_detector();
    
    let short_code1 = r#"
fn get_x() -> i32 {
    42
}
"#;
    
    let short_code2 = r#"
fn get_y() -> i32 {
    24
}
"#;
    
    create_test_file(short_code1, "test1.rs").unwrap();
    create_test_file(short_code2, "test2.rs").unwrap();
    
    let files = vec!["test1.rs".to_string(), "test2.rs".to_string()];
    let clones = detector.detect_duplicates(&files).unwrap();
    
    // Should not detect as clones - too short and different
    assert_eq!(clones.len(), 0);
}
```

## Test Utilities

### Helper Functions

```rust
use std::fs;
use std::path::Path;

/// Creates a test detector with configuration suitable for testing
pub fn create_test_detector() -> CodeDuplicationDetector {
    let config = CodeDuplicationConfig {
        min_tokens: 5,        // Low threshold for testing small functions
        similarity_threshold: 0.6,
        hash_window_size: 3,
        min_shared_hashes: 2,
        normalize_identifiers: true,
        normalize_literals: true,
    };
    CodeDuplicationDetector::new(config)
}

/// Creates a test configuration with specified parameters
pub fn create_test_config() -> CodeDuplicationConfig {
    CodeDuplicationConfig {
        min_tokens: 10,
        similarity_threshold: 0.7,
        hash_window_size: 4,
        min_shared_hashes: 3,
        normalize_identifiers: true,
        normalize_literals: true,
    }
}

/// Creates a temporary test file with the given content
pub fn create_test_file(content: &str, path: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

/// Cleans up test files after tests complete
pub fn cleanup_test_files(files: &[&str]) {
    for file in files {
        let _ = fs::remove_file(file);
    }
}

/// Creates a temporary directory for test files
pub fn create_test_dir(dir_name: &str) -> std::io::Result<()> {
    fs::create_dir_all(dir_name)
}
```

### Test Data Generators

```rust
/// Generates test code with specified characteristics
pub struct TestCodeGenerator;

impl TestCodeGenerator {
    /// Generates a pair of exactly identical functions
    pub fn generate_identical_functions(lang: &str) -> (String, String) {
        match lang {
            "rust" => {
                let code = r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Sum: {}", result);
    result
}
"#;
                (code.to_string(), code.to_string())
            }
            "python" => {
                let code = r#"
def calculate_sum(a, b):
    result = a + b
    print(f"Sum: {result}")
    return result
"#;
                (code.to_string(), code.to_string())
            }
            "javascript" => {
                let code = r#"
function calculateSum(a, b) {
    const result = a + b;
    console.log(`Sum: ${result}`);
    return result;
}
"#;
                (code.to_string(), code.to_string())
            }
            _ => panic!("Unsupported language: {}", lang),
        }
    }
    
    /// Generates a pair of functions with renamed variables
    pub fn generate_renamed_functions(lang: &str) -> (String, String) {
        match lang {
            "rust" => {
                let code1 = r#"
fn process_data(input: i32, factor: i32) -> i32 {
    let temp = input * factor;
    println!("Processing: {}", temp);
    temp + 100
}
"#;
                let code2 = r#"
fn handle_values(x: i32, multiplier: i32) -> i32 {
    let result = x * multiplier;
    println!("Processing: {}", result);
    result + 100
}
"#;
                (code1.to_string(), code2.to_string())
            }
            // Add other languages as needed
            _ => panic!("Unsupported language: {}", lang),
        }
    }
}
```

## Performance Testing

### Benchmark Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_large_file_performance() {
        let detector = create_test_detector();
        
        // Generate large test file
        let mut large_content = String::new();
        for i in 0..1000 {
            large_content.push_str(&format!(
                "fn function_{i}() -> i32 {{ let x = {i}; x * 2 }}\n", 
                i = i
            ));
        }
        
        create_test_file(&large_content, "large_file.rs").unwrap();
        
        let start = Instant::now();
        let files = vec!["large_file.rs".to_string()];
        let clones = detector.detect_duplicates(&files).unwrap();
        let duration = start.elapsed();
        
        println!("Large file analysis took: {:?}", duration);
        assert!(duration.as_secs() < 10); // Should complete within 10 seconds
        
        cleanup_test_files(&["large_file.rs"]);
    }
    
    #[test]
    fn test_many_files_performance() {
        let detector = create_test_detector();
        
        // Create many small files
        let mut files = Vec::new();
        for i in 0..100 {
            let filename = format!("test_file_{}.rs", i);
            let content = format!(
                "fn function_{}() -> i32 {{ let x = {}; x + 1 }}\n", 
                i, i
            );
            create_test_file(&content, &filename).unwrap();
            files.push(filename);
        }
        
        let start = Instant::now();
        let file_strings: Vec<String> = files.iter().map(|f| f.clone()).collect();
        let clones = detector.detect_duplicates(&file_strings).unwrap();
        let duration = start.elapsed();
        
        println!("Many files analysis took: {:?}", duration);
        assert!(duration.as_secs() < 5); // Should complete within 5 seconds
        
        // Cleanup
        let file_refs: Vec<&str> = files.iter().map(|f| f.as_str()).collect();
        cleanup_test_files(&file_refs);
    }
}
```

## Integration Testing

### End-to-End Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use uveddi::analysis::AnalysisEngine;
    
    #[test]
    fn test_integration_with_analysis_engine() {
        let mut engine = AnalysisEngine::new().unwrap();
        
        // Set up test configuration
        let config = CodeDuplicationConfig::default();
        engine.set_code_duplication_config(config);
        
        // Create test files
        let duplicate_code = r#"
fn validate_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }
    email.contains('@')
}
"#;
        
        create_test_file(duplicate_code, "module1.rs").unwrap();
        create_test_file(duplicate_code, "module2.rs").unwrap();
        
        // Run analysis through engine
        let files = vec!["module1.rs".to_string(), "module2.rs".to_string()];
        let results = engine.analyze_code_duplication(&files).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].clone_type, CloneType::Type1);
        
        cleanup_test_files(&["module1.rs", "module2.rs"]);
    }
}
```

## Test Best Practices

### 1. Test Isolation
- Each test should be independent and not rely on other tests
- Use temporary files with unique names to avoid conflicts
- Clean up test files after each test

### 2. Configuration Testing
- Test with different configuration parameters
- Validate threshold effects on detection results
- Test edge cases with extreme configuration values

### 3. Language Coverage
- Include tests for each supported programming language
- Test language-specific patterns and edge cases
- Validate Tree-sitter query correctness

### 4. Performance Validation
- Include performance benchmarks for large files
- Test scalability with many files
- Monitor memory usage during testing

### 5. Error Handling
- Test error conditions (malformed files, unsupported languages)
- Validate graceful degradation
- Test recovery from parse errors

## Running Tests

### Unit Tests
```bash
cargo test code_duplication_detection
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Performance Tests
```bash
cargo test --release performance_tests
```

### All Tests
```bash
cargo test
```

## Continuous Integration

### GitHub Actions Configuration
```yaml
name: Code Duplication Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run tests
      run: cargo test code_duplication_detection
    - name: Run performance tests
      run: cargo test --release performance_tests
```

This comprehensive testing guide ensures the Duplicate Code Detector maintains high quality and reliability across all supported use cases and configurations.
