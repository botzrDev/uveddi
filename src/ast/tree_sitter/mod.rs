use tree_sitter::{Parser, Tree};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod queries;


/// Multi-language AST parser for Rust, Python, and JavaScript/TypeScript using tree-sitter.
/// - Caches ASTs in-memory for performance.
/// - To add new languages, implement dynamic grammar loading (see TODO).
/// - Used for all dependency extraction and anti-pattern detection in Sprint 2.
pub struct AstParser {
    rust_parser: Parser,
    python_parser: Parser,
    javascript_parser: Parser,
    cache: Mutex<HashMap<String, ParsedFile>>, // AST cache by file path
}

impl AstParser {
    /// Initialize parsers for supported languages (ER-F-002)
    pub fn new() -> Result<Self, AstError> {
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language())?;
        
        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language())?;
        
        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(tree_sitter_javascript::language())?;
        
        Ok(AstParser {
            rust_parser,
            python_parser,
            javascript_parser,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Parse file and extract dependencies (ER-F-003), with AST caching.
    /// Returns a parsed AST for the file, using cache if available.
    /// Errors if the file cannot be parsed or language is unsupported.
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let path_str = file_path.to_string_lossy().to_string();
        if let Some(cached) = self.cache.lock().unwrap().get(&path_str) {
            return Ok(cached.clone());
        }
        let source = std::fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        
        let parser = match language {
            SourceLanguage::Rust => &mut self.rust_parser,
            SourceLanguage::Python => &mut self.python_parser,
            SourceLanguage::JavaScript => &mut self.javascript_parser,
        };
        
        let tree = parser.parse(&source, None)
            .ok_or(AstError::ParseFailed)?;
            
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree,
            source,
        };
        self.cache.lock().unwrap().insert(path_str, parsed.clone());
        Ok(parsed)
    }

    /// Detect source language from file extension.
    /// Returns SourceLanguage or an error if unsupported.
    fn detect_language(&self, file_path: &Path) -> Result<SourceLanguage, AstError> {
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .ok_or(AstError::UnsupportedLanguage(format!("No file extension for {:?}", file_path)))?;

        match extension {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "jsx" | "ts" | "tsx" => Ok(SourceLanguage::JavaScript),
            _ => Err(AstError::UnsupportedLanguage(extension.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: std::path::PathBuf,
    pub language: SourceLanguage,
    pub tree: Tree,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),
    #[error("AST parsing failed")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Documentation:
/// - This parser supports Rust, Python, and JavaScript/TypeScript using tree-sitter.
/// - ASTs are cached in-memory for performance. Future: add disk cache if needed.
/// - To add new languages, implement dynamic grammar loading (see TODO below).
///
/// TODO: Implement dynamic grammar loading for extensibility in future sprints.
