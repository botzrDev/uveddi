// Implementation file for tree-sitter enabled builds
// UV-97: Tree-sitter feature gating implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tree_sitter::{Parser, Tree};

mod arc_pathbuf_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::path::PathBuf;
    use std::sync::Arc;

    pub fn serialize<S>(arc_pathbuf: &Arc<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arc_pathbuf.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pathbuf = PathBuf::deserialize(deserializer)?;
        Ok(Arc::new(pathbuf))
    }
}

mod arc_string_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(arc_string: &Arc<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arc_string.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(Arc::new(string))
    }
}

/// Tree-sitter parser implementation (feature enabled)
pub struct AstParser {
    parsers: HashMap<SourceLanguage, Parser>,
    cache: Mutex<HashMap<String, CachedAst>>,
}

impl std::fmt::Debug for AstParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstParser")
            .field("cache", &self.cache)
            .finish()
    }
}

/// Cached AST structure
#[derive(Debug, Clone)]
pub struct CachedAst {
    pub tree: Tree,
    pub timestamp: std::time::SystemTime,
}

/// Source language enumeration
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

impl SourceLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| match ext_str {
                "rs" => Some(SourceLanguage::Rust),
                "py" => Some(SourceLanguage::Python),
                "js" | "ts" | "jsx" | "tsx" => Some(SourceLanguage::JavaScript),
                _ => None,
            })
    }
}

/// Parsed file structure containing AST and metadata

#[derive(Debug, Clone, Serialize, Deserialize)] // Add Serialize/Deserialize
pub struct ParsedFile {
    #[serde(with = "arc_pathbuf_serde")]
    pub file_path: Arc<PathBuf>, // Changed to Arc<PathBuf>
    pub language: SourceLanguage,
    #[serde(with = "arc_string_serde")]
    pub source: Arc<String>, // Renamed from content to source, and changed to Arc<String>
    #[serde(skip)]
    pub tree: Option<Tree>,
    #[serde(skip)]
    pub custom_ast: Option<CustomAst>,
}

/// Custom AST representation
#[derive(Debug, Clone)]
pub enum CustomAst {
    File {
        items: Vec<CustomAst>,
    },
    Struct {
        name: String,
        methods: Vec<String>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
    },
    Variable {
        name: String,
    },
}

/// AST error types for tree-sitter operations
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),
    #[error("Parse failed")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl AstParser {
    /// Create a new AST parser with language support
    pub fn new() -> Result<Self, AstError> {
        let mut parsers = HashMap::new();

        // Initialize Rust parser
        let mut rust_parser = tree_sitter::Parser::new();
        rust_parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(AstError::TreeSitterLanguage)?;
        parsers.insert(SourceLanguage::Rust, rust_parser);

        // Initialize Python parser
        let mut python_parser = tree_sitter::Parser::new();
        python_parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(AstError::TreeSitterLanguage)?;
        parsers.insert(SourceLanguage::Python, python_parser);

        // Initialize JavaScript parser
        let mut javascript_parser = tree_sitter::Parser::new();
        javascript_parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(AstError::TreeSitterLanguage)?;
        parsers.insert(SourceLanguage::JavaScript, javascript_parser);

        Ok(AstParser {
            parsers,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Add a language parser
    pub fn add_language(&mut self, lang: SourceLanguage, parser: Parser) {
        self.parsers.insert(lang, parser);
    }

    /// Parse a file with caching
    pub fn parse_with_cache(
        &mut self,
        file_path: &Path,
        content: &str,
        language: SourceLanguage,
    ) -> Result<ParsedFile, AstError> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .lock()
            .map_err(|e| AstError::Other(format!("Cache lock error: {}", e)))?
            .get(&file_path.to_string_lossy().to_string())
        {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified <= cached.timestamp {
                        return Ok(ParsedFile {
                            file_path: Arc::new(file_path.to_path_buf()),
                            language,
                            source: Arc::new(content.to_string()),
                            tree: Some(cached.tree.clone()),
                            custom_ast: None,
                        });
                    }
                }
            }
        }

        // Parse the file
        let parser = self.parsers.get_mut(&language).ok_or_else(|| {
            AstError::UnsupportedLanguage(format!("No parser for {:?}", language))
        })?;

        let tree = parser.parse(content, None).ok_or(AstError::ParseFailed)?;

        // Cache the result
        self.cache
            .lock()
            .map_err(|e| AstError::Other(format!("Cache lock error: {}", e)))?
            .insert(
                file_path.to_string_lossy().to_string(),
                CachedAst {
                    tree: tree.clone(),
                    timestamp: std::time::SystemTime::now(),
                },
            );

        Ok(ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            language,
            source: Arc::new(content.to_string()),
            tree: Some(tree),
            custom_ast: None,
        })
    }

    /// Parse content and return a ParsedFile
    /// UV-METHODS-001: Implements missing parse_content method for long_methods detector
    pub fn parse_content(
        &mut self,
        content: &str,
        file_path: &std::path::PathBuf,
        language: SourceLanguage,
    ) -> Result<ParsedFile, AstError> {
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{:?}", language)))?;
        let tree = parser
            .parse(content, None)
            .ok_or_else(|| AstError::ParseFailed)?;
        Ok(ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            language,
            source: Arc::new(content.to_string()),
            tree: Some(tree),
            custom_ast: None,
        })
    }

    /// Clear the AST cache
    pub fn clear_cache(&mut self) -> Result<(), AstError> {
        self.cache
            .lock()
            .map_err(|e| AstError::Other(format!("Cache lock error: {}", e)))?
            .clear();
        Ok(())
    }

    /// Get available parsers
    pub fn get_available_parsers(&self) -> Vec<SourceLanguage> {
        self.parsers.keys().copied().collect()
    }
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default AstParser")
    }
}

impl ParsedFile {
    /// Get cache path for a file
    pub fn cache_path(file_path: &Path) -> std::path::PathBuf {
        let mut cache_path = std::env::temp_dir();
        cache_path.push("uveddi_ast_cache");
        cache_path.push(format!(
            "{}.cache",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        cache_path
    }
}

/// Create a test AST parser for testing
pub fn create_test_ast_parser() -> Result<AstParser, AstError> {
    AstParser::new()
}
