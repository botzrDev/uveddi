//! Dead Code anti-pattern detector
//!
//! This detector implements a simplified version of the mark-and-sweep algorithm
//! described in the Dead Code Research document. It identifies potentially unused
//! functions, variables, and other symbols across multiple programming languages.
//!
//! ## Detection Strategy:
//! 1. **Symbol Collection**: Extract all function/variable definitions using Tree-sitter queries
//! 2. **Usage Analysis**: Find all references and calls to these symbols
//! 3. **Entry Point Detection**: Identify main functions, exports, and public APIs
//! 4. **Reachability Analysis**: Mark symbols as live if they're reachable from entry points
//! 5. **Dead Code Reporting**: Report symbols that remain unmarked as potentially dead
//!
//! ## Confidence Scoring:
//! - **High (0.8-1.0)**: Private/internal symbols with no references
//! - **Medium (0.5-0.7)**: Exported symbols in applications with no apparent usage
//! - **Low (0.2-0.4)**: Symbols in files with dynamic features (eval, decorators, etc.)
//!
//! ## Language Support:
//! - **Rust**: Functions, structs, enums, constants, modules
//! - **Python**: Functions, classes, variables, imports
//! - **JavaScript**: Functions, classes, variables, exports

mod analysis;
mod config;
#[cfg(feature = "engine-integration")]
pub mod context_detector;
mod detector;
mod language_support;
mod patterns;
mod types;
mod validation;

// Re-export public API
pub use config::DeadCodeConfig;
pub use detector::DeadCodeDetector;
pub use types::{DeadCodeIssue, Severity, Symbol, SymbolType};

// Re-export for testing
#[cfg(test)]
pub use analysis::{
    unreachable::UnreachableCodeAnalyzer, unused_functions::UnusedFunctionAnalyzer,
    unused_imports::UnusedImportAnalyzer, unused_variables::UnusedVariableAnalyzer,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{compatibility_shim::ParsedFileCompat, SourceLanguage};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dead_code_detection() {
        let config = DeadCodeConfig::default();
        let detector = DeadCodeDetector::new(config);

        // Create a mock parsed file
        let parsed_file = ParsedFileCompat {
            file_path: Arc::new(PathBuf::from("test.rs")),
            source: "fn unused_function() {}\nfn main() { println!(\"Hello\"); }"
                .to_string()
                .into(),
            language: SourceLanguage::Rust,
            tree: None, // Would need actual tree-sitter tree in real test
            custom_ast: Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        // This would fail without an actual AST, but shows the structure
        let result = detector.detect_dead_code(&parsed_file).await;
        assert!(result.is_err()); // Expected since we don't have a real AST
    }

    #[test]
    fn test_config_creation() {
        let config = DeadCodeConfig::default();
        assert_eq!(config.min_confidence, 0.5);
        assert!(!config.library_mode);
        assert_eq!(config.ignore_patterns.len(), 4);
        assert_eq!(config.keep_alive_patterns.len(), 4);
    }

    #[test]
    fn test_symbol_type_display() {
        assert_eq!(SymbolType::Function.to_string(), "function");
        assert_eq!(SymbolType::Class.to_string(), "class");
        assert_eq!(SymbolType::Variable.to_string(), "variable");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::High.to_string(), "High");
        assert_eq!(Severity::Medium.to_string(), "Medium");
        assert_eq!(Severity::Low.to_string(), "Low");
    }
}
