//! Rust pattern detection

use super::super::super::config::GodObjectConfig;
use super::super::super::detector::DetectedPattern;
use super::queries::{RUST_DERIVE_QUERY, RUST_USE_QUERY};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::error::ErrorHelpers;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;

/// Rust pattern detector
pub struct RustPatternDetector<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> RustPatternDetector<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Analyze imports to detect framework usage
    pub fn analyze_imports(&self, parsed_file: &ParsedFile) -> Result<Vec<String>, AnalysisError> {
        let mut detected_frameworks = Vec::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("import analysis"))?;
        let language = tree.language();

        let query = Query::new(&language, RUST_USE_QUERY)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(import_text) = capture.node.utf8_text(source) {
                    let module_name = import_text
                        .split("::")
                        .next()
                        .unwrap_or(import_text)
                        .to_string();
                    if self.config.framework_modules.contains(&module_name) {
                        detected_frameworks.push(module_name);
                    }
                }
            }
        }

        Ok(detected_frameworks)
    }

    /// Analyze derive macros for DTO pattern detection
    pub fn analyze_derive_macros(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashMap<String, Vec<String>>, AnalysisError> {
        let mut derive_attributes: HashMap<String, Vec<String>> = HashMap::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("derive analysis"))?;
        let language = tree.language();

        let derive_query = Query::new(&language, RUST_DERIVE_QUERY)
            .map_err(|e| AnalysisError::QueryError(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&derive_query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(attr_text) = capture.node.utf8_text(source) {
                    if attr_text.contains("Serialize") || attr_text.contains("Deserialize") {
                        // Find the associated struct - simplified approach
                        if let Some(parent) = capture.node.parent() {
                            if let Some(struct_node) = parent.next_sibling() {
                                if let Ok(struct_text) = struct_node.utf8_text(source) {
                                    if struct_text.starts_with("struct") {
                                        let struct_name = "derived_struct".to_string(); // Simplified
                                        derive_attributes
                                            .entry(struct_name)
                                            .or_insert_with(Vec::new)
                                            .push(attr_text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(derive_attributes)
    }

    /// Detect Rust-specific patterns
    pub fn detect_patterns(
        &self,
        name: &str,
        method_count: usize,
        field_count: usize,
        detected_frameworks: &[String],
        derive_attributes: &HashMap<String, Vec<String>>,
    ) -> Option<DetectedPattern> {
        // Check for Serde DTO pattern
        if derive_attributes.contains_key(name) {
            let field_ratio = field_count as f64 / (field_count + method_count) as f64;
            if field_ratio > 0.7 {
                return Some(DetectedPattern::Dto {
                    framework: "serde".to_string(),
                    field_ratio,
                });
            }
        }

        // Check for Builder pattern
        if name.ends_with("Builder") && method_count >= 3 {
            return Some(DetectedPattern::Builder {
                builder_methods: vec!["build_method".to_string()], // Simplified
                build_method: Some("build".to_string()),
            });
        }

        // Check for framework patterns
        for framework in detected_frameworks {
            if ["axum", "rocket", "actix_web", "diesel", "sqlx"].contains(&framework.as_str()) {
                return Some(DetectedPattern::FrameworkController {
                    framework: framework.clone(),
                    base_class: None,
                });
            }
        }

        None
    }
}
