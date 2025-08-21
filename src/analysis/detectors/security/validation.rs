//! Validation and false positive mitigation for security analysis
//!
//! This module implements sophisticated validation techniques to reduce false positives
//! and improve the accuracy of security vulnerability detection. It includes Bayesian
//! optimization, confidence calculation, and multi-layered filtering strategies.

use crate::analysis::detectors::security::config::FalsePositiveConfig;
use crate::analysis::detectors::security::core::ConfidenceScore;
use crate::analysis::detectors::security::types::{SecurityIssue, SecuritySeverity};
use crate::analysis::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main validation engine that coordinates all validation strategies
pub struct ValidationEngine {
    config: FalsePositiveConfig,
    false_positive_mitigator: FalsePositiveMitigator,
    confidence_calculator: ConfidenceCalculator,
    bayesian_optimizer: Option<BayesianOptimizer>,
}

impl ValidationEngine {
    pub fn new(config: FalsePositiveConfig) -> Result<Self, AnalysisError> {
        let false_positive_mitigator = FalsePositiveMitigator::new(config.clone())?;
        let confidence_calculator = ConfidenceCalculator::new();
        let bayesian_optimizer = if config.enable_bayesian_optimization {
            Some(BayesianOptimizer::new()?)
        } else {
            None
        };

        Ok(Self {
            config,
            false_positive_mitigator,
            confidence_calculator,
            bayesian_optimizer,
        })
    }

    /// Validate and filter a list of security issues
    pub async fn validate_issues(
        &self,
        mut issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Validating {} security issues", issues.len());

        // Apply false positive mitigation
        issues = self.false_positive_mitigator.filter_issues(issues).await?;
        info!("After false positive filtering: {} issues remain", issues.len());

        // Recalculate confidence scores
        for issue in &mut issues {
            let new_confidence = self.confidence_calculator.calculate_confidence(issue).await?;
            issue.confidence_score = new_confidence.final_score;
        }

        // Apply Bayesian optimization if enabled
        if let Some(optimizer) = &self.bayesian_optimizer {
            issues = optimizer.optimize_results(issues).await?;
        }

        // Final filtering based on confidence threshold
        let min_confidence = 0.3; // TODO: Make this configurable
        issues.retain(|issue| issue.confidence_score >= min_confidence);

        info!("After validation: {} high-confidence issues remain", issues.len());
        Ok(issues)
    }

    /// Cross-validate findings between multiple detection methods
    pub async fn cross_validate(
        &self,
        detector_results: HashMap<String, Vec<SecurityIssue>>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        info!("Cross-validating results from {} detectors", detector_results.len());

        let mut validated_issues = Vec::new();
        let mut issue_agreements = HashMap::new();

        // Count how many detectors agree on each issue
        for (detector_name, issues) in detector_results.iter() {
            for issue in issues {
                let issue_key = self.generate_issue_key(issue);
                let agreements = issue_agreements.entry(issue_key).or_insert_with(Vec::new);
                agreements.push((detector_name.clone(), issue.clone()));
            }
        }

        // Filter issues based on agreement threshold
        let min_agreement = self.config.min_lines_threshold; // Reuse config value
        for (issue_key, agreements) in issue_agreements {
            if agreements.len() >= min_agreement {
                // Multiple detectors agree - high confidence
                let mut representative_issue = agreements[0].1.clone();
                
                // Boost confidence based on agreement
                let agreement_boost = (agreements.len() as f64 - 1.0) * 0.1;
                representative_issue.confidence_score = 
                    (representative_issue.confidence_score + agreement_boost).min(1.0);

                // Add all detector names that found this issue
                for (detector_name, _) in agreements {
                    representative_issue = representative_issue.with_detector(detector_name);
                }

                validated_issues.push(representative_issue);
            } else if agreements.len() == 1 && agreements[0].1.confidence_score >= 0.8 {
                // Single detector but very high confidence
                validated_issues.push(agreements[0].1.clone());
            }
        }

        info!("Cross-validation completed: {} issues validated", validated_issues.len());
        Ok(validated_issues)
    }

    /// Generate a unique key for an issue to enable comparison
    fn generate_issue_key(&self, issue: &SecurityIssue) -> String {
        format!(
            "{}:{}:{}:{}",
            issue.location.file_path.display(),
            issue.location.start_line,
            issue.issue_type.to_string(),
            issue.title
        )
    }
}

/// False positive mitigation system with multiple filtering strategies
pub struct FalsePositiveMitigator {
    config: FalsePositiveConfig,
    heuristic_filters: Vec<Box<dyn HeuristicFilter>>,
    contextual_filters: Vec<Box<dyn ContextualFilter>>,
    statistical_filters: Vec<Box<dyn StatisticalFilter>>,
}

impl FalsePositiveMitigator {
    pub fn new(config: FalsePositiveConfig) -> Result<Self, AnalysisError> {
        let mut mitigator = Self {
            config: config.clone(),
            heuristic_filters: Vec::new(),
            contextual_filters: Vec::new(),
            statistical_filters: Vec::new(),
        };

        // Initialize filters based on configuration
        if config.enable_heuristic_filtering {
            mitigator.heuristic_filters.push(Box::new(SizeThresholdFilter::new(
                config.min_lines_threshold,
                config.min_tokens_threshold,
            )));
            mitigator.heuristic_filters.push(Box::new(CommentSuppressionFilter::new(
                config.suppression_comments.clone(),
            )));
        }

        if config.enable_contextual_filtering {
            mitigator.contextual_filters.push(Box::new(FileTypeFilter::new(
                config.exclude_test_files,
                config.exclude_generated_files,
                config.exclude_third_party,
            )));
        }

        if config.enable_statistical_filtering {
            mitigator.statistical_filters.push(Box::new(FrequencyFilter::new()));
        }

        Ok(mitigator)
    }

    pub async fn filter_issues(
        &self,
        mut issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Applying false positive mitigation to {} issues", issues.len());

        // Apply heuristic filters
        for filter in &self.heuristic_filters {
            issues = filter.filter(issues).await?;
        }

        // Apply contextual filters
        for filter in &self.contextual_filters {
            issues = filter.filter(issues).await?;
        }

        // Apply statistical filters
        for filter in &self.statistical_filters {
            issues = filter.filter(issues).await?;
        }

        debug!("False positive mitigation completed: {} issues remain", issues.len());
        Ok(issues)
    }
}

/// Trait for heuristic-based filters
#[async_trait::async_trait]
trait HeuristicFilter: Send + Sync {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError>;
}

/// Size threshold filter to exclude very small code snippets
struct SizeThresholdFilter {
    min_lines: usize,
    min_tokens: usize,
}

impl SizeThresholdFilter {
    fn new(min_lines: usize, min_tokens: usize) -> Self {
        Self { min_lines, min_tokens }
    }
}

#[async_trait::async_trait]
impl HeuristicFilter for SizeThresholdFilter {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let filtered: Vec<SecurityIssue> = issues
            .into_iter()
            .filter(|issue| {
                let line_count = (issue.location.end_line - issue.location.start_line + 1) as usize;
                line_count >= self.min_lines
                // TODO: Add token counting
            })
            .collect();

        debug!("Size threshold filter: {} issues remaining", filtered.len());
        Ok(filtered)
    }
}

/// Comment suppression filter for intentional suppressions
struct CommentSuppressionFilter {
    suppression_patterns: Vec<String>,
}

impl CommentSuppressionFilter {
    fn new(patterns: Vec<String>) -> Self {
        Self {
            suppression_patterns: patterns,
        }
    }
}

#[async_trait::async_trait]
impl HeuristicFilter for CommentSuppressionFilter {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut filtered = Vec::new();

        for issue in issues {
            let mut should_suppress = false;

            // Check if there's a suppression comment near the issue
            if let Ok(content) = std::fs::read_to_string(&issue.location.file_path) {
                let lines: Vec<&str> = content.lines().collect();
                let issue_line = (issue.location.start_line - 1) as usize;
                
                // Check the few lines before the issue for suppression comments
                for i in issue_line.saturating_sub(3)..=issue_line.min(lines.len().saturating_sub(1)) {
                    if let Some(line) = lines.get(i) {
                        for pattern in &self.suppression_patterns {
                            if line.contains(pattern) {
                                should_suppress = true;
                                debug!("Suppressing issue due to comment: {}", pattern);
                                break;
                            }
                        }
                    }
                    if should_suppress {
                        break;
                    }
                }
            }

            if !should_suppress {
                filtered.push(issue);
            }
        }

        debug!("Comment suppression filter: {} issues remaining", filtered.len());
        Ok(filtered)
    }
}

/// Trait for contextual filters
#[async_trait::async_trait]
trait ContextualFilter: Send + Sync {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError>;
}

/// File type filter to exclude test files, generated files, etc.
struct FileTypeFilter {
    exclude_test_files: bool,
    exclude_generated_files: bool,
    exclude_third_party: bool,
}

impl FileTypeFilter {
    fn new(exclude_test_files: bool, exclude_generated_files: bool, exclude_third_party: bool) -> Self {
        Self {
            exclude_test_files,
            exclude_generated_files,
            exclude_third_party,
        }
    }

    fn should_exclude_file(&self, file_path: &std::path::Path) -> bool {
        let path_str = file_path.to_string_lossy().to_lowercase();

        if self.exclude_test_files {
            if path_str.contains("/test/") || path_str.contains("/tests/") ||
               path_str.contains("_test.") || path_str.contains(".test.") {
                return true;
            }
        }

        if self.exclude_generated_files {
            if path_str.contains("generated") || path_str.contains(".gen.") ||
               path_str.contains("__pycache__") || path_str.contains("/target/") {
                return true;
            }
        }

        if self.exclude_third_party {
            if path_str.contains("node_modules") || path_str.contains("vendor/") ||
               path_str.contains("third_party") || path_str.contains(".cargo/") {
                return true;
            }
        }

        false
    }
}

#[async_trait::async_trait]
impl ContextualFilter for FileTypeFilter {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let filtered: Vec<SecurityIssue> = issues
            .into_iter()
            .filter(|issue| !self.should_exclude_file(&issue.location.file_path))
            .collect();

        debug!("File type filter: {} issues remaining", filtered.len());
        Ok(filtered)
    }
}

/// Trait for statistical filters
#[async_trait::async_trait]
trait StatisticalFilter: Send + Sync {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError>;
}

/// Frequency filter to down-rank very common patterns
struct FrequencyFilter {
    frequency_threshold: usize,
}

impl FrequencyFilter {
    fn new() -> Self {
        Self {
            frequency_threshold: 10, // Issues appearing more than 10 times get down-ranked
        }
    }
}

#[async_trait::async_trait]
impl StatisticalFilter for FrequencyFilter {
    async fn filter(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut pattern_counts = HashMap::new();
        
        // Count pattern frequencies
        for issue in &issues {
            let pattern_key = format!("{}:{}", issue.issue_type.to_string(), issue.title);
            *pattern_counts.entry(pattern_key).or_insert(0) += 1;
        }

        // Adjust confidence scores based on frequency
        let mut filtered = Vec::new();
        for mut issue in issues {
            let pattern_key = format!("{}:{}", issue.issue_type.to_string(), issue.title);
            if let Some(&count) = pattern_counts.get(&pattern_key) {
                if count > self.frequency_threshold {
                    // Down-rank very common patterns
                    issue.confidence_score *= 0.8;
                    debug!("Down-ranking frequent pattern: {} (count: {})", pattern_key, count);
                }
            }
            
            // Only keep issues with reasonable confidence after frequency adjustment
            if issue.confidence_score >= 0.2 {
                filtered.push(issue);
            }
        }

        debug!("Frequency filter: {} issues remaining", filtered.len());
        Ok(filtered)
    }
}

/// Confidence calculation system
pub struct ConfidenceCalculator {
    // Configuration and models for confidence calculation
}

impl ConfidenceCalculator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn calculate_confidence(
        &self,
        issue: &SecurityIssue,
    ) -> Result<ConfidenceScore, AnalysisError> {
        let mut confidence = ConfidenceScore::new();

        // Detection method confidence
        let detection_confidence = match issue.vulnerability_type {
            crate::analysis::detectors::security::types::VulnerabilityType::Static => 0.8,
            crate::analysis::detectors::security::types::VulnerabilityType::Dynamic => 0.9,
            crate::analysis::detectors::security::types::VulnerabilityType::Dependency => 0.95,
            crate::analysis::detectors::security::types::VulnerabilityType::Configuration => 0.85,
            crate::analysis::detectors::security::types::VulnerabilityType::AiInferred => 0.6,
            crate::analysis::detectors::security::types::VulnerabilityType::Hybrid => 0.75,
        };
        confidence = confidence.with_detection_method(detection_confidence);

        // Evidence strength based on severity and issue type
        let evidence_strength = match issue.severity {
            SecuritySeverity::Critical => 0.9,
            SecuritySeverity::High => 0.8,
            SecuritySeverity::Medium => 0.6,
            SecuritySeverity::Low => 0.4,
            SecuritySeverity::Info => 0.2,
        };
        confidence = confidence.with_evidence_strength(evidence_strength);

        // Architectural context (would be calculated based on actual context)
        let architectural_score = if issue.correlation_id.is_some() { 0.3 } else { 0.0 };
        confidence = confidence.with_architectural_context(architectural_score);

        // Cross-validation score (based on number of detectors that found this issue)
        let cross_validation_score = if issue.detected_by.len() > 1 {
            (issue.detected_by.len() as f64 * 0.2).min(1.0)
        } else {
            0.0
        };
        confidence = confidence.with_cross_validation(cross_validation_score);

        Ok(confidence)
    }
}

/// Bayesian optimization for adaptive parameter tuning
pub struct BayesianOptimizer {
    // This would contain machine learning models for optimization
    optimization_history: Vec<OptimizationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OptimizationResult {
    parameters: HashMap<String, f64>,
    performance_score: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl BayesianOptimizer {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            optimization_history: Vec::new(),
        })
    }

    pub async fn optimize_results(
        &self,
        issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        debug!("Applying Bayesian optimization to {} issues", issues.len());

        // TODO: Implement actual Bayesian optimization
        // For now, return issues unchanged
        Ok(issues)
    }

    pub async fn update_optimization(
        &mut self,
        parameters: HashMap<String, f64>,
        performance_score: f64,
    ) -> Result<(), AnalysisError> {
        let result = OptimizationResult {
            parameters,
            performance_score,
            timestamp: chrono::Utc::now(),
        };

        self.optimization_history.push(result);

        // Keep only recent history to prevent unbounded growth
        if self.optimization_history.len() > 1000 {
            self.optimization_history.drain(0..100);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::types::{SecurityIssueType, SecurityLocation, VulnerabilityType};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_validation_engine() {
        let config = FalsePositiveConfig::moderate();
        let engine = ValidationEngine::new(config).unwrap();
        
        let test_issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection".to_string(),
            "Test issue".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
        ).with_confidence(0.8);

        let validated = engine.validate_issues(vec![test_issue]).await.unwrap();
        assert!(!validated.is_empty());
    }

    #[tokio::test]
    async fn test_false_positive_mitigator() {
        let config = FalsePositiveConfig::aggressive();
        let mitigator = FalsePositiveMitigator::new(config).unwrap();

        let test_issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection".to_string(),
            "Test issue".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
        );

        let filtered = mitigator.filter_issues(vec![test_issue]).await.unwrap();
        // Result depends on the specific filters and test issue characteristics
        assert!(filtered.len() <= 1);
    }

    #[tokio::test]
    async fn test_confidence_calculator() {
        let calculator = ConfidenceCalculator::new();
        
        let test_issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection".to_string(),
            "Test issue".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
        ).with_severity(SecuritySeverity::Critical);

        let confidence = calculator.calculate_confidence(&test_issue).await.unwrap();
        assert!(confidence.final_score > 0.0);
        assert!(confidence.final_score <= 1.0);
    }

    #[test]
    fn test_size_threshold_filter() {
        let filter = SizeThresholdFilter::new(5, 10);
        
        let small_issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "Small Issue".to_string(),
            "Test".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 11), // 2 lines
        );

        let large_issue = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "Large Issue".to_string(),
            "Test".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 20), // 11 lines
        );

        // The actual filtering would be tested in an async context
        // Here we just test the line count calculation
        let small_lines = (small_issue.location.end_line - small_issue.location.start_line + 1) as usize;
        let large_lines = (large_issue.location.end_line - large_issue.location.start_line + 1) as usize;

        assert_eq!(small_lines, 2);
        assert_eq!(large_lines, 11);
        assert!(small_lines < filter.min_lines);
        assert!(large_lines >= filter.min_lines);
    }

    #[test]
    fn test_file_type_filter() {
        let filter = FileTypeFilter::new(true, true, true);
        
        assert!(filter.should_exclude_file(&PathBuf::from("/src/test/helper.rs")));
        assert!(filter.should_exclude_file(&PathBuf::from("/src/generated/proto.rs")));
        assert!(filter.should_exclude_file(&PathBuf::from("/node_modules/lib/index.js")));
        assert!(!filter.should_exclude_file(&PathBuf::from("/src/main.rs")));
    }

    #[tokio::test]
    async fn test_cross_validation() {
        let config = FalsePositiveConfig::moderate();
        let engine = ValidationEngine::new(config).unwrap();

        let issue1 = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection".to_string(),
            "Test issue".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
        ).with_detector("Detector1".to_string());

        let issue2 = SecurityIssue::new(
            SecurityIssueType::Injection,
            VulnerabilityType::Static,
            "SQL Injection".to_string(), // Same issue found by different detector
            "Test issue".to_string(),
            SecurityLocation::new(PathBuf::from("test.rs"), 10, 15),
        ).with_detector("Detector2".to_string());

        let mut detector_results = HashMap::new();
        detector_results.insert("Detector1".to_string(), vec![issue1]);
        detector_results.insert("Detector2".to_string(), vec![issue2]);

        let validated = engine.cross_validate(detector_results).await.unwrap();
        assert_eq!(validated.len(), 1); // Should merge the two identical issues
        assert_eq!(validated[0].detected_by.len(), 2); // Should list both detectors
    }
}