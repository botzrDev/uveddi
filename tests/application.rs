use std::path::PathBuf;
use tempfile::tempdir;
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};

#[test]
fn test_orchestrator_creation() {
    let orchestrator = AnalysisOrchestrator::new();
    assert!(orchestrator.is_ok());
}

#[test]
fn test_orchestrator_default() {
    let _orchestrator = AnalysisOrchestrator::default();
    // If this doesn't panic, the default implementation works
}

#[test]
fn test_analysis_config_creation() {
    let config = AnalysisConfig {
        target_path: PathBuf::from("/tmp"),
        output_format: "json".to_string(),
        output_file: Some(PathBuf::from("output.json")),
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
    };

    assert_eq!(config.target_path, PathBuf::from("/tmp"));
    assert_eq!(config.output_format, "json");
    assert_eq!(config.output_file, Some(PathBuf::from("output.json")));
    assert!(!config.enable_ai);
}

#[tokio::test]
async fn test_execute_analysis_nonexistent_path() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_nonexistent.db");
    let mut orchestrator = AnalysisOrchestrator::with_db_path(&db_path).unwrap();

    let config = AnalysisConfig {
        target_path: PathBuf::from("/nonexistent/path"),
        output_format: "json".to_string(),
        output_file: None,
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
    };

    let result = orchestrator.execute_analysis(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_analysis_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_empty.db");
    let mut orchestrator = AnalysisOrchestrator::with_db_path(&db_path).unwrap();

    let analysis_dir = tempdir().unwrap();
    let config = AnalysisConfig {
        target_path: analysis_dir.path().to_path_buf(),
        output_format: "json".to_string(),
        output_file: None,
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
    };

    let result = orchestrator.execute_analysis(config).await;
    assert!(result.is_ok());

    let report = result.unwrap();
    assert_eq!(report.metadata.files_analyzed, 0);
    assert_eq!(report.metadata.issues_found, 0);
    assert!(!report.metadata.ai_enhanced);
}
