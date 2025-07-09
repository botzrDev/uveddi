use uveddi::analysis::detectors::anti_patterns::dead_code::DeadCodeDetector;
use uveddi::analysis::AnalysisDetector;
use uveddi::ast::tree_sitter::{AstParser, SourceLanguage};
use std::path::PathBuf;

#[test]
fn debug_dead_code_exported_symbols() {
    let rust_code = r#"
pub fn exported_function() {
    println!("This is exported");
}

fn private_unused() {
    println!("This is private and unused");
}
"#;

    let mut parser = AstParser::new().expect("Failed to create parser");
    let parsed_file = parser
        .parse_content(rust_code, &PathBuf::from("lib.rs"), SourceLanguage::Rust)
        .expect("Failed to parse Rust code");

    // Configure for library mode to NOT detect unused exported symbols
    let mut config = uveddi::analysis::detectors::anti_patterns::dead_code::DeadCodeConfig::default();
    config.library_mode = true; // Library mode - don't detect unused exports
    config.min_confidence = 0.0; // Lower threshold to see everything
    let detector = DeadCodeDetector::new(config);
    
    let issues = detector
        .detect_issues(&parsed_file)
        .expect("Failed to detect issues");

    println!("Found {} issues:", issues.len());
    for issue in &issues {
        println!("Issue: {} (severity: {})", issue.description, issue.severity);
    }

    // Should detect private_unused but not exported_function
    let private_detected = issues.iter().any(|issue| issue.description.contains("'private_unused'"));
    let exported_detected = issues.iter().any(|issue| issue.description.contains("'exported_function'"));
    
    println!("Private detected: {}, Exported detected: {}", private_detected, exported_detected);
}
