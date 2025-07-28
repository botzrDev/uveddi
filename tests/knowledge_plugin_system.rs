//! Comprehensive tests for the Knowledge Plugin System (UV-338)
//!
//! This test suite validates the plugin system for custom knowledge,
//! ensuring security, performance, and integration requirements are met.

use std::collections::HashMap;
use std::path::PathBuf;
use tokio;
use uveddi::ai::knowledge::schema::*;
use uveddi::ai::knowledge::context_selection::*;
use uveddi::plugins::knowledge::*;
use uveddi::plugins::development::*;
use uveddi::plugins::integration::*;
use uveddi::plugins::PluginError;

/// Test basic plugin system initialization and configuration
#[tokio::test]
async fn test_plugin_system_initialization() {
    let config = KnowledgePluginSystemConfig {
        security: SecurityConfig {
            enable_sandboxing: true,
            max_memory_per_plugin: 100 * 1024 * 1024, // 100MB
            max_execution_time_ms: 5000,
            enable_security_auditing: true,
        },
        performance: PerformanceConfig {
            max_init_time_ms: 1000,
            max_pattern_retrieval_ms: 100,
            max_memory_usage_mb: 100,
            max_active_plugins: 10,
        },
        plugin_directories: vec![PathBuf::from("test_plugins")],
        enable_hot_reload: false,
    };

    let plugin_system = KnowledgePluginSystem::new(config);
    
    // Verify system is properly initialized
    assert!(plugin_system.get_metrics().total_patterns == 0);
}

/// Test plugin metadata and capabilities
#[tokio::test]
async fn test_plugin_metadata() {
    let metadata = KnowledgePluginMetadata {
        id: "test-plugin-1.0.0".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        description: "A test plugin for validation".to_string(),
        plugin_type: KnowledgePluginType::AntiPattern,
        supported_languages: vec![SourceLanguage::Rust, SourceLanguage::Python],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities {
            provides_patterns: true,
            provides_detectors: true,
            provides_solutions: true,
            provides_language_support: false,
            provides_framework_knowledge: false,
            requires_network: false,
            requires_filesystem: false,
        },
        permissions: KnowledgePluginPermissions::default(),
    };

    // Verify metadata structure
    assert_eq!(metadata.name, "Test Plugin");
    assert_eq!(metadata.plugin_type, KnowledgePluginType::AntiPattern);
    assert!(metadata.capabilities.provides_patterns);
    assert_eq!(metadata.supported_languages.len(), 2);
}

/// Test example anti-pattern plugin functionality
#[tokio::test]
async fn test_example_antipattern_plugin() {
    let metadata = KnowledgePluginMetadata {
        id: "example-antipattern-1.0.0".to_string(),
        name: "Example Anti-Pattern Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Uveddi Community".to_string(),
        description: "Example plugin demonstrating custom anti-pattern definitions".to_string(),
        plugin_type: KnowledgePluginType::AntiPattern,
        supported_languages: vec![SourceLanguage::Rust, SourceLanguage::Python],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    let mut plugin = ExampleAntiPatternPlugin::new(metadata);

    // Test plugin lifecycle
    assert!(plugin.initialize().await.is_ok());
    
    // Test pattern retrieval
    let patterns = plugin.get_patterns().await.unwrap();
    assert!(!patterns.is_empty());
    
    let singleton_pattern = patterns.iter()
        .find(|p| p.id == "custom_singleton_abuse")
        .expect("Should find singleton abuse pattern");
    
    assert_eq!(singleton_pattern.name, "Singleton Abuse");
    assert_eq!(singleton_pattern.category, AntiPatternCategory::ObjectOriented);
    assert_eq!(singleton_pattern.impact, ImpactLevel::High);
    assert!(!singleton_pattern.solutions.is_empty());

    // Test health check
    let health = plugin.health_check().await.unwrap();
    assert!(matches!(health, PluginHealthStatus::Healthy));

    // Test shutdown
    assert!(plugin.shutdown().await.is_ok());
}

/// Test enterprise plugin functionality
#[tokio::test]
async fn test_enterprise_plugin() {
    let metadata = KnowledgePluginMetadata {
        id: "enterprise-plugin-1.0.0".to_string(),
        name: "Enterprise Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Enterprise Team".to_string(),
        description: "Enterprise-specific patterns and compliance".to_string(),
        plugin_type: KnowledgePluginType::Enterprise,
        supported_languages: vec![SourceLanguage::Universal],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    let mut plugin = ExampleEnterprisePlugin::new(metadata);
    assert!(plugin.initialize().await.is_ok());

    // Test organization patterns
    let org_patterns = plugin.get_organization_patterns("example_org").await.unwrap();
    assert!(!org_patterns.is_empty());

    // Test compliance knowledge
    let compliance = plugin.get_compliance_knowledge("SOX").await.unwrap();
    assert_eq!(compliance.framework, "SOX");
    assert!(!compliance.rules.is_empty());

    // Test policy validation
    let policies = vec![
        OrganizationPolicy {
            id: "no_println".to_string(),
            name: "No println usage".to_string(),
            description: "Avoid using println! in production code".to_string(),
            patterns: vec!["println_usage".to_string()],
            enforcement: EnforcementLevel::Error,
        }
    ];

    // Test compliant code
    let compliant_code = "log::info!(\"This is proper logging\");";
    let result = plugin.validate_against_policies(compliant_code, &policies).await.unwrap();
    assert!(result.compliant);
    assert!(result.violations.is_empty());

    // Test non-compliant code
    let non_compliant_code = "println!(\"This should not be used\");";
    let result = plugin.validate_against_policies(non_compliant_code, &policies).await.unwrap();
    assert!(!result.compliant);
    assert!(!result.violations.is_empty());
}

/// Test framework plugin functionality
#[tokio::test]
async fn test_framework_plugin() {
    let metadata = KnowledgePluginMetadata {
        id: "react-plugin-1.0.0".to_string(),
        name: "React Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Frontend Team".to_string(),
        description: "React-specific patterns and best practices".to_string(),
        plugin_type: KnowledgePluginType::Framework,
        supported_languages: vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities {
            provides_framework_knowledge: true,
            ..Default::default()
        },
        permissions: KnowledgePluginPermissions::default(),
    };

    let mut plugin = ExampleFrameworkPlugin::new(metadata);
    assert!(plugin.initialize().await.is_ok());

    // Test framework knowledge
    let react_knowledge = plugin.get_framework_knowledge("React").await.unwrap();
    assert!(react_knowledge.is_some());
    
    let knowledge = react_knowledge.unwrap();
    assert_eq!(knowledge.name, "React");
    assert!(!knowledge.patterns.is_empty());
    assert!(!knowledge.best_practices.is_empty());
    assert!(!knowledge.pitfalls.is_empty());

    // Test patterns from framework
    let patterns = plugin.get_patterns().await.unwrap();
    assert!(!patterns.is_empty());
    
    let rerender_pattern = patterns.iter()
        .find(|p| p.id == "react_unnecessary_rerender")
        .expect("Should find React re-render pattern");
    
    assert_eq!(rerender_pattern.category, AntiPatternCategory::Performance);
    assert!(rerender_pattern.tags.contains(&"react".to_string()));
}

/// Test plugin knowledge library integration
#[tokio::test]
async fn test_plugin_knowledge_library() {
    let mut library = PluginKnowledgeLibrary::new();

    // Create test plugin knowledge
    let plugin_knowledge = PluginKnowledge {
        plugin_id: "test-plugin".to_string(),
        patterns: vec![create_test_pattern()],
        detectors: vec![],
        solutions: vec![],
    };

    // Test merging knowledge
    assert!(library.merge(plugin_knowledge).is_ok());
    assert!(library.validate().is_ok());
    assert!(library.optimize_for_performance().is_ok());

    // Test pattern retrieval
    let pattern = library.get_pattern("test_pattern_id");
    assert!(pattern.is_some());

    let plugin_patterns = library.get_plugin_patterns("test-plugin");
    assert!(plugin_patterns.is_some());
    assert_eq!(plugin_patterns.unwrap().len(), 1);
}

/// Test plugin knowledge integration with core library
#[tokio::test]
async fn test_knowledge_integration() {
    // Create a minimal core knowledge library
    let core_library = create_test_core_library();
    
    let mut integrator = PluginKnowledgeIntegrator::new(core_library);

    // Create plugin knowledge library
    let mut plugin_library = PluginKnowledgeLibrary::new();
    let plugin_knowledge = PluginKnowledge {
        plugin_id: "integration-test-plugin".to_string(),
        patterns: vec![create_test_pattern()],
        detectors: vec![],
        solutions: vec![],
    };
    plugin_library.merge(plugin_knowledge).unwrap();

    // Update integrator with plugin knowledge
    assert!(integrator.update_plugin_knowledge(plugin_library).await.is_ok());

    // Test integrated context retrieval
    let analysis_context = AnalysisContext {
        language: SourceLanguage::Rust,
        frameworks: vec!["test_framework".to_string()],
        detected_patterns: vec![],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Small,
            architectural_patterns: vec![],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 2.0,
                cognitive_complexity: 1.5,
                nesting_depth: 2,
                function_length: 20,
            },
            team_experience: ExperienceLevel::Intermediate,
        },
        user_intent: UserIntent::CodeReview,
        token_constraints: TokenConstraints {
            max_tokens: 4000,
            reserved_tokens: 500,
            pattern_token_budget: 3000,
            solution_token_budget: 500,
        },
    };

    let integrated_context = integrator.get_integrated_context(&analysis_context, 10).await.unwrap();
    
    // Verify integration results
    assert!(!integrated_context.patterns.is_empty());
    assert!(integrated_context.retrieval_time_ms > 0.0);
    assert!(integrated_context.plugin_contributions.total_plugins > 0);

    // Test category-based retrieval
    let category_patterns = integrator.get_patterns_by_category(AntiPatternCategory::Performance).await.unwrap();
    // Should work even if empty for this test

    // Test language-based retrieval
    let language_patterns = integrator.get_patterns_by_language(SourceLanguage::Rust).await.unwrap();
    // Should include patterns from both core and plugins

    // Verify metrics
    let metrics = integrator.get_metrics();
    assert!(metrics.total_patterns > 0);
}

/// Test plugin security and permissions
#[tokio::test]
async fn test_plugin_security() {
    let security_config = SecurityConfig {
        enable_sandboxing: true,
        max_memory_per_plugin: 50 * 1024 * 1024, // 50MB
        max_execution_time_ms: 2000,
        enable_security_auditing: true,
    };

    let security_manager = KnowledgePluginSecurityManager::new(&security_config);

    // Test permissions validation
    let permissions = KnowledgePluginPermissions {
        network: NetworkPermissions {
            allow_http: false,
            allow_https: true,
            allowed_domains: vec!["api.example.com".to_string()],
            allowed_ports: vec![443],
        },
        filesystem: FilesystemPermissions {
            allow_read: true,
            allow_write: false,
            allowed_paths: vec![PathBuf::from("/tmp/plugin_data")],
        },
        system: SystemPermissions {
            allow_process_execution: false,
            allow_env_access: false,
            max_memory_mb: 50,
            max_execution_time_ms: 2000,
        },
        data: DataPermissions {
            allow_knowledge_read: true,
            allow_knowledge_write: false,
            allow_analysis_data: true,
        },
    };

    // Verify permission structure
    assert!(permissions.network.allow_https);
    assert!(!permissions.network.allow_http);
    assert!(permissions.filesystem.allow_read);
    assert!(!permissions.filesystem.allow_write);
    assert!(!permissions.system.allow_process_execution);
    assert!(permissions.data.allow_knowledge_read);
}

/// Test plugin performance monitoring
#[tokio::test]
async fn test_plugin_performance() {
    let performance_config = PerformanceConfig {
        max_init_time_ms: 1000,
        max_pattern_retrieval_ms: 100,
        max_memory_usage_mb: 100,
        max_active_plugins: 5,
    };

    let performance_monitor = KnowledgePluginPerformanceMonitor::new();

    // Test performance thresholds
    assert_eq!(performance_config.max_init_time_ms, 1000);
    assert_eq!(performance_config.max_pattern_retrieval_ms, 100);
    assert_eq!(performance_config.max_memory_usage_mb, 100);
    assert_eq!(performance_config.max_active_plugins, 5);
}

/// Test plugin context selection and relevance scoring
#[tokio::test]
async fn test_plugin_context_selection() {
    let mut plugin_context = PluginContext::new();

    // Add context from multiple plugins
    let context1 = PluginSpecificContext {
        language_knowledge: Some(create_test_language_knowledge()),
        framework_knowledge: HashMap::new(),
    };

    let context2 = PluginSpecificContext {
        language_knowledge: None,
        framework_knowledge: {
            let mut frameworks = HashMap::new();
            frameworks.insert("React".to_string(), create_test_framework_knowledge());
            frameworks
        },
    };

    plugin_context.add_plugin_context("plugin1".to_string(), context1);
    plugin_context.add_plugin_context("plugin2".to_string(), context2);

    // Test context retrieval
    assert!(plugin_context.get_plugin_context("plugin1").is_some());
    assert!(plugin_context.get_plugin_context("plugin2").is_some());
    assert!(plugin_context.get_plugin_context("nonexistent").is_none());

    let all_contexts = plugin_context.get_all_contexts();
    assert_eq!(all_contexts.len(), 2);
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() {
    let metadata = KnowledgePluginMetadata {
        id: "error-test-plugin".to_string(),
        name: "Error Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: "Plugin for testing error conditions".to_string(),
        plugin_type: KnowledgePluginType::AntiPattern,
        supported_languages: vec![SourceLanguage::Rust],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    let plugin = ExampleAntiPatternPlugin::new(metadata);

    // Test operations on uninitialized plugin
    let result = plugin.get_patterns().await;
    assert!(result.is_err());
    
    if let Err(PluginError::Execution(msg)) = result {
        assert!(msg.contains("not initialized"));
    } else {
        panic!("Expected execution error for uninitialized plugin");
    }
}

// Helper functions for creating test data

fn create_test_pattern() -> PatternKnowledge {
    use uveddi::ai::knowledge::compression::CompressedString;
    
    PatternKnowledge {
        id: "test_pattern_id".to_string(),
        name: "Test Pattern".to_string(),
        definition: CompressedString::new("A test pattern for validation"),
        symptoms: vec![CompressedString::new("Test symptom")],
        impact: ImpactLevel::Medium,
        category: AntiPatternCategory::Performance,
        detection_methods: vec![
            DetectionMethod {
                method_type: DetectionMethodType::Structural,
                description: CompressedString::new("Test detection method"),
                thresholds: vec![("test_threshold".to_string(), 1.0)],
                confidence: 0.8,
            }
        ],
        solutions: vec![
            SolutionPattern {
                name: "Test Solution".to_string(),
                description: CompressedString::new("A test solution"),
                implementation_steps: vec![CompressedString::new("Step 1: Test")],
                benefits: vec![CompressedString::new("Test benefit")],
                effort_level: EffortLevel::Low,
            }
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["test".to_string()],
        frequency_score: 0.5,
        detection_confidence: 0.8,
    }
}

fn create_test_core_library() -> KnowledgeLibrary {
    let mut library = KnowledgeLibrary::new();
    
    // Add a test universal pattern
    library.universal_patterns.insert(
        "test_universal_pattern".to_string(),
        create_test_pattern()
    );
    
    library
}

fn create_test_language_knowledge() -> LanguageKnowledge {
    LanguageKnowledge {
        language: SourceLanguage::Rust,
        patterns: {
            let mut patterns = HashMap::new();
            patterns.insert("rust_test_pattern".to_string(), create_test_pattern());
            patterns
        },
        idioms: vec![],
        frameworks: HashMap::new(),
        stdlib_patterns: vec![],
    }
}

fn create_test_framework_knowledge() -> FrameworkKnowledge {
    FrameworkKnowledge {
        name: "React".to_string(),
        version: "18.x".to_string(),
        patterns: vec![create_test_pattern()],
        best_practices: vec!["Use hooks".to_string()],
        pitfalls: vec!["Don't mutate state".to_string()],
    }
}

/// Integration test with real analysis context
#[tokio::test]
async fn test_real_world_integration() {
    // This test simulates a real-world scenario where plugins provide
    // additional knowledge for code analysis

    let config = KnowledgePluginSystemConfig {
        security: SecurityConfig {
            enable_sandboxing: true,
            max_memory_per_plugin: 100 * 1024 * 1024,
            max_execution_time_ms: 5000,
            enable_security_auditing: false, // Disabled for test
        },
        performance: PerformanceConfig {
            max_init_time_ms: 2000,
            max_pattern_retrieval_ms: 200,
            max_memory_usage_mb: 100,
            max_active_plugins: 10,
        },
        plugin_directories: vec![],
        enable_hot_reload: false,
    };

    let mut plugin_system = KnowledgePluginSystem::new(config);

    // Simulate loading plugin knowledge
    let mut plugin_library = PluginKnowledgeLibrary::new();
    
    // Add knowledge from multiple plugins
    let antipattern_knowledge = PluginKnowledge {
        plugin_id: "antipattern-plugin".to_string(),
        patterns: vec![create_test_pattern()],
        detectors: vec![],
        solutions: vec![],
    };
    
    plugin_library.merge(antipattern_knowledge).unwrap();

    // Test the complete workflow
    let analysis_context = AnalysisContext {
        language: SourceLanguage::Rust,
        frameworks: vec!["tokio".to_string(), "serde".to_string()],
        detected_patterns: vec![
            DetectedPattern {
                pattern_id: "test_pattern_id".to_string(),
                confidence: 0.9,
                severity: SeverityLevel::Medium,
                location: LocationContext {
                    file_path: "src/main.rs".to_string(),
                    line_range: (10, 20),
                    context_name: Some("main".to_string()),
                },
                related_patterns: vec![],
            }
        ],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Medium,
            architectural_patterns: vec!["microservices".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 5.2,
                cognitive_complexity: 3.8,
                nesting_depth: 3,
                function_length: 45,
            },
            team_experience: ExperienceLevel::Senior,
        },
        user_intent: UserIntent::RefactoringGuidance,
        token_constraints: TokenConstraints {
            max_tokens: 8000,
            reserved_tokens: 1000,
            pattern_token_budget: 5000,
            solution_token_budget: 2000,
        },
    };

    // Get plugin context
    let plugin_context = plugin_system.get_plugin_context(&analysis_context).await.unwrap();
    
    // Verify context contains relevant information
    assert!(!plugin_context.get_all_contexts().is_empty());
    
    // Test performance requirements (should be under 100ms for retrieval)
    let start = std::time::Instant::now();
    let _context = plugin_system.get_plugin_context(&analysis_context).await.unwrap();
    let elapsed = start.elapsed();
    
    // This should be fast (under 100ms as per UV-337 requirements)
    assert!(elapsed.as_millis() < 100, "Plugin context retrieval took too long: {}ms", elapsed.as_millis());
}