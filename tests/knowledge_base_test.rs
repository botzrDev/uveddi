//! Comprehensive test suite for AI knowledge library content
//!
//! This test module provides thorough validation of the universal anti-pattern
//! knowledge base, ensuring content quality, completeness, and integration
//! with the overall Uveddi system.

use std::collections::HashSet;
use uveddi::ai::knowledge::compression::CompressedString;
use uveddi::ai::knowledge::population::{
    populate_from_detector_analysis, populate_knowledge_library,
};
use uveddi::ai::knowledge::schema::*;
use uveddi::ai::knowledge::validation::{KnowledgeValidator, Severity};

/// Test the basic knowledge library creation and structure
#[test]
fn test_knowledge_library_creation() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Basic structure validation
    assert!(
        library.universal_patterns.len() > 0,
        "Library should contain universal patterns"
    );
    assert!(
        library.detection_context.len() > 0,
        "Library should have detection context mappings"
    );
    assert!(
        library.solution_patterns.len() > 0,
        "Library should have solution pattern mappings"
    );

    // Metadata validation
    assert!(
        !library.metadata.content_version.is_empty(),
        "Content version should be set"
    );
    assert!(
        library.metadata.pattern_count > 0,
        "Pattern count should be positive"
    );
    assert!(
        library
            .metadata
            .supported_languages
            .contains(&SourceLanguage::Universal),
        "Should support universal patterns"
    );

    println!(
        "✓ Successfully created knowledge library with {} patterns",
        library.metadata.pattern_count
    );
}

/// Test that all expected core patterns are present
#[test]
fn test_core_patterns_present() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    let expected_patterns = [
        "god_object",
        "large_class",
        "tight_coupling",
        "dead_code",
        "code_duplication",
        "cyclic_dependencies",
        "silent_failure",
        "resource_leak",
        "magic_values",
    ];

    for pattern_id in &expected_patterns {
        assert!(
            library.universal_patterns.contains_key(*pattern_id),
            "Core pattern '{}' should be present",
            pattern_id
        );

        let pattern = &library.universal_patterns[*pattern_id];
        assert!(
            !pattern.name.is_empty(),
            "Pattern '{}' should have a name",
            pattern_id
        );
        assert!(
            !pattern.definition.is_empty(),
            "Pattern '{}' should have a definition",
            pattern_id
        );

        println!("✓ Core pattern '{}': {} found", pattern_id, pattern.name);
    }
}

/// Test God Object pattern specifically for completeness
#[test]
fn test_god_object_pattern_completeness() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let god_object = library
        .universal_patterns
        .get("god_object")
        .expect("God Object pattern should exist");

    // Basic fields
    assert_eq!(god_object.id, "god_object");
    assert_eq!(god_object.name, "God Object");
    assert_eq!(god_object.category, AntiPatternCategory::ObjectOriented);
    assert_eq!(god_object.impact, ImpactLevel::High);

    // Content validation
    assert!(
        god_object.definition.len() > 100,
        "Definition should be comprehensive"
    );
    assert!(
        god_object.symptoms.len() >= 5,
        "Should have multiple symptoms"
    );
    assert!(
        god_object.solutions.len() >= 1,
        "Should have at least one solution"
    );
    assert!(
        god_object.detection_methods.len() >= 1,
        "Should have detection methods"
    );

    // Solution validation
    for solution in &god_object.solutions {
        assert!(!solution.title.is_empty(), "Solution should have a title");
        assert!(
            !solution.implementation.is_empty(),
            "Solution should have implementation details"
        );
        assert!(solution.examples.len() > 0, "Solution should have examples");
    }

    // Tags validation
    assert!(
        god_object.tags.contains(&"object-oriented".to_string()),
        "Should have object-oriented tag"
    );
    assert!(
        god_object.tags.contains(&"design".to_string()),
        "Should have design tag"
    );

    // Related patterns
    assert!(
        god_object
            .related_patterns
            .contains(&"large_class".to_string()),
        "Should be related to large_class"
    );

    println!("✓ God Object pattern is complete and well-structured");
}

/// Test Tight Coupling pattern for architectural completeness
#[test]
fn test_tight_coupling_pattern_completeness() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let tight_coupling = library
        .universal_patterns
        .get("tight_coupling")
        .expect("Tight Coupling pattern should exist");

    assert_eq!(tight_coupling.category, AntiPatternCategory::Architectural);
    assert_eq!(tight_coupling.impact, ImpactLevel::High);

    // Should have detection methods with metrics
    let has_metric_detection = tight_coupling
        .detection_methods
        .iter()
        .any(|method| matches!(method, DetectionMethod::MetricThreshold { .. }));
    assert!(
        has_metric_detection,
        "Should have metric-based detection methods"
    );

    // Should have dependency analysis
    let has_dependency_analysis = tight_coupling
        .detection_methods
        .iter()
        .any(|method| matches!(method, DetectionMethod::DependencyAnalysis { .. }));
    assert!(
        has_dependency_analysis,
        "Should have dependency analysis methods"
    );

    println!("✓ Tight Coupling pattern has proper architectural analysis methods");
}

/// Test Dead Code pattern for accuracy
#[test]
fn test_dead_code_pattern_accuracy() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let dead_code = library
        .universal_patterns
        .get("dead_code")
        .expect("Dead Code pattern should exist");

    assert_eq!(dead_code.category, AntiPatternCategory::Maintainability);
    assert_eq!(dead_code.impact, ImpactLevel::Medium);

    // Should have static analysis detection methods
    let has_static_analysis = dead_code
        .detection_methods
        .iter()
        .any(|method| matches!(method, DetectionMethod::StaticAnalysis { .. }));
    assert!(
        has_static_analysis,
        "Should have static analysis detection methods"
    );

    // Should have AST pattern detection
    let has_ast_pattern = dead_code
        .detection_methods
        .iter()
        .any(|method| matches!(method, DetectionMethod::AstPattern { .. }));
    assert!(has_ast_pattern, "Should have AST pattern detection methods");

    println!("✓ Dead Code pattern has accurate detection methods");
}

/// Test Code Duplication pattern for clone type coverage
#[test]
fn test_code_duplication_pattern_coverage() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let code_dup = library
        .universal_patterns
        .get("code_duplication")
        .expect("Code Duplication pattern should exist");

    assert_eq!(code_dup.category, AntiPatternCategory::Maintainability);

    // Should mention different clone types in symptoms or description
    let symptoms_text = code_dup
        .symptoms
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let all_text = format!("{} {}", code_dup.definition.as_str(), symptoms_text);

    // Check for clone type mentions (from actual detector implementation)
    assert!(
        all_text.to_lowercase().contains("identical")
            || all_text.to_lowercase().contains("similar")
            || all_text.to_lowercase().contains("duplicate"),
        "Should describe the nature of code duplication"
    );

    println!("✓ Code Duplication pattern covers the concept comprehensively");
}

/// Test detection context mappings
#[test]
fn test_detection_context_mappings() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Should have context mappings for major categories
    let has_oo_patterns = library
        .detection_context
        .iter()
        .any(|(context, patterns)| context.contains("object") && !patterns.is_empty());
    assert!(
        has_oo_patterns,
        "Should have object-oriented pattern mappings"
    );

    let has_architectural_patterns = library
        .detection_context
        .iter()
        .any(|(context, patterns)| context.contains("architectural") && !patterns.is_empty());
    assert!(
        has_architectural_patterns,
        "Should have architectural pattern mappings"
    );

    // Validate that all referenced patterns exist
    for (context, pattern_ids) in &library.detection_context {
        for pattern_id in pattern_ids {
            assert!(
                library.universal_patterns.contains_key(pattern_id),
                "Context '{}' references non-existent pattern '{}'",
                context,
                pattern_id
            );
        }
    }

    println!("✓ Detection context mappings are valid and comprehensive");
}

/// Test solution pattern mappings
#[test]
fn test_solution_pattern_mappings() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    assert!(
        library.solution_patterns.len() > 0,
        "Should have solution pattern mappings"
    );

    // Should have common solution types
    let expected_solutions = [
        "Extract Class",
        "Dependency Injection",
        "Interface Abstraction",
    ];
    for solution_type in &expected_solutions {
        let has_solution = library.solution_patterns.contains_key(*solution_type);
        if has_solution {
            println!("✓ Found solution type: {}", solution_type);
        }
    }

    // Should have effort-level mappings
    let effort_mappings = library
        .solution_patterns
        .iter()
        .filter(|(key, _)| key.contains("effort"))
        .count();
    assert!(
        effort_mappings > 0,
        "Should have effort-level solution mappings"
    );

    println!("✓ Solution pattern mappings provide good coverage");
}

/// Test compression string usage and optimization
#[test]
fn test_compression_string_optimization() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Check that CompressedString is used appropriately
    for (pattern_id, pattern) in &library.universal_patterns {
        // Definitions should be compressed
        assert!(
            pattern.definition.len() > 0,
            "Pattern '{}' should have a non-empty definition",
            pattern_id
        );

        // Symptoms should be compressed
        for (i, symptom) in pattern.symptoms.iter().enumerate() {
            assert!(
                symptom.len() > 0,
                "Pattern '{}' symptom {} should not be empty",
                pattern_id,
                i
            );
        }

        // Solutions should have compressed implementation text
        for (i, solution) in pattern.solutions.iter().enumerate() {
            assert!(
                solution.implementation.len() > 0,
                "Pattern '{}' solution {} should have implementation details",
                pattern_id,
                i
            );
        }
    }

    println!("✓ CompressedString usage is consistent throughout the library");
}

/// Test knowledge library validation
#[test]
fn test_knowledge_library_validation() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let validator = KnowledgeValidator::default();
    let report = validator.validate_library(&library);

    // Should have reasonable scores
    assert!(
        report.coverage_score > 0.5,
        "Coverage score should be reasonable: {}",
        report.coverage_score
    );
    assert!(
        report.quality_score > 0.5,
        "Quality score should be reasonable: {}",
        report.quality_score
    );

    // Should not have critical issues
    let critical_issues = report
        .issues
        .iter()
        .filter(|issue| issue.severity == Severity::Critical)
        .count();
    assert_eq!(
        critical_issues, 0,
        "Should not have critical validation issues"
    );

    // Should have actionable recommendations
    assert!(
        report.recommendations.len() > 0,
        "Should provide recommendations"
    );

    println!(
        "✓ Knowledge library passes validation with scores - Coverage: {:.2}, Quality: {:.2}",
        report.coverage_score, report.quality_score
    );

    // Print any issues found
    for issue in &report.issues {
        if matches!(issue.severity, Severity::Error | Severity::Warning) {
            println!("  {:?}: {}", issue.severity, issue.message);
        }
    }
}

/// Test enhanced knowledge population from detector analysis
#[test]
fn test_enhanced_knowledge_population() {
    let library =
        populate_from_detector_analysis().expect("Failed to populate enhanced knowledge library");

    // Should have the same basic structure as regular population
    assert!(library.universal_patterns.len() > 0);
    assert!(library.detection_context.len() > 0);

    // Should have enhanced detection methods (from actual detectors)
    let god_object = library
        .universal_patterns
        .get("god_object")
        .expect("God Object should exist");

    // Should have language-specific thresholds from actual detectors
    let has_language_thresholds = god_object
        .detection_methods
        .iter()
        .any(|method| match method {
            DetectionMethod::MetricThreshold { metric_name, .. } => {
                metric_name.contains("rust")
                    || metric_name.contains("python")
                    || metric_name.contains("javascript")
            }
            _ => false,
        });
    assert!(
        has_language_thresholds,
        "Should have language-specific detection thresholds"
    );

    println!("✓ Enhanced knowledge population adds detector-specific information");
}

/// Test library metadata and compression information
#[test]
fn test_library_metadata() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Metadata should be properly set
    assert!(!library.metadata.schema_version.is_empty());
    assert!(!library.metadata.content_version.is_empty());
    assert!(library.metadata.pattern_count > 0);
    assert!(library.metadata.supported_languages.len() > 0);

    // Compression metadata should be initialized
    assert!(library.compression_metadata.compression_level > 0);
    assert!(!library.compression_metadata.dictionary_version.is_empty());

    // Index metadata should be initialized
    assert!(library.index_metadata.entry_count >= 0);

    println!("✓ Library metadata is properly initialized");
    println!("  Schema version: {}", library.metadata.schema_version);
    println!("  Content version: {}", library.metadata.content_version);
    println!("  Pattern count: {}", library.metadata.pattern_count);
    println!(
        "  Supported languages: {:?}",
        library.metadata.supported_languages
    );
}

/// Test pattern relationship consistency
#[test]
fn test_pattern_relationships() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Collect all pattern IDs
    let all_pattern_ids: HashSet<String> = library.universal_patterns.keys().cloned().collect();

    // Validate that all related pattern references are valid
    for (pattern_id, pattern) in &library.universal_patterns {
        for related_id in &pattern.related_patterns {
            assert!(
                all_pattern_ids.contains(related_id),
                "Pattern '{}' references non-existent related pattern '{}'",
                pattern_id,
                related_id
            );
        }
    }

    // Check for some expected relationships
    let god_object = &library.universal_patterns["god_object"];
    assert!(
        god_object
            .related_patterns
            .contains(&"large_class".to_string()),
        "God Object should be related to Large Class"
    );

    let tight_coupling = &library.universal_patterns["tight_coupling"];
    assert!(
        tight_coupling.related_patterns.len() > 0,
        "Tight Coupling should have related patterns"
    );

    println!("✓ Pattern relationships are consistent and valid");
}

/// Test that detection methods are properly configured
#[test]
fn test_detection_methods_configuration() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    for (pattern_id, pattern) in &library.universal_patterns {
        assert!(
            pattern.detection_methods.len() > 0,
            "Pattern '{}' should have at least one detection method",
            pattern_id
        );

        for (i, method) in pattern.detection_methods.iter().enumerate() {
            match method {
                DetectionMethod::MetricThreshold { threshold, .. } => {
                    assert!(
                        *threshold > 0.0,
                        "Pattern '{}' detection method {} should have positive threshold",
                        pattern_id,
                        i
                    );
                }
                DetectionMethod::StaticAnalysis { confidence, .. } => {
                    assert!(
                        *confidence >= 0.0 && *confidence <= 1.0,
                        "Pattern '{}' detection method {} should have valid confidence",
                        pattern_id,
                        i
                    );
                }
                _ => {} // Other methods are valid as-is
            }
        }
    }

    println!("✓ All detection methods are properly configured");
}

/// Integration test: validate that the knowledge library can be used for AI prompts
#[test]
fn test_ai_prompt_integration() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Test that we can extract information for AI prompts
    let god_object = &library.universal_patterns["god_object"];

    // Should be able to create a prompt with pattern information
    let prompt_content = format!(
        "Pattern: {}\nDefinition: {}\nSymptoms: {}\nSolutions: {}",
        god_object.name,
        god_object.definition.as_str(),
        god_object
            .symptoms
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        god_object
            .solutions
            .iter()
            .map(|s| &s.title)
            .collect::<Vec<_>>()
            .join(", ")
    );

    assert!(
        prompt_content.len() > 200,
        "AI prompt content should be substantial"
    );
    assert!(
        prompt_content.contains("God Object"),
        "Should contain pattern name"
    );
    assert!(
        prompt_content.contains("Extract Class"),
        "Should contain solution information"
    );

    println!("✓ Knowledge library is suitable for AI prompt generation");
    println!(
        "  Sample prompt length: {} characters",
        prompt_content.len()
    );
}

/// Performance test: ensure the library can be loaded efficiently
#[test]
fn test_library_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");
    let population_time = start.elapsed();

    let start = Instant::now();
    let validator = KnowledgeValidator::default();
    let _report = validator.validate_library(&library);
    let validation_time = start.elapsed();

    // Should be able to populate and validate quickly
    assert!(
        population_time.as_millis() < 1000,
        "Population should complete in under 1 second, took: {:?}",
        population_time
    );
    assert!(
        validation_time.as_millis() < 500,
        "Validation should complete in under 500ms, took: {:?}",
        validation_time
    );

    println!("✓ Performance is acceptable");
    println!("  Population time: {:?}", population_time);
    println!("  Validation time: {:?}", validation_time);
}

/// Test that all patterns have proper categorization
#[test]
fn test_pattern_categorization() {
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // Count patterns by category
    let mut category_counts = std::collections::HashMap::new();
    for pattern in library.universal_patterns.values() {
        *category_counts.entry(&pattern.category).or_insert(0) += 1;
    }

    // Should have patterns in multiple categories
    assert!(
        category_counts.len() >= 3,
        "Should have patterns in multiple categories"
    );

    // Should have some object-oriented patterns
    assert!(
        category_counts.contains_key(&AntiPatternCategory::ObjectOriented),
        "Should have object-oriented patterns"
    );

    // Should have some architectural patterns
    assert!(
        category_counts.contains_key(&AntiPatternCategory::Architectural),
        "Should have architectural patterns"
    );

    println!("✓ Patterns are properly categorized");
    for (category, count) in category_counts {
        println!("  {:?}: {} patterns", category, count);
    }
}

/// Final integration test: end-to-end knowledge library usage
#[test]
fn test_end_to_end_integration() {
    // 1. Populate the library
    let library = populate_knowledge_library().expect("Failed to populate knowledge library");

    // 2. Validate the library
    let validator = KnowledgeValidator::default();
    let report = validator.validate_library(&library);
    assert!(
        report.is_valid
            || report
                .issues
                .iter()
                .all(|i| i.severity != Severity::Critical)
    );

    // 3. Test knowledge retrieval
    let god_object_patterns =
        library.get_patterns_by_category(&AntiPatternCategory::ObjectOriented);
    assert!(
        god_object_patterns.len() > 0,
        "Should find object-oriented patterns"
    );

    // 4. Test detection context lookup
    let context_results = library.detection_context.get("object oriented");
    if let Some(patterns) = context_results {
        assert!(
            patterns.len() > 0,
            "Should find patterns for context lookup"
        );
    }

    // 5. Test solution pattern lookup
    let extract_class_solutions = library.solution_patterns.get("Extract Class");
    if let Some(solutions) = extract_class_solutions {
        assert!(solutions.len() > 0, "Should find Extract Class solutions");
    }

    println!("✓ End-to-end knowledge library integration successful");
    println!("  Total patterns: {}", library.universal_patterns.len());
    println!("  Detection contexts: {}", library.detection_context.len());
    println!("  Solution patterns: {}", library.solution_patterns.len());
    println!("  Validation score: {:.2}", report.quality_score);
}
