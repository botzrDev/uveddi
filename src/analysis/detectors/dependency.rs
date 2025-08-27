use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(feature = "tree-sitter")]
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Query, QueryCursor, StreamingIterator};

// Queries will be loaded dynamically when needed
use crate::ast::tree_sitter_impl::{AstError, AstParser, ParsedFile, SourceLanguage};
pub use crate::database::models::{Dependency, DependencyType};

/// Errors that can occur during dependency extraction
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("AST parsing error: {0}")]
    AstError(#[from] AstError),
    #[error("IO error for path {0}: {1}")]
    IoError(PathBuf, #[source] std::io::Error),
    #[error("Query compilation error: {0}")]
    QueryError(String),
    #[error("Invalid file path: {path} - Reason: {reason}")]
    InvalidPath { path: PathBuf, reason: String },
    #[error("Unsupported language for path: {0}")]
    UnsupportedLanguage(String),
}

/// Extracts dependencies from source code files using Abstract Syntax Tree (AST) parsing.
///
/// This extractor leverages `tree-sitter` to parse source files for various languages
/// (Rust, Python, JavaScript) and identify import/use statements. It is responsible for
/// finding direct dependencies within a single file.
///
/// # Features
///
/// - Supports multiple programming languages through `tree-sitter` grammars.
/// - Identifies different types of dependencies (e.g., `use`, `import`).
/// - Resolves module paths for Rust `mod` statements.
///
/// # Errors
///
/// Returns `ExtractionError` if the `tree-sitter` parser cannot be initialized or if
/// a query fails to compile.
pub struct DependencyExtractor {
    parser: AstParser,
}

impl DependencyExtractor {
    /// Creates a new `DependencyExtractor`.
    ///
    /// This initializes the underlying `AstParser`, which may fail if `tree-sitter`
    /// grammars are not available.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DependencyExtractor` or an `ExtractionError`.
    pub fn new() -> Result<Self, ExtractionError> {
        let parser = AstParser::new().map_err(ExtractionError::AstError)?;
        Ok(Self { parser })
    }

    /// Extracts dependencies from a single source file by its path.
    ///
    /// This is a convenience method that first parses the file into an AST and then
    /// calls `extract_from_ast`.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the source file to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Dependency>` or an `ExtractionError`.
    pub fn extract_from_file(
        &mut self,
        file_path: &Path,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ExtractionError::IoError(file_path.to_path_buf(), e))?;

        let language = SourceLanguage::from_path(file_path).ok_or_else(|| {
            ExtractionError::UnsupportedLanguage(file_path.to_string_lossy().to_string())
        })?;

        let parsed_file = self
            .parser
            .parse_content(&content, file_path, language)
            .map_err(ExtractionError::AstError)?;
        self.extract_from_ast(&parsed_file)
    }

    /// Extracts dependencies from a `ParsedFile` containing a pre-existing AST.
    ///
    /// This method runs a language-specific `tree-sitter` query against the AST
    /// to find all import-like statements. It captures the module paths and constructs
    /// a list of `Dependency` objects.
    ///
    /// # Arguments
    ///
    /// * `parsed_file` - A reference to the `ParsedFile` to be analyzed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Dependency>` or an `ExtractionError` if the
    /// `tree-sitter` query fails.
    pub fn extract_from_ast(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        #[cfg(feature = "tree-sitter")]
        {
            let (query_str, dependency_type) = match parsed_file.language {
                SourceLanguage::Rust => (RUST_IMPORTS_QUERY, DependencyType::Use),
                SourceLanguage::Python => (PYTHON_IMPORTS_QUERY, DependencyType::Import),
                SourceLanguage::JavaScript => (JAVASCRIPT_IMPORTS_QUERY, DependencyType::Import),
                SourceLanguage::TypeScript => (JAVASCRIPT_IMPORTS_QUERY, DependencyType::Import), // Reuse JS query for TS placeholder
            };

            // UV-2: Semantic dependency classification for function calls and data transformations
            // TODO: Extend tree-sitter queries to capture function calls (ControlFlow) and assignments (DataFlow)
            // Example: If capture_name == "call", set dependency_type = DependencyType::ControlFlow
            // Example: If capture_name == "assignment", set dependency_type = DependencyType::DataFlow

            let query = Query::new(
                &parsed_file
                    .tree
                    .as_ref()
                    .expect("AST tree missing")
                    .language(),
                query_str,
            )
            .map_err(|e| ExtractionError::QueryError(e.to_string()))?;

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(
                &query,
                parsed_file
                    .tree
                    .as_ref()
                    .expect("AST tree missing")
                    .root_node(),
                parsed_file.source.as_bytes(),
            );

            let mut dependencies = Vec::new();
            while let Some(mat) = matches.next() {
                for capture in mat.captures {
                    // Only process captures named "path"
                    let capture_name = query.capture_names()[capture.index as usize];
                    if capture_name != "path" {
                        continue;
                    }

                    let node = capture.node;
                    let line_number = node.start_position().row + 1;
                    let mut module_name = node
                        .utf8_text(parsed_file.source.as_bytes())
                        .unwrap_or("")
                        .to_string();

                    // For Rust, resolve `mod` statements to file paths
                    if parsed_file.language == SourceLanguage::Rust
                        && dependency_type == DependencyType::Use
                    {
                        // UV-150: Strategic error handling for path operations (Category V)
                        let parent_dir = parsed_file.file_path.parent()
                        .ok_or_else(|| ExtractionError::InvalidPath {
                            path: parsed_file.file_path.as_ref().clone(),
                            reason: "File path has no parent directory (see UV-150 error handling policy)".to_string(),
                        })?;
                        let mut potential_path = parent_dir.join(&module_name);
                        if !potential_path.exists() {
                            potential_path.set_extension("rs");
                            if !potential_path.exists() {
                                // Check for module/mod.rs
                                let mod_parent = parsed_file.file_path.as_path().parent().ok_or_else(|| ExtractionError::InvalidPath {
                                path: parsed_file.file_path.as_ref().clone(),
                                reason: "File path has no parent directory for mod.rs (see UV-150)".to_string(),
                            })?;
                                let mod_path = mod_parent.join(&module_name).join("mod.rs");
                                if mod_path.exists() {
                                    potential_path = mod_path;
                                }
                            }
                        }
                        if potential_path.exists() {
                            module_name = potential_path.to_string_lossy().into_owned();
                        }
                    }

                    // Clean up the module name (e.g., remove quotes from strings)
                    if module_name.starts_with('"') && module_name.ends_with('"')
                        || module_name.starts_with('\'') && module_name.ends_with('\'')
                    {
                        module_name = module_name[1..module_name.len() - 1].to_string();
                    }

                    // Normalize JS/TS import paths to match file stem (e.g., './b.js' -> 'b')
                    if let SourceLanguage::JavaScript = parsed_file.language {
                        if module_name.starts_with("./") {
                            let name = module_name.trim_start_matches("./");
                            if let Some(stripped) = name.strip_suffix(".js") {
                                module_name = stripped.to_string();
                            } else if let Some(stripped) = name.strip_suffix(".ts") {
                                module_name = stripped.to_string();
                            } else {
                                module_name = name.to_string();
                            }
                        }
                    }

                    if let SourceLanguage::Rust = parsed_file.language {
                        if let Some(parent) = parsed_file.file_path.as_path().parent() {
                            let mut path = parent.join(&module_name);
                            if !path.exists() {
                                path.set_extension("rs");
                            }
                        }
                    }

                    let mut detected_type = dependency_type.clone();
                    // UV-2: Classify dependency type based on AST capture name
                    if capture_name == "call" {
                        detected_type = DependencyType::ControlFlow;
                    } else if capture_name == "assignment" {
                        detected_type = DependencyType::DataFlow;
                    }

                    dependencies.push(Dependency {
                        from_file: parsed_file.file_path.as_ref().clone(),
                        to_module: module_name,
                        dependency_type: detected_type,
                        line_number: Some((line_number + 1) as u32),
                    });
                }
            }

            Ok(dependencies)
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            // Fallback regex-based dependency extraction
            self.extract_with_regex(parsed_file)
        }
    }
    /// Fallback dependency extraction using regex (for when tree-sitter is disabled)
    fn extract_with_regex(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        // Implement basic regex-based dependency extraction
        // This is a simplified fallback - should be expanded based on language
        let pattern = match parsed_file.language {
            SourceLanguage::Rust => r"use\s+([\w:]+)",
            SourceLanguage::Python => r"import\s+([\w.]+)",
            SourceLanguage::JavaScript => {
                r#"import\s+.*from\s+['"]([^'"]+)['"]|require\(['"]([^'"]+)['"]\)"#
            }
            SourceLanguage::TypeScript => {
                r#"import\s+.*from\s+['"]([^'"]+)['"]|require\(['"]([^'"]+)['"]\)"#
            } // reuse JS regex
        };

        // Create regex pattern
        let re =
            regex::Regex::new(pattern).map_err(|e| ExtractionError::QueryError(e.to_string()))?;
        let mut dependencies = Vec::new();

        // Process each line
        for (line_num, line) in parsed_file.source.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                // Get module name from capture group 1 or 2
                let module_name = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str().to_string());

                if let Some(name) = module_name {
                    dependencies.push(Dependency {
                        from_file: parsed_file.file_path.as_ref().clone(), // UV-222: Convert Arc<PathBuf> to PathBuf
                        to_module: name,
                        dependency_type: DependencyType::Import,
                        line_number: Some((line_num + 1) as u32),
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Async version of extract_from_ast for compatibility with service layer
    pub async fn extract_dependencies(
        &self,
        parsed_file: &crate::analysis::components::ast_provider::ParsedFile,
    ) -> Result<Vec<Dependency>, ExtractionError> {
        // Convert between ParsedFile types - this is a compatibility layer
        let ast_parsed_file = ParsedFile {
            file_path: Arc::clone(&parsed_file.file_path),
            language: parsed_file.language,
            source: Arc::clone(&parsed_file.source),
            tree: parsed_file.tree.clone(),
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        // Call the sync version
        self.extract_from_ast(&ast_parsed_file)
    }
}
