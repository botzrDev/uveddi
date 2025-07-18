//! Comprehensive testing for dependency injection system
//!
//! This module tests the dependency injection infrastructure including:
//! - Constructor injection with trait-based dependencies
//! - Builder pattern with fluent API
//! - Enhanced detector configuration
//! - Mock implementations for testing
//! - Configuration-driven setup

use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;

use uveddi::analysis::{
    AnalysisEngine, AnalysisEngineBuilder, AnalysisConfig, EnhancedDetectorConfig, IssueSeverity,
    DetectorFactory,
    traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait, CacheStats},
    adapters::{AstParserAdapter, DependencyExtractorAdapter, ResultCacheAdapter},
    errors::AnalysisError,
};
use uveddi::ast::{ParsedFile, SourceLanguage};
use uveddi::analysis::detectors::dependency::Dependency;
use uveddi::models::ApiDependencyType;

/// Mock AST parser for testing dependency injection
struct MockAstParser {
    initialized: bool,
    supported_languages: Vec<SourceLanguage>,
}

impl MockAstParser {
    fn new() -> Self {
        Self {
            initialized: true,
            supported_languages: vec![SourceLanguage::Rust, SourceLanguage::Python],
        }
    }
    
    fn new_uninitialized() -> Self {
        Self {
            initialized: false,
            supported_languages: vec![],
        }
    }
}

impl AstParserTrait for MockAstParser {
    fn parse_file(&self, path: &Path) -> Result<ParsedFile, AnalysisError> {
        if !self.initialized {
            return Err(AnalysisError::DetectionError("Parser not initialized".to_string()));
        }
        
        // Create a minimal mock parsed file
        Ok(ParsedFile {
            file_path: Arc::new(path.to_path_buf()),
            language: SourceLanguage::Rust,
            tree: None,
            source: Arc::new("// Mock file content".to_string()),
            custom_ast: Arc::new(None),
            modified_at: std::time::SystemTime::now(),
        })
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn supported_languages(&self) -> Vec<SourceLanguage> {
        self.supported_languages.clone()
    }
}

/// Mock dependency extractor for testing
struct MockDependencyExtractor {
    supported_languages: Vec<SourceLanguage>,
}

impl MockDependencyExtractor {
    fn new() -> Self {
        Self {
            supported_languages: vec![SourceLanguage::Rust, SourceLanguage::Python],
        }
    }
}

impl DependencyExtractorTrait for MockDependencyExtractor {
    fn extract_from_ast(&self, _parsed_file: &ParsedFile) -> Result<Vec<Dependency>, AnalysisError> {
        // Return mock dependencies
        Ok(vec![
            Dependency {
                from_file: std::path::PathBuf::from("test.rs"),
                to_module: "std::collections".to_string(),
                dependency_type: ApiDependencyType::Import,
                line_number: Some(1),
            }
        ])
    }
    
    fn supports_language(&self, language: &SourceLanguage) -> bool {
        self.supported_languages.contains(language)
    }
}

/// Mock result cache for testing
struct MockResultCache {
    hits: u64,
    misses: u64,
}

impl MockResultCache {
    fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }
}

impl ResultCacheTrait for MockResultCache {
    fn get_json(&self, _key: &str) -> Result<Option<String>, AnalysisError> {
        // Always return cache miss for testing
        Ok(None)
    }
    
    fn set_json(&self, _key: &str, _value: &str) -> Result<(), AnalysisError> {
        Ok(())
    }
    
    fn clear(&self) {
        // No-op for mock
    }
    
    fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            entries: 0,
            memory_usage: 1024,
        }
    }
}

#[tokio::test]
async fn test_constructor_injection_with_valid_dependencies() {
    let ast_parser = Box::new(MockAstParser::new()) as Box<dyn AstParserTrait>;
    let dependency_extractor = Box::new(MockDependencyExtractor::new()) as Box<dyn DependencyExtractorTrait>;
    let cache = Box::new(MockResultCache::new()) as Box<dyn ResultCacheTrait>;
    let detectors = vec![];

    let result = AnalysisEngine::with_injected_dependencies(
        ast_parser,
        dependency_extractor,
        cache,
        detectors,
    );

    assert!(result.is_ok(), "Engine creation should succeed with valid dependencies");
}

#[tokio::test]
async fn test_constructor_injection_with_uninitialized_parser() {
    let ast_parser = Box::new(MockAstParser::new_uninitialized()) as Box<dyn AstParserTrait>;
    let dependency_extractor = Box::new(MockDependencyExtractor::new()) as Box<dyn DependencyExtractorTrait>;
    let cache = Box::new(MockResultCache::new()) as Box<dyn ResultCacheTrait>;
    let detectors = vec![];

    let result = AnalysisEngine::with_injected_dependencies(
        ast_parser,
        dependency_extractor,
        cache,
        detectors,
    );

    assert!(result.is_err(), "Engine creation should fail with uninitialized parser");
    if let Err(error) = result {
        assert!(format!("{}", error).contains("AST parser not properly initialized"));
    }
}

#[tokio::test]
async fn test_builder_pattern_with_dependency_injection() {
    let engine = AnalysisEngineBuilder::new()
        .with_injected_ast_parser(Box::new(MockAstParser::new()))
        .with_injected_dependency_extractor(Box::new(MockDependencyExtractor::new()))
        .with_cache(Box::new(MockResultCache::new()))
        .build()
        .await;

    assert!(engine.is_ok(), "Builder should create engine with dependency injection");
}

#[tokio::test]
async fn test_builder_pattern_with_adapters() {
    let ast_parser_adapter = AstParserAdapter::new_default();
    assert!(ast_parser_adapter.is_ok(), "AST parser adapter should be created successfully");

    let dependency_extractor_adapter = DependencyExtractorAdapter::new_default();
    assert!(dependency_extractor_adapter.is_ok(), "Dependency extractor adapter should be created successfully");

    let cache_adapter = ResultCacheAdapter::new_memory();
    assert!(cache_adapter.is_ok(), "Cache adapter should be created successfully");

    let engine = AnalysisEngineBuilder::new()
        .with_injected_ast_parser(Box::new(ast_parser_adapter.unwrap()))
        .with_dependency_extractor(Box::new(dependency_extractor_adapter.unwrap()))
        .with_cache(Box::new(cache_adapter.unwrap()))
        .build()
        .await;

    assert!(engine.is_ok(), "Builder should create engine with adapter-based dependency injection");
}

#[tokio::test]
async fn test_enhanced_detector_configuration() {
    let config = EnhancedDetectorConfig::new()
        .with_enabled(true)
        .with_severity(IssueSeverity::High)
        .with_max_methods(25)
        .with_max_fields(20);

    assert!(config.enabled);
    assert_eq!(config.severity, IssueSeverity::High);
    assert_eq!(config.thresholds.max_methods, Some(25));
    assert_eq!(config.thresholds.max_fields, Some(20));

    // Test detector creation with enhanced config
    let factory = DetectorFactory::new();
    let detector = factory.create_detector_enhanced("god_object", &config);
    assert!(detector.is_ok(), "Enhanced detector creation should succeed");
}

#[tokio::test]
async fn test_configuration_driven_setup() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    let config_content = r#"
cache_size = 500
enable_plugins = false

[cache_settings]
cache_type = "memory"
max_size = "50MB"
eviction_policy = "lru"

[performance_settings]
max_concurrent_files = 5
timeout_seconds = 15
parallel_processing = true

[enhanced_detectors.god_object]
enabled = true
severity = "High"
thresholds = { max_methods = 25, max_fields = 20 }

[enhanced_detectors.code_duplication]
enabled = true
severity = "Medium"
thresholds = { min_similarity = 0.9 }
"#;

    std::fs::write(&config_path, config_content)?;

    let engine = AnalysisEngineBuilder::new()
        .from_config_file_di(&config_path)?
        .with_detectors_from_config()?
        .build()
        .await;

    assert!(engine.is_ok(), "Configuration-driven engine creation should succeed");
    Ok(())
}

#[tokio::test]
async fn test_mock_detector_integration() {
    use uveddi::analysis::AnalysisDetector;
    use uveddi::database::models::{AntiPatternType, ArchitecturalIssue};
    
    struct MockDetector;
    
    #[async_trait::async_trait]
    impl AnalysisDetector for MockDetector {
        async fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
            Ok(vec![
                ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,
                    anti_pattern_type_id: 1,
                    file_path: "test.rs".to_string(),
                    start_line: Some(42),
                    end_line: Some(50),
                    severity: "High".to_string(),
                    description: "Test issue detected by mock detector".to_string(),
                    code_snippet: Some("mock code".to_string()),
                    ai_explanation: Some("Mock AI explanation".to_string()),
                }
            ])
        }
        
        fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
            vec![
                AntiPatternType {
                    anti_pattern_type_id: None,
                    name: "Mock Anti-Pattern".to_string(),
                    description: "Mock anti-pattern for testing".to_string(),
                    category: "Testing".to_string(),
                }
            ]
        }
        
        fn get_detector_name(&self) -> &'static str {
            "MockDetector"
        }
    }

    let engine = AnalysisEngineBuilder::new()
        .with_detector(Box::new(MockDetector))
        .build()
        .await
        .unwrap();

    let anti_pattern_types = engine.get_anti_pattern_types();
    assert!(anti_pattern_types.len() >= 1, "Mock detector should be included in anti-pattern types");
    
    let mock_types: Vec<_> = anti_pattern_types.iter()
        .filter(|t| t.name == "Mock Anti-Pattern")
        .collect();
    assert_eq!(mock_types.len(), 1, "Should have exactly one mock anti-pattern type");
}

#[tokio::test]
async fn test_backward_compatibility() {
    // Test that the new dependency injection system doesn't break existing functionality
    let engine_original = AnalysisEngine::new();
    assert!(engine_original.is_ok(), "Original constructor should still work");

    let engine_defaults = AnalysisEngine::new_with_defaults();
    assert!(engine_defaults.is_ok(), "New defaults constructor should work");

    let engine_memory = AnalysisEngine::new_with_memory_cache();
    assert!(engine_memory.is_ok(), "Memory cache constructor should still work");
}

#[tokio::test]
async fn test_dependency_validation() {
    // Test that incompatible dependencies are rejected
    struct IncompatibleExtractor;
    
    impl DependencyExtractorTrait for IncompatibleExtractor {
        fn extract_from_ast(&self, _parsed_file: &ParsedFile) -> Result<Vec<Dependency>, AnalysisError> {
            Ok(vec![])
        }
        
        fn supports_language(&self, _language: &SourceLanguage) -> bool {
            false // Supports no languages
        }
    }

    let ast_parser = Box::new(MockAstParser::new()) as Box<dyn AstParserTrait>;
    let incompatible_extractor = Box::new(IncompatibleExtractor) as Box<dyn DependencyExtractorTrait>;
    let cache = Box::new(MockResultCache::new()) as Box<dyn ResultCacheTrait>;
    let detectors = vec![];

    let result = AnalysisEngine::with_injected_dependencies(
        ast_parser,
        incompatible_extractor,
        cache,
        detectors,
    );

    assert!(result.is_err(), "Engine creation should fail with incompatible dependencies");
    if let Err(error) = result {
        assert!(format!("{}", error).contains("no common language support"));
    }
}

#[test]
fn test_cache_stats() {
    let cache = MockResultCache::new();
    let stats = cache.get_stats();
    
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.memory_usage, 1024);
}

#[test]
fn test_enhanced_config_serialization() {
    let config = EnhancedDetectorConfig::new()
        .with_enabled(true)
        .with_severity(IssueSeverity::Critical)
        .with_max_methods(30);

    // Test TOML serialization
    let toml_string = toml::to_string(&config);
    assert!(toml_string.is_ok(), "Enhanced config should serialize to TOML");

    let serialized = toml_string.unwrap();
    assert!(serialized.contains("enabled = true"));
    assert!(serialized.contains("severity = \"Critical\""));
    assert!(serialized.contains("max_methods = 30"));

    // Test deserialization
    let deserialized: EnhancedDetectorConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.severity, config.severity);
    assert_eq!(deserialized.thresholds.max_methods, config.thresholds.max_methods);
}

#[tokio::test]
async fn test_analysis_config_with_enhanced_detectors() {
    let config = AnalysisConfig::default();
    
    // Verify default enhanced detectors are configured
    assert!(!config.enhanced_detectors.is_empty(), "Default config should have enhanced detectors");
    assert!(config.enhanced_detectors.contains_key("god_object"));
    assert!(config.enhanced_detectors.contains_key("code_duplication"));

    // Test engine creation from enhanced config
    let engine = config.create_engine().await;
    assert!(engine.is_ok(), "Engine creation from enhanced config should succeed");
}

#[test]
fn test_issue_severity_serialization() {
    let severities = vec![
        IssueSeverity::Low,
        IssueSeverity::Medium,
        IssueSeverity::High,
        IssueSeverity::Critical,
    ];

    for severity in severities {
        let serialized = serde_json::to_string(&severity).unwrap();
        let deserialized: IssueSeverity = serde_json::from_str(&serialized).unwrap();
        assert_eq!(severity, deserialized);
    }
}

#[tokio::test]
async fn test_factory_available_detectors() {
    let factory = DetectorFactory::new();
    let available = factory.available_detectors();
    
    assert!(available.contains(&"god_object".to_string()));
    assert!(available.contains(&"code_duplication".to_string()));
    assert!(available.contains(&"dead_code".to_string()));
    assert!(available.contains(&"large_classes".to_string()));
    assert!(available.contains(&"tight_coupling".to_string()));
    
    assert_eq!(available.len(), 5, "Should have exactly 5 built-in detectors");
}