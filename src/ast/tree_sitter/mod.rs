use tree_sitter::{Parser, Tree};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use bincode;
use std::fs;
use std::io::{Read, Write};

pub mod queries;

const CACHE_DIR: &str = ".codeatlas_cache";

/// Multi-language AST parser for Rust, Python, and JavaScript/TypeScript using tree-sitter.
/// - Caches ASTs in-memory for performance.
/// - To add new languages, implement dynamic grammar loading (see TODO).
/// - Used for all dependency extraction and anti-pattern detection in Sprint 2.
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>,
    cache: Mutex<HashMap<String, ParsedFile>>, // AST cache by file path
}

impl AstParser {
    /// Initialize parsers for supported languages (ER-F-002)
    pub fn new() -> Result<Self, AstError> {
        let mut parsers = HashMap::new();
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language())?;
        parsers.insert(SourceLanguage::Rust, rust_parser);

        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language())?;
        parsers.insert(SourceLanguage::Python, python_parser);

        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(tree_sitter_javascript::language())?;
        parsers.insert(SourceLanguage::JavaScript, javascript_parser);

        Ok(AstParser {
            parsers,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Dynamically add a new language parser at runtime
    pub fn add_language(&mut self, lang: SourceLanguage, parser: Parser) {
        self.parsers.insert(lang, parser);
    }

    /// Parse file and extract dependencies (ER-F-003), with AST caching.
    /// Returns a parsed AST for the file, using cache if available.
    /// Errors if the file cannot be parsed or language is unsupported.
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let path_str = file_path.to_string_lossy().to_string();
        if let Some(cached) = self.cache.lock().unwrap().get(&path_str) {
            return Ok(cached.clone());
        }
        // Try disk cache
        let cache_path = ParsedFile::cache_path(file_path);
        if let Ok(mut f) = fs::File::open(&cache_path) {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).ok();
            if let Ok(mut parsed) = bincode::deserialize::<ParsedFile>(&buf) {
                // Re-parse tree from source
                let language = self.detect_language(file_path)?;
                let parser = self.parsers.get_mut(&language)
                    .ok_or_else(|| AstError::UnsupportedLanguage(format!("{:?}", language)))?;
                if let Some(tree) = parser.parse(&parsed.source, None) {
                    parsed.tree = Some(tree);
                    self.cache.lock().unwrap().insert(path_str.clone(), parsed.clone());
                    return Ok(parsed);
                }
            }
        }
        // Parse and cache
        let source = fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        let parser = self.parsers.get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{:?}", language)))?;
        let tree = parser.parse(&source, None)
            .ok_or(AstError::ParseFailed)?;
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree: Some(tree),
            source: source.clone(),
        };
        // Save to disk
        let mut disk_parsed = parsed.clone();
        disk_parsed.tree = None; // Tree can't be serialized, skip
        let encoded = bincode::serialize(&disk_parsed).unwrap();
        if let Ok(mut f) = fs::File::create(&cache_path) {
            f.write_all(&encoded).ok();
        }
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

/// Represents a parsed source file and its AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    /// Path to the source file on disk.
    pub path: std::path::PathBuf,
    /// Detected programming language of the file.
    pub language: SourceLanguage,
    /// The parsed tree-sitter AST. Not serialized.
    #[serde(skip)]
    pub tree: Option<Tree>,
    /// The full source code as a string.
    pub source: String,
}

impl ParsedFile {
    pub fn cache_path(file_path: &Path) -> std::path::PathBuf {
        let mut cache_dir = std::env::current_dir().unwrap();
        cache_dir.push(CACHE_DIR);
        fs::create_dir_all(&cache_dir).ok();
        let file_hash = format!("{:x}", md5::compute(file_path.to_string_lossy().as_bytes()));
        cache_dir.push(file_hash);
        cache_dir.set_extension("bin");
        cache_dir
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

// Documentation:
// - This parser supports Rust, Python, and JavaScript/TypeScript using tree-sitter.
// - ASTs are cached in-memory for performance. Future: add disk cache if needed.
// - To add new languages, implement dynamic grammar loading (see TODO below).
//
// TODO: Implement dynamic grammar loading for extensibility in future sprints.
