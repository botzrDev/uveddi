//! # Dead Code Detector - New Context Implementation
//!
//! This detector uses the new AnalysisContext interface for dead code detection.

use crate::ast::SourceLanguage;
use crate::database::models::ArchitecturalIssue;
use crate::engine::analysis::context::AnalysisContext;
use crate::engine::analysis::pipeline::{Detector, PipelineError};
use crate::engine::knowledge_graph::{KnowledgeGraph, QueryBuilder};
use crate::engine::parsing::{RelationKind, Symbol, SymbolKind};
use std::collections::{HashMap, HashSet};

/// Dead code detector using analysis context and knowledge graph
pub struct ContextDeadCodeDetector {
    /// Confidence threshold for dead code detection (0.0 to 1.0)
    confidence_threshold: f64,
    /// Whether to include private symbols in analysis
    include_private: bool,
}

impl ContextDeadCodeDetector {
    /// Create a new context-based dead code detector
    pub fn new(confidence_threshold: f64, include_private: bool) -> Self {
        Self {
            confidence_threshold,
            include_private,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(0.8, true)
    }

    /// Find symbols that appear to be unused
    fn find_unused_symbols(&self, context: &AnalysisContext) -> Vec<UnusedSymbol> {
        let mut unused_symbols = Vec::new();

        // Build usage map from relations
        let usage_map = self.build_usage_map(context);

        // Analyze each symbol
        for symbol in &context.symbols {
            if self.should_analyze_symbol(symbol) {
                let usage_count = usage_map.get(&symbol.name).unwrap_or(&0);
                let confidence = self.calculate_dead_code_confidence(symbol, *usage_count, context);

                if confidence >= self.confidence_threshold {
                    unused_symbols.push(UnusedSymbol {
                        symbol: symbol.clone(),
                        confidence,
                        usage_count: *usage_count,
                        reason: self.determine_dead_code_reason(symbol, *usage_count),
                    });
                }
            }
        }

        unused_symbols
    }

    /// Build a map of symbol usage counts
    fn build_usage_map(&self, context: &AnalysisContext) -> HashMap<String, usize> {
        let mut usage_map = HashMap::new();

        // Count direct usages from relations
        for relation in &context.relations {
            if matches!(relation.kind, RelationKind::Uses | RelationKind::Calls) {
                *usage_map.entry(relation.to.clone()).or_insert(0) += 1;
            }
        }

        // Also check for symbol references in source code (simple text matching)
        for symbol in &context.symbols {
            if matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Method | SymbolKind::Variable
            ) {
                let occurrences = self.count_symbol_occurrences(&context.source, &symbol.name);
                // Subtract 1 for the definition itself
                let usage_count = occurrences.saturating_sub(1);

                let current_count = usage_map.get(&symbol.name).unwrap_or(&0);
                usage_map.insert(
                    symbol.name.clone(),
                    std::cmp::max(*current_count, usage_count),
                );
            }
        }

        usage_map
    }

    /// Count occurrences of a symbol name in source code
    fn count_symbol_occurrences(&self, source: &str, symbol_name: &str) -> usize {
        // Simple word boundary matching - could be improved with proper tokenization
        let mut count = 0;
        let mut start = 0;

        while let Some(pos) = source[start..].find(symbol_name) {
            let absolute_pos = start + pos;

            // Check word boundaries
            let before_char = if absolute_pos > 0 {
                source.chars().nth(absolute_pos - 1)
            } else {
                Some(' ')
            };

            let after_pos = absolute_pos + symbol_name.len();
            let after_char = source.chars().nth(after_pos);

            let is_word_boundary = |c: Option<char>| match c {
                Some(ch) => !ch.is_alphanumeric() && ch != '_',
                None => true,
            };

            if is_word_boundary(before_char) && is_word_boundary(after_char) {
                count += 1;
            }

            start = absolute_pos + 1;
        }

        count
    }

    /// Check if a symbol should be analyzed for dead code
    fn should_analyze_symbol(&self, symbol: &Symbol) -> bool {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                // Skip main functions and entry points
                if symbol.name == "main" || symbol.name.starts_with("test_") {
                    return false;
                }
                true
            }
            SymbolKind::Variable | SymbolKind::Field => {
                // Include private variables if configured
                self.include_private || !self.is_private_symbol(symbol)
            }
            SymbolKind::Class | SymbolKind::Struct => {
                // Only analyze if configured to include private symbols
                self.include_private || !self.is_private_symbol(symbol)
            }
            SymbolKind::Module => false, // Modules are rarely completely unused
            _ => false,
        }
    }

    /// Simple heuristic to determine if a symbol is private
    fn is_private_symbol(&self, symbol: &Symbol) -> bool {
        // This is language-specific and simplified
        symbol.name.starts_with('_') || symbol.name.starts_with("private")
    }

    /// Calculate confidence that a symbol is dead code
    fn calculate_dead_code_confidence(
        &self,
        symbol: &Symbol,
        usage_count: usize,
        context: &AnalysisContext,
    ) -> f64 {
        let mut confidence: f64 = match usage_count {
            0 => 0.9,     // High confidence for truly unused symbols
            1 => 0.6,     // Medium confidence for barely used symbols
            2..=3 => 0.3, // Low confidence for occasionally used symbols
            _ => 0.0,     // No confidence for frequently used symbols
        };

        // Adjust confidence based on symbol type
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                // Functions with no usages are likely dead
                if usage_count == 0 {
                    confidence = 0.95;
                }
            }
            SymbolKind::Variable => {
                // Variables should definitely be used
                if usage_count == 0 {
                    confidence = 0.85;
                }
            }
            _ => {}
        }

        // Reduce confidence for public APIs (heuristic)
        if symbol.name.starts_with("pub_") || symbol.name == "new" {
            confidence *= 0.5;
        }

        // Reduce confidence for exports in JavaScript/TypeScript
        if matches!(
            context.file_info.language,
            SourceLanguage::JavaScript | SourceLanguage::TypeScript
        ) {
            if context.source.contains(&format!("export {}", symbol.name)) {
                confidence *= 0.3;
            }
        }

        confidence.clamp(0.0_f64, 1.0_f64)
    }

    /// Determine the reason for dead code classification
    fn determine_dead_code_reason(&self, symbol: &Symbol, usage_count: usize) -> String {
        match usage_count {
            0 => format!(
                "{} '{}' is never used",
                symbol.kind.description(),
                symbol.name
            ),
            1 => format!(
                "{} '{}' is only used once and may be unnecessary",
                symbol.kind.description(),
                symbol.name
            ),
            _ => format!(
                "{} '{}' has limited usage ({})",
                symbol.kind.description(),
                symbol.name,
                usage_count
            ),
        }
    }
}

impl Detector for ContextDeadCodeDetector {
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError> {
        let mut issues = Vec::new();

        // Find unused symbols
        let unused_symbols = self.find_unused_symbols(context);

        // Create issues for each unused symbol
        for unused in unused_symbols {
            // Create metadata with issue_type and rule_id
            let mut metadata = serde_json::Map::new();
            metadata.insert("issue_type".to_string(), serde_json::Value::String("Dead Code".to_string()));
            metadata.insert("rule_id".to_string(), serde_json::Value::String("dead_code".to_string()));
            metadata.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(unused.confidence).unwrap_or(serde_json::Number::from(0))));
            metadata.insert("usage_count".to_string(), serde_json::Value::Number(serde_json::Number::from(unused.usage_count)));
            metadata.insert("symbol_kind".to_string(), serde_json::Value::String(unused.symbol.kind.description().to_string()));

            let description = format!(
                "{} (confidence: {:.1}%, usage count: {})",
                unused.reason,
                unused.confidence * 100.0,
                unused.usage_count
            );

            let issue = ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0, // TODO: Get from context
                anti_pattern_type_id: 1, // TODO: Get from anti-pattern mapping
                file_path: context.file_info.path.to_string_lossy().to_string(),
                start_line: Some(unused.symbol.line as i32),
                end_line: Some(unused.symbol.line as i32),
                line_number: Some(unused.symbol.line as i32),
                column_number: Some(unused.symbol.column as i32),
                message: description.clone(),
                metadata: serde_json::to_string(&metadata).unwrap_or("{}".to_string()),
                detector_name: "DeadCodeDetector".to_string(),
                created_at: chrono::Utc::now(),
                severity: if unused.confidence > 0.85 {
                    "high".to_string()
                } else if unused.confidence > 0.6 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                },
                description,
                code_snippet: self.extract_context_snippet(context, &unused.symbol),
                ai_explanation: Some(format!(
                    "Consider removing {} '{}' if it is truly unused, or mark it with appropriate annotations if it's intentionally unused",
                    unused.symbol.kind.description(),
                    unused.symbol.name
                )),
            };

            issues.push(issue);
        }

        Ok(issues)
    }

    fn name(&self) -> &str {
        "ContextDeadCodeDetector"
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

impl ContextDeadCodeDetector {
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

        let start = (symbol.line - 1).saturating_sub(1);
        let end = std::cmp::min(symbol.end_line + 1, lines.len());

        Some(lines[start..end].join("\n"))
    }
}

/// Represents a potentially unused symbol
#[derive(Debug)]
struct UnusedSymbol {
    symbol: Symbol,
    confidence: f64,
    usage_count: usize,
    reason: String,
}

impl SymbolKind {
    fn description(&self) -> &'static str {
        match self {
            SymbolKind::Function => "Function",
            SymbolKind::Method => "Method",
            SymbolKind::Variable => "Variable",
            SymbolKind::Field => "Field",
            SymbolKind::Class => "Class",
            SymbolKind::Struct => "Struct",
            SymbolKind::Module => "Module",
            SymbolKind::Interface => "Interface",
            SymbolKind::Trait => "Trait",
            SymbolKind::Enum => "Enum",
        }
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
    fn test_context_dead_code_detector() {
        let detector = ContextDeadCodeDetector::default();

        let source_code = r#"
fn used_function() {
    println!("This is used");
}

fn unused_function() {
    println!("This is not used");
}

fn main() {
    used_function();
}
"#;

        let symbols = vec![
            Symbol {
                name: "used_function".to_string(),
                kind: SymbolKind::Function,
                line: 2,
                column: 0,
                end_line: 4,
                end_column: 1,
                parent: None,
            },
            Symbol {
                name: "unused_function".to_string(),
                kind: SymbolKind::Function,
                line: 6,
                column: 0,
                end_line: 8,
                end_column: 1,
                parent: None,
            },
            Symbol {
                name: "main".to_string(),
                kind: SymbolKind::Function,
                line: 10,
                column: 0,
                end_line: 12,
                end_column: 1,
                parent: None,
            },
        ];

        let relations = vec![Relation {
            from: "main".to_string(),
            to: "used_function".to_string(),
            kind: RelationKind::Calls,
        }];

        let file_info = FileInfo {
            path: PathBuf::from("test.rs"),
            language: SourceLanguage::Rust,
            lines_of_code: source_code.lines().count(),
            size_bytes: source_code.len(),
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
            None,
            source_code.to_string(),
            symbols,
            relations,
            project_context,
        );

        let issues = detector.detect(&context).unwrap();

        // Should find unused_function but not used_function or main
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("unused_function"));
        assert!(issues[0].description.contains("never used"));
    }
}
