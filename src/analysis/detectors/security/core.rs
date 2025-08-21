//! Core structures and components for the security detection system
//!
//! This module provides the fundamental building blocks for the security detector,
//! including the analysis context, results structures, confidence scoring mechanisms,
//! and vulnerability database interfaces.

use crate::analysis::detectors::security::owasp::OwaspVulnerability;
use crate::analysis::detectors::security::types::{SecuritySeverity, VulnerabilityMetadata};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Security analysis context containing all necessary information for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Path to the file being analyzed
    pub file_path: PathBuf,
    /// File content for analysis
    pub content: String,
    /// Programming language of the file
    pub language: SourceLanguage,
    /// Additional metadata about the file and analysis context
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SecurityContext {
    pub fn new(file_path: PathBuf, content: String, language: SourceLanguage) -> Self {
        Self {
            file_path,
            content,
            language,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn from_parsed_file(file: &ParsedFile) -> Result<Self, AnalysisError> {
        let content = std::fs::read_to_string(&**file.file_path)
            .map_err(|e| AnalysisError::file_system_error(format!("Failed to read file {}: {}", file.file_path.display(), e), e))?;

        Ok(Self::new(file.file_path.as_ref().to_path_buf(), content, file.language))
    }

    pub fn to_parsed_file(&self) -> Result<ParsedFile, AnalysisError> {
        // This is a simplified conversion - in a real implementation, we'd need to parse the AST
        use std::sync::Arc;
        use crate::analysis::cache::wrappers::ArchivableSystemTime;
        
        Ok(ParsedFile {
            file_path: Arc::new(self.file_path.clone()),
            language: self.language,
            tree: None,
            source: Arc::new(self.content.clone()),
            custom_ast: Arc::new(None),
            modified_at: ArchivableSystemTime::now(),
        })
    }

    pub fn get_file_extension(&self) -> Option<String> {
        self.file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    pub fn is_source_file(&self) -> bool {
        match self.get_file_extension().as_deref() {
            Some("rs") | Some("py") | Some("js") | Some("ts") => true,
            _ => false,
        }
    }

    pub fn is_config_file(&self) -> bool {
        match self.get_file_extension().as_deref() {
            Some("toml") | Some("json") | Some("yaml") | Some("yml") => true,
            _ => false,
        }
    }

    pub fn get_lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    pub fn get_line_count(&self) -> usize {
        self.content.lines().count()
    }
}

/// Comprehensive security analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    /// List of detected vulnerabilities
    pub vulnerabilities: Vec<OwaspVulnerability>,
    /// Overall analysis statistics
    pub statistics: AnalysisStatistics,
    /// Confidence scores and validation information
    pub confidence_info: ConfidenceInfo,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Architectural correlations found
    pub architectural_correlations: HashMap<String, Vec<String>>,
}

impl SecurityAnalysisResult {
    pub fn new() -> Self {
        Self {
            vulnerabilities: Vec::new(),
            statistics: AnalysisStatistics::default(),
            confidence_info: ConfidenceInfo::default(),
            performance_metrics: PerformanceMetrics::default(),
            architectural_correlations: HashMap::new(),
        }
    }

    pub fn add_vulnerability(&mut self, vulnerability: OwaspVulnerability) {
        self.vulnerabilities.push(vulnerability);
        self.update_statistics();
    }

    pub fn extend_vulnerabilities(&mut self, vulnerabilities: Vec<OwaspVulnerability>) {
        self.vulnerabilities.extend(vulnerabilities);
        self.update_statistics();
    }

    fn update_statistics(&mut self) {
        self.statistics.total_issues = self.vulnerabilities.len();
        
        // Count by severity
        self.statistics.critical_count = self.vulnerabilities.iter()
            .filter(|v| v.severity == SecuritySeverity::Critical)
            .count();
        self.statistics.high_count = self.vulnerabilities.iter()
            .filter(|v| v.severity == SecuritySeverity::High)
            .count();
        self.statistics.medium_count = self.vulnerabilities.iter()
            .filter(|v| v.severity == SecuritySeverity::Medium)
            .count();
        self.statistics.low_count = self.vulnerabilities.iter()
            .filter(|v| v.severity == SecuritySeverity::Low)
            .count();

        // Calculate average confidence
        if !self.vulnerabilities.is_empty() {
            let total_confidence: f64 = self.vulnerabilities.iter()
                .map(|v| v.confidence_score)
                .sum();
            self.confidence_info.average_confidence = total_confidence / self.vulnerabilities.len() as f64;
        }
    }

    pub fn get_high_confidence_issues(&self) -> Vec<&OwaspVulnerability> {
        self.vulnerabilities.iter()
            .filter(|v| v.confidence_score >= 0.8)
            .collect()
    }

    pub fn get_urgent_issues(&self) -> Vec<&OwaspVulnerability> {
        self.vulnerabilities.iter()
            .filter(|v| matches!(v.severity, SecuritySeverity::Critical | SecuritySeverity::High))
            .collect()
    }
}

impl Default for SecurityAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the security analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatistics {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub files_analyzed: usize,
    pub lines_analyzed: usize,
}

impl Default for AnalysisStatistics {
    fn default() -> Self {
        Self {
            total_issues: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            info_count: 0,
            files_analyzed: 0,
            lines_analyzed: 0,
        }
    }
}

/// Confidence information for the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInfo {
    pub average_confidence: f64,
    pub min_confidence: f64,
    pub max_confidence: f64,
    pub confidence_distribution: HashMap<String, usize>, // Confidence range -> count
    pub validation_results: ValidationResults,
}

impl Default for ConfidenceInfo {
    fn default() -> Self {
        Self {
            average_confidence: 0.0,
            min_confidence: 1.0,
            max_confidence: 0.0,
            confidence_distribution: HashMap::new(),
            validation_results: ValidationResults::default(),
        }
    }
}

/// Results from validation processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub cross_validated_count: usize,
    pub false_positive_count: usize,
    pub hallucination_detected_count: usize,
    pub grounding_success_rate: f64,
}

impl Default for ValidationResults {
    fn default() -> Self {
        Self {
            cross_validated_count: 0,
            false_positive_count: 0,
            hallucination_detected_count: 0,
            grounding_success_rate: 1.0,
        }
    }
}

/// Performance metrics for the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_analysis_time_ms: u64,
    pub taint_analysis_time_ms: u64,
    pub owasp_analysis_time_ms: u64,
    pub validation_time_ms: u64,
    pub memory_usage_mb: f64,
    pub throughput_files_per_second: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_analysis_time_ms: 0,
            taint_analysis_time_ms: 0,
            owasp_analysis_time_ms: 0,
            validation_time_ms: 0,
            memory_usage_mb: 0.0,
            throughput_files_per_second: 0.0,
        }
    }
}

/// Multi-factor confidence scoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    /// Detection method confidence (deterministic vs heuristic vs AI)
    pub detection_method_score: f64,
    /// Evidence strength score
    pub evidence_strength_score: f64,
    /// Architectural context amplification
    pub architectural_context_score: f64,
    /// Cross-validation score
    pub cross_validation_score: f64,
    /// Final combined confidence score
    pub final_score: f64,
}

impl ConfidenceScore {
    pub fn new() -> Self {
        Self {
            detection_method_score: 0.5,
            evidence_strength_score: 0.5,
            architectural_context_score: 0.0,
            cross_validation_score: 0.0,
            final_score: 0.5,
        }
    }

    /// Calculate final confidence score from component scores
    pub fn calculate_final_score(&mut self) {
        // Weighted combination of different factors
        let weights = ConfidenceWeights::default();
        
        self.final_score = (
            self.detection_method_score * weights.detection_method +
            self.evidence_strength_score * weights.evidence_strength +
            self.architectural_context_score * weights.architectural_context +
            self.cross_validation_score * weights.cross_validation
        ) / (weights.detection_method + weights.evidence_strength + weights.architectural_context + weights.cross_validation);

        // Clamp to valid range
        self.final_score = self.final_score.clamp(0.0, 1.0);
    }

    pub fn with_detection_method(mut self, score: f64) -> Self {
        self.detection_method_score = score.clamp(0.0, 1.0);
        self.calculate_final_score();
        self
    }

    pub fn with_evidence_strength(mut self, score: f64) -> Self {
        self.evidence_strength_score = score.clamp(0.0, 1.0);
        self.calculate_final_score();
        self
    }

    pub fn with_architectural_context(mut self, score: f64) -> Self {
        self.architectural_context_score = score.clamp(0.0, 1.0);
        self.calculate_final_score();
        self
    }

    pub fn with_cross_validation(mut self, score: f64) -> Self {
        self.cross_validation_score = score.clamp(0.0, 1.0);
        self.calculate_final_score();
        self
    }

    pub fn is_high_confidence(&self) -> bool {
        self.final_score >= 0.8
    }

    pub fn is_low_confidence(&self) -> bool {
        self.final_score < 0.4
    }
}

impl Default for ConfidenceScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Weights for combining confidence factors
#[derive(Debug, Clone)]
struct ConfidenceWeights {
    pub detection_method: f64,
    pub evidence_strength: f64,
    pub architectural_context: f64,
    pub cross_validation: f64,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            detection_method: 0.4,  // Highest weight for detection method
            evidence_strength: 0.3, // High weight for evidence quality
            architectural_context: 0.2, // Medium weight for architectural context
            cross_validation: 0.1,  // Lower weight for cross-validation
        }
    }
}

/// Vulnerability database interface for SCA and CVE lookups
pub struct VulnerabilityDatabase {
    /// Cache of known vulnerabilities
    vulnerability_cache: Arc<HashMap<String, VulnerabilityRecord>>,
    // Database connection or API client would go here
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityRecord {
    pub cve_id: Option<String>,
    pub cwe_id: Option<String>,
    pub severity: SecuritySeverity,
    pub cvss_score: Option<f64>,
    pub description: String,
    pub affected_versions: Vec<String>,
    pub fixed_versions: Vec<String>,
    pub references: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl VulnerabilityDatabase {
    pub fn new() -> Result<Self, AnalysisError> {
        info!("Initializing vulnerability database");
        
        let vulnerability_cache = Arc::new(HashMap::new());
        
        Ok(Self {
            vulnerability_cache,
        })
    }

    /// Look up vulnerability information by package and version
    pub async fn lookup_vulnerability(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<VulnerabilityRecord>, AnalysisError> {
        debug!("Looking up vulnerabilities for package: {} version: {}", package_name, version);
        
        // TODO: Implement actual vulnerability database lookup
        // This would involve querying:
        // - RustSec Advisory Database for Rust packages
        // - GitHub Advisory Database
        // - OSV database
        // - National Vulnerability Database (NVD)
        
        Ok(Vec::new())
    }

    /// Check if a specific vulnerability affects a package version
    pub async fn is_vulnerable(
        &self,
        package_name: &str,
        version: &str,
        vulnerability_id: &str,
    ) -> Result<bool, AnalysisError> {
        debug!("Checking if package {}:{} is affected by {}", package_name, version, vulnerability_id);
        
        // TODO: Implement vulnerability matching logic
        
        Ok(false)
    }

    /// Update vulnerability database from external sources
    pub async fn update_database(&mut self) -> Result<(), AnalysisError> {
        info!("Updating vulnerability database from external sources");
        
        // TODO: Implement database update logic
        // This would involve:
        // - Fetching latest advisories from RustSec
        // - Syncing with GitHub Advisory Database
        // - Updating local cache
        
        Ok(())
    }

    /// Get vulnerability statistics
    pub fn get_statistics(&self) -> VulnerabilityDatabaseStats {
        VulnerabilityDatabaseStats {
            total_vulnerabilities: self.vulnerability_cache.len(),
            last_updated: chrono::Utc::now(), // TODO: Track actual update time
            sources: vec!["RustSec".to_string(), "GitHub".to_string(), "OSV".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityDatabaseStats {
    pub total_vulnerabilities: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub sources: Vec<String>,
}

/// Detection method categorization for confidence scoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Deterministic static analysis (high confidence)
    Deterministic(f64),
    /// Heuristic pattern matching (medium confidence)
    Heuristic(f64),
    /// AI-inferred vulnerability (variable confidence)
    AiInferred(f64),
    /// Hybrid detection using multiple methods
    Hybrid(Vec<DetectionMethod>),
}

impl DetectionMethod {
    pub fn confidence_score(&self) -> f64 {
        match self {
            DetectionMethod::Deterministic(score) => *score,
            DetectionMethod::Heuristic(score) => *score,
            DetectionMethod::AiInferred(score) => *score,
            DetectionMethod::Hybrid(methods) => {
                if methods.is_empty() {
                    0.5
                } else {
                    let sum: f64 = methods.iter().map(|m| m.confidence_score()).sum();
                    (sum / methods.len() as f64).min(1.0)
                }
            }
        }
    }

    pub fn is_high_confidence(&self) -> bool {
        self.confidence_score() >= 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_security_context_creation() {
        let context = SecurityContext::new(
            PathBuf::from("test.rs"),
            "fn main() {}".to_string(),
            SourceLanguage::Rust,
        );

        assert_eq!(context.file_path, PathBuf::from("test.rs"));
        assert_eq!(context.language, SourceLanguage::Rust);
        assert!(context.is_source_file());
        assert!(!context.is_config_file());
    }

    #[test]
    fn test_security_analysis_result() {
        let mut result = SecurityAnalysisResult::new();
        assert_eq!(result.statistics.total_issues, 0);

        // Add a test vulnerability
        let location = crate::analysis::detectors::security::types::SecurityLocation::new(
            PathBuf::from("test.rs"),
            1,
            1,
        );
        
        let vulnerability = OwaspVulnerability::new(
            crate::analysis::detectors::security::owasp::OwaspCategory::Injection,
            crate::analysis::detectors::security::types::SecurityIssueType::Injection,
            "Test Vulnerability".to_string(),
            "Test description".to_string(),
            location,
        ).with_severity(SecuritySeverity::High);

        result.add_vulnerability(vulnerability);
        assert_eq!(result.statistics.total_issues, 1);
        assert_eq!(result.statistics.high_count, 1);
    }

    #[test]
    fn test_confidence_score_calculation() {
        let mut score = ConfidenceScore::new()
            .with_detection_method(0.9)  // High confidence detection
            .with_evidence_strength(0.8) // Strong evidence
            .with_architectural_context(0.6) // Some architectural issues
            .with_cross_validation(0.7); // Good cross-validation

        assert!(score.final_score > 0.7);
        assert!(score.is_high_confidence());
    }

    #[test]
    fn test_detection_method_confidence() {
        let deterministic = DetectionMethod::Deterministic(0.95);
        let heuristic = DetectionMethod::Heuristic(0.7);
        let ai_inferred = DetectionMethod::AiInferred(0.6);

        assert!(deterministic.is_high_confidence());
        assert!(!heuristic.is_high_confidence());
        assert!(!ai_inferred.is_high_confidence());

        let hybrid = DetectionMethod::Hybrid(vec![deterministic, heuristic]);
        assert_eq!(hybrid.confidence_score(), 0.825); // (0.95 + 0.7) / 2
    }

    #[tokio::test]
    async fn test_vulnerability_database_creation() {
        let db = VulnerabilityDatabase::new();
        assert!(db.is_ok());

        let db = db.unwrap();
        let stats = db.get_statistics();
        assert_eq!(stats.total_vulnerabilities, 0); // Empty initially
    }

    #[test]
    fn test_context_file_type_detection() {
        let rust_context = SecurityContext::new(
            PathBuf::from("main.rs"),
            "".to_string(),
            SourceLanguage::Rust,
        );
        assert!(rust_context.is_source_file());
        assert!(!rust_context.is_config_file());

        let config_context = SecurityContext::new(
            PathBuf::from("config.toml"),
            "".to_string(),
            SourceLanguage::Rust,
        );
        assert!(!config_context.is_source_file());
        assert!(config_context.is_config_file());
    }
}