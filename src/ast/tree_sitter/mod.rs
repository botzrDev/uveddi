use bincode;
use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
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
                                structs.entry(type_name).or_default().extend(methods);
                            }
                        }
                        _ => {}
                    }
                }
                // Build CustomAst::Struct for each struct
                for (name, methods) in structs {
                    items.push(CustomAst::Struct { name, methods });
                }
            }
            SourceLanguage::Python | SourceLanguage::JavaScript => {
                // Fallback: keep previous logic for now
                for child in root.children(&mut root.walk()) {
                    match (language, child.kind()) {
                        (SourceLanguage::Python, "class_definition")
                        | (SourceLanguage::JavaScript, "class_declaration") => {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                items.push(CustomAst::Variable { name });
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

        if let Some(cached) = self.cache.lock().unwrap().get(&path_str) {
            if cached.modified_at == modified_time {
                return Ok(cached.clone());
            }
        }

        // Try disk cache
        let cache_path = ParsedFile::cache_path(file_path);
        if let Ok(mut f) = fs::File::open(&cache_path) {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).ok();
            if let Ok(parsed) = bincode::deserialize::<ParsedFile>(&buf) {
                if parsed.modified_at == modified_time {
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
            .ok_or_else(|| AstError::UnsupportedLanguage(format!("{:?}", language)))?;
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
        let extension =
            file_path
                .extension()
                .and_then(|s| s.to_str())
                .ok_or(AstError::UnsupportedLanguage(format!(
                    "No file extension for {:?}",
                    file_path
                )))?;

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
    /// The custom, serializable AST for this file.
    pub custom_ast: Option<CustomAst>,
    /// The last modification time of the file.
    pub modified_at: std::time::SystemTime,
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

/// A simplified, serializable Rust-native AST node for demonstration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomAst {
    File { items: Vec<CustomAst> },
    Struct { name: String, methods: Vec<String> },
    Function { name: String, params: Vec<String> },
    Variable { name: String },
    // Extend as needed for more node types
}

impl CustomAst {
    /// Returns a summary string of the AST structure (node types, relationships).
    pub fn summary(&self) -> String {
        match self {
            CustomAst::File { items } => {
                let mut summary = String::from("File containing:");
                for item in items {
                    summary.push_str(&format!("\n- {}", item.node_type()));
                }
                summary
            }
            CustomAst::Struct { name, methods } => {
                format!("Struct: {} ({} methods)", name, methods.len())
            }
            CustomAst::Function { name, params } => {
                format!("Function: {} ({} params)", name, params.len())
            }
            CustomAst::Variable { name } => {
                format!("Variable: {}", name)
            }
        }
    }

    /// Returns a string representing the node type for summary purposes.
    fn node_type(&self) -> &'static str {
        match self {
            CustomAst::File { .. } => "File",
            CustomAst::Struct { .. } => "Struct",
            CustomAst::Function { .. } => "Function",
            CustomAst::Variable { .. } => "Variable",
        }
    }

    /// Extracts a relevant code snippet for the given issue context, if present.
    pub fn extract_relevant_code(&self, issue_context: &str) -> Option<String> {
        // For demonstration, just return the name of the first struct/function/variable matching the context
        match self {
            CustomAst::File { items } => {
                for item in items {
                    if let Some(snippet) = item.extract_relevant_code(issue_context) {
                        return Some(snippet);
                    }
                }
                None
            }
            CustomAst::Struct { name, methods } => {
                if issue_context.contains(name) {
                    Some(format!("struct {} {{ ... }}\nmethods: {:?}", name, methods))
                } else {
                    None
                }
            }
            CustomAst::Function { name, params } => {
                if issue_context.contains(name) {
                    Some(format!("fn {}({}) {{ ... }}", name, params.join(", ")))
                } else {
                    None
                }
            }
            CustomAst::Variable { name } => {
                if issue_context.contains(name) {
                    Some(format!("let {} = ...;", name))
                } else {
                    None
                }
            }
        }
    }
}

impl Default for CustomAst {
    fn default() -> Self {
        CustomAst::File { items: vec![] }
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
