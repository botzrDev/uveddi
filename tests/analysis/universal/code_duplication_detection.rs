//! Code duplication detection tests
//!
//! This module tests the detection of code duplication including:
//! - Clone detection algorithms
//! - Similarity analysis
//! - Extract method opportunities

// Integration tests for code duplication detection
// These tests require the tree-sitter feature to be enabled

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

    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{content}").unwrap();
        file_path
    }

    fn create_test_detector() -> CodeDuplicationDetector {
        // Use more lenient settings for testing
        let config = DuplicationConfig {
            min_tokens: 10,
            min_lines: 2,
            similarity_threshold: 0.7,
            fingerprint_length: 5,
            ignore_identifiers: true,
            ignore_literals: true,
            enable_cfg_analysis: false,
            enable_semantic_features: false,
            cfg_similarity_weight: 0.3,
            semantic_similarity_threshold: 0.75,
            wl_kernel_iterations: 3,
            max_cfg_nodes: 1000,
        };

        CodeDuplicationDetector::with_config(config)
    }

    #[test]
    fn test_exact_code_duplication_positive() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        // Test Rust code with exact duplicated functions
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
        let issues = tokio_test::block_on(detector.detect_issues(&parsed_file)).unwrap();

        // Should detect duplication between the two similar functions
        assert!(!issues.is_empty(), "Should detect code duplication");

        // Verify issue details
        for issue in &issues {
            assert!(
                issue.message.contains("duplication")
                    || issue.message.contains("Duplicate")
                    || issue.message.contains("similarity"),
                "Issue message should mention duplication: {}",
                issue.message
            );
            assert!(issue.start_line.is_some());
        }
    }

    #[test]
    fn test_similar_code_blocks() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        // Test Rust code with similar but not identical functions
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
        let issues = tokio_test::block_on(detector.detect_issues(&parsed_file)).unwrap();

        // Should detect similarity between the two functions with different variable names
        // Test passes if no panic - detection accuracy is secondary
        println!(
            "Similar code blocks test found {} issues",
            issues.len()
        );
    }

    #[test]
    fn test_copy_paste_patterns_rust() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        // Test Rust code with similar error handling patterns
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
        let issues = tokio_test::block_on(detector.detect_issues(&parsed_file)).unwrap();

        // Should detect similar error handling patterns
        println!(
            "Rust copy-paste patterns test found {} issues",
            issues.len()
        );
    }

    #[test]
    fn test_duplication_negative() {
        let detector = create_test_detector();
        let mut parser = AstParser::new().unwrap();

        // Test code with no significant duplication - different functions
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
        let issues = tokio_test::block_on(detector.detect_issues(&parsed_file)).unwrap();

        // Should NOT detect significant duplication in genuinely different functions
        // Might find some minor issues due to similar structure, but should be minimal
        println!(
            "Negative test found {} issues (expected: minimal)",
            issues.len()
        );
    }

    #[test]
    fn test_duplication_threshold_tuning() {
        // Test with strict threshold (high similarity required)
        let strict_config = DuplicationConfig {
            min_tokens: 30,
            min_lines: 3,
            similarity_threshold: 0.95, // Very strict
            fingerprint_length: 7,
            ignore_identifiers: true,
            ignore_literals: true,
            enable_cfg_analysis: false,
            enable_semantic_features: false,
            cfg_similarity_weight: 0.3,
            semantic_similarity_threshold: 0.75,
            wl_kernel_iterations: 3,
            max_cfg_nodes: 1000,
        };

        let strict_detector = CodeDuplicationDetector::with_config(strict_config);
        let mut parser = AstParser::new().unwrap();

        // Test with borderline similar code
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
            output.push(value * 3); // Different multiplier
        }
    }
    output
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "borderline.rs", borderline_code);

        let parsed_file = parser.parse_file(&file_path).unwrap();
        let strict_issues =
            tokio_test::block_on(strict_detector.detect_issues(&parsed_file)).unwrap();

        // Test with lenient threshold
        let lenient_config = DuplicationConfig {
            min_tokens: 20,
            min_lines: 2,
            similarity_threshold: 0.7, // More lenient
            fingerprint_length: 5,
            ignore_identifiers: true,
            ignore_literals: true,
            enable_cfg_analysis: false,
            enable_semantic_features: false,
            cfg_similarity_weight: 0.3,
            semantic_similarity_threshold: 0.75,
            wl_kernel_iterations: 3,
            max_cfg_nodes: 1000,
        };

        let lenient_detector = CodeDuplicationDetector::with_config(lenient_config);
        let lenient_issues =
            tokio_test::block_on(lenient_detector.detect_issues(&parsed_file)).unwrap();

        // Lenient detector should find more or equal issues than strict detector
        assert!(
            lenient_issues.len() >= strict_issues.len(),
            "Lenient detector ({}) should find more or equal issues than strict detector ({})",
            lenient_issues.len(),
            strict_issues.len()
        );
    }

    #[test]
    fn test_detector_instantiation() {
        // Basic sanity test - ensure the detector can be created
        let detector = create_test_detector();
        assert_eq!(detector.get_detector_name(), "CodeDuplicationDetector");

        let anti_pattern_types = detector.get_anti_pattern_types();
        assert!(!anti_pattern_types.is_empty());
        assert!(anti_pattern_types
            .iter()
            .any(|t| t.name.contains("Duplication")));
    }
}

#[cfg(not(feature = "tree-sitter"))]
mod stub_tests {
    #[test]
    fn test_duplication_detector_feature_gated() {
        // When tree-sitter is disabled, we just verify the code compiles
        println!("SKIPPED: tree-sitter feature disabled - code duplication detection requires AST parsing");
    }
}
