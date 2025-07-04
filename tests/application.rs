use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};
use std::path::PathBuf;
use tempfile::tempdir;

fn setup() {
    // Clean up previous test runs
    let cache_files = ["uveddi_cache.db", "uveddi.db"];
    for file in &cache_files {
        if std::path::Path::new(file).exists() {
            let _ = std::fs::remove_file(file);
        }
    }
}

#[test]
fn test_orchestrator_creation() {
    setup();
    let orchestrator = AnalysisOrchestrator::new();
    assert!(orchestrator.is_ok());
}

#[test]
fn test_orchestrator_default() {
    setup();
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
    };
    
    assert_eq!(config.target_path, PathBuf::from("/tmp"));
    assert_eq!(config.output_format, "json");
    assert_eq!(config.output_file, Some(PathBuf::from("output.json")));
    assert!(!config.enable_ai);
}

#[tokio::test]
async fn test_execute_analysis_nonexistent_path() {
    setup();
    let mut orchestrator = AnalysisOrchestrator::new().unwrap();
    
    let config = AnalysisConfig {
        target_path: PathBuf::from("/nonexistent/path"),
        output_format: "json".to_string(),
        output_file: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
    };
    
    let result = orchestrator.execute_analysis(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_analysis_empty_directory() {
    setup();
    let mut orchestrator = AnalysisOrchestrator::new().unwrap();
    
    let temp_dir = tempdir().unwrap();
    let config = AnalysisConfig {
        target_path: temp_dir.path().to_path_buf(),
        output_format: "json".to_string(),
        output_file: None,
        enable_ai: false,
        ollama_api_url: None,
        ollama_model: None,
    };
    
    let result = orchestrator.execute_analysis(config).await;
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert_eq!(report.metadata.files_analyzed, 0);
    assert_eq!(report.metadata.issues_found, 0);
    assert!(!report.metadata.ai_enhanced);
}
