//! Unused import detection

use std::collections::HashSet;

use crate::analysis::AnalysisError;
use crate::analysis::detectors::anti_patterns::dead_code::types::{
    DeadCodeIssue, Severity, Symbol, SymbolType,
};

/// Detects unused imports in code
pub struct UnusedImportAnalyzer;

impl UnusedImportAnalyzer {
    /// Analyzes symbols for unused imports
    pub fn analyze(
        symbols: &[Symbol],
        references: &HashSet<String>,
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for symbol in symbols {
            if symbol.symbol_type == SymbolType::Import && !symbol.is_live {
                if !Self::is_import_used(symbol, references) {
                    issues.push(DeadCodeIssue {
                        symbol: symbol.clone(),
                        severity: Severity::Medium,
                        removal_safe: true,
                        related_symbols: Vec::new(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Checks if an import is actually used in the code
    fn is_import_used(import: &Symbol, references: &HashSet<String>) -> bool {
        // Extract the imported name from the import statement
        let imported_name = Self::extract_imported_name(&import.name);
        references.contains(&imported_name)
    }

    /// Extracts the actual imported name from an import statement
    fn extract_imported_name(import_stmt: &str) -> String {
        // This is simplified; real implementation would parse the import properly
        if let Some(last_part) = import_stmt.split("::").last() {
            last_part.to_string()
        } else {
            import_stmt.to_string()
        }
    }

    /// Identifies different import patterns
    pub fn identify_import_pattern(symbol: &Symbol) -> ImportPattern {
        let import_text = &symbol.code_snippet;

        if import_text.contains("*") {
            ImportPattern::Wildcard
        } else if import_text.contains(" as ") {
            ImportPattern::Aliased
        } else if import_text.contains("{") && import_text.contains("}") {
            ImportPattern::Destructured
        } else {
            ImportPattern::Direct
        }
    }

    /// Checks if an import is a common utility that might be used indirectly
    pub fn is_utility_import(import_name: &str) -> bool {
        const UTILITY_IMPORTS: &[&str] = &[
            "std",
            "fmt",
            "io",
            "collections",
            "vec",
            "string",
            "debug",
            "error",
        ];

        UTILITY_IMPORTS
            .iter()
            .any(|&util| import_name.contains(util))
    }
}

/// Patterns for import statements
#[derive(Debug, Clone, PartialEq)]
pub enum ImportPattern {
    Direct,
    Aliased,
    Destructured,
    Wildcard,
}