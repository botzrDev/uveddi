//! # God Object Detector - New Context Implementation
//!
//! This detector uses the new AnalysisContext interface and pipeline integration.
//! It demonstrates migration from the legacy ParsedFile interface.

use super::config::GodObjectConfig;
use super::detector::{ComplexityMetrics, DetectedPattern};
use crate::ast::SourceLanguage;
use crate::database::models::ArchitecturalIssue;
use crate::engine::analysis::context::AnalysisContext;
use crate::engine::analysis::pipeline::{Detector, PipelineError};
use crate::engine::knowledge_graph::{KnowledgeGraph, QueryBuilder};
use crate::engine::parsing::{Symbol, SymbolKind};

/// God Object detector using the new analysis context
pub struct ContextGodObjectDetector {
    config: GodObjectConfig,
}

impl ContextGodObjectDetector {
    /// Create a new context-based God Object detector
    pub fn new(config: GodObjectConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self {
            config: GodObjectConfig::default(),
        }
    }

    /// Analyze a class/struct symbol using the analysis context
    fn analyze_symbol(
        &self,
        context: &AnalysisContext,
        symbol: &Symbol,
    ) -> Result<Option<ArchitecturalIssue>, PipelineError> {
        // Skip if not a class/struct/module symbol
        if !matches!(
            symbol.kind,
            SymbolKind::Class | SymbolKind::Struct | SymbolKind::Module
        ) {
            return Ok(None);
        }

        // Calculate complexity metrics using context
        let metrics = self.calculate_metrics_from_context(context, symbol)?;

        // Get language-specific thresholds
        let method_threshold = self
            .config
            .get_method_threshold(context.file_info.language.clone());
        let field_threshold = self
            .config
            .get_field_threshold(context.file_info.language.clone());

        // Check thresholds
        let exceeds_method_threshold = metrics.method_count > method_threshold;
        let exceeds_field_threshold = metrics.field_count > field_threshold;

        if !exceeds_method_threshold && !exceeds_field_threshold {
            return Ok(None);
        }

        // Check for exclusion patterns using knowledge graph
        if let Some(_pattern) = self.detect_exclusion_patterns(context, symbol)? {
            return Ok(None);
        }

        // Create metadata with issue_type and rule_id
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "issue_type".to_string(),
            serde_json::Value::String("God Object".to_string()),
        );
        metadata.insert(
            "rule_id".to_string(),
            serde_json::Value::String("god_object".to_string()),
        );
        metadata.insert(
            "method_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.method_count)),
        );
        metadata.insert(
            "field_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.field_count)),
        );
        metadata.insert(
            "method_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from(method_threshold)),
        );
        metadata.insert(
            "field_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from(field_threshold)),
        );

        let description = format!(
            "Class '{}' has {} methods and {} fields, exceeding thresholds (methods: {}, fields: {})",
            symbol.name, metrics.method_count, metrics.field_count, method_threshold, field_threshold
        );

        // Create the architectural issue
        let issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 0, // TODO: Get from context
            anti_pattern_type_id: 1, // TODO: Get from anti-pattern mapping
            file_path: context.file_info.path.to_string_lossy().to_string(),
            start_line: Some(symbol.line as i32),
            end_line: Some(symbol.line as i32),
            line_number: Some(symbol.line as i32),
            column_number: Some(symbol.column as i32),
            message: description.clone(),
            metadata: serde_json::to_string(&metadata).unwrap_or("{}".to_string()),
            detector_name: "GodObjectDetector".to_string(),
            created_at: chrono::Utc::now(),
            severity: "medium".to_string(),
            description,
            code_snippet: self.extract_context_snippet(context, symbol),
            ai_explanation: Some(format!(
                "Consider breaking down '{}' into smaller, more focused classes following the Single Responsibility Principle",
                symbol.name
            )),
        };

        Ok(Some(issue))
    }

    /// Calculate complexity metrics from analysis context
    fn calculate_metrics_from_context(
        &self,
        context: &AnalysisContext,
        symbol: &Symbol,
    ) -> Result<ComplexityMetrics, PipelineError> {
        let mut metrics = ComplexityMetrics::default();

        // Count methods and fields related to this symbol
        for related_symbol in &context.symbols {
            if related_symbol.parent.as_ref() == Some(&symbol.name) {
                match related_symbol.kind {
                    SymbolKind::Method | SymbolKind::Function => {
                        metrics.method_count += 1;
                        // Simple heuristic: methods with more than 10 lines are complex
                        if related_symbol.end_line.saturating_sub(related_symbol.line) > 10 {
                            metrics.complex_methods += 1;
                        } else {
                            metrics.trivial_methods += 1;
                        }
                    }
                    SymbolKind::Field | SymbolKind::Variable => {
                        metrics.field_count += 1;
                    }
                    _ => {}
                }
            }
        }

        // Use relations to count dependencies
        for relation in &context.relations {
            if relation.from == symbol.name {
                metrics.dependency_count += 1;
            }
        }

        Ok(metrics)
    }

    /// Detect exclusion patterns using the knowledge graph
    fn detect_exclusion_patterns(
        &self,
        _context: &AnalysisContext,
        _symbol: &Symbol,
    ) -> Result<Option<DetectedPattern>, PipelineError> {
        // TODO: Implement pattern detection using knowledge graph
        // For now, return None (no exclusions)
        Ok(None)
    }

    /// Extract context snippet for the issue
    fn extract_context_snippet(
        &self,
        context: &AnalysisContext,
        symbol: &Symbol,
    ) -> Option<String> {
        let lines: Vec<&str> = context.source.lines().collect();
        if symbol.line == 0 || symbol.line > lines.len() {
            return None;
        }

        let start = (symbol.line - 1).saturating_sub(2);
        let end = std::cmp::min(symbol.line + 3, lines.len());

        Some(lines[start..end].join("\n"))
    }
}

impl Detector for ContextGodObjectDetector {
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError> {
        let mut issues = Vec::new();

        // Analyze each symbol that could be a God Object
        for symbol in &context.symbols {
            if let Some(issue) = self.analyze_symbol(context, symbol)? {
                issues.push(issue);
            }
        }

        Ok(issues)
    }

    fn name(&self) -> &str {
        "ContextGodObjectDetector"
    }

    fn supports_language(&self, language: &SourceLanguage) -> bool {
        matches!(
            language,
            SourceLanguage::Rust
                | SourceLanguage::Python
                | SourceLanguage::JavaScript
                | SourceLanguage::TypeScript
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analysis::context::{FileInfo, ProjectContext};
    use crate::engine::parsing::{Relation, RelationKind, Symbol, SymbolKind};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_context_god_object_detector() {
        let detector = ContextGodObjectDetector::default();

        // Create a test context with a large class
        let mut symbols = Vec::new();

        // Main class symbol
        let main_class = Symbol {
            name: "LargeClass".to_string(),
            kind: SymbolKind::Class,
            line: 10,
            column: 0,
            end_line: 100,
            end_column: 1,
            parent: None,
        };
        symbols.push(main_class);

        // Add many methods to exceed threshold
        for i in 1..=8 {
            symbols.push(Symbol {
                name: format!("method{}", i),
                kind: SymbolKind::Method,
                line: 10 + i,
                column: 4,
                end_line: 15 + i,
                end_column: 5,
                parent: Some("LargeClass".to_string()),
            });
        }

        // Add many fields to exceed threshold
        for i in 1..=10 {
            symbols.push(Symbol {
                name: format!("field{}", i),
                kind: SymbolKind::Field,
                line: 5 + i,
                column: 4,
                end_line: 5 + i,
                end_column: 20,
                parent: Some("LargeClass".to_string()),
            });
        }

        let file_info = FileInfo {
            path: PathBuf::from("test.rs"),
            language: SourceLanguage::Rust,
            lines_of_code: 100,
            size_bytes: 2000,
            modified_at: SystemTime::now(),
        };

        let project_context = ProjectContext {
            project_root: PathBuf::from("/test"),
            project_files: vec![],
            dependencies: vec![],
            global_symbols: vec![],
        };

        let context = AnalysisContext::new(
            file_info,
            None, // No syntax tree for this test
            "class LargeClass { /* ... */ }".to_string(),
            symbols,
            vec![], // No relations for this test
            project_context,
        );

        let issues = detector.detect(&context).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("LargeClass"));
    }
}
