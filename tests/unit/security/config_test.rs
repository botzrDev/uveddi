//! Unit tests for security configuration module

use uveddi::analysis::detectors::security::config::*;

#[test]
fn test_security_config_production() {
    let config = SecurityConfig::production();
    
    assert!(config.enable_taint_analysis);
    assert!(config.enable_sca);
    assert!(config.enable_ai_enhancement);
    assert_eq!(config.confidence_threshold, 0.7);
    assert_eq!(config.max_issues_per_file, 50);
    assert!(config.enable_caching);
    assert!(config.parallel_analysis);
    assert_eq!(config.timeout_seconds, 300);
}

#[test]
fn test_security_config_development() {
    let config = SecurityConfig::development();
    
    assert!(config.enable_taint_analysis);
    assert!(!config.enable_sca); // Disabled in dev for speed
    assert!(!config.enable_ai_enhancement); // Disabled in dev
    assert_eq!(config.confidence_threshold, 0.5); // Lower threshold
    assert_eq!(config.max_issues_per_file, 100); // Higher limit for debugging
    assert!(config.enable_caching);
    assert!(config.parallel_analysis);
    assert_eq!(config.timeout_seconds, 600); // Longer timeout for debugging
}

#[test]
fn test_security_config_ci_cd() {
    let config = SecurityConfig::ci_cd();
    
    assert!(config.enable_taint_analysis);
    assert!(config.enable_sca);
    assert!(!config.enable_ai_enhancement); // Disabled for reproducibility
    assert_eq!(config.confidence_threshold, 0.8); // Higher threshold
    assert_eq!(config.max_issues_per_file, 30); // Lower limit
    assert!(!config.enable_caching); // Disabled for clean runs
    assert!(config.parallel_analysis);
    assert_eq!(config.timeout_seconds, 180); // Shorter timeout
}

#[test]
fn test_security_config_minimal() {
    let config = SecurityConfig::minimal();
    
    assert!(config.enable_taint_analysis); // Core feature
    assert!(!config.enable_sca);
    assert!(!config.enable_ai_enhancement);
    assert_eq!(config.confidence_threshold, 0.6);
    assert_eq!(config.max_issues_per_file, 20);
    assert!(config.enable_caching);
    assert!(!config.parallel_analysis); // Disabled for minimal
    assert_eq!(config.timeout_seconds, 120);
}

#[test]
fn test_multi_agent_config_production() {
    let config = MultiAgentConfig::production();
    
    assert!(config.enable_orchestrator);
    assert!(config.enable_taint_agent);
    assert!(config.enable_config_agent);
    assert!(config.enable_dependency_agent);
    assert!(config.enable_validation_agent);
    assert_eq!(config.max_concurrent_agents, 4);
    assert_eq!(config.agent_timeout_seconds, 300);
    assert_eq!(config.message_queue_size, 1000);
    assert_eq!(config.retry_attempts, 3);
}

#[test]
fn test_multi_agent_config_development() {
    let config = MultiAgentConfig::development();
    
    assert!(config.enable_orchestrator);
    assert!(config.enable_taint_agent);
    assert!(config.enable_config_agent);
    assert!(!config.enable_dependency_agent); // Disabled for speed
    assert!(config.enable_validation_agent);
    assert_eq!(config.max_concurrent_agents, 2); // Lower for debugging
    assert_eq!(config.agent_timeout_seconds, 600); // Longer for debugging
    assert_eq!(config.message_queue_size, 100); // Smaller queue
    assert_eq!(config.retry_attempts, 1);
}

#[test]
fn test_taint_analysis_config_defaults() {
    let config = TaintAnalysisConfig::default();
    
    assert!(config.track_implicit_flows);
    assert!(config.context_sensitive);
    assert!(config.field_sensitive);
    assert_eq!(config.max_path_length, 100);
    assert_eq!(config.max_iterations, 1000);
    assert!(config.enable_sanitizers);
    assert!(config.detect_second_order);
}

#[test]
fn test_false_positive_config_aggressive() {
    let config = FalsePositiveConfig::aggressive();
    
    assert!(config.enable_heuristic_filtering);
    assert!(config.enable_contextual_filtering);
    assert!(config.enable_statistical_filtering);
    assert!(config.enable_ml_filtering);
    assert!(config.exclude_test_files);
    assert!(config.exclude_generated_files);
    assert_eq!(config.min_lines_threshold, 10);
    assert_eq!(config.min_confidence_threshold, 0.7);
    assert_eq!(config.max_similar_issues, 3);
}

#[test]
fn test_false_positive_config_balanced() {
    let config = FalsePositiveConfig::balanced();
    
    assert!(config.enable_heuristic_filtering);
    assert!(config.enable_contextual_filtering);
    assert!(!config.enable_statistical_filtering);
    assert!(!config.enable_ml_filtering);
    assert!(config.exclude_test_files);
    assert!(!config.exclude_generated_files);
    assert_eq!(config.min_lines_threshold, 5);
    assert_eq!(config.min_confidence_threshold, 0.5);
    assert_eq!(config.max_similar_issues, 5);
}

#[test]
fn test_false_positive_config_minimal() {
    let config = FalsePositiveConfig::minimal();
    
    assert!(!config.enable_heuristic_filtering);
    assert!(config.enable_contextual_filtering);
    assert!(!config.enable_statistical_filtering);
    assert!(!config.enable_ml_filtering);
    assert!(config.exclude_test_files);
    assert!(!config.exclude_generated_files);
    assert_eq!(config.min_lines_threshold, 3);
    assert_eq!(config.min_confidence_threshold, 0.3);
    assert_eq!(config.max_similar_issues, 10);
}

#[test]
fn test_language_config_rust() {
    let config = LanguageConfig::rust();
    
    assert!(config.custom_sources.contains(&"std::env::args".to_string()));
    assert!(config.custom_sources.contains(&"std::env::var".to_string()));
    assert!(config.custom_sinks.contains(&"std::process::Command::new".to_string()));
    assert!(config.custom_sinks.contains(&"sqlx::query".to_string()));
    assert!(config.custom_sanitizers.contains(&"html_escape::encode".to_string()));
    assert!(config.enable_unsafe_analysis);
    assert!(config.check_memory_safety);
    assert!(config.detect_race_conditions);
}

#[test]
fn test_language_config_python() {
    let config = LanguageConfig::python();
    
    assert!(config.custom_sources.contains(&"input(".to_string()));
    assert!(config.custom_sources.contains(&"sys.argv".to_string()));
    assert!(config.custom_sinks.contains(&"eval(".to_string()));
    assert!(config.custom_sinks.contains(&"os.system".to_string()));
    assert!(config.custom_sanitizers.contains(&"html.escape".to_string()));
    assert!(config.check_type_confusion);
    assert!(config.detect_deserialization);
    assert!(config.analyze_imports);
}

#[test]
fn test_language_config_javascript() {
    let config = LanguageConfig::javascript();
    
    assert!(config.custom_sources.contains(&"req.query".to_string()));
    assert!(config.custom_sources.contains(&"req.body".to_string()));
    assert!(config.custom_sinks.contains(&"eval(".to_string()));
    assert!(config.custom_sinks.contains(&"document.write".to_string()));
    assert!(config.custom_sanitizers.contains(&"DOMPurify.sanitize".to_string()));
    assert!(config.detect_prototype_pollution);
    assert!(config.check_dom_xss);
    assert!(config.analyze_dependencies);
}

#[test]
fn test_security_config_with_modifications() {
    let mut config = SecurityConfig::production();
    
    // Modify some settings
    config.confidence_threshold = 0.9;
    config.max_issues_per_file = 100;
    config.enable_ai_enhancement = false;
    
    assert_eq!(config.confidence_threshold, 0.9);
    assert_eq!(config.max_issues_per_file, 100);
    assert!(!config.enable_ai_enhancement);
    
    // Other settings should remain unchanged
    assert!(config.enable_taint_analysis);
    assert!(config.enable_sca);
}

#[test]
fn test_multi_agent_config_with_selective_agents() {
    let mut config = MultiAgentConfig::production();
    
    // Disable some agents
    config.enable_dependency_agent = false;
    config.enable_config_agent = false;
    
    assert!(config.enable_orchestrator);
    assert!(config.enable_taint_agent);
    assert!(!config.enable_config_agent);
    assert!(!config.enable_dependency_agent);
    assert!(config.enable_validation_agent);
}

#[test]
fn test_config_serialization() {
    let config = SecurityConfig::production();
    
    // Test that config can be serialized to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"enable_taint_analysis\":true"));
    
    // Test that it can be deserialized back
    let deserialized: SecurityConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.enable_taint_analysis, deserialized.enable_taint_analysis);
    assert_eq!(config.confidence_threshold, deserialized.confidence_threshold);
}