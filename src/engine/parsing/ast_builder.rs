//! # AST Builder
//!
//! Responsible for coordinating language-specific parsers and building ASTs.
//! Provides a unified interface for parsing files of different languages.

use super::{LanguageParser, ParseError, ParseResult};
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use std::path::Path;

// Tree-sitter imports with feature gate
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::Tree;

/// Parse result containing the AST and metadata
#[derive(Debug)]
pub struct ParseResult {
    pub language: SourceLanguage,
    pub tree: Option<Tree>,
    pub source: String,
    pub file_path: std::path::PathBuf,
}

/// Coordinates language-specific parsers
pub struct AstBuilder {
    /// Map of language parsers
    parsers: HashMap<SourceLanguage, Box<dyn LanguageParser>>,
}

impl AstBuilder {
    /// Create a new AST builder with default parsers
    pub fn new() -> Result<Self, ParseError> {
        // TODO: Initialize language parsers
        // This will be populated when we migrate parser implementations
        Ok(Self {
            parsers: HashMap::new(),
        })
    }

    /// Parse a file into an AST
    pub fn parse_file(&self, file_path: &Path) -> Result<ParseResult, ParseError> {
        // TODO: Implement file parsing logic
        // 1. Detect language
        // 2. Read file contents
        // 3. Select appropriate parser
        // 4. Parse and return result

        Err(ParseError::InvalidSource)
    }

    /// Parse source code with explicit language
    pub fn parse_source(
        &self,
        source: &str,
        language: SourceLanguage,
    ) -> Result<ParseResult, ParseError> {
        // TODO: Implement direct source parsing
        // 1. Get parser for language
        // 2. Parse source
        // 3. Return result

        Err(ParseError::UnsupportedLanguage(language))
    }

    /// Register a custom language parser
    pub fn register_parser(&mut self, parser: Box<dyn LanguageParser>) {
        self.parsers.insert(parser.language(), parser);
    }
}