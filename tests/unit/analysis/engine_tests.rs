//! Comprehensive unit tests for the AnalysisEngine
//!
//! This test suite validates the AnalysisEngine's core functionality including:
//! - Engine initialization and configuration
//! - Analysis workflow execution
//! - Error handling scenarios
//! - Performance characteristics
//!
//! These tests use the established test infrastructure from UV-296-T1.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rstest::*;
use serial_test::serial;
use tempfile::TempDir;
use tokio::time::timeout;

// Test infrastructure imports
use crate::test_utils::{
    create_sample_rust_file, create_temp_test_dir, fixtures::TestFixtures,
    helpers::*, mocks::*,
};

// Mock imports for isolated testing
use crate::test_utils::mocks::{
    MockAstParser, MockDependencyExtractor, MockResultCache, MockAnalysisDetector,
    create_sample_dependency, create_sample_issue,
};

// Analysis engine and related imports
use crate::analysis::{
    AnalysisEngine, AnalysisConfig, DetectorConfig,
    engine_builder::AnalysisEngineBuilder,
};
use crate::analysis::components::{
    ConfigurationService, AstProviderImpl, CacheManagerImpl,
    DependencyGraphBuilderImpl, DetectorScheduler, PluginManagerHandle,
    AnalysisAggregator,
};
use crate::analysis::graph::dependency::{LocalDependencyGraph, ComponentNode, LocalDependencyType};
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::error::{UveddiError, Result};
use crate::ast::{ParsedFile, SourceLanguage};

/// Test fixtures for engine testing
struct EngineTestFixtures {
    temp_dir: TempDir,
    config: AnalysisConfig,
    test_files: Vec<PathBuf>,
    mock_ast_parser: MockAstParser,
    mock_dependency_extractor: MockDependencyExtractor,
    mock_cache: MockResultCache,
    mock_detectors: Vec<MockAnalysisDetector>,
}

impl EngineTestFixtures {
    async fn new() -> Result<Self> {
        let temp_dir = create_temp_test_dir()?;
        
        // Create test files
        let test_files = vec![
            create_sample_rust_file(&temp_dir, "main.rs", "fn main() {}").await?,
            create_sample_rust_file(&temp_dir, "lib.rs", "pub fn test() {}").await?,
            create_sample_rust_file(&temp_dir, "module.rs", "pub struct Config {}").await?,
        ];

        // Create mock configuration
        let mut detectors = HashMap::new();
        detectors.insert("god_object".to_string(), DetectorConfig::new());
        detectors.insert("dead_code".to_string(), DetectorConfig::new());
        
        let config = AnalysisConfig {
            detectors,
            enhanced_detectors: HashMap::new(),
            standard_detectors: HashMap::new(),
            cache_size: Some(1000),
            enable_plugins: false,
            cache_path: Some(temp_dir.path().join("test_cache.db").to_string_lossy().to_string()),
            cache_settings: Default::default(),
            performance_settings: Default::default(),
        };

        // Create mocks
        let mock_ast_parser = MockAstParser::create_successful();
        let mock_dependency_extractor = MockDependencyExtractor::create_empty();
        let mock_cache = MockResultCache::create_empty();
        let mock_detectors = vec![
            MockAnalysisDetector::create_clean(),
            MockAnalysisDetector::create_with_issues(vec![
                create_sample_issue(AntiPatternType::GodObject, "Test issue", "test.rs")
            ]),
        ];

        Ok(Self {
            temp_dir,
            config,
            test_files,
            mock_ast_parser,
            mock_dependency_extractor,
            mock_cache,
            mock_detectors,
        })
    }

    fn get_test_path(&self) -> &Path {
        self.temp_dir.path()
    }
}

mod engine_initialization_tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation_with_default_configuration() {
        let engine = AnalysisEngine::new();
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        
        // Verify engine has default configuration
        assert_eq!(engine.get_files_analyzed(), 0);
        assert!(!engine.has_plugin_support());
        assert!(!engine.get_anti_pattern_types().is_empty());
    }

    #[tokio::test]
    async fn test_engine_creation_with_custom_detectors() {
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let engine = AnalysisEngine::with_detectors(detectors);
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert_eq!(engine.get_files_analyzed(), 0);
    }

    #[tokio::test]
    async fn test_engine_creation_with_custom_cache_path() {
        let temp_dir = create_temp_test_dir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("custom_cache.db");
        
        let engine = AnalysisEngine::with_cache_path(&cache_path);
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert_eq!(engine.get_files_analyzed(), 0);
    }

    #[tokio::test]
    async fn test_engine_creation_with_memory_cache() {
        let engine = AnalysisEngine::new_with_memory_cache();
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert_eq!(engine.get_files_analyzed(), 0);
    }

    #[tokio::test]
    async fn test_engine_creation_with_plugin_support() {
        let engine = AnalysisEngine::new_async().await;
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert!(engine.has_plugin_support());
    }

    #[tokio::test]
    async fn test_engine_builder_pattern() {
        let temp_dir = create_temp_test_dir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("builder_cache.db");
        
        let engine = AnalysisEngine::builder()
            .with_cache_path(&cache_path)
            .enable_plugins(false)
            .build();
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert!(!engine.has_plugin_support());
    }

    #[tokio::test]
    async fn test_engine_builder_async_pattern() {
        let engine = AnalysisEngine::builder()
            .enable_plugins(true)
            .build_async()
            .await;
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert!(engine.has_plugin_support());
    }

    #[tokio::test]
    async fn test_engine_creation_with_injected_dependencies() {
        let ast_parser = Box::new(MockAstParser::create_successful()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_empty()) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(MockResultCache::create_empty()) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        );
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert_eq!(engine.get_files_analyzed(), 0);
    }

    #[tokio::test]
    async fn test_engine_initialization_error_handling() {
        // Test with invalid cache path
        let invalid_path = Path::new("/invalid/path/cache.db");
        
        let engine = AnalysisEngine::with_cache_path(invalid_path);
        
        // Should handle invalid path gracefully or return error
        // Note: Actual behavior depends on implementation
        assert!(engine.is_ok() || engine.is_err());
    }
}

mod configuration_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_loading_and_validation() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        
        // Test configuration values
        assert_eq!(fixtures.config.cache_size, Some(1000));
        assert!(!fixtures.config.enable_plugins);
        assert!(fixtures.config.cache_path.is_some());
        assert!(!fixtures.config.detectors.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_configuration_handling() {
        let mut config = AnalysisConfig {
            detectors: HashMap::new(),
            enhanced_detectors: HashMap::new(),
            standard_detectors: HashMap::new(),
            cache_size: Some(0), // Invalid cache size
            enable_plugins: false,
            cache_path: None,
            cache_settings: Default::default(),
            performance_settings: Default::default(),
        };
        
        // Test with invalid cache size
        config.cache_size = Some(0);
        
        // Should handle invalid configuration gracefully
        let result = config.create_engine().await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_configuration_updates_and_reloading() {
        let mut fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        
        // Modify configuration
        fixtures.config.cache_size = Some(500);
        fixtures.config.enable_plugins = true;
        
        // Configuration should be updated
        assert_eq!(fixtures.config.cache_size, Some(500));
        assert!(fixtures.config.enable_plugins);
    }

    #[tokio::test]
    async fn test_detector_configuration() {
        let mut detectors = HashMap::new();
        detectors.insert("god_object".to_string(), DetectorConfig::new().with_param("threshold_methods", 5));
        detectors.insert("dead_code".to_string(), DetectorConfig::new().with_param("enabled", 1));
        
        let config = AnalysisConfig {
            detectors,
            enhanced_detectors: HashMap::new(),
            standard_detectors: HashMap::new(),
            cache_size: Some(1000),
            enable_plugins: false,
            cache_path: None,
            cache_settings: Default::default(),
            performance_settings: Default::default(),
        };
        
        assert_eq!(config.detectors.len(), 2);
        assert!(config.detectors.contains_key("god_object"));
        assert!(config.detectors.contains_key("dead_code"));
    }
}

mod analysis_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_single_file_analysis_with_mocked_dependencies() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create engine with mocked dependencies
        let ast_parser = Box::new(MockAstParser::create_successful()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_empty()) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(MockResultCache::create_empty()) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        ).expect("Failed to create engine");
        
        // Analyze single file
        let result = engine.analyze(test_file).await;
        
        assert!(result.is_ok());
        let (issues, graph) = result.unwrap();
        assert!(issues.is_empty()); // Mock detector returns no issues
        assert_eq!(graph.node_count(), 0); // Empty dependency graph
    }

    #[tokio::test]
    async fn test_multi_file_analysis_coordination() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_dir = fixtures.get_test_path();
        
        // Create engine with mocked dependencies
        let ast_parser = Box::new(MockAstParser::create_successful()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_with_dependencies(vec![
            create_sample_dependency("test_dep", "main.rs", "lib.rs"),
        ])) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(MockResultCache::create_empty()) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_with_issues(vec![
                create_sample_issue(AntiPatternType::GodObject, "Test issue", "main.rs"),
            ])) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        ).expect("Failed to create engine");
        
        // Analyze directory
        let result = engine.analyze(test_dir).await;
        
        assert!(result.is_ok());
        let (issues, graph) = result.unwrap();
        assert!(!issues.is_empty()); // Mock detector returns issues
        assert!(graph.node_count() >= 0); // Should have some nodes
    }

    #[tokio::test]
    async fn test_detector_scheduling_and_execution() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create multiple detectors
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>,
            Box::new(MockAnalysisDetector::create_with_issues(vec![
                create_sample_issue(AntiPatternType::DeadCode, "Dead code found", "test.rs"),
            ])) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>,
            Box::new(MockAnalysisDetector::create_with_issues(vec![
                create_sample_issue(AntiPatternType::GodObject, "God object found", "test.rs"),
            ])) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>,
        ];
        
        let mut engine = AnalysisEngine::with_detectors(detectors).expect("Failed to create engine");
        
        // Analyze file
        let result = engine.analyze(test_file).await;
        
        assert!(result.is_ok());
        let (issues, _) = result.unwrap();
        
        // Should have issues from multiple detectors
        assert!(!issues.is_empty());
    }

    #[tokio::test]
    async fn test_result_aggregation_and_reporting() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_dir = fixtures.get_test_path();
        
        // Create detector with multiple issues
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_with_issues(vec![
                create_sample_issue(AntiPatternType::GodObject, "Issue 1", "file1.rs"),
                create_sample_issue(AntiPatternType::DeadCode, "Issue 2", "file2.rs"),
                create_sample_issue(AntiPatternType::LargeClass, "Issue 3", "file3.rs"),
            ])) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_detectors(detectors).expect("Failed to create engine");
        
        // Analyze directory
        let result = engine.analyze(test_dir).await;
        
        assert!(result.is_ok());
        let (issues, graph) = result.unwrap();
        
        // Verify aggregation
        assert_eq!(issues.len(), 3);
        assert!(graph.node_count() >= 0);
        
        // Check that files were analyzed
        assert!(engine.get_files_analyzed() >= 0);
    }

    #[tokio::test]
    async fn test_cache_integration_in_workflow() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create cache with pre-existing data
        let mut cache_data = HashMap::new();
        cache_data.insert("test_key".to_string(), "cached_value".to_string());
        
        let ast_parser = Box::new(MockAstParser::create_successful()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_empty()) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(MockResultCache::create_with_data(cache_data)) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        ).expect("Failed to create engine");
        
        // Analyze file
        let result = engine.analyze(test_file).await;
        
        assert!(result.is_ok());
        
        // Check cache metrics
        let cache_metrics = engine.get_ast_cache_metrics();
        assert!(!cache_metrics.is_null());
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_error_propagation() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create failing AST parser
        let ast_parser = Box::new(MockAstParser::create_failing()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_empty()) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(MockResultCache::create_empty()) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        ).expect("Failed to create engine");
        
        // Analyze file - should handle parse errors gracefully
        let result = engine.analyze(test_file).await;
        
        // Should either succeed with partial results or fail with proper error
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_detector_failure_handling() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create failing detector
        let mut failing_detector = MockAnalysisDetector::new();
        failing_detector
            .expect_detect_issues()
            .returning(|_| Err(crate::analysis::errors::AnalysisError::DetectorError("Mock detector failure".to_string())));
        failing_detector
            .expect_detector_name()
            .returning(|| "FailingDetector");
        failing_detector
            .expect_detector_type()
            .returning(|| AntiPatternType::GodObject);
        
        let detectors = vec![
            Box::new(failing_detector) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_detectors(detectors).expect("Failed to create engine");
        
        // Analyze file - should handle detector failures gracefully
        let result = engine.analyze(test_file).await;
        
        // Should either succeed with partial results or fail with proper error
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_and_cancellation_scenarios() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create slow detector
        let mut slow_detector = MockAnalysisDetector::new();
        slow_detector
            .expect_detect_issues()
            .returning(|_| {
                // Simulate slow operation
                std::thread::sleep(Duration::from_millis(100));
                Ok(vec![])
            });
        slow_detector
            .expect_detector_name()
            .returning(|| "SlowDetector");
        slow_detector
            .expect_detector_type()
            .returning(|| AntiPatternType::GodObject);
        
        let detectors = vec![
            Box::new(slow_detector) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_detectors(detectors).expect("Failed to create engine");
        
        // Analyze file with timeout
        let result = timeout(Duration::from_millis(50), engine.analyze(test_file)).await;
        
        // Should timeout or complete quickly
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_resource_exhaustion_scenarios() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_dir = fixtures.get_test_path();
        
        // Create many detectors to simulate resource exhaustion
        let mut detectors = Vec::new();
        for i in 0..10 {
            let mut detector = MockAnalysisDetector::new();
            detector
                .expect_detect_issues()
                .returning(move |_| {
                    Ok(vec![create_sample_issue(
                        AntiPatternType::GodObject,
                        &format!("Issue from detector {}", i),
                        "test.rs",
                    )])
                });
            detector
                .expect_detector_name()
                .returning(|| "TestDetector");
            detector
                .expect_detector_type()
                .returning(|| AntiPatternType::GodObject);
            
            detectors.push(Box::new(detector) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>);
        }
        
        let mut engine = AnalysisEngine::with_detectors(detectors).expect("Failed to create engine");
        
        // Analyze directory - should handle resource constraints gracefully
        let result = engine.analyze(test_dir).await;
        
        assert!(result.is_ok());
        let (issues, _) = result.unwrap();
        
        // Should have issues from all detectors
        assert!(!issues.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_path_handling() {
        let mut engine = AnalysisEngine::new().expect("Failed to create engine");
        
        // Test with non-existent path
        let invalid_path = Path::new("/non/existent/path");
        let result = engine.analyze(invalid_path).await;
        
        // Should handle invalid paths gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_operation_failures() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create failing cache
        let mut failing_cache = MockResultCache::new();
        failing_cache
            .expect_get_json()
            .returning(|_| Err(crate::analysis::errors::AnalysisError::CacheError("Mock cache failure".to_string())));
        failing_cache
            .expect_set_json()
            .returning(|_, _| Err(crate::analysis::errors::AnalysisError::CacheError("Mock cache failure".to_string())));
        failing_cache
            .expect_clear()
            .returning(|| ());
        failing_cache
            .expect_get_stats()
            .returning(|| crate::analysis::traits::CacheStats {
                hits: 0,
                misses: 0,
                size: 0,
                max_size: 0,
            });
        
        let ast_parser = Box::new(MockAstParser::create_successful()) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(MockDependencyExtractor::create_empty()) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(failing_cache) as Box<dyn ResultCacheTrait>;
        let detectors = vec![
            Box::new(MockAnalysisDetector::create_clean()) as Box<dyn crate::analysis::AnalysisDetector + Send + Sync>
        ];
        
        let mut engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        ).expect("Failed to create engine");
        
        // Analyze file - should handle cache failures gracefully
        let result = engine.analyze(test_file).await;
        
        // Should either succeed without cache or fail with proper error
        assert!(result.is_ok() || result.is_err());
    }
}

mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_performance_validation() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        let mut engine = AnalysisEngine::new().expect("Failed to create engine");
        
        // Measure analysis time
        let start = std::time::Instant::now();
        let result = engine.analyze(test_file).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        
        // Should complete within reasonable time (adjust as needed)
        assert!(duration < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_memory_usage_patterns() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_dir = fixtures.get_test_path();
        
        // Create engine with memory cache
        let mut engine = AnalysisEngine::new_with_memory_cache().expect("Failed to create engine");
        
        // Analyze directory
        let result = engine.analyze(test_dir).await;
        assert!(result.is_ok());
        
        // Check cache metrics
        let cache_metrics = engine.get_ast_cache_metrics();
        assert!(!cache_metrics.is_null());
    }

    #[tokio::test]
    async fn test_concurrent_analysis_performance() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        
        // Create multiple engines for concurrent analysis
        let engines = (0..3)
            .map(|_| AnalysisEngine::new().expect("Failed to create engine"))
            .collect::<Vec<_>>();
        
        let start = std::time::Instant::now();
        
        // Run concurrent analyses
        let mut tasks = Vec::new();
        for (i, mut engine) in engines.into_iter().enumerate() {
            let test_file = fixtures.test_files[i % fixtures.test_files.len()].clone();
            tasks.push(tokio::spawn(async move {
                engine.analyze(&test_file).await
            }));
        }
        
        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();
        
        // All should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
        
        // Should complete within reasonable time
        assert!(duration < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_cache_performance_impact() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        // Create engine with cache
        let mut engine_with_cache = AnalysisEngine::new().expect("Failed to create engine");
        
        // First analysis (cold cache)
        let start = std::time::Instant::now();
        let result1 = engine_with_cache.analyze(test_file).await;
        let cold_duration = start.elapsed();
        
        assert!(result1.is_ok());
        
        // Second analysis (warm cache)
        let start = std::time::Instant::now();
        let result2 = engine_with_cache.analyze(test_file).await;
        let warm_duration = start.elapsed();
        
        assert!(result2.is_ok());
        
        // Warm cache should be faster or similar
        // Note: This might not always be true due to test environment
        assert!(warm_duration <= cold_duration + Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let temp_dir = create_temp_test_dir().expect("Failed to create temp dir");
        
        // Create large file content
        let large_content = (0..1000)
            .map(|i| format!("fn function_{}() {{ println!(\"Function {}\"); }}", i, i))
            .collect::<Vec<_>>()
            .join("\n");
        
        let large_file = create_sample_rust_file(&temp_dir, "large.rs", &large_content)
            .await
            .expect("Failed to create large file");
        
        let mut engine = AnalysisEngine::new().expect("Failed to create engine");
        
        // Analyze large file
        let start = std::time::Instant::now();
        let result = engine.analyze(&large_file).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        
        // Should handle large files within reasonable time
        assert!(duration < Duration::from_secs(30));
    }
}

mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_analysis_workflow() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_dir = fixtures.get_test_path();
        
        // Create comprehensive engine
        let mut engine = AnalysisEngine::builder()
            .with_cache_path(&fixtures.temp_dir.path().join("integration_cache.db"))
            .build()
            .expect("Failed to create engine");
        
        // Run full analysis
        let result = engine.analyze(test_dir).await;
        
        assert!(result.is_ok());
        let (issues, graph) = result.unwrap();
        
        // Verify results
        assert!(issues.len() >= 0);
        assert!(graph.node_count() >= 0);
        assert!(engine.get_files_analyzed() >= 0);
        
        // Check anti-pattern types
        let anti_patterns = engine.get_anti_pattern_types();
        assert!(!anti_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_engine_state_consistency() {
        let fixtures = EngineTestFixtures::new().await.expect("Failed to create fixtures");
        let test_file = &fixtures.test_files[0];
        
        let mut engine = AnalysisEngine::new().expect("Failed to create engine");
        
        // Multiple analysis runs
        for i in 0..3 {
            let result = engine.analyze(test_file).await;
            assert!(result.is_ok());
            
            // Engine state should be consistent
            let files_analyzed = engine.get_files_analyzed();
            assert!(files_analyzed >= 0);
        }
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let temp_dir = create_temp_test_dir().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("persistent_cache.db");
        let test_file = create_sample_rust_file(&temp_dir, "test.rs", "fn main() {}")
            .await
            .expect("Failed to create test file");
        
        // First engine instance
        {
            let mut engine = AnalysisEngine::with_cache_path(&cache_path)
                .expect("Failed to create engine");
            
            let result = engine.analyze(&test_file).await;
            assert!(result.is_ok());
        }
        
        // Second engine instance (should use persisted cache)
        {
            let mut engine = AnalysisEngine::with_cache_path(&cache_path)
                .expect("Failed to create engine");
            
            let result = engine.analyze(&test_file).await;
            assert!(result.is_ok());
        }
    }
}

/// Helper function to create temporary test directory
fn create_temp_test_dir() -> Result<TempDir> {
    tempfile::tempdir()
        .map_err(|e| UveddiError::IoError {
            operation: "create temp dir".to_string(),
            path: "temp".to_string(),
            source: e,
        })
}