//! Knowledge Base Content Validation and Quality Assurance
//!
//! This module provides comprehensive validation and quality assurance for the
//! AI knowledge library, ensuring accuracy, completeness, and consistency of
//! anti-pattern knowledge content.

use crate::ai::knowledge::schema::*;
use crate::ai::knowledge::population::PopulationError;
use std::collections::{HashMap, HashSet};
use tracing::{info, debug, warn, error};

/// Severity levels for validation issues
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Individual validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub pattern_id: Option<String>,
    pub field: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Validation rule categories
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Completeness checks (required fields, minimum content)
    Completeness,
    /// Consistency checks (naming, formatting, cross-references)
    Consistency,
    /// Quality checks (content depth, accuracy, usefulness)
    Quality,
    /// Compression optimization checks
    Compression,
    /// Performance optimization checks
    Performance,
}

/// Comprehensive validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub pattern_count: usize,
    pub coverage_score: f64,
    pub quality_score: f64,
    pub consistency_score: f64,
    pub compression_readiness: f64,
    pub issues: Vec<ValidationIssue>,
    pub recommendations: Vec<String>,
    pub summary: ValidationSummary,
}

/// Validation summary statistics
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_patterns: usize,
    pub complete_patterns: usize,
    pub patterns_with_examples: usize,
    pub patterns_with_solutions: usize,
    pub patterns_with_detection_methods: usize,
    pub average_symptoms_per_pattern: f64,
    pub average_solutions_per_pattern: f64,
    pub critical_issues: usize,
    pub error_issues: usize,
    pub warning_issues: usize,
    pub info_issues: usize,
}

/// Comprehensive validation of knowledge base content
pub struct KnowledgeValidator {
    strict_mode: bool,
    validation_rules: Vec<ValidationRule>,
    min_symptoms_per_pattern: usize,
    min_solutions_per_pattern: usize,
    min_examples_per_pattern: usize,
    min_detection_methods_per_pattern: usize,
}

impl Default for KnowledgeValidator {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validation_rules: vec![
                ValidationRule::Completeness,
                ValidationRule::Consistency,
                ValidationRule::Quality,
                ValidationRule::Compression,
            ],
            min_symptoms_per_pattern: 3,
            min_solutions_per_pattern: 1,
            min_examples_per_pattern: 1,
            min_detection_methods_per_pattern: 1,
        }
    }
}

impl KnowledgeValidator {
    /// Create a new validator with custom configuration
    pub fn new(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            ..Default::default()
        }
    }

    /// Create a validator for production use with strict requirements
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            validation_rules: vec![
                ValidationRule::Completeness,
                ValidationRule::Consistency,
                ValidationRule::Quality,
                ValidationRule::Compression,
                ValidationRule::Performance,
            ],
            min_symptoms_per_pattern: 5,
            min_solutions_per_pattern: 2,
            min_examples_per_pattern: 2,
            min_detection_methods_per_pattern: 2,
        }
    }

    /// Validate the entire knowledge library
    pub fn validate_library(&self, library: &KnowledgeLibrary) -> ValidationReport {
        info!("Starting comprehensive knowledge library validation");
        
        let mut issues = Vec::new();
        
        // 1. Validate pattern completeness
        self.validate_pattern_completeness(library, &mut issues);
        
        // 2. Validate content quality
        self.validate_content_quality(library, &mut issues);
        
        // 3. Validate relationships and consistency
        self.validate_relationships(library, &mut issues);
        
        // 4. Validate compression readiness
        self.validate_compression_readiness(library, &mut issues);
        
        // 5. Validate detection methods
        self.validate_detection_methods(library, &mut issues);
        
        // 6. Calculate scores
        let coverage_score = self.calculate_coverage_score(library);
        let quality_score = self.calculate_quality_score(library, &issues);
        let consistency_score = self.calculate_consistency_score(library, &issues);
        let compression_readiness = self.calculate_compression_readiness(library);
        
        // 7. Generate summary
        let summary = self.generate_summary(library, &issues);
        
        // 8. Generate recommendations
        let recommendations = self.generate_recommendations(library, &issues);
        
        let is_valid = issues.iter().all(|i| i.severity != Severity::Critical && i.severity != Severity::Error);
        
        info!("Validation completed: {} issues found, valid: {}", issues.len(), is_valid);
        
        ValidationReport {
            is_valid,
            pattern_count: library.metadata.pattern_count,
            coverage_score,
            quality_score,
            consistency_score,
            compression_readiness,
            issues,
            recommendations,
            summary,
        }
    }

    /// Validate pattern completeness (required fields and content minimums)
    fn validate_pattern_completeness(&self, library: &KnowledgeLibrary, issues: &mut Vec<ValidationIssue>) {
        debug!("Validating pattern completeness");
        
        for (pattern_id, pattern) in &library.universal_patterns {
            // Check required fields
            if pattern.definition.is_empty() {
                issues.push(ValidationIssue {
                    severity: Severity::Critical,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("definition".to_string()),
                    message: "Pattern definition cannot be empty".to_string(),
                    suggestion: Some("Provide a comprehensive definition explaining what this anti-pattern is".to_string()),
                });
            }
            
            // Check minimum symptoms
            if pattern.symptoms.len() < self.min_symptoms_per_pattern {
                let severity = if self.strict_mode { Severity::Error } else { Severity::Warning };
                issues.push(ValidationIssue {
                    severity,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("symptoms".to_string()),
                    message: format!("Pattern has only {} symptoms, minimum is {}", 
                                   pattern.symptoms.len(), self.min_symptoms_per_pattern),
                    suggestion: Some("Add more observable symptoms to help developers identify this pattern".to_string()),
                });
            }
            
            // Check minimum solutions
            if pattern.solutions.len() < self.min_solutions_per_pattern {
                let severity = if self.strict_mode { Severity::Error } else { Severity::Warning };
                issues.push(ValidationIssue {
                    severity,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("solutions".to_string()),
                    message: format!("Pattern has only {} solutions, minimum is {}", 
                                   pattern.solutions.len(), self.min_solutions_per_pattern),
                    suggestion: Some("Provide at least one practical solution for addressing this anti-pattern".to_string()),
                });
            }
            
            // Check minimum examples
            if pattern.examples.primary.len() < self.min_examples_per_pattern {
                let severity = if self.strict_mode { Severity::Error } else { Severity::Warning };
                issues.push(ValidationIssue {
                    severity,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("examples".to_string()),
                    message: format!("Pattern has only {} examples, minimum is {}", 
                                   pattern.examples.primary.len(), self.min_examples_per_pattern),
                    suggestion: Some("Add concrete code examples showing the anti-pattern and its solution".to_string()),
                });
            }
            
            // Check minimum detection methods
            if pattern.detection_methods.len() < self.min_detection_methods_per_pattern {
                let severity = if self.strict_mode { Severity::Error } else { Severity::Warning };
                issues.push(ValidationIssue {
                    severity,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("detection_methods".to_string()),
                    message: format!("Pattern has only {} detection methods, minimum is {}", 
                                   pattern.detection_methods.len(), self.min_detection_methods_per_pattern),
                    suggestion: Some("Define how automated tools can detect this anti-pattern".to_string()),
                });
            }
            
            // Validate solution completeness
            for (i, solution) in pattern.solutions.iter().enumerate() {
                if solution.implementation.is_empty() {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        pattern_id: Some(pattern_id.clone()),
                        field: Some(format!("solutions[{}].implementation", i)),
                        message: "Solution implementation cannot be empty".to_string(),
                        suggestion: Some("Provide detailed implementation guidance for this solution".to_string()),
                    });
                }
                
                if solution.examples.is_empty() && self.strict_mode {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        pattern_id: Some(pattern_id.clone()),
                        field: Some(format!("solutions[{}].examples", i)),
                        message: "Solution has no code examples".to_string(),
                        suggestion: Some("Add code examples demonstrating how to implement this solution".to_string()),
                    });
                }
            }
        }
        
        // Validate language-specific patterns
        for (language, lang_knowledge) in &library.language_specific {
            for (pattern_id, pattern) in &lang_knowledge.patterns {
                // Apply similar validation rules to language-specific patterns
                if pattern.definition.is_empty() {
                    issues.push(ValidationIssue {
                        severity: Severity::Critical,
                        pattern_id: Some(format!("{}:{}", format!("{:?}", language), pattern_id)),
                        field: Some("definition".to_string()),
                        message: "Language-specific pattern definition cannot be empty".to_string(),
                        suggestion: Some("Provide a definition specific to this programming language".to_string()),
                    });
                }
            }
        }
    }

    /// Validate content quality (depth, accuracy, usefulness)
    fn validate_content_quality(&self, library: &KnowledgeLibrary, issues: &mut Vec<ValidationIssue>) {
        debug!("Validating content quality");
        
        for (pattern_id, pattern) in &library.universal_patterns {
            // Check definition quality
            if pattern.definition.len() < 50 {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("definition".to_string()),
                    message: "Pattern definition is very brief - consider expanding".to_string(),
                    suggestion: Some("Provide more context and explanation for better understanding".to_string()),
                });
            }
            
            // Check symptom quality
            for (i, symptom) in pattern.symptoms.iter().enumerate() {
                if symptom.len() < 10 {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        pattern_id: Some(pattern_id.clone()),
                        field: Some(format!("symptoms[{}]", i)),
                        message: "Symptom description is too brief".to_string(),
                        suggestion: Some("Provide more detailed symptom descriptions".to_string()),
                    });
                }
                
                // Check for vague language
                let vague_terms = ["too much", "too many", "bad", "poor", "wrong"];
                let symptom_text = symptom.as_str().to_lowercase();
                for term in &vague_terms {
                    if symptom_text.contains(term) && !symptom_text.contains("(") {
                        issues.push(ValidationIssue {
                            severity: Severity::Info,
                            pattern_id: Some(pattern_id.clone()),
                            field: Some(format!("symptoms[{}]", i)),
                            message: format!("Consider making '{}' more specific with thresholds or examples", term),
                            suggestion: Some("Replace vague terms with specific criteria or examples".to_string()),
                        });
                        break;
                    }
                }
            }
            
            // Check tag quality
            if pattern.tags.is_empty() {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("tags".to_string()),
                    message: "Pattern has no tags for categorization".to_string(),
                    suggestion: Some("Add relevant tags to improve discoverability".to_string()),
                });
            }
            
            // Check frequency and confidence scores
            if pattern.frequency_score < 0.0 || pattern.frequency_score > 1.0 {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("frequency_score".to_string()),
                    message: "Frequency score must be between 0.0 and 1.0".to_string(),
                    suggestion: Some("Set frequency score based on how common this pattern is in practice".to_string()),
                });
            }
            
            if pattern.detection_confidence < 0.0 || pattern.detection_confidence > 1.0 {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("detection_confidence".to_string()),
                    message: "Detection confidence must be between 0.0 and 1.0".to_string(),
                    suggestion: Some("Set confidence based on how reliably this pattern can be detected".to_string()),
                });
            }
        }
    }

    /// Validate relationships and consistency
    fn validate_relationships(&self, library: &KnowledgeLibrary, issues: &mut Vec<ValidationIssue>) {
        debug!("Validating relationships and consistency");
        
        let all_pattern_ids: HashSet<String> = library.universal_patterns.keys()
            .chain(
                library.language_specific.values()
                    .flat_map(|lang| lang.patterns.keys())
            )
            .cloned()
            .collect();
        
        for (pattern_id, pattern) in &library.universal_patterns {
            // Validate related pattern references
            for related_id in &pattern.related_patterns {
                if !all_pattern_ids.contains(related_id) {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        pattern_id: Some(pattern_id.clone()),
                        field: Some("related_patterns".to_string()),
                        message: format!("References non-existent pattern: {}", related_id),
                        suggestion: Some("Remove invalid reference or create the referenced pattern".to_string()),
                    });
                }
            }
            
            // Check for naming consistency
            if !pattern.id.chars().all(|c| c.is_lowercase() || c == '_') {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("id".to_string()),
                    message: "Pattern ID should use snake_case convention".to_string(),
                    suggestion: Some("Use lowercase letters and underscores for pattern IDs".to_string()),
                });
            }
            
            // Validate category consistency
            let category_str = format!("{:?}", pattern.category).to_lowercase();
            if !pattern.tags.iter().any(|tag| category_str.contains(&tag.to_lowercase())) {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("tags".to_string()),
                    message: "Consider adding a tag that matches the pattern category".to_string(),
                    suggestion: Some(format!("Consider adding '{}' or related tag", category_str)),
                });
            }
        }
        
        // Validate detection context mappings
        for (context, pattern_ids) in &library.detection_context {
            for pattern_id in pattern_ids {
                if !all_pattern_ids.contains(pattern_id) {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        pattern_id: None,
                        field: Some("detection_context".to_string()),
                        message: format!("Detection context '{}' references non-existent pattern: {}", 
                                       context, pattern_id),
                        suggestion: Some("Update detection context mappings".to_string()),
                    });
                }
            }
        }
    }

    /// Validate compression readiness
    fn validate_compression_readiness(&self, library: &KnowledgeLibrary, issues: &mut Vec<ValidationIssue>) {
        debug!("Validating compression readiness");
        
        // Check for compression optimization opportunities
        let mut repeated_phrases = HashMap::new();
        
        for (pattern_id, pattern) in &library.universal_patterns {
            // Analyze definition for repeated content
            let words: Vec<&str> = pattern.definition.as_str().split_whitespace().collect();
            for window in words.windows(3) {
                let phrase = window.join(" ");
                *repeated_phrases.entry(phrase.clone()).or_insert(0) += 1;
            }
            
            // Check for overly long content that might not compress well
            if pattern.definition.len() > 2000 {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("definition".to_string()),
                    message: "Very long definition may impact compression efficiency".to_string(),
                    suggestion: Some("Consider breaking down into smaller, more focused sections".to_string()),
                });
            }
            
            // Check for compression-unfriendly characters
            let problematic_chars = pattern.definition.as_str().chars()
                .filter(|c| !c.is_ascii() && !c.is_whitespace())
                .count();
            
            if problematic_chars > pattern.definition.len() / 10 {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    pattern_id: Some(pattern_id.clone()),
                    field: Some("definition".to_string()),
                    message: "High ratio of non-ASCII characters may reduce compression efficiency".to_string(),
                    suggestion: Some("Consider using ASCII alternatives where possible".to_string()),
                });
            }
        }
        
        // Report highly repeated phrases as dictionary candidates
        for (phrase, count) in repeated_phrases {
            if count >= 3 && phrase.len() > 10 {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    pattern_id: None,
                    field: Some("compression".to_string()),
                    message: format!("Phrase '{}' repeated {} times - good dictionary candidate", 
                                   phrase, count),
                    suggestion: Some("This phrase should be included in the compression dictionary".to_string()),
                });
            }
        }
    }

    /// Validate detection methods
    fn validate_detection_methods(&self, library: &KnowledgeLibrary, issues: &mut Vec<ValidationIssue>) {
        debug!("Validating detection methods");
        
        for (pattern_id, pattern) in &library.universal_patterns {
            for (i, method) in pattern.detection_methods.iter().enumerate() {
                match method {
                    DetectionMethod::MetricThreshold { threshold, .. } => {
                        if *threshold <= 0.0 {
                            issues.push(ValidationIssue {
                                severity: Severity::Warning,
                                pattern_id: Some(pattern_id.clone()),
                                field: Some(format!("detection_methods[{}]", i)),
                                message: "Metric threshold should be positive".to_string(),
                                suggestion: Some("Set a meaningful positive threshold value".to_string()),
                            });
                        }
                    },
                    DetectionMethod::StaticAnalysis { confidence, .. } => {
                        if *confidence < 0.0 || *confidence > 1.0 {
                            issues.push(ValidationIssue {
                                severity: Severity::Error,
                                pattern_id: Some(pattern_id.clone()),
                                field: Some(format!("detection_methods[{}]", i)),
                                message: "Static analysis confidence must be between 0.0 and 1.0".to_string(),
                                suggestion: Some("Set confidence based on method reliability".to_string()),
                            });
                        }
                    },
                    DetectionMethod::AstPattern { node_types, .. } => {
                        if node_types.is_empty() {
                            issues.push(ValidationIssue {
                                severity: Severity::Warning,
                                pattern_id: Some(pattern_id.clone()),
                                field: Some(format!("detection_methods[{}]", i)),
                                message: "AST pattern has no node types specified".to_string(),
                                suggestion: Some("Specify the AST node types this pattern matches".to_string()),
                            });
                        }
                    },
                    _ => {} // Other detection methods are valid as-is
                }
            }
        }
    }

    /// Calculate coverage score (0.0 to 1.0)
    fn calculate_coverage_score(&self, library: &KnowledgeLibrary) -> f64 {
        let total_patterns = library.universal_patterns.len() as f64;
        if total_patterns == 0.0 {
            return 0.0;
        }
        
        let mut score = 0.0;
        let max_score_per_pattern = 1.0;
        
        for pattern in library.universal_patterns.values() {
            let mut pattern_score = 0.0;
            
            // Definition completeness (0.2)
            if !pattern.definition.is_empty() {
                pattern_score += 0.2;
            }
            
            // Symptoms completeness (0.2)
            if pattern.symptoms.len() >= self.min_symptoms_per_pattern {
                pattern_score += 0.2;
            }
            
            // Solutions completeness (0.2)
            if pattern.solutions.len() >= self.min_solutions_per_pattern {
                pattern_score += 0.2;
            }
            
            // Examples completeness (0.2)
            if pattern.examples.primary.len() >= self.min_examples_per_pattern {
                pattern_score += 0.2;
            }
            
            // Detection methods completeness (0.2)
            if pattern.detection_methods.len() >= self.min_detection_methods_per_pattern {
                pattern_score += 0.2;
            }
            
            score += pattern_score / max_score_per_pattern;
        }
        
        score / total_patterns
    }

    /// Calculate quality score based on content depth and accuracy
    fn calculate_quality_score(&self, library: &KnowledgeLibrary, issues: &[ValidationIssue]) -> f64 {
        let total_patterns = library.universal_patterns.len() as f64;
        if total_patterns == 0.0 {
            return 0.0;
        }
        
        // Base score starts at 1.0 and decreases based on issues
        let mut score = 1.0;
        
        // Deduct points for quality issues
        for issue in issues {
            match issue.severity {
                Severity::Critical => score -= 0.1,
                Severity::Error => score -= 0.05,
                Severity::Warning => score -= 0.02,
                Severity::Info => score -= 0.01,
            }
        }
        
        // Add points for quality indicators
        let mut quality_bonus = 0.0;
        for pattern in library.universal_patterns.values() {
            // Bonus for comprehensive definitions
            if pattern.definition.len() > 200 {
                quality_bonus += 0.01;
            }
            
            // Bonus for multiple examples
            if pattern.examples.primary.len() > 1 {
                quality_bonus += 0.01;
            }
            
            // Bonus for multiple solutions
            if pattern.solutions.len() > 1 {
                quality_bonus += 0.01;
            }
            
            // Bonus for good tags
            if pattern.tags.len() >= 3 {
                quality_bonus += 0.005;
            }
        }
        
        score += quality_bonus / total_patterns;
        score.max(0.0).min(1.0)
    }

    /// Calculate consistency score
    fn calculate_consistency_score(&self, library: &KnowledgeLibrary, issues: &[ValidationIssue]) -> f64 {
        let consistency_issues = issues.iter()
            .filter(|issue| {
                issue.field.as_ref().map_or(false, |f| 
                    f.contains("id") || f.contains("tags") || f.contains("related_patterns")
                )
            })
            .count();
        
        let total_patterns = library.universal_patterns.len();
        if total_patterns == 0 {
            return 1.0;
        }
        
        let consistency_ratio = consistency_issues as f64 / total_patterns as f64;
        (1.0 - consistency_ratio).max(0.0)
    }

    /// Calculate compression readiness score
    fn calculate_compression_readiness(&self, library: &KnowledgeLibrary) -> f64 {
        // This would analyze the content for compression optimization potential
        // For now, return a baseline score
        0.8
    }

    /// Generate validation summary statistics
    fn generate_summary(&self, library: &KnowledgeLibrary, issues: &[ValidationIssue]) -> ValidationSummary {
        let total_patterns = library.universal_patterns.len();
        
        let complete_patterns = library.universal_patterns.values()
            .filter(|p| !p.definition.is_empty() && 
                       p.symptoms.len() >= self.min_symptoms_per_pattern &&
                       p.solutions.len() >= self.min_solutions_per_pattern)
            .count();
        
        let patterns_with_examples = library.universal_patterns.values()
            .filter(|p| p.examples.primary.len() >= self.min_examples_per_pattern)
            .count();
        
        let patterns_with_solutions = library.universal_patterns.values()
            .filter(|p| p.solutions.len() >= self.min_solutions_per_pattern)
            .count();
        
        let patterns_with_detection_methods = library.universal_patterns.values()
            .filter(|p| p.detection_methods.len() >= self.min_detection_methods_per_pattern)
            .count();
        
        let total_symptoms: usize = library.universal_patterns.values()
            .map(|p| p.symptoms.len())
            .sum();
        let average_symptoms_per_pattern = if total_patterns > 0 {
            total_symptoms as f64 / total_patterns as f64
        } else {
            0.0
        };
        
        let total_solutions: usize = library.universal_patterns.values()
            .map(|p| p.solutions.len())
            .sum();
        let average_solutions_per_pattern = if total_patterns > 0 {
            total_solutions as f64 / total_patterns as f64
        } else {
            0.0
        };
        
        let critical_issues = issues.iter().filter(|i| i.severity == Severity::Critical).count();
        let error_issues = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warning_issues = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        let info_issues = issues.iter().filter(|i| i.severity == Severity::Info).count();
        
        ValidationSummary {
            total_patterns,
            complete_patterns,
            patterns_with_examples,
            patterns_with_solutions,
            patterns_with_detection_methods,
            average_symptoms_per_pattern,
            average_solutions_per_pattern,
            critical_issues,
            error_issues,
            warning_issues,
            info_issues,
        }
    }

    /// Generate actionable recommendations
    fn generate_recommendations(&self, library: &KnowledgeLibrary, issues: &[ValidationIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze issues and generate recommendations
        let critical_count = issues.iter().filter(|i| i.severity == Severity::Critical).count();
        let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
        
        if critical_count > 0 {
            recommendations.push(format!(
                "URGENT: Fix {} critical issues before deploying the knowledge library", 
                critical_count
            ));
        }
        
        if error_count > 0 {
            recommendations.push(format!(
                "Address {} error-level issues to improve library quality", 
                error_count
            ));
        }
        
        // Pattern-specific recommendations
        let incomplete_patterns = library.universal_patterns.values()
            .filter(|p| p.symptoms.len() < self.min_symptoms_per_pattern)
            .count();
        
        if incomplete_patterns > 0 {
            recommendations.push(format!(
                "Expand symptom descriptions for {} patterns to improve detection accuracy",
                incomplete_patterns
            ));
        }
        
        let patterns_without_examples = library.universal_patterns.values()
            .filter(|p| p.examples.primary.is_empty())
            .count();
        
        if patterns_without_examples > 0 {
            recommendations.push(format!(
                "Add code examples to {} patterns to improve developer understanding",
                patterns_without_examples
            ));
        }
        
        // Compression recommendations
        recommendations.push(
            "Run compression analysis to optimize dictionary training for zstd compression".to_string()
        );
        
        // Quality recommendations
        if library.universal_patterns.len() < 15 {
            recommendations.push(
                "Consider adding more anti-pattern definitions to provide comprehensive coverage".to_string()
            );
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::knowledge::population::populate_knowledge_library;
    
    #[test]
    fn test_validate_empty_library() {
        let library = KnowledgeLibrary::new();
        let validator = KnowledgeValidator::default();
        let report = validator.validate_library(&library);
        
        assert!(!report.is_valid); // Empty library should not be valid
        assert_eq!(report.pattern_count, 0);
    }
    
    #[test]
    fn test_validate_populated_library() {
        let library = populate_knowledge_library().unwrap();
        let validator = KnowledgeValidator::default();
        let report = validator.validate_library(&library);
        
        assert!(report.pattern_count > 0);
        assert!(report.coverage_score > 0.0);
        assert!(report.quality_score > 0.0);
        
        // Should have some issues but not be completely invalid
        println!("Validation report: {} issues, coverage: {:.2}, quality: {:.2}", 
                 report.issues.len(), report.coverage_score, report.quality_score);
    }
    
    #[test]
    fn test_strict_validation() {
        let library = populate_knowledge_library().unwrap();
        let validator = KnowledgeValidator::strict();
        let report = validator.validate_library(&library);
        
        // Strict validation should find more issues
        assert!(report.issues.len() > 0);
    }
    
    #[test]
    fn test_validation_issue_severity() {
        let issue = ValidationIssue {
            severity: Severity::Critical,
            pattern_id: Some("test".to_string()),
            field: Some("definition".to_string()),
            message: "Test issue".to_string(),
            suggestion: None,
        };
        
        assert_eq!(issue.severity, Severity::Critical);
    }
}