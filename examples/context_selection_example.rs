//! Example demonstrating the Dynamic Context Selection Engine (UV-334)
//!
//! This example shows how to use the intelligent context selection system to
//! dynamically choose the most relevant knowledge from the pattern library
//! based on analysis context and AI prompt requirements.

use std::collections::HashMap;
use uveddi::ai::knowledge::{
    context_selection::*,
    language_integration::{EnhancedPattern, LanguageContext, PatternMetadata},
    schema::*,
    compression::CompressedString,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 UV-334 Dynamic Context Selection Engine Demo");
    println!("================================================\n");

    // 1. Create enhanced patterns (normally populated from UV-332 and UV-333)
    let enhanced_patterns = create_sample_enhanced_patterns();
    
    // 2. Configure the context selector
    let config = ContextSelectionConfig {
        max_patterns: 5,
        token_budget: 2000,
        relevance_threshold: 0.4,
        scoring_weights: ScoringWeights {
            language_match: 0.30,
            framework_relevance: 0.20,
            pattern_similarity: 0.25,
            detection_confidence: 0.15,
            solution_applicability: 0.08,
            usage_frequency: 0.02,
        },
        enable_adaptive_learning: true,
        selection_strategy: SelectionStrategy::Diversified,
    };
    
    // 3. Create the dynamic context selector
    let mut selector = DynamicContextSelector::new(enhanced_patterns, config);
    
    // 4. Simulate an analysis context (what Uveddi detected in the codebase)
    let analysis_context = create_sample_analysis_context();
    
    println!("📊 Analysis Context:");
    println!("   Language: {:?}", analysis_context.language);
    println!("   Frameworks: {:?}", analysis_context.frameworks);
    println!("   Detected Patterns: {} issues found", analysis_context.detected_patterns.len());
    for pattern in &analysis_context.detected_patterns {
        println!("     - {} (confidence: {:.1}%)", pattern.pattern_id, pattern.confidence * 100.0);
    }
    println!("   Token Budget: {} tokens\n", analysis_context.token_constraints.available_tokens);
    
    // 5. Perform intelligent context selection
    println!("🧠 Performing Intelligent Context Selection...");
    let selected_context = selector.select_context(&analysis_context)?;
    
    // 6. Display results
    println!("\n✅ Context Selection Results:");
    println!("   Strategy Used: {:?}", selected_context.metadata.strategy_used);
    println!("   Selection Time: {}ms", selected_context.metadata.selection_time_ms);
    println!("   Patterns Considered: {}", selected_context.metadata.total_candidates);
    println!("   Patterns Selected: {}", selected_context.metadata.selected_count);
    println!("   Average Relevance: {:.3}", selected_context.metadata.average_relevance);
    println!("   Token Utilization: {:.1}% ({}/{} tokens)",
             selected_context.metadata.token_usage.utilization_percent,
             selected_context.metadata.token_usage.used_tokens,
             selected_context.metadata.token_usage.available_tokens);
    
    println!("\n📋 Selected Patterns for AI Context:");
    for (i, scored_pattern) in selected_context.patterns.iter().enumerate() {
        let pattern = &scored_pattern.candidate.pattern.universal;
        println!("   {}. {} (Score: {:.3})", 
                 i + 1, 
                 pattern.name, 
                 scored_pattern.relevance_score);
        println!("      Category: {:?}", pattern.category);
        println!("      Match Type: {:?}", scored_pattern.candidate.match_type);
        
        // Show detailed score breakdown
        let breakdown = &scored_pattern.score_breakdown;
        println!("      Score Breakdown:");
        println!("        - Language: {:.3}", breakdown.language_score);
        println!("        - Framework: {:.3}", breakdown.framework_score);
        println!("        - Similarity: {:.3}", breakdown.similarity_score);
        println!("        - Confidence: {:.3}", breakdown.confidence_score);
        println!("        - Solution: {:.3}", breakdown.solution_score);
        println!();
    }
    
    // 7. Show performance metrics
    println!("⚡ Performance Metrics:");
    let perf = &selected_context.selection_metrics;
    println!("   Candidate Generation: {}ms", perf.candidate_generation_ms);
    println!("   Pattern Scoring: {}ms", perf.scoring_time_ms);
    println!("   Strategy Application: {}ms", perf.strategy_time_ms);
    println!("   Token Optimization: {}ms", perf.optimization_time_ms);
    println!("   Total Time: {}ms", perf.total_time_ms);
    
    // 8. Demonstrate adaptive learning
    println!("\n🎓 Adaptive Learning Demo:");
    println!("   Simulating user feedback...");
    let mut analytics = ContextAnalytics::new();
    analytics.add_feedback("god_object".to_string(), 5, "Excellent detection!".to_string());
    analytics.add_feedback("tight_coupling".to_string(), 3, "Somewhat helpful".to_string());
    
    println!("   Usage scores updated:");
    println!("     - god_object: {:.3}", analytics.get_usage_score("god_object"));
    println!("     - tight_coupling: {:.3}", analytics.get_usage_score("tight_coupling"));
    println!("     - new_pattern: {:.3} (neutral)", analytics.get_usage_score("new_pattern"));
    
    println!("\n🎯 Context selection completed successfully!");
    println!("The AI system now has intelligently selected, contextually relevant");
    println!("knowledge to provide targeted guidance and explanations.");
    
    Ok(())
}

fn create_sample_enhanced_patterns() -> HashMap<String, EnhancedPattern> {
    let mut patterns = HashMap::new();
    
    // Create God Object pattern
    let mut god_object_contexts = HashMap::new();
    god_object_contexts.insert(SourceLanguage::Rust, LanguageContext {
        language: SourceLanguage::Rust,
        specific_symptoms: vec![
            CompressedString::new("Large struct with many fields"),
            CompressedString::new("Impl block with many methods"),
        ],
        detection_methods: vec![
            DetectionMethod::MetricThreshold {
                metric_name: "struct_field_count".to_string(),
                threshold: 20.0,
                operator: ComparisonOperator::GreaterThan,
            }
        ],
        solutions: vec![],
        tools: vec![CompressedString::new("clippy::too_many_arguments")],
        frameworks: vec![],
        stdlib_guidance: Some(CompressedString::new("Use composition over inheritance")),
    });
    
    let god_object = EnhancedPattern {
        universal: PatternKnowledge {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            definition: CompressedString::new("A class or struct that knows too much or does too much"),
            symptoms: vec![
                CompressedString::new("Large class/struct size"),
                CompressedString::new("Too many responsibilities"),
            ],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![],
            solutions: vec![
                SolutionPattern {
                    id: "decompose_god_object".to_string(),
                    title: "Decompose into Smaller Components".to_string(),
                    implementation: CompressedString::new("Break down the large class into smaller, focused components"),
                    examples: vec![],
                    effort_level: EffortLevel::High,
                    prerequisites: vec!["Understanding of Single Responsibility Principle".to_string()],
                    expected_impact: ImpactLevel::High,
                }
            ],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec!["tight_coupling".to_string(), "spaghetti_code".to_string()],
            tags: vec!["oop".to_string(), "design".to_string(), "maintainability".to_string()],
            frequency_score: 0.8,
            detection_confidence: 0.9,
        },
        language_contexts: god_object_contexts,
        metadata: PatternMetadata {
            complete_languages: vec![SourceLanguage::Rust],
            partial_languages: vec![SourceLanguage::Python, SourceLanguage::JavaScript],
            completeness_score: 0.85,
            language_confidence: [
                (SourceLanguage::Rust, 0.9),
                (SourceLanguage::Python, 0.7),
                (SourceLanguage::JavaScript, 0.6),
            ].into_iter().collect(),
        },
    };
    
    patterns.insert("god_object".to_string(), god_object);
    
    // Create Tight Coupling pattern
    let mut coupling_contexts = HashMap::new();
    coupling_contexts.insert(SourceLanguage::Rust, LanguageContext {
        language: SourceLanguage::Rust,
        specific_symptoms: vec![
            CompressedString::new("Excessive use of pub fields"),
            CompressedString::new("Direct struct field access"),
        ],
        detection_methods: vec![],
        solutions: vec![],
        tools: vec![CompressedString::new("cargo-deps for dependency analysis")],
        frameworks: vec![],
        stdlib_guidance: Some(CompressedString::new("Use traits for loose coupling")),
    });
    
    let tight_coupling = EnhancedPattern {
        universal: PatternKnowledge {
            id: "tight_coupling".to_string(),
            name: "Tight Coupling".to_string(),
            definition: CompressedString::new("Classes or modules that are overly dependent on each other"),
            symptoms: vec![
                CompressedString::new("Changes in one module require changes in many others"),
                CompressedString::new("Difficult to test components in isolation"),
            ],
            impact: ImpactLevel::Medium,
            category: AntiPatternCategory::Architectural,
            detection_methods: vec![],
            solutions: vec![],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec!["god_object".to_string()],
            tags: vec!["coupling".to_string(), "architecture".to_string()],
            frequency_score: 0.7,
            detection_confidence: 0.8,
        },
        language_contexts: coupling_contexts,
        metadata: PatternMetadata {
            complete_languages: vec![SourceLanguage::Rust],
            partial_languages: vec![],
            completeness_score: 0.9,
            language_confidence: [(SourceLanguage::Rust, 0.8)].into_iter().collect(),
        },
    };
    
    patterns.insert("tight_coupling".to_string(), tight_coupling);
    
    patterns
}

fn create_sample_analysis_context() -> AnalysisContext {
    AnalysisContext {
        language: SourceLanguage::Rust,
        frameworks: vec!["tokio".to_string(), "serde".to_string()],
        detected_patterns: vec![
            DetectedPattern {
                pattern_id: "god_object".to_string(),
                confidence: 0.9,
                severity: SeverityLevel::High,
                location: LocationContext {
                    file_path: "src/analysis/engine.rs".to_string(),
                    line_range: (45, 150),
                    context_name: Some("AnalysisEngine".to_string()),
                },
                related_patterns: vec!["tight_coupling".to_string()],
            },
            DetectedPattern {
                pattern_id: "tight_coupling".to_string(),
                confidence: 0.7,
                severity: SeverityLevel::Medium,
                location: LocationContext {
                    file_path: "src/analysis/engine.rs".to_string(),
                    line_range: (80, 95),
                    context_name: Some("AnalysisEngine::process_file".to_string()),
                },
                related_patterns: vec![],
            },
        ],
        codebase_info: CodebaseInfo {
            size_category: CodebaseSizeCategory::Medium,
            architectural_patterns: vec!["Layered Architecture".to_string()],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 8.5,
                nesting_depth: 3.2,
                coupling_score: 0.6,
                tech_debt_ratio: 0.15,
            },
            team_experience: ExperienceLevel::Mid,
        },
        user_intent: UserIntent {
            goal: AnalysisGoal::DetailedAnalysis,
            explanation_depth: ExplanationDepth::Standard,
            focus_areas: vec![AntiPatternCategory::ObjectOriented, AntiPatternCategory::Architectural],
            solution_preferences: vec![SolutionType::Comprehensive],
        },
        token_constraints: TokenConstraints {
            max_tokens: 2000,
            reserved_tokens: 500,
            available_tokens: 1500,
            tokens_per_pattern: 200,
        },
    }
}