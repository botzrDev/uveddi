//! TypeScript pattern detection

use super::super::super::config::GodObjectConfig;
use super::super::super::detector::DetectedPattern;
use super::queries::TYPESCRIPT_IMPORT_QUERY;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::ParsedFile;
use crate::error::ErrorHelpers;
use std::collections::HashSet;
use streaming_iterator::StreamingIterator;

/// TypeScript pattern detector
pub struct TypeScriptPatternDetector<'a> {
    config: &'a GodObjectConfig,
}

impl<'a> TypeScriptPatternDetector<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self { config }
    }

    /// Analyze imports to detect framework usage
    pub fn analyze_imports(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let mut detected_frameworks = HashSet::new();
        let source = parsed_file.source.as_bytes();
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| ErrorHelpers::ast_error("import analysis"))?;
        let language = tree.language();

        let query = Query::new(&language, TYPESCRIPT_IMPORT_QUERY)
            .map_err(|e| ErrorHelpers::query_error(&e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source);

        while let Some(mat) = matches.next() {
            for capture in mat.captures {
                if let Ok(import_text) = capture.node.utf8_text(source) {
                    let module_name = self.extract_module_name(import_text);
                    if self.config.framework_modules.contains(&module_name) {
                        detected_frameworks.insert(module_name);
                    }
                }
            }
        }

        Ok(detected_frameworks)
    }

    /// Extract module name from import path
    fn extract_module_name(&self, import_text: &str) -> String {
        if import_text.starts_with('"') || import_text.starts_with('\'') {
            let path = &import_text[1..import_text.len() - 1];
            if path.starts_with("./") || path.starts_with("../") {
                return "local".to_string();
            }
            path.split('/').next().unwrap_or(path).to_string()
        } else {
            import_text.to_string()
        }
    }

    /// Detect TypeScript-specific patterns
    pub fn detect_patterns(
        &self,
        name: &str,
        method_count: usize,
        field_count: usize,
        detected_frameworks: &HashSet<String>,
    ) -> Option<DetectedPattern> {
        // Check for Angular Component/Service patterns
        if detected_frameworks.contains("angular") {
            if name.ends_with("Component") || name.ends_with("Service") || name.ends_with("Module")
            {
                return Some(DetectedPattern::FrameworkController {
                    framework: "angular".to_string(),
                    base_class: Some(name.to_string()),
                });
            }
        }

        // Check for NestJS Controller/Service patterns
        if detected_frameworks.contains("nest") || detected_frameworks.contains("nestjs") {
            if name.ends_with("Controller") || name.ends_with("Service") || name.ends_with("Module")
            {
                return Some(DetectedPattern::FrameworkController {
                    framework: "nestjs".to_string(),
                    base_class: Some(name.to_string()),
                });
            }
        }

        // Check for DTO pattern with TypeScript interfaces
        if field_count > 0 {
            let field_ratio = field_count as f64 / (field_count + method_count) as f64;
            if field_ratio > 0.8 {
                // Stricter for TypeScript due to type safety
                for framework in detected_frameworks {
                    if ["class-validator", "class-transformer", "nestjs"]
                        .iter()
                        .any(|f| framework.contains(f))
                    {
                        return Some(DetectedPattern::Dto {
                            framework: framework.clone(),
                            field_ratio,
                        });
                    }
                }

                // Check for TypeScript DTO naming conventions
                if name.to_lowercase().contains("dto") {
                    return Some(DetectedPattern::Dto {
                        framework: "typescript_naming".to_string(),
                        field_ratio,
                    });
                }
            }
        }

        // Check for Builder pattern
        if name.ends_with("Builder") && method_count >= 3 {
            return Some(DetectedPattern::Builder {
                builder_methods: vec!["build_method".to_string()], // Simplified
                build_method: Some("build".to_string()),
            });
        }

        None
    }
}
