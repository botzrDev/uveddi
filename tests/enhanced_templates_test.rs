//! Comprehensive tests for UV-336 Enhanced AI Prompt Templates
//!
//! This test suite validates the enhanced prompt template system with knowledge
//! library integration, ensuring template quality, knowledge injection, and
//! AI integration functionality.

#![cfg(feature = "ai")]

use std::collections::HashMap;
use std::time::Duration;
use uveddi::ai::knowledge::context_selection::*;
use uveddi::ai::knowledge::schema::*;
use uveddi::ai::prompts::enhanced_templates::*;
use uveddi::ai::prompts::knowledge_integration::*;

#[tokio::test]
async fn test_enhanced_template_system_creation() {
    println!("🧪 Testing Enhanced Template System Creation");

    // Create mock context selector
    let enhanced_patterns = create_mock_enhanced_patterns();
    let config = ContextSelectionConfig::default();
    let context_selector = DynamicContextSelector::new(enhanced_patterns, config);

    // Create template system
    let optimization_config = TemplateOptimizationConfig::default();
    let template_system = EnhancedPromptTemplateSystem::new(context_selector, optimization_config);

    println!("✅ Enhanced template system created successfully");
}

#[tokio::test]
async fn test_god_object_template_generation() {
    println!("🧪 Testing God Object Template Generation");

    let mut template_system = create_test_template_system().await;
    let analysis_context = create_god_object_analysis_context();

    let enhanced_prompt = template_system
        .generate_enhanced_prompt(&analysis_context, AnalysisType::GodObjectAnalysis)
        .await
        .expect("Failed to generate enhanced prompt");

    // Validate prompt structure
    assert!(!enhanced_prompt.content.is_empty());
    assert!(enhanced_prompt.content.contains("God Object"));
    assert!(enhanced_prompt.token_usage.total_tokens > 0);
    assert!(enhanced_prompt.knowledge_context.patterns.len() > 0);

    println!(
        "✅ God Object template generated with {} tokens",
        enhanced_prompt.token_usage.total_tokens
    );
}

#[tokio::test]
async fn test_knowledge_injection_points() {
    println!("🧪 Testing Knowledge Injection Points");

    let mut template_system = create_test_template_system().await;
    let analysis_context = create_comprehensive_analysis_context();

    let enhanced_prompt = template_system
        .generate_enhanced_prompt(&analysis_context, AnalysisType::GeneralAnalysis)
        .await
        .expect("Failed to generate enhanced prompt");

    // Validate injection points were used
    let content = &enhanced_prompt.content;

    // Check for pattern definitions
    assert!(content.contains("Definition:") || content.contains("Pattern:"));

    // Check for language-specific content
    assert!(content.contains("Rust") || content.contains("Python"));

    // Check for solution patterns
    assert!(content.contains("Solution") || content.contains("Refactor"));

    println!("✅ Knowledge injection points validated");
}

#[tokio::test]
async fn test_token_budget_optimization() {
    println!("🧪 Testing Token Budget Optimization");

    let mut template_system = create_test_template_system().await;
    let mut analysis_context = create_comprehensive_analysis_context();

    // Set strict token budget
    analysis_context.token_constraints.max_tokens = 1000;
    analysis_context.token_constraints.available_tokens = 800;

    let enhanced_prompt = template_system
        .generate_enhanced_prompt(&analysis_context, AnalysisType::GodObjectAnalysis)
        .await
        .expect("Failed to generate enhanced prompt");

    // Validate token budget compliance
    assert!(enhanced_prompt.token_usage.total_tokens <= 1000);
    assert!(enhanced_prompt.token_usage.efficiency_score >= 0.8);

    println!(
        "✅ Token budget optimization: {}/{} tokens ({}% efficiency)",
        enhanced_prompt.token_usage.total_tokens,
        1000,
        (enhanced_prompt.token_usage.efficiency_score * 100.0) as u32
    );
}

#[tokio::test]
async fn test_language_specific_templates() {
    println!("🧪 Testing Language-Specific Templates");

    let mut template_system = create_test_template_system().await;

    // Test Rust-specific template
    let mut rust_context = create_comprehensive_analysis_context();
    rust_context.language = SourceLanguage::Rust;

    let rust_prompt = template_system
        .generate_enhanced_prompt(&rust_context, AnalysisType::GodObjectAnalysis)
        .await
        .expect("Failed to generate Rust prompt");

    // Test Python-specific template
    let mut python_context = create_comprehensive_analysis_context();
    python_context.language = SourceLanguage::Python;

    let python_prompt = template_system
        .generate_enhanced_prompt(&python_context, AnalysisType::GodObjectAnalysis)
        .await
        .expect("Failed to generate Python prompt");

    // Validate language-specific content
    assert_ne!(rust_prompt.content, python_prompt.content);

    println!("✅ Language-specific templates generated successfully");
}

#[tokio::test]
async fn test_ai_integration_layer() {
    println!("🧪 Testing AI Integration Layer");

    let template_system = create_test_template_system().await;
    let ai_engine = create_mock_ai_engine();
    let config = IntegrationConfig::default();

    let mut knowledge_ai = KnowledgeEnhancedAIEngine::new(ai_engine, template_system, config);

    let issue = create_mock_architectural_issue();
    let context = create_comprehensive_analysis_context();

    let enhanced_response = knowledge_ai
        .generate_enhanced_explanation(&issue, &context)
        .await
        .expect("Failed to generate enhanced explanation");

    // Validate enhanced response
    assert!(!enhanced_response.explanation.is_empty());
    assert!(enhanced_response.enhancement_metadata.patterns_used > 0);
    assert!(enhanced_response.confidence_score > 0.0);

    println!(
        "✅ AI integration layer working: {} patterns used",
        enhanced_response.enhancement_metadata.patterns_used
    );
}

#[tokio::test]
async fn test_template_performance() {
    println!("🧪 Testing Template Performance");

    let mut template_system = create_test_template_system().await;
    let analysis_context = create_comprehensive_analysis_context();

    let start_time = std::time::Instant::now();

    let enhanced_prompt = template_system
        .generate_enhanced_prompt(&analysis_context, AnalysisType::GodObjectAnalysis)
        .await
        .expect("Failed to generate enhanced prompt");

    let generation_time = start_time.elapsed();

    // Validate performance targets
    assert!(generation_time < Duration::from_millis(50)); // <50ms target
    assert!(enhanced_prompt.quality_metrics.relevance_score >= 0.8);

    println!(
        "✅ Template generation performance: {}ms",
        generation_time.as_millis()
    );
}

#[tokio::test]
async fn test_comprehensive_template_coverage() {
    println!("🧪 Testing Comprehensive Template Coverage");

    let mut template_system = create_test_template_system().await;
    let analysis_context = create_comprehensive_analysis_context();

    let analysis_types = vec![
        AnalysisType::GodObjectAnalysis,
        AnalysisType::TightCouplingAnalysis,
        AnalysisType::DeadCodeAnalysis,
        AnalysisType::PerformanceAnalysis,
        AnalysisType::GeneralAnalysis,
    ];

    for analysis_type in analysis_types {
        let enhanced_prompt = template_system
            .generate_enhanced_prompt(&analysis_context, analysis_type.clone())
            .await
            .expect(&format!(
                "Failed to generate template for {:?}",
                analysis_type
            ));

        assert!(!enhanced_prompt.content.is_empty());
        assert!(enhanced_prompt.token_usage.total_tokens > 0);

        println!("✅ {:?} template generated successfully", analysis_type);
    }

    println!("✅ All template types covered");
}

// Helper functions for test setup

async fn create_test_template_system() -> EnhancedPromptTemplateSystem {
    let enhanced_patterns = create_mock_enhanced_patterns();
    let config = ContextSelectionConfig::default();
    let context_selector = DynamicContextSelector::new(enhanced_patterns, config);
    let optimization_config = TemplateOptimizationConfig::default();

    EnhancedPromptTemplateSystem::new(context_selector, optimization_config)
}

fn create_mock_enhanced_patterns(
) -> HashMap<String, crate::ai::knowledge::language_integration::EnhancedPattern> {
    let mut patterns = HashMap::new();

    // Create mock God Object pattern
    let god_object_pattern = crate::ai::knowledge::language_integration::EnhancedPattern {
        universal: create_mock_pattern_knowledge("god_object", "God Object"),
        language_contexts: HashMap::new(),
        completeness_score: 0.9,
        confidence_score: 0.85,
    };

    patterns.insert("god_object".to_string(), god_object_pattern);
    patterns
}

fn create_mock_pattern_knowledge(id: &str, name: &str) -> PatternKnowledge {
    PatternKnowledge {
        id: id.to_string(),
        name: name.to_string(),
        definition: CompressedString::new(&format!("{} anti-pattern definition", name)),
        symptoms: vec![
            CompressedString::new("Excessive methods and fields"),
            CompressedString::new("Multiple unrelated responsibilities"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        detection_methods: vec![],
        solutions: vec![],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec!["object-oriented".to_string()],
        frequency_score: 0.8,
        detection_confidence: 0.85,
    }
}

fn create_god_object_analysis_context() -> AnalysisContext {
    AnalysisContext {
        language: SourceLanguage::Rust,
        frameworks: vec!["tokio".to_string()],
        detected_patterns: vec![DetectedPattern {
            pattern_id: "god_object".to_string(),
            confidence: 0.9,
            severity: SeverityLevel::High,
            location: LocationContext {
                file_path: "src/main.rs".to_string(),
                line_range: (1, 100),
                function_name: Some("UserManager".to_string()),
            },
            related_patterns: vec![],
        }],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Medium,
            architectural_patterns: vec!["MVC".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 15.0,
                cognitive_complexity: 20.0,
                nesting_depth: 4,
            },
            team_experience: ExperienceLevel::Intermediate,
        },
        user_intent: UserIntent {
            goal: AnalysisGoal::CodeQualityImprovement,
            explanation_depth: ExplanationDepth::Detailed,
            focus_areas: vec![AntiPatternCategory::ObjectOriented],
            solution_preferences: vec![SolutionType::Refactoring],
        },
        token_constraints: TokenConstraints {
            max_tokens: 2000,
            reserved_tokens: 200,
            available_tokens: 1800,
            tokens_per_pattern: 300,
        },
    }
}

fn create_comprehensive_analysis_context() -> AnalysisContext {
    create_god_object_analysis_context()
}

fn create_mock_ai_engine() -> crate::ai::engine::AIEngine {
    // Create a mock AI engine for testing
    crate::ai::engine::AIEngine::new_mock()
}

fn create_mock_architectural_issue() -> crate::database::models::ArchitecturalIssue {
    crate::database::models::ArchitecturalIssue {
        id: 1,
        file_path: "src/main.rs".to_string(),
        issue_type: "god_object".to_string(),
        severity: "high".to_string(),
        description: "Large class with too many responsibilities".to_string(),
        line_start: 1,
        line_end: 100,
        function_name: Some("UserManager".to_string()),
        suggestion: Some("Consider breaking into smaller classes".to_string()),
        confidence_score: Some(0.9),
    }
}

impl Default for TemplateOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_token_compression: true,
            max_template_size: 2000,
            enable_caching: true,
            cache_ttl: Duration::from_secs(300),
            enable_performance_tracking: true,
            optimization_level: OptimizationLevel::Balanced,
        }
    }
}

impl Default for ContextSelectionConfig {
    fn default() -> Self {
        Self {
            max_patterns: 5,
            token_budget: 2000,
            relevance_threshold: 0.7,
            scoring_weights: ScoringWeights::default(),
            enable_adaptive_learning: true,
            selection_strategy: SelectionStrategy::TopScoring,
        }
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            language_match: 0.25,
            framework_relevance: 0.20,
            pattern_similarity: 0.20,
            detection_confidence: 0.15,
            solution_applicability: 0.15,
            usage_frequency: 0.05,
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_enhancement: true,
            fallback_on_error: true,
            max_enhancement_time: Duration::from_millis(100),
            enable_quality_tracking: true,
        }
    }
}

// Mock implementations for testing
impl crate::ai::engine::AIEngine {
    pub fn new_mock() -> Self {
        // Mock implementation for testing
        Self::new(crate::ai::config::AIConfig::default()).expect("Failed to create mock AI engine")
    }
}
