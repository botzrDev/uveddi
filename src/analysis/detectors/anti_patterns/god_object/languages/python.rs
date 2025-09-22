//! Python-specific God Object detection

use super::super::config::GodObjectConfig;
use super::super::detector::{ComplexityMetrics, DetectedPattern};
use super::super::metrics::MetricsCalculator;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use streaming_iterator::StreamingIterator;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use crate::error::ErrorHelpers;
use std::collections::HashSet;
use tracing::debug;

// Python-specific Tree-sitter queries
const PYTHON_CLASS_QUERY: &str = r#"
(class_definition
  name: (identifier) @name
  body: (block) @body
)
"#;

const PYTHON_IMPORT_QUERY: &str = r#"
[
  (import_statement
    name: (dotted_name) @import_path
  )
  (import_from_statement
    module_name: (dotted_name) @import_path
  )
]
"#;

const PYTHON_DECORATOR_QUERY: &str = r#"
(decorator
  (dotted_name) @decorator_name
)
"#;

const PYTHON_CLASS_BASE_QUERY: &str = r#"
(class_definition
  superclasses: (argument_list) @bases
)
"#;

/// Python-specific God Object analyzer
pub struct PythonGodObjectAnalyzer<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> PythonGodObjectAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Analyze a Python file for God Objects
    pub fn analyze(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError("AST tree missing".to_string())
        })?;
        let language = tree.language();

        // Stage 1: Framework Detection
        let detected_frameworks = self.analyze_imports(parsed_file)?;
        debug!("Detected Python frameworks: {:?}", detected_frameworks);

        let class_query = Query::new(&language, PYTHON_CLASS_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            let name_node = mat.captures[0].node;
            let body_node = mat.captures[1].node;
            let container_node = name_node.parent().unwrap_or(name_node);

            let name = name_node.utf8_text(source).unwrap_or("Unnamed");

            // Calculate metrics
            let method_count = MetricsCalculator::calculate_method_count(parsed_file, body_node)?;
            let field_count = MetricsCalculator::calculate_field_count(parsed_file, body_node)?;

            let method_threshold = self.config.get_method_threshold(parsed_file.language);
            let field_threshold = self.config.get_field_threshold(parsed_file.language);

            debug!(
                "Analyzing Python class {}: {} methods, {} fields (thresholds: >{}, >{})",
                name, method_count, field_count, method_threshold, field_threshold
            );

            // Only proceed if thresholds are exceeded
            if method_count <= method_threshold && field_count <= field_threshold {
                continue;
            }

            // Stage 3: Pattern Recognition
            let excluded_pattern = if self.config.recognize_patterns {
                self.detect_python_patterns(
                    parsed_file,
                    container_node,
                    name,
                    method_count,
                    field_count,
                    &detected_frameworks,
                )
            } else {
                None
            };

            // Stage 4: Create metrics and issue
            let mut metrics = MetricsCalculator::calculate_metrics(parsed_file, container_node)?;
            metrics.method_count = method_count;
            metrics.field_count = field_count;

            if let Some(issue) = self.create_issue(
                parsed_file,
                name,
                name_node,
                container_node,
                &metrics,
                excluded_pattern,
            ) {
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    /// Analyze imports to detect framework usage
    fn analyze_imports(&self, parsed_file: &ParsedFile) -> Result<HashSet<String>, AnalysisError> {
        let mut detected_frameworks = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("import analysis"))?;
        let language = tree.language();

        let query = Query::new(&language, PYTHON_IMPORT_QUERY)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(import_text) = capture.node.utf8_text(source) {
                    let module_name = import_text
                        .split('.')
                        .next()
                        .unwrap_or(import_text)
                        .to_string();
                    if self.config.framework_modules.contains(&module_name) {
                        detected_frameworks.insert(module_name);
                    }
                }
            }
        }

        Ok(detected_frameworks)
    }

    /// Detect Python-specific patterns
    fn detect_python_patterns(
        &self,
        _parsed_file: &ParsedFile,
        _container_node: Node,
        name: &str,
        method_count: usize,
        field_count: usize,
        detected_frameworks: &HashSet<String>,
    ) -> Option<DetectedPattern> {
        // Check for DTO pattern based on field-to-method ratio
        if field_count > 0 {
            let field_ratio = field_count as f64 / (field_count + method_count) as f64;
            if field_ratio > 0.7 {
                // Check for Python DTO frameworks
                for framework in detected_frameworks {
                    if ["pydantic", "dataclass"]
                        .iter()
                        .any(|f| framework.contains(f))
                    {
                        return Some(DetectedPattern::Dto {
                            framework: framework.clone(),
                            field_ratio,
                        });
                    }
                }

                // Check for DTO naming conventions
                if name.to_lowercase().contains("dto") {
                    return Some(DetectedPattern::Dto {
                        framework: "naming_convention".to_string(),
                        field_ratio,
                    });
                }
            }
        }

        // Check for framework controller patterns
        for framework in detected_frameworks {
            if ["django", "flask", "fastapi"].contains(&framework.as_str()) {
                if name.ends_with("View")
                    || name.ends_with("Controller")
                    || name.ends_with("Handler")
                {
                    return Some(DetectedPattern::FrameworkController {
                        framework: framework.clone(),
                        base_class: Some(name.to_string()),
                    });
                }
            }
        }

        None
    }

    /// Create an ArchitecturalIssue for a detected God Object
    fn create_issue(
        &self,
        parsed_file: &ParsedFile,
        name: &str,
        name_node: Node,
        container_node: Node,
        metrics: &ComplexityMetrics,
        excluded_pattern: Option<DetectedPattern>,
    ) -> Option<ArchitecturalIssue> {
        // If excluded by pattern recognition, return None
        if excluded_pattern.is_some() {
            debug!(
                "Excluding '{}' due to detected pattern: {:?}",
                name, excluded_pattern
            );
            return None;
        }

        let method_threshold = self.config.get_method_threshold(parsed_file.language);
        let field_threshold = self.config.get_field_threshold(parsed_file.language);

        let method_excess = metrics.method_count.saturating_sub(method_threshold);
        let field_excess = metrics.field_count.saturating_sub(field_threshold);

        // Only consider it an issue if at least one threshold is exceeded
        if method_excess == 0 && field_excess == 0 {
            return None;
        }

        let total_excess = method_excess + field_excess;
        let severity = match total_excess {
            0..=4 => "Medium",
            5..=10 => "High",
            _ => "Critical",
        };

        let mut description = format!(
            "God Object detected: '{}' has {} methods and {} fields. (Thresholds: methods>{}, fields>{})",
            name, metrics.method_count, metrics.field_count, method_threshold, field_threshold
        );

        if let Some(lcom4) = metrics.lcom4_score {
            description.push_str(&format!(
                " LCOM4 score: {} (>1 indicates low cohesion)",
                lcom4
            ));
        }

        let mut issue = ArchitecturalIssue::new(
            0, // analysis_run_id will be set by the engine
            1, // anti_pattern_type_id for God Object
            parsed_file.file_path.display().to_string(),
            Some((name_node.start_position().row + 1) as i32),
            description.clone(),
            "GodObjectDetector".to_string(),
            severity.to_string(),
            description.clone(),
        );

        issue.start_line = Some((name_node.start_position().row + 1) as i32);
        issue.end_line = Some((name_node.end_position().row + 1) as i32);
        issue.code_snippet = Some(
            container_node
                .utf8_text(parsed_file.source.as_bytes())
                .unwrap_or("")
                .to_string(),
        );

        Some(issue)
    }
}
