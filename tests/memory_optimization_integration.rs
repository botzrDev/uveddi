//! Integration tests for memory optimization configuration validation and graceful fallback
//! UV-210/UV-26 Phase 4 - Production integration tests

use std::path::PathBuf;
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};

#[cfg(feature = "memory-optimization")]
use uveddi::analysis::memory::MemoryOptimizationConfig;

#[tokio::test]
async fn test_memory_optimization_validation_invalid_limit() {
    let config = AnalysisConfig {
        target_path: PathBuf::from("src/lib.rs"),
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

        // Memory optimization with invalid limit
        #[cfg(feature = "memory-optimization")]
        memory_optimization: None,
        enable_memory_optimization: true,
        memory_limit_gb: Some(-1.0), // Invalid negative limit
        memory_profile: None,
    };

    let mut orchestrator = AnalysisOrchestrator::new().unwrap();

    // Should not fail, but should log warnings and fallback to standard mode
    let result = orchestrator.execute_analysis(config).await;
    assert!(
        result.is_ok(),
        "Analysis should succeed with graceful fallback"
    );
}

#[tokio::test]
async fn test_memory_optimization_validation_invalid_profile() {
    let config = AnalysisConfig {
        target_path: PathBuf::from("src/lib.rs"),
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

        // Memory optimization with invalid profile
        #[cfg(feature = "memory-optimization")]
        memory_optimization: None,
        enable_memory_optimization: true,
        memory_limit_gb: None,
        memory_profile: Some("invalid_profile".to_string()),
    };

    let mut orchestrator = AnalysisOrchestrator::new().unwrap();

    // Should not fail, but should log warnings and fallback to standard mode
    let result = orchestrator.execute_analysis(config).await;
    assert!(
        result.is_ok(),
        "Analysis should succeed with graceful fallback"
    );
}

#[tokio::test]
async fn test_memory_optimization_validation_valid_config() {
    let config = AnalysisConfig {
        target_path: PathBuf::from("src/lib.rs"),
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

        // Memory optimization with valid config
        #[cfg(feature = "memory-optimization")]
        memory_optimization: None,
        enable_memory_optimization: true,
        memory_limit_gb: Some(4.0), // Valid 4GB limit
        memory_profile: Some("small".to_string()),
    };

    let mut orchestrator = AnalysisOrchestrator::new().unwrap();

    // Should succeed with memory optimization enabled
    let result = orchestrator.execute_analysis(config).await;
    assert!(
        result.is_ok(),
        "Analysis should succeed with valid memory optimization config"
    );
}

#[tokio::test]
async fn test_memory_optimization_disabled() {
    let config = AnalysisConfig {
        target_path: PathBuf::from("src/lib.rs"),
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

        // Memory optimization disabled
        #[cfg(feature = "memory-optimization")]
        memory_optimization: None,
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
    };

    let mut orchestrator = AnalysisOrchestrator::new().unwrap();

    // Should succeed with standard analysis mode
    let result = orchestrator.execute_analysis(config).await;
    assert!(result.is_ok(), "Analysis should succeed with standard mode");
}

#[cfg(feature = "memory-optimization")]
#[tokio::test]
async fn test_memory_optimization_custom_config() {
    let mut custom_config = MemoryOptimizationConfig::default();
    custom_config.target_max_memory_bytes = 2 * 1024 * 1024 * 1024; // 2GB

    let config = AnalysisConfig {
        target_path: PathBuf::from("src/lib.rs"),
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

        // Memory optimization with custom config
        memory_optimization: Some(custom_config),
        enable_memory_optimization: true,
        memory_limit_gb: None,
        memory_profile: None,
    };

    let mut orchestrator = AnalysisOrchestrator::new().unwrap();

    // Should succeed with custom memory optimization config
    let result = orchestrator.execute_analysis(config).await;
    assert!(
        result.is_ok(),
        "Analysis should succeed with custom memory optimization config"
    );
}
