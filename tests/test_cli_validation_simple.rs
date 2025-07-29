//! Simple test for CLI input validation integration

use std::path::PathBuf;
use uveddi::cli::analyze_command::AnalyzeCommand;

#[test]
fn test_cli_validation_integration() {
    // Test valid command
    let valid_command = AnalyzeCommand {
        path: PathBuf::from("./src"),
        output_format: "json".to_string(),
        output: Some(PathBuf::from("report.json")),
        enable_ai: true,
        ollama_api_url: Some("http://localhost:11434".to_string()),
        ollama_model: Some("deepseek-coder".to_string()),
        dead_code_confidence: Some(0.8),
        dead_code_library_mode: false,
        dead_code_ignore_patterns: Some(vec!["test".to_string()]),
        dead_code_keep_alive: Some(vec!["main".to_string()]),
        large_classes_max_loc: Some(500),
        large_classes_max_methods: Some(20),
        large_classes_max_fields: Some(15),
        large_classes_max_complexity: Some(50),
        large_classes_max_lcom: Some(0.8),
        large_classes_ignore_patterns: Some(vec!["generated".to_string()]),
        large_classes_min_severity: Some(25),
        enable_memory_optimization: false,
        memory_limit_gb: Some(4.0),
        memory_profile: Some("default".to_string()),
        enable_image_rendering: false,
        mermaid_only: false,
        no_fallback: false,
        rendering_service_url: "http://localhost:3000".to_string(),
        check_rendering_service: false,
        #[cfg(feature = "enterprise")]
        diagram_format: "png".to_string(),
    };

    let result = valid_command.validate_inputs();
    assert!(result.is_ok(), "Valid command should pass validation");
}

#[test]
fn test_cli_malicious_input_rejection() {
    // Test malicious command
    let malicious_command = AnalyzeCommand {
        path: PathBuf::from("'; DROP TABLE users; --"),
        output_format: "json'; DELETE FROM data; --".to_string(),
        output: Some(PathBuf::from("normal.json")),
        enable_ai: true,
        ollama_api_url: Some("javascript:alert(1)".to_string()),
        ollama_model: Some("../../../etc/passwd".to_string()),
        dead_code_confidence: Some(150.0), // Invalid range
        dead_code_library_mode: false,
        dead_code_ignore_patterns: Some(vec!["'; DROP TABLE test; --".to_string()]),
        dead_code_keep_alive: Some(vec!["normal".to_string()]),
        large_classes_max_loc: Some(500),
        large_classes_max_methods: Some(20),
        large_classes_max_fields: Some(15),
        large_classes_max_complexity: Some(50),
        large_classes_max_lcom: Some(0.8),
        large_classes_ignore_patterns: Some(vec!["generated".to_string()]),
        large_classes_min_severity: Some(25),
        enable_memory_optimization: false,
        memory_limit_gb: Some(4.0),
        memory_profile: Some("default".to_string()),
        enable_image_rendering: false,
        mermaid_only: false,
        no_fallback: false,
        rendering_service_url: "http://localhost:3000".to_string(),
        check_rendering_service: false,
        #[cfg(feature = "enterprise")]
        diagram_format: "png".to_string(),
    };

    let result = malicious_command.validate_inputs();
    assert!(result.is_err(), "Malicious command should be rejected");
}
