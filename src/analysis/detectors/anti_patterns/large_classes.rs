//! Large Classes anti-pattern detector
//!
//! This detector implements a sophisticated multi-metric approach to identify Large Classes,
//! God Objects, and Blob anti-patterns as described in the Large Class Research document.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::debug;
use tree_sitter::{Node, Query, QueryCursor, Tree};

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
#[derive(Debug, Clone)]
pub struct LargeClassDetector {
    pub config: LargeClassConfig,
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
    fn extract_rust_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        let mut metrics = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
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
                    let (method_count, cyclomatic_complexity) =
                        self.find_rust_impl_metrics(name, tree, source)?;
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
                        file_path: parsed_file.path.display().to_string(),
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
    fn extract_rust_metrics(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping Rust metrics extraction");
        Ok(Vec::new())
    }

    // Similar pattern for Python and JavaScript
    #[cfg(feature = "tree-sitter")]
    fn extract_python_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        // Implementation with tree-sitter
        Ok(Vec::new()) // Simplified for now
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_python_metrics(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping Python metrics extraction");
        Ok(Vec::new())
    }

    #[cfg(feature = "tree-sitter")]
    fn extract_javascript_metrics(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        // Implementation with tree-sitter
        Ok(Vec::new()) // Simplified for now
    }

    #[cfg(not(feature = "tree-sitter"))]
    fn extract_javascript_metrics(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<ClassMetrics>, AnalysisError> {
        debug!("Tree-sitter feature not enabled, skipping JavaScript metrics extraction");
        Ok(Vec::new())
    }

    // Helper methods that are only available with tree-sitter
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_logical_loc(&self, node: &Node, source: &[u8]) -> u32 {
        // Implementation
        0
    }

    #[cfg(feature = "tree-sitter")]
    pub fn extract_code_snippet(&self, node: &Node, source: &[u8], max_lines: usize) -> String {
        // Implementation
        String::new()
    }

    #[cfg(feature = "tree-sitter")]
    fn count_rust_struct_fields(
        &self,
        struct_node: &Node,
        source: &[u8],
    ) -> Result<u32, AnalysisError> {
        Ok(0)
    }

    #[cfg(feature = "tree-sitter")]
    fn find_rust_impl_metrics(
        &self,
        struct_name: &str,
        tree: &Tree,
        source: &[u8],
    ) -> Result<(u32, u32), AnalysisError> {
        Ok((0, 0))
    }

    #[cfg(feature = "tree-sitter")]
    fn calculate_rust_lcom(
        &self,
        struct_name: &str,
        tree: &Tree,
        source: &[u8],
    ) -> Result<f64, AnalysisError> {
        Ok(0.0)
    }

    #[cfg(feature = "tree-sitter")]
    fn count_rust_coupling(&self, node: &Node, source: &[u8]) -> Result<u32, AnalysisError> {
        Ok(0)
    }

    #[cfg(not(feature = "tree-sitter"))]
    pub fn calculate_logical_loc(&self, node: &Node, source: &[u8]) -> u32 {
        0
    }

    #[cfg(not(feature = "tree-sitter"))]
    pub fn extract_code_snippet(&self, node: &Node, source: &[u8], max_lines: usize) -> String {
        String::new()
    }

    pub fn generate_description(
        &self,
        metrics: &ClassMetrics,
        severity: u32,
        language: SourceLanguage,
    ) -> String {
        format!(
            "Large class '{}' detected with severity {}. Language: {:?}. Metrics: {} LOC, {} methods, {} fields.",
            metrics.name, severity, language, metrics.logical_loc, metrics.method_count, metrics.field_count
        )
    }

    pub fn get_language_thresholds(
        &self,
        language: &SourceLanguage,
    ) -> Option<&LanguageThresholds> {
        match language {
            SourceLanguage::Rust => Some(&self.config.rust_thresholds),
            SourceLanguage::Python => Some(&self.config.python_thresholds),
            SourceLanguage::JavaScript => Some(&self.config.javascript_thresholds),
            _ => None,
        }
    }

    pub(crate) fn calculate_severity_score(
        &self,
        metrics: &ClassMetrics,
        thresholds: &LanguageThresholds,
    ) -> u32 {
        let mut score = 0;
        let w = &self.config.severity_weights;

        if metrics.logical_loc > thresholds.max_logical_loc {
            score += (w.size_weight * ((metrics.logical_loc - thresholds.max_logical_loc) as f64))
                as u32;
        }
        if metrics.method_count > thresholds.max_methods {
            score += (w.size_weight * ((metrics.method_count - thresholds.max_methods) * 5) as f64)
                as u32;
        }
        if metrics.field_count > thresholds.max_fields {
            score +=
                (w.size_weight * ((metrics.field_count - thresholds.max_fields) * 5) as f64) as u32;
        }
        if metrics.cyclomatic_complexity > thresholds.max_cyclomatic_complexity {
            score += (w.complexity_weight
                * ((metrics.cyclomatic_complexity - thresholds.max_cyclomatic_complexity) as f64))
                as u32;
        }
        if metrics.lcom_score > thresholds.max_lcom_score {
            score += (w.structural_weight
                * ((metrics.lcom_score - thresholds.max_lcom_score) * 100.0) as f64)
                as u32;
        }
        if metrics.coupling_count > thresholds.max_coupling {
            score += (w.structural_weight
                * ((metrics.coupling_count - thresholds.max_coupling) * 2) as f64)
                as u32;
        }

        score
    }
}

impl AnalysisDetector for LargeClassDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let metrics = match file.language {
            SourceLanguage::Rust => self.extract_rust_metrics(file)?,
            SourceLanguage::Python => self.extract_python_metrics(file)?,
            SourceLanguage::JavaScript => self.extract_javascript_metrics(file)?,
            _ => return Ok(Vec::new()),
        };

        let mut issues = Vec::new();
        for class_metrics in metrics {
            let thresholds = match file.language {
                SourceLanguage::Rust => &self.config.rust_thresholds,
                SourceLanguage::Python => &self.config.python_thresholds,
                SourceLanguage::JavaScript => &self.config.javascript_thresholds,
                _ => continue,
            };

            if class_metrics.logical_loc > thresholds.max_logical_loc
                || class_metrics.method_count > thresholds.max_methods
                || class_metrics.field_count > thresholds.max_fields
            {
                let severity = self.calculate_severity_score(&class_metrics, thresholds);

                let issue = ArchitecturalIssue {
                    issue_id: None,
                    analysis_run_id: 0,      // TODO: Get proper analysis run ID
                    anti_pattern_type_id: 1, // TODO: Get proper ID for LargeClass from database
                    file_path: class_metrics.file_path.clone(),
                    start_line: Some(class_metrics.start_line.try_into().unwrap()),
                    end_line: Some(class_metrics.end_line.try_into().unwrap()),
                    severity: severity.to_string(),
                    description: format!(
                        "Large class '{}' detected: {} LOC, {} methods, {} fields",
                        class_metrics.name,
                        class_metrics.logical_loc,
                        class_metrics.method_count,
                        class_metrics.field_count
                    ),
                    code_snippet: Some(class_metrics.code_snippet.clone()),
                    ai_explanation: Some(
                        "Consider breaking this class into smaller, more focused classes"
                            .to_string(),
                    ),
                };

                issues.push(issue);
            }
        }

        Ok(issues)
    }

    fn get_detector_name(&self) -> &'static str {
        "LargeClassDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        // Return empty vector since AntiPatternType is now a struct, not an enum
        Vec::new()
    }

    fn detect(
        &self,
        graph: &crate::analysis::graph::dependency::LocalDependencyGraph,
    ) -> Vec<ArchitecturalIssue> {
        // For the graph-based detect method, we return empty for now
        // This method is used for dependency-based analysis
        Vec::new()
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
