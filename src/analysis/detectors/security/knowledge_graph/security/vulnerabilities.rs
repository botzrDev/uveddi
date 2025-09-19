//! Vulnerability analysis through knowledge graph traversal

use crate::analysis::detectors::security::knowledge_graph::types::{CodeEntity, StructuralSemanticGraph};
use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType, SecuritySeverity};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet};

/// Vulnerability analyzer using graph traversal
pub struct VulnerabilityAnalyzer {
    vulnerability_patterns: Vec<VulnerabilityPattern>,
}

impl VulnerabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            vulnerability_patterns: vec![
                VulnerabilityPattern::SqlInjection,
                VulnerabilityPattern::XssVulnerability,
                VulnerabilityPattern::BufferOverflow,
                VulnerabilityPattern::UseAfterFree,
                VulnerabilityPattern::RaceCondition,
                VulnerabilityPattern::IntegerOverflow,
            ],
        }
    }

    /// Analyze graph for security vulnerabilities
    pub async fn analyze_vulnerabilities(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<VulnerabilityAnalysisResult, AnalysisError> {
        let mut vulnerabilities = Vec::new();
        let mut risk_scores = HashMap::new();

        for pattern in &self.vulnerability_patterns {
            let pattern_vulns = self.detect_vulnerability_pattern(pattern, graph, entities).await?;
            vulnerabilities.extend(pattern_vulns);
        }

        // Calculate risk scores for entities
        for entity in entities {
            let risk_score = self.calculate_entity_risk_score(entity, graph, &vulnerabilities);
            risk_scores.insert(entity.id.clone(), risk_score);
        }

        Ok(VulnerabilityAnalysisResult {
            vulnerabilities,
            risk_scores,
            overall_risk: self.calculate_overall_risk(&vulnerabilities),
        })
    }

    async fn detect_vulnerability_pattern(
        &self,
        pattern: &VulnerabilityPattern,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        match pattern {
            VulnerabilityPattern::SqlInjection => self.detect_sql_injection(graph, entities).await,
            VulnerabilityPattern::XssVulnerability => self.detect_xss_vulnerability(graph, entities).await,
            VulnerabilityPattern::BufferOverflow => self.detect_buffer_overflow(entities).await,
            VulnerabilityPattern::UseAfterFree => self.detect_use_after_free(graph, entities).await,
            VulnerabilityPattern::RaceCondition => self.detect_race_condition(graph, entities).await,
            VulnerabilityPattern::IntegerOverflow => self.detect_integer_overflow(entities).await,
        }
    }

    async fn detect_sql_injection(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.is_sql_related(entity) && self.has_injection_risk(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("sql_injection_{}", entity.id)),
                    title: "Potential SQL Injection".to_string(),
                    description: format!("Entity '{}' may be vulnerable to SQL injection", entity.name),
                    issue_type: SecurityIssueType::Injection,
                    severity: SecuritySeverity::High,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Use parameterized queries and input validation".to_string(),
                    cwe_id: Some("CWE-89".to_string()),
                    confidence: 0.8,
                });
            }
        }

        Ok(issues)
    }

    async fn detect_xss_vulnerability(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.is_web_related(entity) && self.has_xss_risk(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("xss_vulnerability_{}", entity.id)),
                    title: "Potential XSS Vulnerability".to_string(),
                    description: format!("Entity '{}' may be vulnerable to XSS attacks", entity.name),
                    issue_type: SecurityIssueType::Injection,
                    severity: SecuritySeverity::High,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Implement proper output encoding and input validation".to_string(),
                    cwe_id: Some("CWE-79".to_string()),
                    confidence: 0.7,
                });
            }
        }

        Ok(issues)
    }

    async fn detect_buffer_overflow(&self, entities: &[CodeEntity]) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.has_buffer_operations(entity) && self.lacks_bounds_checking(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("buffer_overflow_{}", entity.id)),
                    title: "Potential Buffer Overflow".to_string(),
                    description: format!("Entity '{}' may have buffer overflow vulnerabilities", entity.name),
                    issue_type: SecurityIssueType::BufferOverflow,
                    severity: SecuritySeverity::Critical,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Add bounds checking and use safe buffer operations".to_string(),
                    cwe_id: Some("CWE-120".to_string()),
                    confidence: 0.6,
                });
            }
        }

        Ok(issues)
    }

    async fn detect_use_after_free(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.has_memory_operations(entity) && self.has_use_after_free_pattern(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("use_after_free_{}", entity.id)),
                    title: "Potential Use After Free".to_string(),
                    description: format!("Entity '{}' may have use-after-free vulnerabilities", entity.name),
                    issue_type: SecurityIssueType::MemoryCorruption,
                    severity: SecuritySeverity::Critical,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Review memory management and use safe patterns".to_string(),
                    cwe_id: Some("CWE-416".to_string()),
                    confidence: 0.5,
                });
            }
        }

        Ok(issues)
    }

    async fn detect_race_condition(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.has_concurrent_access(entity) && self.lacks_synchronization(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("race_condition_{}", entity.id)),
                    title: "Potential Race Condition".to_string(),
                    description: format!("Entity '{}' may have race condition vulnerabilities", entity.name),
                    issue_type: SecurityIssueType::RaceCondition,
                    severity: SecuritySeverity::Medium,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Add proper synchronization mechanisms".to_string(),
                    cwe_id: Some("CWE-362".to_string()),
                    confidence: 0.6,
                });
            }
        }

        Ok(issues)
    }

    async fn detect_integer_overflow(&self, entities: &[CodeEntity]) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in entities {
            if self.has_arithmetic_operations(entity) && self.lacks_overflow_checks(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("integer_overflow_{}", entity.id)),
                    title: "Potential Integer Overflow".to_string(),
                    description: format!("Entity '{}' may have integer overflow vulnerabilities", entity.name),
                    issue_type: SecurityIssueType::IntegerOverflow,
                    severity: SecuritySeverity::Medium,
                    location: self.entity_to_location(entity),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Add overflow checking for arithmetic operations".to_string(),
                    cwe_id: Some("CWE-190".to_string()),
                    confidence: 0.4,
                });
            }
        }

        Ok(issues)
    }

    fn calculate_entity_risk_score(
        &self,
        entity: &CodeEntity,
        _graph: &StructuralSemanticGraph,
        vulnerabilities: &[SecurityIssue],
    ) -> f64 {
        let entity_vulns = vulnerabilities.iter()
            .filter(|v| v.id.as_ref().map(|id| id.contains(&entity.id)).unwrap_or(false))
            .count();

        (entity_vulns as f64 * 0.2).min(1.0)
    }

    fn calculate_overall_risk(&self, vulnerabilities: &[SecurityIssue]) -> f64 {
        if vulnerabilities.is_empty() {
            0.0
        } else {
            let critical_count = vulnerabilities.iter().filter(|v| matches!(v.severity, SecuritySeverity::Critical)).count();
            let high_count = vulnerabilities.iter().filter(|v| matches!(v.severity, SecuritySeverity::High)).count();

            ((critical_count as f64 * 1.0 + high_count as f64 * 0.7) / vulnerabilities.len() as f64).min(1.0)
        }
    }

    // Helper methods for vulnerability detection
    fn is_sql_related(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("sql") || name_lower.contains("query") || name_lower.contains("database")
    }

    fn has_injection_risk(&self, entity: &CodeEntity) -> bool {
        entity.metadata.get("uses_dynamic_queries").is_some() ||
        entity.name.to_lowercase().contains("concat")
    }

    fn is_web_related(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("html") || name_lower.contains("web") || name_lower.contains("render")
    }

    fn has_xss_risk(&self, entity: &CodeEntity) -> bool {
        entity.metadata.get("outputs_html").is_some() ||
        entity.name.to_lowercase().contains("unsafe")
    }

    fn has_buffer_operations(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("copy") || name_lower.contains("buffer") || name_lower.contains("memcpy")
    }

    fn lacks_bounds_checking(&self, _entity: &CodeEntity) -> bool {
        // Placeholder: would check for bounds checking patterns
        true // Conservative assumption
    }

    fn has_memory_operations(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("malloc") || name_lower.contains("free") || name_lower.contains("new") || name_lower.contains("delete")
    }

    fn has_use_after_free_pattern(&self, _entity: &CodeEntity) -> bool {
        // Placeholder: would analyze for use-after-free patterns
        false
    }

    fn has_concurrent_access(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("thread") || name_lower.contains("async") || name_lower.contains("parallel")
    }

    fn lacks_synchronization(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        !name_lower.contains("mutex") && !name_lower.contains("lock") && !name_lower.contains("sync")
    }

    fn has_arithmetic_operations(&self, entity: &CodeEntity) -> bool {
        entity.metadata.get("has_arithmetic").is_some() ||
        entity.name.to_lowercase().contains("calc")
    }

    fn lacks_overflow_checks(&self, _entity: &CodeEntity) -> bool {
        // Placeholder: would check for overflow protection
        true // Conservative assumption
    }

    fn entity_to_location(&self, entity: &CodeEntity) -> crate::analysis::detectors::security::types::SecurityLocation {
        crate::analysis::detectors::security::types::SecurityLocation::new(
            entity.location.file_path.clone(),
            entity.location.start_line,
            entity.location.end_line,
        )
    }
}

/// Types of vulnerabilities to detect
#[derive(Debug, Clone)]
pub enum VulnerabilityPattern {
    SqlInjection,
    XssVulnerability,
    BufferOverflow,
    UseAfterFree,
    RaceCondition,
    IntegerOverflow,
}

/// Result of vulnerability analysis
#[derive(Debug, Clone)]
pub struct VulnerabilityAnalysisResult {
    pub vulnerabilities: Vec<SecurityIssue>,
    pub risk_scores: HashMap<String, f64>,
    pub overall_risk: f64,
}