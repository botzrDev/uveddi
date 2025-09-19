//! Main security knowledge graph detector implementation
//!
//! This module provides the main detector that integrates all knowledge graph components
//! to provide comprehensive security analysis through graph-based reasoning.

use crate::analysis::detectors::security::knowledge_graph::{
    config::KnowledgeGraphConfig,
    types::{
        SecurityQuery, SecurityKnowledgeResult,
        FileAnalysis, ArchitecturalCorrelation,
    },
    graph::GraphManager,
    knowledge::KnowledgeManager,
    security::SecurityAnalyzer,
    language_support::LanguageEntityProcessor,
};
use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType, SecuritySeverity, SecurityLocation, VulnerabilityType};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Main knowledge graph detector for security analysis
pub struct KnowledgeGraphDetector {
    config: KnowledgeGraphConfig,
    graph_manager: GraphManager,
    knowledge_manager: KnowledgeManager,
    security_analyzer: SecurityAnalyzer,
    language_processor: LanguageEntityProcessor,
}

impl KnowledgeGraphDetector {
    /// Create a new knowledge graph detector
    pub fn new(config: KnowledgeGraphConfig) -> Self {
        // Use the primary language from config, default to Rust
        let primary_language = SourceLanguage::Rust; // Would be configurable

        Self {
            config,
            graph_manager: GraphManager::new(primary_language.clone()),
            knowledge_manager: KnowledgeManager::new(primary_language),
            security_analyzer: SecurityAnalyzer::new(),
            language_processor: LanguageEntityProcessor::new(),
        }
    }

    /// Analyze files and build security knowledge graph
    pub async fn analyze_files(&mut self, file_paths: &[PathBuf]) -> Result<SecurityKnowledgeResult, AnalysisError> {
        info!("Starting knowledge graph analysis for {} files", file_paths.len());

        // Process files to extract entities
        let mut file_analyses = Vec::new();
        for file_path in file_paths {
            if let Ok(analysis) = self.process_file(file_path).await {
                file_analyses.push(analysis);
            }
        }

        if file_analyses.is_empty() {
            warn!("No files could be processed for knowledge graph analysis");
            return Ok(SecurityKnowledgeResult::empty());
        }

        // Build graph from analyses
        self.graph_manager.build_from_analyses(file_analyses.clone()).await?;

        // Extract all entities for further processing
        let all_entities: Vec<_> = file_analyses.iter()
            .flat_map(|analysis| analysis.entities.clone())
            .collect();

        // Process knowledge extraction and inference
        let processed_knowledge = self.knowledge_manager
            .process_entities(&all_entities)
            .await?;

        // Perform security analysis on the graph
        let security_result = self.security_analyzer
            .analyze_security(self.graph_manager.graph(), &all_entities)
            .await?;

        // Analyze graph patterns
        let graph_patterns = self.graph_manager.analyze_security_patterns().await?;

        // Combine results
        let mut all_security_issues = security_result.security_issues.clone();
        all_security_issues.extend(self.convert_patterns_to_issues(&graph_patterns));

        // Calculate confidence based on multiple factors
        let confidence_score = self.calculate_overall_confidence(
            &processed_knowledge,
            &security_result,
            &all_security_issues,
        );

        Ok(SecurityKnowledgeResult {
            structural_facts: crate::analysis::detectors::security::knowledge_graph::types::StructuralQueryResult {
                entities: all_entities,
                relationships: processed_knowledge.extracted.relationships,
                anti_patterns: graph_patterns,
            },
            semantic_insights: crate::analysis::detectors::security::knowledge_graph::types::SemanticQueryResult {
                insights: processed_knowledge.inferred.security_insights.iter()
                    .map(|insight| insight.description.clone())
                    .collect(),
                concepts: Vec::new(),
                confidence_score: processed_knowledge.inference_validation.overall_confidence,
            },
            contextual_information: crate::analysis::detectors::security::knowledge_graph::types::RAGQueryResult {
                relevant_contexts: Vec::new(),
                similarity_scores: HashMap::new(),
                retrieved_facts: processed_knowledge.inferred.inferred_facts.iter()
                    .map(|fact| format!("{:?}: {}", fact.fact_type, fact.subject))
                    .collect(),
            },
            historical_patterns: Vec::new(),
            confidence_score,
        })
    }

    /// Query the knowledge graph for specific security information
    pub async fn query_security_context(
        &self,
        query: SecurityQuery,
    ) -> Result<SecurityKnowledgeResult, AnalysisError> {
        debug!("Querying knowledge graph: {:?}", query);

        // This would implement querying logic based on the built graph
        // For now, return empty result as placeholder
        Ok(SecurityKnowledgeResult::empty())
    }

    /// Correlate security issues with architectural patterns
    pub async fn correlate_with_architecture(
        &self,
        security_issue: &SecurityIssue,
    ) -> Result<ArchitecturalCorrelation, AnalysisError> {
        debug!("Correlating security issue with architecture: {}", security_issue.title);

        // This would implement correlation logic
        // For now, return placeholder correlation
        Ok(ArchitecturalCorrelation {
            security_issue_id: security_issue.id.clone().unwrap_or_default(),
            architectural_patterns: Vec::new(),
            correlation_strength: 0.5,
            amplification_factors: HashMap::new(),
            recommendations: vec![
                "Review architectural patterns in affected area".to_string(),
                "Consider refactoring to reduce coupling".to_string(),
            ],
        })
    }

    /// Process a single file for entity extraction
    async fn process_file(&self, file_path: &PathBuf) -> Result<FileAnalysis, AnalysisError> {
        debug!("Processing file: {}", file_path.display());

        // Read file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read file {}: {}", file_path.display(), e);
                return Err(AnalysisError::Other(format!("File read error: {}", e)));
            }
        };

        // Detect language
        let language = self.language_processor.detect_language(file_path);
        let language = match language {
            Some(lang) => lang,
            None => {
                debug!("Skipping file with unknown language: {}", file_path.display());
                return Ok(FileAnalysis {
                    file_path: file_path.clone(),
                    entities: Vec::new(),
                    security_issues: Vec::new(),
                    anti_patterns: Vec::new(),
                });
            }
        };

        // Extract entities
        let entities = self.language_processor
            .process_file_entities(file_path, &content, language)
            .await?;

        debug!("Extracted {} entities from {}", entities.len(), file_path.display());

        Ok(FileAnalysis {
            file_path: file_path.clone(),
            entities,
            security_issues: Vec::new(), // Would be populated by other detectors
            anti_patterns: Vec::new(),   // Would be populated by pattern analysis
        })
    }

    /// Convert anti-pattern info to security issues
    fn convert_patterns_to_issues(&self, patterns: &[crate::analysis::detectors::security::knowledge_graph::types::AntiPatternInfo]) -> Vec<SecurityIssue> {
        patterns.iter().map(|pattern| {
            SecurityIssue {
                id: Some(format!("knowledge_graph_pattern_{}", pattern.pattern_type)),
                title: format!("Architectural Anti-Pattern: {}", pattern.pattern_type),
                description: pattern.description.clone(),
                issue_type: self.map_pattern_to_issue_type(&pattern.pattern_type),
                vulnerability_type: VulnerabilityType::Static,
                severity: self.map_severity_score_to_severity(pattern.severity),
                confidence_score: pattern.severity,
                location: self.default_location(),
                language: Some(SourceLanguage::Rust),
                remediation: Some(format!("Address the {} anti-pattern to improve security", pattern.pattern_type)),
                context: std::collections::HashMap::new(),
                metadata: Default::default(),
                detected_by: vec!["KnowledgeGraphDetector".to_string()],
                correlation_id: None,
            }
        }).collect()
    }

    /// Map pattern type to security issue type
    fn map_pattern_to_issue_type(&self, pattern_type: &str) -> SecurityIssueType {
        match pattern_type.to_lowercase().as_str() {
            s if s.contains("privilege") => SecurityIssueType::PrivilegeEscalation,
            s if s.contains("injection") => SecurityIssueType::Injection,
            s if s.contains("trust") => SecurityIssueType::BrokenAccessControl,
            s if s.contains("auth") => SecurityIssueType::AuthenticationFailures,
            s if s.contains("coupling") => SecurityIssueType::InsecureDesign,
            _ => SecurityIssueType::Custom("Unknown".to_string()),
        }
    }

    /// Map severity score to security severity
    fn map_severity_score_to_severity(&self, score: f64) -> crate::analysis::detectors::security::types::SecuritySeverity {
        if score >= 0.9 {
            crate::analysis::detectors::security::types::SecuritySeverity::Critical
        } else if score >= 0.7 {
            crate::analysis::detectors::security::types::SecuritySeverity::High
        } else if score >= 0.5 {
            crate::analysis::detectors::security::types::SecuritySeverity::Medium
        } else if score >= 0.3 {
            crate::analysis::detectors::security::types::SecuritySeverity::Low
        } else {
            crate::analysis::detectors::security::types::SecuritySeverity::Info
        }
    }

    /// Calculate overall confidence score
    fn calculate_overall_confidence(
        &self,
        processed_knowledge: &crate::analysis::detectors::security::knowledge_graph::knowledge::ProcessedKnowledge,
        security_result: &crate::analysis::detectors::security::knowledge_graph::security::SecurityAnalysisResult,
        security_issues: &[SecurityIssue],
    ) -> f64 {
        let knowledge_confidence = processed_knowledge.extraction_validation.validation_score;
        let inference_confidence = processed_knowledge.inference_validation.overall_confidence;
        let security_confidence = security_result.confidence_score;

        // Weight the different confidence scores
        let overall = (knowledge_confidence * 0.3 +
                      inference_confidence * 0.3 +
                      security_confidence * 0.4);

        // Adjust based on number of issues found (more issues = higher confidence in detection)
        let issue_boost = if security_issues.is_empty() {
            1.0
        } else {
            1.0 + (security_issues.len() as f64 * 0.05).min(0.2)
        };

        (overall * issue_boost).min(1.0)
    }

    /// Default location for issues without specific location
    fn default_location(&self) -> crate::analysis::detectors::security::types::SecurityLocation {
        crate::analysis::detectors::security::types::SecurityLocation::new(
            PathBuf::from("unknown"),
            0,
            0,
        )
    }

    /// Get detector statistics
    pub fn get_statistics(&self) -> DetectorStatistics {
        let graph_stats = self.graph_manager.get_statistics();

        DetectorStatistics {
            nodes_analyzed: graph_stats.node_count,
            edges_analyzed: graph_stats.edge_count,
            languages_supported: 3, // Rust, Python, JavaScript
            analysis_modules: 5,    // Graph, Knowledge, Security, Language, Main
        }
    }
}

impl SecurityKnowledgeResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            structural_facts: crate::analysis::detectors::security::knowledge_graph::types::StructuralQueryResult {
                entities: Vec::new(),
                relationships: Vec::new(),
                anti_patterns: Vec::new(),
            },
            semantic_insights: crate::analysis::detectors::security::knowledge_graph::types::SemanticQueryResult {
                insights: Vec::new(),
                concepts: Vec::new(),
                confidence_score: 0.0,
            },
            contextual_information: crate::analysis::detectors::security::knowledge_graph::types::RAGQueryResult {
                relevant_contexts: Vec::new(),
                similarity_scores: HashMap::new(),
                retrieved_facts: Vec::new(),
            },
            historical_patterns: Vec::new(),
            confidence_score: 0.0,
        }
    }
}

/// Statistics about the detector performance
#[derive(Debug, Clone)]
pub struct DetectorStatistics {
    pub nodes_analyzed: usize,
    pub edges_analyzed: usize,
    pub languages_supported: usize,
    pub analysis_modules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_creation() {
        let config = KnowledgeGraphConfig::default();
        let detector = KnowledgeGraphDetector::new(config);

        let stats = detector.get_statistics();
        assert_eq!(stats.languages_supported, 3);
        assert_eq!(stats.analysis_modules, 5);
    }

    #[tokio::test]
    async fn test_empty_file_analysis() {
        let config = KnowledgeGraphConfig::default();
        let mut detector = KnowledgeGraphDetector::new(config);

        let result = detector.analyze_files(&[]).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.confidence_score, 0.0);
    }
}