//! # AST Builder
//!
//! Responsible for coordinating language-specific parsers and building ASTs.
//! Provides a unified interface for parsing files of different languages.
//!
//! Migrated from tree_sitter_impl.rs to implement the new engine architecture.

use super::{LanguageParser, ParseError, Relation, Symbol};
use crate::ast::SourceLanguage;
use crate::engine::parsing::parsers::{
    javascript_parser::JavaScriptParser,
    python_parser::PythonParser,
    rust_parser::RustParser,
    typescript_parser::TypeScriptParser,
};
use crate::security;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

// Tree-sitter imports with feature gate
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Tree};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Parser, Tree};

/// Parse result containing the AST and metadata
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// File path that was parsed
    pub file_path: Arc<std::path::PathBuf>,
    /// Detected language
    pub language: SourceLanguage,
    /// Parsed tree (optional for compatibility)
    pub tree: Option<Tree>,
    /// Source code content
    pub source: Arc<String>,
    /// Extracted symbols
    pub symbols: Vec<Symbol>,
    /// Relations between symbols
    pub relations: Vec<Relation>,
    /// File modification time
    pub modified_at: std::time::SystemTime,
}

impl ParseResult {
    /// Get the source code as a string slice
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// Create a new parse result
    pub fn new(
        file_path: std::path::PathBuf,
        language: SourceLanguage,
        tree: Option<Tree>,
        source: String,
        symbols: Vec<Symbol>,
        relations: Vec<Relation>,
        modified_at: std::time::SystemTime,
    ) -> Self {
        Self {
            file_path: Arc::new(file_path),
            language,
            tree,
            source: Arc::new(source),
            symbols,
            relations,
            modified_at,
        }
    }
}

/// Coordinates language-specific parsers
pub struct AstBuilder {
    /// Map of language parsers
    parsers: HashMap<SourceLanguage, Box<dyn LanguageParser>>,
}

impl AstBuilder {
    /// Create a new AST builder with default parsers
    pub fn new() -> Result<Self, ParseError> {
        let mut parsers: HashMap<SourceLanguage, Box<dyn LanguageParser>> = HashMap::new();

        // Initialize language parsers with feature gates
        #[cfg(feature = "rust-lang")]
        {
            match RustParser::new() {
                Ok(parser) => {
                    parsers.insert(SourceLanguage::Rust, Box::new(parser));
                }
                Err(e) => {
                    warn!("Failed to initialize Rust parser: {}", e);
                }
            }
        }

        #[cfg(feature = "python-lang")]
        {
            match PythonParser::new() {
                Ok(parser) => {
                    parsers.insert(SourceLanguage::Python, Box::new(parser));
                }
                Err(e) => {
                    warn!("Failed to initialize Python parser: {}", e);
                }
            }
        }

        #[cfg(feature = "javascript-lang")]
        {
            match JavaScriptParser::new() {
                Ok(parser) => {
                    parsers.insert(SourceLanguage::JavaScript, Box::new(parser));
                }
                Err(e) => {
                    warn!("Failed to initialize JavaScript parser: {}", e);
                }
            }
        }

        #[cfg(feature = "typescript-lang")]
        {
            match TypeScriptParser::new() {
                Ok(parser) => {
                    parsers.insert(SourceLanguage::TypeScript, Box::new(parser));
                }
                Err(e) => {
                    warn!("Failed to initialize TypeScript parser: {}", e);
                }
            }
        }

        info!(
            "Initialized AST builder with {} language parsers",
            parsers.len()
        );

        Ok(Self { parsers })
    }

    /// Parse a file into an AST
    pub fn parse_file(&self, file_path: &Path) -> Result<ParseResult, ParseError> {
        // Security validation
        security::validate_file_size(file_path)
            .map_err(|e| ParseError::ParseFailed(format!("Security validation failed: {}", e)))?;
        security::validate_file_type(file_path)
            .map_err(|e| ParseError::ParseFailed(format!("Security validation failed: {}", e)))?;

        // Get file modification time
        let modified_time = std::fs::metadata(file_path)
            .and_then(|meta| meta.modified())
            .map_err(|e| ParseError::ParseFailed(format!("Failed to get file metadata: {}", e)))?;

        // Read source safely
        let source = self.read_source_safely(file_path)?;

        // Detect language
        let language = self.detect_language(file_path)?;

        // Get appropriate parser
        let parser = self
            .parsers
            .get(&language)
            .ok_or_else(|| {
                ParseError::UnsupportedLanguage(language)
            })?;

        // Parse the source
        let tree = parser.parse(&source)?;

        // Extract symbols and relations
        let symbols = parser.extract_symbols(&tree, &source);
        let relations = parser.build_relations(&tree, &source);

        Ok(ParseResult::new(
            file_path.to_path_buf(),
            language,
            Some(tree),
            source,
            symbols,
            relations,
            modified_time,
        ))
    }

    /// Parse source code with explicit language
    pub fn parse_source(
        &self,
        source: &str,
        language: SourceLanguage,
        file_path: Option<&Path>,
    ) -> Result<ParseResult, ParseError> {
        let parser = self
            .parsers
            .get(&language)
            .ok_or_else(|| ParseError::UnsupportedLanguage(language))?;

        let tree = parser.parse(source)?;
        let symbols = parser.extract_symbols(&tree, source);
        let relations = parser.build_relations(&tree, source);

        let path = file_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("<inline>"));

        Ok(ParseResult::new(
            path,
            language,
            Some(tree),
            source.to_string(),
            symbols,
            relations,
            std::time::SystemTime::now(),
        ))
    }

    /// Register a custom language parser
    pub fn register_parser(&mut self, parser: Box<dyn LanguageParser>) {
        let language = parser.language();
        self.parsers.insert(language, parser);
        info!("Registered custom parser for {:?}", language);
    }

    /// Get available languages
    pub fn available_languages(&self) -> Vec<SourceLanguage> {
        self.parsers.keys().copied().collect()
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: &SourceLanguage) -> bool {
        self.parsers.contains_key(language)
    }

    /// Safely read source code with robust UTF-8 handling
    fn read_source_safely(&self, file_path: &Path) -> Result<String, ParseError> {
        let source_bytes = std::fs::read(file_path)
            .map_err(|e| ParseError::ParseFailed(format!("Failed to read file: {}", e)))?;

        match String::from_utf8(source_bytes.clone()) {
            Ok(valid_string) => Ok(valid_string),
            Err(utf8_error) => {
                let file_display = file_path.display();
                let error_pos = utf8_error.utf8_error().valid_up_to();

                warn!(
                    "Invalid UTF-8 found in file '{}' at byte position {}. Using lossy conversion.",
                    file_display, error_pos
                );

                let lossy_string = String::from_utf8_lossy(&source_bytes);
                Ok(lossy_string.into_owned())
            }
        }
    }

    /// Detect language from file extension
    fn detect_language(&self, file_path: &Path) -> Result<SourceLanguage, ParseError> {
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match ext {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "jsx" => Ok(SourceLanguage::JavaScript),
            "ts" | "tsx" => Ok(SourceLanguage::TypeScript),
            _ => Err(ParseError::ParseFailed(format!(
                "Unsupported file extension: {}",
                ext
            ))),
        }
    }
}