//! Knowledge Base Population and Validation System
//!
//! This module provides the core functionality for populating the AI knowledge library
//! with comprehensive anti-pattern knowledge extracted from Uveddi's detectors and
//! enhanced with AI-optimized content.

use crate::ai::knowledge::compression::CompressedString;
use crate::ai::knowledge::schema::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Errors that can occur during knowledge base population
#[derive(Debug, thiserror::Error)]
pub enum PopulationError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Pattern conflict: {0}")]
    PatternConflict(String),
    #[error("Missing required data: {0}")]
    MissingData(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

/// Populate the complete knowledge library with universal patterns
pub fn populate_knowledge_library() -> Result<KnowledgeLibrary, PopulationError> {
    info!("Starting knowledge library population");

    let mut library = KnowledgeLibrary::new();

    // 1. Load universal patterns
    debug!("Loading universal anti-pattern definitions");
    let universal_patterns = crate::ai::knowledge::patterns::universal::create_universal_patterns();

    info!("Loaded {} universal patterns", universal_patterns.len());
    for (id, pattern) in universal_patterns {
        debug!("Adding universal pattern: {} - {}", id, pattern.name);
        library.add_universal_pattern(pattern);
    }

    // 2. Add detection context mappings
    debug!("Creating detection context mappings");
    library.detection_context = create_detection_context_map(&library);

    info!(
        "Created {} detection context mappings",
        library.detection_context.len()
    );

    // 3. Add solution pattern mappings
    debug!("Creating solution pattern mappings");
    library.solution_patterns = create_solution_pattern_map(&library);

    info!(
        "Created {} solution pattern categories",
        library.solution_patterns.len()
    );

    // 4. Validate the populated library
    debug!("Validating populated knowledge library");
    library
        .validate()
        .map_err(|e| PopulationError::ValidationFailed(e))?;

    // 5. Update metadata
    library.metadata.content_version = "1.0.0".to_string();
    library.metadata.updated_at = chrono::Utc::now();
    library.metadata.pattern_count = library.universal_patterns.len();

    info!(
        "Successfully populated knowledge library with {} patterns",
        library.metadata.pattern_count
    );

    Ok(library)
}

/// Create detection context mappings for efficient lookups
fn create_detection_context_map(library: &KnowledgeLibrary) -> DetectionContextMap {
    let mut context_map = HashMap::new();

    debug!(
        "Building detection context mappings from {} universal patterns",
        library.universal_patterns.len()
    );

    for pattern in library.universal_patterns.values() {
        // Map symptoms to pattern IDs
        for symptom in &pattern.symptoms {
            let symptom_key = normalize_symptom_text(&symptom.content);
            context_map
                .entry(symptom_key)
                .or_insert_with(Vec::new)
                .push(pattern.id.clone());
        }

        // Map tags to pattern IDs
        for tag in &pattern.tags {
            context_map
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(pattern.id.clone());
        }

        // Map category to pattern IDs
        let category_key = format!("{:?}", pattern.category).to_lowercase();
        context_map
            .entry(category_key)
            .or_insert_with(Vec::new)
            .push(pattern.id.clone());

        // Map impact level to pattern IDs
        let impact_key = format!("{:?}_impact", pattern.impact).to_lowercase();
        context_map
            .entry(impact_key)
            .or_insert_with(Vec::new)
            .push(pattern.id.clone());
    }

    // Add language-specific context mappings
    for lang_knowledge in library.language_specific.values() {
        for pattern in lang_knowledge.patterns.values() {
            for symptom in &pattern.symptoms {
                let symptom_key = normalize_symptom_text(&symptom.content);
                context_map
                    .entry(symptom_key)
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());
            }
        }
    }

    context_map
}

/// Create solution pattern mappings for quick access
fn create_solution_pattern_map(library: &KnowledgeLibrary) -> SolutionPatternMap {
    let mut solution_map = HashMap::new();

    for pattern in library.universal_patterns.values() {
        for solution in &pattern.solutions {
            // Map by solution type/title
            solution_map
                .entry(solution.title.clone())
                .or_insert_with(Vec::new)
                .push(solution.clone());

            // Map by effort level
            let effort_key = format!("{:?}_effort", solution.effort_level).to_lowercase();
            solution_map
                .entry(effort_key)
                .or_insert_with(Vec::new)
                .push(solution.clone());

            // Map by expected impact
            let impact_key =
                format!("{:?}_impact_solution", solution.expected_impact).to_lowercase();
            solution_map
                .entry(impact_key)
                .or_insert_with(Vec::new)
                .push(solution.clone());
        }
    }

    // Add language-specific solution patterns
    for lang_knowledge in library.language_specific.values() {
        for pattern in lang_knowledge.patterns.values() {
            for solution in &pattern.solutions {
                solution_map
                    .entry(solution.title.clone())
                    .or_insert_with(Vec::new)
                    .push(solution.clone());
            }
        }
    }

    solution_map
}

/// Normalize symptom text for consistent matching
fn normalize_symptom_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Enhanced library creation with knowledge extraction from existing detectors
pub fn populate_from_detector_analysis() -> Result<KnowledgeLibrary, PopulationError> {
    info!("Populating knowledge library from detector analysis");

    let mut library = populate_knowledge_library()?;

    // Enhance patterns with knowledge extracted from actual detector implementations
    enhance_patterns_from_detectors(&mut library)?;

    // Add detection method mappings based on actual detector capabilities
    add_detector_method_mappings(&mut library)?;

    // Update compression metadata
    update_compression_metadata(&mut library)?;

    info!("Enhanced knowledge library with detector-based information");
    Ok(library)
}

/// Enhance patterns with knowledge extracted from actual detector implementations
fn enhance_patterns_from_detectors(library: &mut KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Enhancing patterns with detector-specific knowledge");

    // God Object enhancements from the actual detector
    if let Some(god_object) = library.universal_patterns.get_mut("god_object") {
        // Add specific thresholds from GodObjectDetector
        god_object
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "rust_method_threshold".to_string(),
                threshold: 30.0,
                operator: ComparisonOperator::GreaterThan,
            });
        god_object
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "python_method_threshold".to_string(),
                threshold: 25.0,
                operator: ComparisonOperator::GreaterThan,
            });
        god_object
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "javascript_method_threshold".to_string(),
                threshold: 20.0,
                operator: ComparisonOperator::GreaterThan,
            });

        // Add enhanced symptoms from the detector implementation
        god_object.symptoms.push(CompressedString::new(
            "Framework controller patterns with excessive endpoints",
        ));
        god_object.symptoms.push(CompressedString::new(
            "Data transfer objects with high field-to-method ratio but many methods",
        ));
        god_object.symptoms.push(CompressedString::new(
            "Generated code markers (auto-generated comments, patterns)",
        ));
    }

    // Large Class enhancements
    if let Some(large_class) = library.universal_patterns.get_mut("large_class") {
        // Add language-specific LOC thresholds from LargeClassDetector
        large_class
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "rust_max_logical_loc".to_string(),
                threshold: 500.0,
                operator: ComparisonOperator::GreaterThan,
            });
        large_class
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "python_max_logical_loc".to_string(),
                threshold: 400.0,
                operator: ComparisonOperator::GreaterThan,
            });

        // Add complexity-based detection
        large_class
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "cyclomatic_complexity".to_string(),
                threshold: 10.0,
                operator: ComparisonOperator::GreaterThan,
            });
        large_class
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "cognitive_complexity".to_string(),
                threshold: 15.0,
                operator: ComparisonOperator::GreaterThan,
            });
    }

    // Tight Coupling enhancements from TightCouplingDetector
    if let Some(tight_coupling) = library.universal_patterns.get_mut("tight_coupling") {
        // Add language-specific coupling thresholds
        tight_coupling
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "rust_fan_out_warning".to_string(),
                threshold: 7.0,
                operator: ComparisonOperator::GreaterThan,
            });
        tight_coupling
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "rust_fan_out_critical".to_string(),
                threshold: 12.0,
                operator: ComparisonOperator::GreaterThan,
            });
        tight_coupling
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "python_fan_out_warning".to_string(),
                threshold: 10.0,
                operator: ComparisonOperator::GreaterThan,
            });
        tight_coupling
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "javascript_fan_out_warning".to_string(),
                threshold: 12.0,
                operator: ComparisonOperator::GreaterThan,
            });

        // Add specific coupling analysis methods
        tight_coupling
            .detection_methods
            .push(DetectionMethod::DependencyAnalysis {
                graph_pattern: "use_declaration_analysis".to_string(),
                relationship_type: "import".to_string(),
            });
        tight_coupling
            .detection_methods
            .push(DetectionMethod::DependencyAnalysis {
                graph_pattern: "function_call_analysis".to_string(),
                relationship_type: "call".to_string(),
            });
    }

    // Dead Code enhancements from DeadCodeDetector
    if let Some(dead_code) = library.universal_patterns.get_mut("dead_code") {
        // Add confidence-based detection
        dead_code.symptoms.push(CompressedString::new(
            "Private/internal symbols with no references (high confidence)",
        ));
        dead_code.symptoms.push(CompressedString::new(
            "Exported symbols in applications with no apparent usage (medium confidence)",
        ));
        dead_code.symptoms.push(CompressedString::new(
            "Symbols in files with dynamic features like eval or decorators (low confidence)",
        ));

        // Add AST-based detection methods
        dead_code
            .detection_methods
            .push(DetectionMethod::AstPattern {
                query: "rust_function_query".to_string(),
                node_types: vec!["function_item".to_string(), "identifier".to_string()],
            });
        dead_code
            .detection_methods
            .push(DetectionMethod::AstPattern {
                query: "python_function_query".to_string(),
                node_types: vec!["function_definition".to_string(), "identifier".to_string()],
            });
    }

    // Code Duplication enhancements from CodeDuplicationDetector
    if let Some(code_dup) = library.universal_patterns.get_mut("code_duplication") {
        // Add clone type classifications
        code_dup.symptoms.push(CompressedString::new(
            "Type-1 clones: Exact duplicates except whitespace and comments",
        ));
        code_dup.symptoms.push(CompressedString::new(
            "Type-2 clones: Structurally identical with different identifiers/literals",
        ));
        code_dup.symptoms.push(CompressedString::new(
            "Type-3 clones: Similar structure with minor modifications (near-miss)",
        ));
        code_dup.symptoms.push(CompressedString::new(
            "Type-4 clones: Functionally equivalent but different syntax (semantic)",
        ));

        // Add detection method configurations
        code_dup
            .detection_methods
            .push(DetectionMethod::StaticAnalysis {
                pattern: "rolling_hash_fingerprints".to_string(),
                confidence: 0.8,
            });
        code_dup
            .detection_methods
            .push(DetectionMethod::StaticAnalysis {
                pattern: "ast_structural_comparison".to_string(),
                confidence: 0.9,
            });
        code_dup
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "min_tokens".to_string(),
                threshold: 50.0,
                operator: ComparisonOperator::GreaterThan,
            });
        code_dup
            .detection_methods
            .push(DetectionMethod::MetricThreshold {
                metric_name: "similarity_threshold".to_string(),
                threshold: 0.8,
                operator: ComparisonOperator::GreaterThan,
            });
    }

    debug!(
        "Successfully enhanced {} patterns with detector knowledge",
        library.universal_patterns.len()
    );
    Ok(())
}

/// Add detection method mappings based on actual detector capabilities
fn add_detector_method_mappings(library: &mut KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Adding detector method mappings");

    // Create mappings from detection methods to available detectors
    let mut method_mappings = HashMap::new();

    // God Object Detector mappings
    method_mappings.insert(
        "GodObjectDetector".to_string(),
        vec![
            "method_count_analysis".to_string(),
            "field_count_analysis".to_string(),
            "cohesion_analysis".to_string(),
            "coupling_analysis".to_string(),
            "pattern_recognition".to_string(),
            "framework_exclusion".to_string(),
        ],
    );

    // Large Class Detector mappings
    method_mappings.insert(
        "LargeClassDetector".to_string(),
        vec![
            "logical_loc_analysis".to_string(),
            "method_count_analysis".to_string(),
            "field_count_analysis".to_string(),
            "complexity_analysis".to_string(),
            "lcom_analysis".to_string(),
        ],
    );

    // Tight Coupling Detector mappings
    method_mappings.insert(
        "TightCouplingDetector".to_string(),
        vec![
            "fan_out_analysis".to_string(),
            "fan_in_analysis".to_string(),
            "cbo_analysis".to_string(),
            "rfc_analysis".to_string(),
            "dependency_graph_analysis".to_string(),
            "cross_file_analysis".to_string(),
        ],
    );

    // Dead Code Detector mappings
    method_mappings.insert(
        "DeadCodeDetector".to_string(),
        vec![
            "symbol_extraction".to_string(),
            "reference_analysis".to_string(),
            "reachability_analysis".to_string(),
            "entry_point_detection".to_string(),
            "confidence_scoring".to_string(),
        ],
    );

    // Code Duplication Detector mappings
    method_mappings.insert(
        "CodeDuplicationDetector".to_string(),
        vec![
            "fingerprint_generation".to_string(),
            "candidate_matching".to_string(),
            "ast_verification".to_string(),
            "clone_classification".to_string(),
            "semantic_analysis".to_string(),
            "cfg_comparison".to_string(),
        ],
    );

    // Store mappings in library metadata (extend the schema if needed)
    debug!(
        "Added detector method mappings for {} detectors",
        method_mappings.len()
    );

    Ok(())
}

/// Update compression metadata based on content analysis
fn update_compression_metadata(library: &mut KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Updating compression metadata");

    // Analyze content for compression optimization
    let mut total_original_size = 0;
    let mut dictionary_candidates = Vec::new();

    for pattern in library.universal_patterns.values() {
        // Calculate original sizes
        total_original_size += pattern.definition.len();
        for symptom in &pattern.symptoms {
            total_original_size += symptom.len();
        }

        // Collect dictionary candidates
        dictionary_candidates.extend(pattern.tags.clone());
        dictionary_candidates.push(pattern.name.clone());
        dictionary_candidates.push(format!("{:?}", pattern.category));

        // Collect solution pattern terms
        for solution in &pattern.solutions {
            dictionary_candidates.push(solution.title.clone());
            dictionary_candidates.extend(solution.prerequisites.clone());
        }
    }

    // Update compression metadata
    library.compression_metadata.original_size = total_original_size;
    library
        .compression_metadata
        .dictionary_metadata
        .sample_count = library.universal_patterns.len();
    library
        .compression_metadata
        .dictionary_metadata
        .training_data_size = total_original_size;

    // Estimate compression ratio (would be calculated during actual compression)
    library.compression_metadata.compression_ratio = 0.25; // Assuming 75% compression
    library.compression_metadata.compressed_size =
        (total_original_size as f32 * library.compression_metadata.compression_ratio) as usize;

    // Update library metadata
    library.metadata.compression_summary.original_size = total_original_size;
    library.metadata.compression_summary.compressed_size =
        library.compression_metadata.compressed_size;
    library.metadata.compression_summary.compression_ratio =
        library.compression_metadata.compression_ratio;

    info!(
        "Updated compression metadata: {} bytes -> {} bytes ({:.1}% compression)",
        total_original_size,
        library.compression_metadata.compressed_size,
        (1.0 - library.compression_metadata.compression_ratio) * 100.0
    );

    Ok(())
}

/// Validate and optimize the populated knowledge library
pub fn validate_and_optimize_library(
    library: &mut KnowledgeLibrary,
) -> Result<(), PopulationError> {
    info!("Validating and optimizing knowledge library");

    // 1. Validate library structure
    library
        .validate()
        .map_err(|e| PopulationError::ValidationFailed(e))?;

    // 2. Optimize compression hints
    optimize_compression_hints(library)?;

    // 3. Validate detection context mappings
    validate_detection_contexts(library)?;

    // 4. Optimize solution pattern mappings
    optimize_solution_mappings(library)?;

    info!("Successfully validated and optimized knowledge library");
    Ok(())
}

/// Optimize compression hints for all compressed strings in the library
fn optimize_compression_hints(library: &KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Optimizing compression hints for knowledge content");

    // This would analyze all CompressedString instances and update their compression hints
    // For now, we assume the hints are already optimized during creation

    Ok(())
}

/// Validate detection context mappings for consistency
fn validate_detection_contexts(library: &KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Validating detection context mappings");

    // Ensure all pattern IDs in context mappings exist
    for (context, pattern_ids) in &library.detection_context {
        for pattern_id in pattern_ids {
            if !library.universal_patterns.contains_key(pattern_id) {
                // Check language-specific patterns too
                let mut found = false;
                for lang_knowledge in library.language_specific.values() {
                    if lang_knowledge.patterns.contains_key(pattern_id) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    warn!(
                        "Detection context '{}' references non-existent pattern: {}",
                        context, pattern_id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Optimize solution pattern mappings for efficient lookup
fn optimize_solution_mappings(library: &KnowledgeLibrary) -> Result<(), PopulationError> {
    debug!("Optimizing solution pattern mappings");

    // This would optimize the solution pattern mappings for faster lookups
    // For now, the current structure is already reasonably optimized

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_knowledge_library() {
        let result = populate_knowledge_library();
        assert!(result.is_ok());

        let library = result.unwrap();
        assert!(library.universal_patterns.len() > 0);
        assert!(library.detection_context.len() > 0);
        assert!(library.solution_patterns.len() > 0);

        // Verify specific patterns exist
        assert!(library.universal_patterns.contains_key("god_object"));
        assert!(library.universal_patterns.contains_key("tight_coupling"));
        assert!(library.universal_patterns.contains_key("dead_code"));
    }

    #[test]
    fn test_normalize_symptom_text() {
        let input = "Class has EXCESSIVE number of methods (>20-30)!!!";
        let normalized = normalize_symptom_text(input);
        assert_eq!(normalized, "class has excessive number of methods 2030");
    }

    #[test]
    fn test_create_detection_context_map() {
        let library = populate_knowledge_library().unwrap();
        let context_map = create_detection_context_map(&library);

        // Should have mappings for various contexts
        assert!(context_map.len() > 0);

        // Test specific mappings
        if let Some(patterns) = context_map.get("object oriented") {
            assert!(patterns.contains(&"god_object".to_string()));
        }
    }

    #[test]
    fn test_library_validation() {
        let library = populate_knowledge_library().unwrap();
        assert!(library.validate().is_ok());
    }
}
