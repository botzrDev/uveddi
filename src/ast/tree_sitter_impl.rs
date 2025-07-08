// Real tree-sitter implementation - compiled when feature "tree-sitter" is enabled

use bincode;
use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tree_sitter::{Parser, Tree};

pub mod queries;

const CACHE_DIR: &str = ".uveddi_cache";

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
        rust_parser.set_language(&tree_sitter_rust::language())?;
        parsers.insert(SourceLanguage::Rust, rust_parser);

        let mut python_parser = Parser::new();
        python_parser.set_language(&tree_sitter_python::language())?;
        parsers.insert(SourceLanguage::Python, python_parser);

        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(&tree_sitter_javascript::language())?;
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

    /// Transform tree-sitter CST to custom AST (basic implementation for demonstration)
    fn tree_to_custom_ast(
        tree: &Tree,
        source: &str,
        language: &SourceLanguage,
    ) -> Option<CustomAst> {
        let root = tree.root_node();
        let mut items = Vec::new();
        match language {
            SourceLanguage::Rust => {
                // Collect structs and their methods
                let mut structs: HashMap<String, Vec<String>> = HashMap::new();
                let mut struct_names = Vec::new();
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "struct_item" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                struct_names.push(name.clone());
                                structs.insert(name, Vec::new());
                            }
                        }
                        "impl_item" => {
                            if let Some(type_node) = child.child_by_field_name("type") {
                                let type_name = type_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                let mut methods = Vec::new();
                                if let Some(body_node) = child.child_by_field_name("body") {
                                    for decl in body_node.children(&mut body_node.walk()) {
                                        if decl.kind() == "function_item" {
                                            if let Some(name_node) =
                                                decl.child_by_field_name("name")
                                            {
                                                let method_name = name_node
                                                    .utf8_text(source.as_bytes())
                                                    .unwrap_or("")
                                                    .to_string();
                                                methods.push(method_name);
                                            }
                                        }
                                    }
                                }
                                if let Some(existing_methods) = structs.get_mut(&type_name) {
                                    existing_methods.extend(methods);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                for struct_name in struct_names {
                    let methods = structs.get(&struct_name).cloned().unwrap_or_default();
                    items.push(CustomAst::Struct {
                        name: struct_name,
                        methods,
                    });
                }
            }
            SourceLanguage::Python => {
                // Basic Python class/function extraction
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "class_definition" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                items.push(CustomAst::Struct {
                                    name,
                                    methods: Vec::new(),
                                });
                            }
                        }
                        "function_definition" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                items.push(CustomAst::Function {
                                    name,
                                    params: Vec::new(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            SourceLanguage::JavaScript => {
                // Basic JavaScript function/class extraction
                for child in root.children(&mut root.walk()) {
                    match child.kind() {
                        "function_declaration" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                items.push(CustomAst::Function {
                                    name,
                                    params: Vec::new(),
                                });
                            }
                        }
                        "class_declaration" => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                items.push(CustomAst::Struct {
                                    name,
                                    methods: Vec::new(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Some(CustomAst::File { items })
    }

    /// Parse file and extract dependencies (ER-F-003), with AST caching.
    /// Returns a parsed AST for the file, using cache if available.
    /// Errors if the file cannot be parsed or language is unsupported.
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let path_str = file_path.to_string_lossy().to_string();
        let modified_time = fs::metadata(file_path)?.modified()?;

        if let Some(cached) = self.cache.lock()
            .map_err(|_| AstError::Other("Cache lock poisoned".to_string()))?
            .get(&path_str) {
            if cached.modified_at == modified_time {
                return Ok(cached.clone());
            }
        }

        // Try disk cache
        let cache_path = ParsedFile::cache_path(file_path);
        if let Ok(mut f) = fs::File::open(&cache_path) {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).ok();
            if let Ok(mut parsed) = bincode::deserialize::<ParsedFile>(&buf) {
                if parsed.modified_at == modified_time {
                    // Re-parse the AST since Tree is not serializable
                    let parser = self.parsers.get_mut(&parsed.language).ok_or_else(|| {
                        AstError::UnsupportedLanguage(format!("{:?}", parsed.language))
                    })?;
                    let tree = parser
                        .parse(&parsed.source, None)
                        .ok_or(AstError::ParseFailed)?;
                    parsed.tree = Some(tree);

                    self.cache
                        .lock()
                        .unwrap()
                        .insert(path_str.clone(), parsed.clone());
                    return Ok(parsed);
                }
            }
        }

        // Parse and cache
        let source = fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{language:?}")))?;
        let tree = parser.parse(&source, None).ok_or(AstError::ParseFailed)?;
        if tree.root_node().has_error() {
            return Err(AstError::ParseFailed);
        }
        let custom_ast = Self::tree_to_custom_ast(&tree, &source, &language);
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree: Some(tree),
            source: source.clone(),
            custom_ast: custom_ast.clone(),
            modified_at: modified_time,
        };
        let mut disk_parsed = parsed.clone();
        disk_parsed.tree = None;
        let encoded = bincode::serialize(&disk_parsed)
            .map_err(|e| AstError::Other(format!("Failed to serialize cache: {}", e)))?;
        if let Ok(mut f) = fs::File::create(&cache_path) {
            f.write_all(&encoded).ok();
        }
        self.cache.lock()
            .map_err(|_| AstError::Other("Cache lock poisoned".to_string()))?
            .insert(path_str, parsed.clone());
        Ok(parsed)
    }

    /// Parse content directly from a string (useful for testing)
    pub fn parse_content(&mut self, content: &str, file_path: &Path, language: SourceLanguage) -> Result<ParsedFile, AstError> {
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{language:?}")))?;
        
        let tree = parser.parse(content, None).ok_or(AstError::ParseFailed)?;
        if tree.root_node().has_error() {
            return Err(AstError::ParseFailed);
        }
        
        let custom_ast = Self::tree_to_custom_ast(&tree, content, &language);
        let parsed = ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree: Some(tree),
            source: content.to_string(),
            custom_ast,
            modified_at: std::time::SystemTime::now(),
        };
        
        Ok(parsed)
    }

    fn detect_language(&self, file_path: &Path) -> Result<SourceLanguage, AstError> {
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match ext {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "jsx" | "ts" | "tsx" => Ok(SourceLanguage::JavaScript),
            _ => Err(AstError::UnsupportedLanguage(format!(
                "Unsupported file extension: {}",
                ext
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: SourceLanguage,
    #[serde(skip)]
    pub tree: Option<Tree>,
    pub source: String,
    pub custom_ast: Option<CustomAst>,
    pub modified_at: std::time::SystemTime,
}

impl ParsedFile {
    pub fn cache_path(file_path: &Path) -> std::path::PathBuf {
        let mut hasher = md5::Context::new();
        hasher.consume(file_path.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.compute());
        std::env::temp_dir()
            .join(CACHE_DIR)
            .join(format!("{}.ast", hash))
    }

    /// Extract a tree-sitter parsing summary for debugging.
    /// Returns a human-readable summary of the parsed AST structure.
    pub fn summary(&self) -> String {
        match &self.custom_ast {
            Some(ast) => format!("Parsed {}: {:?}", self.path.display(), ast),
            None => format!("Parsed {} (no AST)", self.path.display()),
        }
    }

    /// Get a specific code segment from the parsed file
    /// This is a placeholder implementation for code segment extraction
    pub fn extract_relevant_code(&self, issue_context: &str) -> Option<String> {
        // For now, just return a simple context around the issue
        // In the future, this could use the AST to find the exact function/class
        let lines: Vec<&str> = self.source.lines().collect();
        
        // Look for lines containing the issue context
        for (i, line) in lines.iter().enumerate() {
            if line.contains(issue_context) {
                // Return 3 lines of context around the match
                let start = i.saturating_sub(3);
                let end = std::cmp::min(i + 4, lines.len());
                let context_lines = &lines[start..end];
                return Some(context_lines.join("\n"));
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomAst {
    File { items: Vec<CustomAst> },
    Struct { name: String, methods: Vec<String> },
    Function { name: String, params: Vec<String> },
    Variable { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[error("Other error: {0}")]
    Other(String),
}

impl Clone for AstParser {
    fn clone(&self) -> Self {
        // Re-initialize parsers for each clone
        let mut parsers = std::collections::HashMap::new();
        let mut rust_parser = tree_sitter::Parser::new();
        rust_parser
            .set_language(&tree_sitter_rust::language())
            .unwrap();
        parsers.insert(SourceLanguage::Rust, rust_parser);
        let mut python_parser = tree_sitter::Parser::new();
        python_parser
            .set_language(&tree_sitter_python::language())
            .unwrap();
        parsers.insert(SourceLanguage::Python, python_parser);
        let mut javascript_parser = tree_sitter::Parser::new();
        javascript_parser
            .set_language(&tree_sitter_javascript::language())
            .unwrap();
        parsers.insert(SourceLanguage::JavaScript, javascript_parser);
        AstParser {
            parsers,
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}
