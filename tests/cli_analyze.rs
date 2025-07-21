use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use uveddi::cli::analyze_command::AnalyzeCommand;

fn setup() {
    // Clean up any previous cache files
    if std::path::Path::new("uveddi_cache.db").exists() {
        let _ = fs::remove_file("uveddi_cache.db");
    }
    if std::path::Path::new("uveddi.db").exists() {
        let _ = fs::remove_file("uveddi.db");
    }
}

#[test]
fn test_analyze_command_creation() {
    setup();

    let command = AnalyzeCommand {
        path: PathBuf::from("test_path"),
        output_format: "markdown".to_string(),
        output: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    assert_eq!(command.path, PathBuf::from("test_path"));
    assert_eq!(command.output_format, "markdown");
    assert_eq!(command.output, None);
    assert!(!command.enable_ai);
    assert_eq!(command.ollama_api_url, None);
    assert_eq!(command.ollama_model, None);
}

#[test]
fn test_analyze_command_with_all_options() {
    setup();

    let output_path = PathBuf::from("output.md");
    let command = AnalyzeCommand {
        path: PathBuf::from("src/"),
        output_format: "json".to_string(),
        output: Some(output_path.clone()),
        enable_ai: true,
        ollama_api_url: Some("http://localhost:11434".to_string()),
        ollama_model: Some("llama2".to_string()),
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    assert_eq!(command.path, PathBuf::from("src/"));
    assert_eq!(command.output_format, "json");
    assert_eq!(command.output, Some(output_path));
    assert!(command.enable_ai);
    assert_eq!(
        command.ollama_api_url,
        Some("http://localhost:11434".to_string())
    );
    assert_eq!(command.ollama_model, Some("llama2".to_string()));
}

#[tokio::test]
async fn test_execute_with_empty_directory() {
    setup();

    let temp_dir = tempdir().unwrap();
    let command = AnalyzeCommand {
        path: temp_dir.path().to_path_buf(),
        output_format: "markdown".to_string(),
        output: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let result = command.execute().await;
    assert!(
        result.is_ok(),
        "Should successfully analyze empty directory"
    );
}

#[tokio::test]
async fn test_execute_with_simple_rust_file() {
    setup();

    let temp_dir = tempdir().unwrap();
    let rust_file = temp_dir.path().join("main.rs");

    // Create a simple Rust file
    fs::write(
        &rust_file,
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    let command = AnalyzeCommand {
        path: temp_dir.path().to_path_buf(),
        output_format: "markdown".to_string(),
        output: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let result = command.execute().await;
    assert!(
        result.is_ok(),
        "Should successfully analyze directory with Rust file"
    );
}

#[tokio::test]
async fn test_execute_with_output_file() {
    setup();

    let temp_dir = tempdir().unwrap();
    let output_file = temp_dir.path().join("analysis_output.md");

    let command = AnalyzeCommand {
        path: temp_dir.path().to_path_buf(),
        output_format: "markdown".to_string(),
        output: Some(output_file.clone()),
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let result = command.execute().await;
    assert!(
        result.is_ok(),
        "Should successfully execute with output file"
    );

    // Check that the output file was created
    assert!(output_file.exists(), "Output file should be created");
}

#[tokio::test]
async fn test_execute_with_json_format() {
    setup();

    let temp_dir = tempdir().unwrap();
    let command = AnalyzeCommand {
        path: temp_dir.path().to_path_buf(),
        output_format: "json".to_string(),
        output: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let result = command.execute().await;
    assert!(
        result.is_ok(),
        "Should successfully execute with JSON format"
    );
}

#[tokio::test]
async fn test_execute_nonexistent_path() {
    setup();

    let command = AnalyzeCommand {
        path: PathBuf::from("/nonexistent/path/to/analyze"),
        output_format: "markdown".to_string(),
        output: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: None,
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: None,
        large_classes_max_loc: None,
        large_classes_max_methods: None,
        large_classes_max_fields: None,
        large_classes_max_complexity: None,
        large_classes_max_lcom: None,
        large_classes_ignore_patterns: None,
        large_classes_min_severity: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let result = command.execute().await;
    assert!(result.is_err(), "Should fail with nonexistent path");
}
