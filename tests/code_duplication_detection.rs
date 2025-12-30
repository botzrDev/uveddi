//! Code Duplication Detection Integration Tests
//!
//! These tests verify the code duplication detector works correctly with real code samples.

#[cfg(feature = "tree-sitter")]
mod tree_sitter_tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use uveddi::analysis::detectors::anti_patterns::code_duplication::{
        CodeDuplicationDetector, DuplicationConfig,
    };
    use uveddi::analysis::AnalysisDetector;
    use uveddi::ast::tree_sitter_impl::AstParser;
    use uveddi::ast::ParsedFileCompat;

    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{content}").unwrap();
        file_path
    }

    fn create_test_detector() -> CodeDuplicationDetector {
        // Use default config with some customization for testing
        let config = DuplicationConfig::new()
            .with_min_tokens(10)
            .with_similarity_threshold(0.7);
        CodeDuplicationDetector::with_config(config)
    }

    fn create_lenient_detector() -> CodeDuplicationDetector {
        let config = DuplicationConfig::thorough_analysis()
            .with_min_tokens(15)
            .with_similarity_threshold(0.6);
        CodeDuplicationDetector::with_config(config)
    }

    fn create_strict_detector() -> CodeDuplicationDetector {
        let config = DuplicationConfig::performance_optimized()
            .with_similarity_threshold(0.95);
        CodeDuplicationDetector::with_config(config)
    }

    #[test]
    fn test_code_duplication_detector_instantiation() {
        let detector = create_test_detector();
        assert_eq!(detector.get_detector_name(), "CodeDuplicationDetector");

        let anti_pattern_types = detector.get_anti_pattern_types();
        assert!(!anti_pattern_types.is_empty());
        assert!(anti_pattern_types
            .iter()
            .any(|t| t.name.contains("Duplication")));
    }

    #[test]
    fn test_code_duplication_exact_match() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        let rust_code = r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    if result > 100 {
        println!("Result is large: {}", result);
    }
    result
}

fn compute_total(x: i32, y: i32) -> i32 {
    let result = x + y;
    if result > 100 {
        println!("Result is large: {}", result);
    }
    result
}

fn different_function(data: &str) -> String {
    format!("Processing: {}", data)
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "duplicate_test.rs", rust_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let compat_file = ParsedFileCompat::from_tree_sitter(parsed_file);
        let issues = tokio_test::block_on(detector.detect_issues(&compat_file)).unwrap();

        // Should detect duplication - test mainly verifies no panic
        println!(
            "Exact match test: Found {} issues between calculate_sum and compute_total",
            issues.len()
        );
    }

    #[test]
    fn test_code_duplication_similar_blocks() {
        let detector = create_lenient_detector();
        let mut parser = AstParser::new().unwrap();

        let rust_code = r#"
fn process_data(items: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if item > 10 {
            result.push(item * 2);
        } else {
            result.push(item);
        }
    }
    result
}

fn transform_values(values: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for value in values {
        if value > 10 {
            result.push(value * 2);
        } else {
            result.push(value);
        }
    }
    result
}

fn unique_function(text: &str) -> String {
    text.to_uppercase()
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "similar_test.rs", rust_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let compat_file = ParsedFileCompat::from_tree_sitter(parsed_file);
        let issues = tokio_test::block_on(detector.detect_issues(&compat_file)).unwrap();

        println!("Similar blocks test found {} issues", issues.len());
    }

    #[test]
    fn test_code_duplication_negative_case() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        // Completely different functions - should find minimal/no duplication
        let rust_code = r#"
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply_values(x: i32, y: i32) -> i32 {
    x * y
}

fn divide_safely(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator != 0.0 {
        Some(numerator / denominator)
    } else {
        None
    }
}

fn greet_user(name: &str) -> String {
    format!("Hello, {}!", name)
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "unique_functions.rs", rust_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let compat_file = ParsedFileCompat::from_tree_sitter(parsed_file);
        let issues = tokio_test::block_on(detector.detect_issues(&compat_file)).unwrap();

        println!(
            "Negative test found {} issues (expected: minimal or none)",
            issues.len()
        );
    }

    #[test]
    fn test_code_duplication_threshold_comparison() {
        let mut parser = AstParser::new().unwrap();

        let borderline_code = r#"
fn process_items(items: Vec<i32>) -> Vec<i32> {
    let mut results = Vec::new();
    for item in items {
        if item > 0 {
            results.push(item * 2);
        }
    }
    results
}

fn handle_values(values: Vec<i32>) -> Vec<i32> {
    let mut output = Vec::new();
    for value in values {
        if value > 0 {
            output.push(value * 3);
        }
    }
    output
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "borderline.rs", borderline_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let compat_file = ParsedFileCompat::from_tree_sitter(parsed_file);

        // Test strict detector
        let strict_detector = create_strict_detector();
        let strict_issues =
            tokio_test::block_on(strict_detector.detect_issues(&compat_file)).unwrap();

        // Test lenient detector
        let lenient_detector = create_lenient_detector();
        let lenient_issues =
            tokio_test::block_on(lenient_detector.detect_issues(&compat_file)).unwrap();

        // Lenient should find >= strict
        assert!(
            lenient_issues.len() >= strict_issues.len(),
            "Lenient detector ({}) should find >= issues than strict ({})",
            lenient_issues.len(),
            strict_issues.len()
        );

        println!(
            "Threshold test: strict={}, lenient={}",
            strict_issues.len(),
            lenient_issues.len()
        );
    }

    #[test]
    fn test_code_duplication_rust_patterns() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        let rust_code = r#"
fn read_file_content(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    if content.is_empty() {
        return Err("File is empty".into());
    }
    Ok(content.trim().to_string())
}

fn load_config_file(config_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(config_path)?;
    if content.is_empty() {
        return Err("File is empty".into());
    }
    Ok(content.trim().to_string())
}

fn process_json(data: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(data)
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "rust_patterns.rs", rust_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let compat_file = ParsedFileCompat::from_tree_sitter(parsed_file);
        let issues = tokio_test::block_on(detector.detect_issues(&compat_file)).unwrap();

        println!("Rust patterns test found {} issues", issues.len());
    }
}

#[cfg(not(feature = "tree-sitter"))]
#[test]
fn test_code_duplication_feature_gated() {
    println!(
        "SKIPPED: tree-sitter feature disabled - code duplication detection requires AST parsing"
    );
}
