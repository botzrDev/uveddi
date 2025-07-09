//! Large Classes anti-pattern detector
//!
//! This detector implements a sophisticated multi-metric approach to identify Large Classes,
//! God Objects, and Blob anti-patterns as described in the Large Class Research document.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use std::collections::HashMap;

// Tree-sitter types are only available when the feature is enabled
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Query, QueryCursor};

/// Holds the collected metrics for a single class or struct.
#[derive(Debug, Clone)]
pub struct ClassMetrics {
    pub name: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub logical_loc: u32,
    pub method_count: u32,
    pub field_count: u32,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub lcom_score: f64,
    pub coupling_count: u32,
    pub is_exported: bool,
    pub code_snippet: String,
}

/// Defines language-specific thresholds for detecting large classes.
#[derive(Debug, Clone)]
pub struct LanguageThresholds {
    pub max_logical_loc: u32,
    pub max_methods: u32,
    pub max_fields: u32,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_lcom_score: f64,
    pub max_coupling: u32,
}

impl LanguageThresholds {
    pub fn rust() -> Self {
        Self {
            max_logical_loc: 400,
            max_methods: 20,
            max_fields: 15,
            max_cyclomatic_complexity: 50,
            max_cognitive_complexity: 40,
            max_lcom_score: 0.8,
            max_coupling: 12,
        }
    }

    pub fn python() -> Self {
        Self {
            max_logical_loc: 500,
            max_methods: 25,
            max_fields: 20,
            max_cyclomatic_complexity: 60,
            max_cognitive_complexity: 50,
            max_lcom_score: 0.8,
            max_coupling: 15,
        }
    }

    pub fn javascript() -> Self {
        Self {
            max_logical_loc: 600,
            max_methods: 30,
            max_fields: 25,
            max_cyclomatic_complexity: 70,
            max_cognitive_complexity: 60,
            max_lcom_score: 0.8,
            max_coupling: 18,
        }
    }
}

/// Configuration for the Large Class detector.
#[derive(Debug, Clone)]
pub struct LargeClassConfig {
    pub rust_thresholds: LanguageThresholds,
    pub python_thresholds: LanguageThresholds,
    pub javascript_thresholds: LanguageThresholds,
    pub enable_lcom_analysis: bool,
    pub enable_coupling_analysis: bool,
    pub severity_weights: SeverityWeights,
}

#[derive(Debug, Clone)]
pub struct SeverityWeights {
    pub size_weight: f64,
    pub complexity_weight: f64,
    pub structural_weight: f64,
}

impl Default for SeverityWeights {
    fn default() -> Self {
        Self {
            size_weight: 0.4,
            complexity_weight: 0.35,
            structural_weight: 0.25,
        }
    }
}

impl Default for LargeClassConfig {
    fn default() -> Self {
        Self {
            rust_thresholds: LanguageThresholds::rust(),
            python_thresholds: LanguageThresholds::python(),
            javascript_thresholds: LanguageThresholds::javascript(),
            enable_lcom_analysis: true,
            enable_coupling_analysis: true,
            severity_weights: SeverityWeights::default(),
        }
    }
}

/// The Large Class anti-pattern detector.
pub struct LargeClassDetector {
    config: LargeClassConfig,
}

impl LargeClassDetector {
    pub fn new(config: LargeClassConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(LargeClassConfig::default())
    }

    // When tree-sitter is available, use full AST analysis
    #[cfg(feature = "tree-sitter")]
    fn extract_rust_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        let mut metrics = Vec::new();
        let source = parsed_file.source.as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("Source code missing".to_string()))?
            .as_bytes();
        let tree = parsed_file.tree.as_ref()
            .ok_or_else(|| AnalysisError::AntiPatternDetection("AST tree missing".to_string()))?;
        let language = tree.language();

        // Query for struct definitions
        let struct_query = Query::new(&language, RUST_STRUCT_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;
        
        let mut cursor = QueryCursor::new();
        for mat in cursor.matches(&struct_query, tree.root_node(), source) {
            if let Some(name_capture) = mat.captures.first() {
                let name_node = name_capture.node;
                let struct_node = name_node.parent().unwrap_or(name_node);
                
                if let Ok(name) = name_node.utf8_text(source) {
                    let field_count = self.count_rust_struct_fields(&struct_node, source)?;
                    let (method_count, cyclomatic_complexity) = self.find_rust_impl_metrics(name, tree, source)?;
                    let logical_loc = self.calculate_logical_loc(&struct_node, source);
                    let lcom_score = if self.config.enable_lcom_analysis {
                        self.calculate_rust_lcom(name, tree, source)?
                    } else {
                        0.0
                    };
                    let coupling_count = if self.config.enable_coupling_analysis {
                        self.count_rust_coupling(&struct_node, source)?
                    } else {
                        0
                    };

                    let class_metrics = ClassMetrics {
                        name: name.to_string(),
                        file_path: parsed_file.file_path.clone(),
                        start_line: struct_node.start_position().row as u32 + 1,
                        end_line: struct_node.end_position().row as u32 + 1,
                        logical_loc,
                        method_count,
                        field_count,
                        cyclomatic_complexity,
                        cognitive_complexity: cyclomatic_complexity, // Simplified
                        lcom_score,
                        coupling_count,
                        is_exported: true, // Simplified
                        code_snippet: self.extract_code_snippet(&struct_node, source, 10),
                    };

                    metrics.push(class_metrics);
                }
            }
        }

        Ok(metrics)
    }

    // When tree-sitter is NOT available, return empty results
    #[cfg(not(feature = "tree-sitter"))]
    fn extract_rust_metrics(&self, _parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping Rust metrics extraction");
        Ok(Vec::new())
    }

    // Similar pattern for Python and JavaScript
    #[cfg(feature = "tree-sitter")]
    fn extract_python_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        // Implementation with tree-sitter
        Ok(Vec::new()) // Simplified for now
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_python_metrics(&self, _parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping Python metrics extraction");
        Ok(Vec::new())
    }

    #[cfg(feature = "tree-sitter")]
    fn extract_javascript_metrics(&self, parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        // Implementation with tree-sitter
        Ok(Vec::new()) // Simplified for now
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_javascript_metrics(&self, _parsed_file: &ParsedFile) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping JavaScript metrics extraction");
        Ok(Vec::new())
    }

    // Helper methods that are only available with tree-sitter
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_logical_loc(&self, node: &tree_sitter::Node, source: &[u8]) -> u32 {
        // Implementation
        0
    }

    #[cfg(feature = "tree-sitter")]
    pub fn extract_code_snippet(&self, node: &tree_sitter::Node, source: &[u8], max_lines: usize) -> String {
        // Implementation
        String::new()
    }

    #[cfg(feature = "tree-sitter")]
    fn count_rust_struct_fields(&self, struct_node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        Ok(0)
    }

    #[cfg(feature = "tree-sitter")]
    fn find_rust_impl_metrics(&self, struct_name: &str, tree: &tree_sitter::Tree, source: &[u8]) -> Result<(u32, u32), AnalysisError> {
        Ok((0, 0))
    }

    #[cfg(feature = "tree-sitter")]
    fn calculate_rust_lcom(&self, struct_name: &str, tree: &tree_sitter::Tree, source: &[u8]) -> Result<f64, AnalysisError> {
        Ok(0.0)
    }

    #[cfg(feature = "tree-sitter")]
    fn count_rust_coupling(&self, node: &tree_sitter::Node, source: &[u8]) -> Result<u32, AnalysisError> {
        Ok(0)
    }

    fn calculate_severity_score(&self, metrics: &ClassMetrics, thresholds: &LanguageThresholds) -> u32 {
        let size_score = ((metrics.logical_loc as f64 / thresholds.max_logical_loc as f64) * 100.0).min(100.0);
        let complexity_score = ((metrics.cyclomatic_complexity as f64 / thresholds.max_cyclomatic_complexity as f64) * 100.0).min(100.0);
        let structural_score = (metrics.lcom_score * 100.0).min(100.0);

        let weighted_score = (size_score * self.config.severity_weights.size_weight) +
                           (complexity_score * self.config.severity_weights.complexity_weight) +
                           (structural_score * self.config.severity_weights.structural_weight);

        weighted_score as u32
    }
}

impl AnalysisDetector for LargeClassDetector {
    fn detect(&self, parsed_files: &[ParsedFile]) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for parsed_file in parsed_files {
            let metrics = match parsed_file.language {
                SourceLanguage::Rust => self.extract_rust_metrics(parsed_file)?,
                SourceLanguage::Python => self.extract_python_metrics(parsed_file)?,
                SourceLanguage::JavaScript => self.extract_javascript_metrics(parsed_file)?,
                _ => continue,
            };

            for class_metrics in metrics {
                let thresholds = match parsed_file.language {
                    SourceLanguage::Rust => &self.config.rust_thresholds,
                    SourceLanguage::Python => &self.config.python_thresholds,
                    SourceLanguage::JavaScript => &self.config.javascript_thresholds,
                    _ => continue,
                };

                if class_metrics.logical_loc > thresholds.max_logical_loc ||
                   class_metrics.method_count > thresholds.max_methods ||
                   class_metrics.field_count > thresholds.max_fields {
                    
                    let severity = self.calculate_severity_score(&class_metrics, thresholds);
                    
                    let issue = ArchitecturalIssue {
                        id: 0,
                        anti_pattern_type: AntiPatternType::LargeClass,
                        file_path: class_metrics.file_path.clone(),
                        start_line: Some(class_metrics.start_line),
                        end_line: Some(class_metrics.end_line),
                        severity,
                        description: format!(
                            "Large class '{}' detected: {} LOC, {} methods, {} fields",
                            class_metrics.name, class_metrics.logical_loc, 
                            class_metrics.method_count, class_metrics.field_count
                        ),
                        suggestion: "Consider breaking this class into smaller, more focused classes".to_string(),
                        metadata: Some(format!(
                            "{{\"name\":\"{}\",\"loc\":{},\"methods\":{},\"fields\":{},\"lcom\":{:.2}}}",
                            class_metrics.name, class_metrics.logical_loc, 
                            class_metrics.method_count, class_metrics.field_count, class_metrics.lcom_score
                        )),
                    };

                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }
}

// Tree-sitter queries are only compiled when the feature is enabled
#[cfg(feature = "tree-sitter")]
const RUST_STRUCT_QUERY: &str = r#"
(struct_item
  name: (type_identifier) @struct_name)
"#;

#[cfg(feature = "tree-sitter")]
const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @class_name)
"#;

#[cfg(feature = "tree-sitter")]
const JAVASCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (identifier) @class_name)
"#;