//! Knowledge Plugin System Example
//!
//! This example demonstrates how to create, register, and use custom knowledge plugins
//! with the Uveddi AI Knowledge Library system (UV-338).

use std::collections::HashMap;
use std::path::PathBuf;
use tokio;
use uveddi::ai::knowledge::context_selection::*;
use uveddi::ai::knowledge::schema::*;
use uveddi::plugins::development::*;
use uveddi::plugins::integration::*;
use uveddi::plugins::knowledge::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Uveddi Knowledge Plugin System Example");
    println!("==========================================");

    // 1. Create and configure the plugin system
    println!("\n1. Initializing Plugin System...");
    let config = create_plugin_system_config();
    let mut plugin_system = KnowledgePluginSystem::new(config);
    println!("✅ Plugin system initialized");

    // 2. Create example plugins
    println!("\n2. Creating Example Plugins...");
    let (antipattern_plugin, enterprise_plugin, framework_plugin) = create_example_plugins();
    println!("✅ Created 3 example plugins:");
    println!("   - Anti-Pattern Plugin (Singleton Abuse detection)");
    println!("   - Enterprise Plugin (SOX compliance)");
    println!("   - Framework Plugin (React performance patterns)");

    // 3. Initialize plugins and load their knowledge
    println!("\n3. Loading Plugin Knowledge...");
    let plugin_knowledge = load_plugin_knowledge(vec![
        Box::new(antipattern_plugin),
        Box::new(enterprise_plugin),
        Box::new(framework_plugin),
    ])
    .await?;

    println!(
        "✅ Loaded knowledge from {} plugins",
        plugin_knowledge.plugin_knowledge.len()
    );
    println!(
        "   - Total patterns: {}",
        plugin_knowledge
            .plugin_knowledge
            .values()
            .map(|k| k.patterns.len())
            .sum::<usize>()
    );

    // 4. Create knowledge integrator with core library
    println!("\n4. Setting up Knowledge Integration...");
    let core_library = create_minimal_core_library();
    let mut integrator = PluginKnowledgeIntegrator::new(core_library);
    integrator.update_plugin_knowledge(plugin_knowledge).await?;
    println!("✅ Knowledge integration configured");

    // 5. Demonstrate context-aware knowledge retrieval
    println!("\n5. Demonstrating Context-Aware Knowledge Retrieval...");

    // Example 1: Rust code analysis context
    let rust_context = create_rust_analysis_context();
    demonstrate_context_retrieval(&integrator, &rust_context, "Rust Codebase").await?;

    // Example 2: JavaScript/React analysis context
    let react_context = create_react_analysis_context();
    demonstrate_context_retrieval(&integrator, &react_context, "React Application").await?;

    // Example 3: Enterprise compliance context
    let enterprise_context = create_enterprise_analysis_context();
    demonstrate_context_retrieval(&integrator, &enterprise_context, "Enterprise Application")
        .await?;

    // 6. Demonstrate plugin-specific features
    println!("\n6. Demonstrating Plugin-Specific Features...");
    demonstrate_enterprise_compliance().await?;
    demonstrate_framework_knowledge().await?;

    // 7. Show performance metrics
    println!("\n7. Performance Metrics...");
    let metrics = integrator.get_metrics();
    println!("📊 Integration Metrics:");
    println!("   - Total patterns: {}", metrics.total_patterns);
    println!("   - Core patterns: {}", metrics.core_patterns);
    println!("   - Plugin patterns: {}", metrics.plugin_patterns);
    println!(
        "   - Avg retrieval time: {:.2}ms",
        metrics.avg_retrieval_time_ms
    );

    println!("\n🎉 Knowledge Plugin System Example Complete!");
    println!("The plugin system successfully demonstrated:");
    println!("  ✓ Plugin registration and initialization");
    println!("  ✓ Custom anti-pattern definitions");
    println!("  ✓ Enterprise compliance validation");
    println!("  ✓ Framework-specific knowledge");
    println!("  ✓ Context-aware knowledge retrieval");
    println!("  ✓ Performance monitoring");

    Ok(())
}

/// Create plugin system configuration
fn create_plugin_system_config() -> KnowledgePluginSystemConfig {
    KnowledgePluginSystemConfig {
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
        plugin_directories: vec![
            PathBuf::from("plugins/community"),
            PathBuf::from("plugins/enterprise"),
        ],
        enable_hot_reload: false,
    }
}

/// Create example plugins for demonstration
fn create_example_plugins() -> (
    ExampleAntiPatternPlugin,
    ExampleEnterprisePlugin,
    ExampleFrameworkPlugin,
) {
    let antipattern_metadata = KnowledgePluginMetadata {
        id: "example-antipattern-1.0.0".to_string(),
        name: "Example Anti-Pattern Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Uveddi Community".to_string(),
        description: "Demonstrates custom anti-pattern definitions including singleton abuse"
            .to_string(),
        plugin_type: KnowledgePluginType::AntiPattern,
        supported_languages: vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::Java,
        ],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities {
            provides_patterns: true,
            provides_detectors: true,
            provides_solutions: true,
            ..Default::default()
        },
        permissions: KnowledgePluginPermissions::default(),
    };

    let enterprise_metadata = KnowledgePluginMetadata {
        id: "enterprise-compliance-1.0.0".to_string(),
        name: "Enterprise Compliance Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Enterprise Security Team".to_string(),
        description: "Provides SOX compliance validation and enterprise-specific patterns"
            .to_string(),
        plugin_type: KnowledgePluginType::Enterprise,
        supported_languages: vec![SourceLanguage::Universal],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities {
            provides_patterns: true,
            provides_solutions: true,
            ..Default::default()
        },
        permissions: KnowledgePluginPermissions {
            data: DataPermissions {
                allow_knowledge_read: true,
                allow_knowledge_write: false,
                allow_analysis_data: true,
            },
            ..Default::default()
        },
    };

    let framework_metadata = KnowledgePluginMetadata {
        id: "react-framework-1.0.0".to_string(),
        name: "React Framework Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Frontend Development Team".to_string(),
        description: "React-specific anti-patterns and performance optimization guidance"
            .to_string(),
        plugin_type: KnowledgePluginType::Framework,
        supported_languages: vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities {
            provides_patterns: true,
            provides_framework_knowledge: true,
            provides_solutions: true,
            ..Default::default()
        },
        permissions: KnowledgePluginPermissions::default(),
    };

    (
        ExampleAntiPatternPlugin::new(antipattern_metadata),
        ExampleEnterprisePlugin::new(enterprise_metadata),
        ExampleFrameworkPlugin::new(framework_metadata),
    )
}

/// Load knowledge from plugins
async fn load_plugin_knowledge(
    mut plugins: Vec<Box<dyn KnowledgePlugin>>,
) -> Result<PluginKnowledgeLibrary, Box<dyn std::error::Error>> {
    let mut library = PluginKnowledgeLibrary::new();

    for plugin in &mut plugins {
        // Initialize plugin
        plugin.initialize().await?;

        // Load knowledge
        let patterns = plugin.get_patterns().await?;
        let detectors = plugin.get_detectors().await?;
        let solutions = plugin.get_solutions().await?;

        let plugin_knowledge = PluginKnowledge {
            plugin_id: plugin.metadata().id.clone(),
            patterns,
            detectors,
            solutions,
        };

        library.merge(plugin_knowledge)?;
    }

    library.validate()?;
    library.optimize_for_performance()?;

    Ok(library)
}

/// Create a minimal core library for demonstration
fn create_minimal_core_library() -> KnowledgeLibrary {
    use uveddi::ai::knowledge::compression::CompressedString;

    let mut library = KnowledgeLibrary::new();

    // Add a sample universal pattern
    let god_object_pattern = PatternKnowledge {
        id: "god_object".to_string(),
        name: "God Object".to_string(),
        definition: CompressedString::new("A class that knows too much or does too much"),
        symptoms: vec![
            CompressedString::new("Class with many responsibilities"),
            CompressedString::new("Large number of methods and fields"),
            CompressedString::new("High coupling with many other classes"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![DetectionMethod {
            method_type: DetectionMethodType::Structural,
            description: CompressedString::new("Count methods and fields in class"),
            thresholds: vec![
                ("method_count".to_string(), 20.0),
                ("field_count".to_string(), 15.0),
            ],
            confidence: 0.85,
        }],
        solutions: vec![SolutionPattern {
            name: "Single Responsibility Principle".to_string(),
            description: CompressedString::new(
                "Break down the class into smaller, focused classes",
            ),
            implementation_steps: vec![
                CompressedString::new("1. Identify distinct responsibilities"),
                CompressedString::new("2. Extract related methods into new classes"),
                CompressedString::new("3. Use composition or delegation"),
            ],
            benefits: vec![
                CompressedString::new("Improved maintainability"),
                CompressedString::new("Better testability"),
                CompressedString::new("Reduced coupling"),
            ],
            effort_level: EffortLevel::High,
        }],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec!["large_class".to_string()],
        tags: vec!["object-oriented".to_string(), "design".to_string()],
        frequency_score: 0.7,
        detection_confidence: 0.85,
    };

    library
        .universal_patterns
        .insert("god_object".to_string(), god_object_pattern);
    library.metadata.pattern_count = 1;

    library
}

/// Create analysis context for Rust codebase
fn create_rust_analysis_context() -> AnalysisContext {
    AnalysisContext {
        language: SourceLanguage::Rust,
        frameworks: vec!["tokio".to_string(), "serde".to_string()],
        detected_patterns: vec![DetectedPattern {
            pattern_id: "custom_singleton_abuse".to_string(),
            confidence: 0.8,
            severity: SeverityLevel::High,
            location: LocationContext {
                file_path: "src/config.rs".to_string(),
                line_range: (45, 120),
                context_name: Some("ConfigManager".to_string()),
            },
            related_patterns: vec!["god_object".to_string()],
        }],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Medium,
            architectural_patterns: vec!["microservices".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 4.2,
                cognitive_complexity: 3.1,
                nesting_depth: 3,
                function_length: 35,
            },
            team_experience: ExperienceLevel::Senior,
        },
        user_intent: UserIntent::RefactoringGuidance,
        token_constraints: TokenConstraints {
            max_tokens: 4000,
            reserved_tokens: 500,
            pattern_token_budget: 2500,
            solution_token_budget: 1000,
        },
    }
}

/// Create analysis context for React application
fn create_react_analysis_context() -> AnalysisContext {
    AnalysisContext {
        language: SourceLanguage::TypeScript,
        frameworks: vec!["React".to_string(), "Next.js".to_string()],
        detected_patterns: vec![DetectedPattern {
            pattern_id: "react_unnecessary_rerender".to_string(),
            confidence: 0.9,
            severity: SeverityLevel::Medium,
            location: LocationContext {
                file_path: "components/UserProfile.tsx".to_string(),
                line_range: (25, 80),
                context_name: Some("UserProfile".to_string()),
            },
            related_patterns: vec![],
        }],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Large,
            architectural_patterns: vec!["component-based".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 3.8,
                cognitive_complexity: 2.9,
                nesting_depth: 4,
                function_length: 28,
            },
            team_experience: ExperienceLevel::Intermediate,
        },
        user_intent: UserIntent::PerformanceOptimization,
        token_constraints: TokenConstraints {
            max_tokens: 6000,
            reserved_tokens: 800,
            pattern_token_budget: 3500,
            solution_token_budget: 1700,
        },
    }
}

/// Create analysis context for enterprise application
fn create_enterprise_analysis_context() -> AnalysisContext {
    AnalysisContext {
        language: SourceLanguage::Java,
        frameworks: vec!["Spring".to_string(), "Hibernate".to_string()],
        detected_patterns: vec![DetectedPattern {
            pattern_id: "enterprise_logging_standard".to_string(),
            confidence: 0.95,
            severity: SeverityLevel::High,
            location: LocationContext {
                file_path: "com/company/service/PaymentService.java".to_string(),
                line_range: (150, 200),
                context_name: Some("processPayment".to_string()),
            },
            related_patterns: vec![],
        }],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::VeryLarge,
            architectural_patterns: vec!["layered".to_string(), "enterprise".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 6.5,
                cognitive_complexity: 4.8,
                nesting_depth: 5,
                function_length: 65,
            },
            team_experience: ExperienceLevel::Expert,
        },
        user_intent: UserIntent::ComplianceValidation,
        token_constraints: TokenConstraints {
            max_tokens: 8000,
            reserved_tokens: 1000,
            pattern_token_budget: 5000,
            solution_token_budget: 2000,
        },
    }
}

/// Demonstrate context-aware knowledge retrieval
async fn demonstrate_context_retrieval(
    integrator: &PluginKnowledgeIntegrator,
    context: &AnalysisContext,
    context_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Analyzing {} Context:", context_name);
    println!("   Language: {:?}", context.language);
    println!("   Frameworks: {:?}", context.frameworks);
    println!("   User Intent: {:?}", context.user_intent);

    let start_time = std::time::Instant::now();
    let integrated_context = integrator.get_integrated_context(context, 5).await?;
    let retrieval_time = start_time.elapsed();

    println!("   ⚡ Retrieval time: {:.2}ms", retrieval_time.as_millis());
    println!("   📊 Results:");
    println!(
        "      - Patterns found: {}",
        integrated_context.patterns.len()
    );
    println!(
        "      - Solutions available: {}",
        integrated_context.solutions.len()
    );
    println!(
        "      - Plugin contributions: {}",
        integrated_context.plugin_contributions.total_plugins
    );

    // Show top patterns
    for (i, ranked_pattern) in integrated_context.patterns.iter().take(3).enumerate() {
        println!(
            "      {}. {} (score: {:.2}, source: {:?})",
            i + 1,
            ranked_pattern.pattern.name,
            ranked_pattern.relevance_score,
            match &ranked_pattern.source {
                PatternSource::Core => "Core",
                PatternSource::Plugin(id) => id,
            }
        );
    }

    Ok(())
}

/// Demonstrate enterprise compliance features
async fn demonstrate_enterprise_compliance() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏢 Enterprise Compliance Demonstration:");

    let metadata = KnowledgePluginMetadata {
        id: "compliance-demo".to_string(),
        name: "Compliance Demo".to_string(),
        version: "1.0.0".to_string(),
        author: "Demo".to_string(),
        description: "Demo".to_string(),
        plugin_type: KnowledgePluginType::Enterprise,
        supported_languages: vec![SourceLanguage::Universal],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    let mut enterprise_plugin = ExampleEnterprisePlugin::new(metadata);
    enterprise_plugin.initialize().await?;

    // Test compliance validation
    let policies = vec![OrganizationPolicy {
        id: "logging_policy".to_string(),
        name: "Structured Logging Policy".to_string(),
        description: "All applications must use structured logging".to_string(),
        patterns: vec!["println_usage".to_string()],
        enforcement: EnforcementLevel::Error,
    }];

    // Test compliant code
    let compliant_code = r#"
        log::info!("Processing payment", 
            "user_id" => user_id, 
            "amount" => amount,
            "correlation_id" => correlation_id
        );
    "#;

    let result = enterprise_plugin
        .validate_against_policies(compliant_code, &policies)
        .await?;
    println!(
        "   ✅ Compliant code validation: {} (score: {:.2})",
        if result.compliant { "PASS" } else { "FAIL" },
        result.score
    );

    // Test non-compliant code
    let non_compliant_code = r#"
        println!("Processing payment for user {}", user_id);
    "#;

    let result = enterprise_plugin
        .validate_against_policies(non_compliant_code, &policies)
        .await?;
    println!(
        "   ❌ Non-compliant code validation: {} (score: {:.2})",
        if result.compliant { "PASS" } else { "FAIL" },
        result.score
    );
    println!("      Violations: {}", result.violations.len());

    // Show compliance knowledge
    let sox_compliance = enterprise_plugin.get_compliance_knowledge("SOX").await?;
    println!("   📋 SOX Compliance Framework:");
    println!(
        "      - Required patterns: {}",
        sox_compliance.required_patterns.len()
    );
    println!(
        "      - Forbidden patterns: {}",
        sox_compliance.forbidden_patterns.len()
    );
    println!("      - Compliance rules: {}", sox_compliance.rules.len());

    Ok(())
}

/// Demonstrate framework-specific knowledge
async fn demonstrate_framework_knowledge() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚛️ Framework Knowledge Demonstration:");

    let metadata = KnowledgePluginMetadata {
        id: "react-demo".to_string(),
        name: "React Demo".to_string(),
        version: "1.0.0".to_string(),
        author: "Demo".to_string(),
        description: "Demo".to_string(),
        plugin_type: KnowledgePluginType::Framework,
        supported_languages: vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    let mut framework_plugin = ExampleFrameworkPlugin::new(metadata);
    framework_plugin.initialize().await?;

    // Get React framework knowledge
    let react_knowledge = framework_plugin.get_framework_knowledge("React").await?;

    if let Some(knowledge) = react_knowledge {
        println!("   📚 React Framework Knowledge:");
        println!(
            "      - Framework: {} v{}",
            knowledge.name, knowledge.version
        );
        println!("      - Patterns: {}", knowledge.patterns.len());
        println!("      - Best practices: {}", knowledge.best_practices.len());
        println!("      - Common pitfalls: {}", knowledge.pitfalls.len());

        println!("   🎯 Best Practices:");
        for (i, practice) in knowledge.best_practices.iter().enumerate() {
            println!("      {}. {}", i + 1, practice);
        }

        println!("   ⚠️ Common Pitfalls:");
        for (i, pitfall) in knowledge.pitfalls.iter().enumerate() {
            println!("      {}. {}", i + 1, pitfall);
        }

        // Show React-specific patterns
        let patterns = framework_plugin.get_patterns().await?;
        println!("   🔍 React-Specific Patterns:");
        for pattern in patterns {
            println!(
                "      - {} (category: {:?})",
                pattern.name, pattern.category
            );
        }
    }

    Ok(())
}
