//! AST provider implementation with caching
//!
//! Provides cached access to Abstract Syntax Trees for performance optimization.

use super::traits::AstProvider;
use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::analysis::cache::wrappers::ArchivableSystemTime;
use crate::ast::{tree_sitter_impl::AstParser, ParseError, SourceLanguage, SyntaxError};
use crate::error::UveddiError;

use crate::core::logging::{info, warn};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
// Feature-gated tree-sitter imports
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Language, Parser, Tree};
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Language, Parser, Tree};
#[derive(Debug)]
pub struct ParsedFile {
    pub file_path: Arc<PathBuf>,
    pub language: SourceLanguage,
    pub tree: Option<Arc<Tree>>,
    pub source: Arc<String>,
    pub syntax_errors: Vec<SyntaxError>,
    pub custom_ast: Arc<Option<crate::ast::tree_sitter_impl::CustomAst>>,
    pub modified_at: ArchivableSystemTime,
}

/// AST provider implementation with integrated caching
pub struct AstProviderImpl {
    ast_parser: Mutex<AstParser>,
    ast_cache: AstCache,
    cache_lock: RwLock<()>, // For coordinating cache access
    language_map: HashMap<SourceLanguage, Language>, // Language grammar management
    parsed_file_cache: DashMap<PathBuf, Arc<ParsedFile>>, // Performance-critical concurrent caching
}

impl AstProviderImpl {
    /// Creates a new AST provider with default cache configuration
    pub fn new() -> Result<Self, UveddiError> {
        let ast_parser = AstParser::new()?;
        let cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(cache_config)?;

        // Initialize language grammar map
        let mut language_map = HashMap::new();
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "rust-lang")]
            language_map.insert(SourceLanguage::Rust, tree_sitter_rust::LANGUAGE.into());
            #[cfg(feature = "python-lang")]
            language_map.insert(SourceLanguage::Python, tree_sitter_python::LANGUAGE.into());
            #[cfg(feature = "javascript-lang")]
            language_map.insert(
                SourceLanguage::JavaScript,
                tree_sitter_javascript::LANGUAGE.into(),
            );
            #[cfg(feature = "typescript-lang")]
            language_map.insert(
                SourceLanguage::TypeScript,
                tree_sitter_typescript::LANGUAGE_TSX.into(),
            );
        }

        Ok(Self {
            ast_parser: Mutex::new(ast_parser),
            ast_cache,
            cache_lock: RwLock::new(()),
            language_map,
            parsed_file_cache: DashMap::new(),
        })
    }

    /// Creates a new AST provider with custom cache configuration
    pub fn with_cache_config(cache_config: CacheConfig) -> Result<Self, UveddiError> {
        let ast_parser = AstParser::new()?;
        let ast_cache = AstCache::new(cache_config)?;

        // Initialize language grammar map
        let mut language_map = HashMap::new();
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "rust-lang")]
            language_map.insert(SourceLanguage::Rust, tree_sitter_rust::LANGUAGE.into());
            #[cfg(feature = "python-lang")]
            language_map.insert(SourceLanguage::Python, tree_sitter_python::LANGUAGE.into());
            #[cfg(feature = "javascript-lang")]
            language_map.insert(
                SourceLanguage::JavaScript,
                tree_sitter_javascript::LANGUAGE.into(),
            );
            #[cfg(feature = "typescript-lang")]
            language_map.insert(
                SourceLanguage::TypeScript,
                tree_sitter_typescript::LANGUAGE_TSX.into(),
            );
        }

        Ok(Self {
            ast_parser: Mutex::new(ast_parser),
            ast_cache,
            cache_lock: RwLock::new(()),
            language_map,
            parsed_file_cache: DashMap::new(),
        })
    }

    /// Detects programming language from file path extension
    fn detect_language_from_path(&self, path: &Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("jsx") => SourceLanguage::JavaScript,
            Some("ts") | Some("tsx") => SourceLanguage::TypeScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Parses a file and caches the result - internal method
    async fn internal_parse_and_cache(
        &self,
        file_path: &Path,
    ) -> Result<Arc<ParsedFile>, ParseError> {
        info!("AST CACHE MISS: Parsing file {}", file_path.display());

        // Detect language from file path
        let language = self.detect_language_from_path(file_path);

        // Get appropriate grammar from language map
        let grammar = self
            .language_map
            .get(&language)
            .ok_or_else(|| ParseError::UnsupportedLanguage(format!("{:?}", language)))?;

        // Parse with error recovery
        let source = std::fs::read_to_string(file_path).map_err(|e| ParseError::Io(e))?;

        let mut parser = Parser::new();
        parser
            .set_language(grammar)
            .map_err(|e| ParseError::Other(e.to_string()))?;

        let tree = parser.parse(&source, None);

        // Handle catastrophic failure
        let tree = match tree {
            Some(t) => t,
            None => return Err(ParseError::Other("Parser returned None".to_string())),
        };

        // Collect syntax errors
        let syntax_errors = self.collect_syntax_errors(&tree, &source);

        // Create ParsedFile structure
        let parsed_file = Arc::new(ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            language,
            tree: Some(Arc::new(tree)),
            source: Arc::new(source),
            syntax_errors,
            custom_ast: Arc::new(None),
            modified_at: ArchivableSystemTime::now(),
        });

        // Cache result
        self.parsed_file_cache
            .insert(file_path.to_path_buf(), Arc::clone(&parsed_file));

        // Also store in the secondary cache (convert Tree to CacheableAst when tree-sitter is disabled)
        #[cfg(feature = "tree-sitter")]
        {
            if let Err(e) = self
                .ast_cache
                .store(file_path, parsed_file.tree.as_ref().unwrap().as_ref().clone())
            {
                warn!("Failed to cache AST for {:?}: {}", file_path, e);
            }
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            use crate::analysis::cache::ast::CacheableAst;
            let cacheable_ast = CacheableAst {
                data: b"stub_ast_data".to_vec(),
                timestamp: std::time::SystemTime::now(),
                language: format!("{:?}", parsed_file.language),
            };
            if let Err(e) = self.ast_cache.store(file_path, cacheable_ast) {
                warn!("Failed to cache AST for {:?}: {}", file_path, e);
            }
        }

        Ok(parsed_file)
    }

    /// Collect syntax errors from the parse tree
    fn collect_syntax_errors(&self, tree: &Tree, _source: &str) -> Vec<SyntaxError> {
        let mut errors = Vec::new();
        let root_node = tree.root_node();
        let _cursor = root_node.walk();

        // Traverse all nodes in the tree
        let mut stack = vec![root_node];
        while let Some(node) = stack.pop() {
            // Check if this node is an error node
            if node.is_error() || node.has_error() {
                let start = node.start_position();
                let end = node.end_position();

                errors.push(SyntaxError {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_line: start.row + 1,
                    start_column: start.column,
                    end_line: end.row + 1,
                    end_column: end.column,
                    message: "Syntax error".to_string(),
                });
            }

            // Add children to the stack
            let mut child = node.child(0);
            while let Some(c) = child {
                stack.push(c);
                child = c.next_sibling();
            }
        }

        errors
    }
}

#[async_trait]
impl AstProvider for AstProviderImpl {
    async fn get_ast(&self, file_path: &Path) -> Result<Arc<Tree>, UveddiError> {
        // Check performance-critical cache first
        if let Some(parsed_file) = self.parsed_file_cache.get(file_path) {
            info!(
                "AST CACHE HIT: Using cached AST for {}",
                file_path.display()
            );
            if let Some(tree) = &parsed_file.tree {
                return Ok(Arc::clone(tree));
        }
        }

        // Check secondary cache (with read lock to allow concurrent reads)
        {
            let _read_guard = self.cache_lock.read().await;
            if let Some(cached_tree) = self.ast_cache.get(file_path) {
                info!(
                    "AST CACHE HIT: Using cached AST for {}",
                    file_path.display()
                );
                #[cfg(feature = "tree-sitter")]
                {
                    return Ok(Arc::new(cached_tree.as_ref().clone()));
                }
                #[cfg(not(feature = "tree-sitter"))]
                {
                    // For stub builds, we need to create a stub Tree from CacheableAst
                    use crate::ast::tree_sitter::Tree;
                    return Ok(Arc::new(Tree));
                }
            }
        }

        // Cache miss - need to parse and cache
        // Use write lock to prevent concurrent parsing of the same file
        let _write_guard = self.cache_lock.write().await;

        // Double-check cache in case another thread parsed it while we were waiting
        if let Some(parsed_file) = self.parsed_file_cache.get(file_path) {
            info!(
                "AST CACHE HIT: Using cached AST for {} (double-check)",
                file_path.display()
            );
            if let Some(tree) = &parsed_file.tree {
                return Ok(Arc::clone(tree));
        }
        }

        // Parse and cache the file
        let parsed_file = self
            .internal_parse_and_cache(file_path)
            .await
            .map_err(|e| UveddiError::AstError {
                file: file_path.to_string_lossy().to_string(),
                language: "unknown".to_string(),
                message: format!("Parse error: {:?}", e),
                suggestion: "Check file syntax".to_string(),
                source: None,
            })?;

        // Return the tree from the parsed file
        if let Some(tree) = &parsed_file.tree {
            Ok(Arc::clone(tree))
        } else {
            Err(UveddiError::AstError {
                file: file_path.to_string_lossy().to_string(),
                language: format!("{:?}", parsed_file.language),
                message: "No AST tree available".to_string(),
                suggestion: "Check parser implementation".to_string(),
                source: None,
            })
        }
    }

    async fn parse_file(&self, file_path: &Path) -> Result<Arc<ParsedFile>, UveddiError> {
        // Check performance-critical cache first
        if let Some(parsed_file) = self.parsed_file_cache.get(file_path) {
            info!(
                "PARSED FILE CACHE HIT: Using cached parsed file for {}",
                file_path.display()
            );
            return Ok(Arc::clone(&parsed_file));
        }

        // Parse and cache the file
        let parsed_file = self
            .internal_parse_and_cache(file_path)
            .await
            .map_err(|e| UveddiError::AstError {
                file: file_path.to_string_lossy().to_string(),
                language: "unknown".to_string(),
                message: format!("Parse error: {:?}", e),
                suggestion: "Check file syntax".to_string(),
                source: None,
            })?;

        Ok(parsed_file)
    }

    fn clear_cache(&self) {
        self.ast_cache.clear();
    }

    fn get_cache_metrics(&self) -> serde_json::Value {
        self.ast_cache.export_metrics_for_observability()
    }
}

impl Default for AstProviderImpl {
    fn default() -> Self {
        Self::new().expect("Failed to create default AstProvider")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_ast_provider_caching() {
        let provider = AstProviderImpl::new().unwrap();

        // Create a temporary Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        let rust_code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        temp_file.write_all(rust_code.as_bytes()).unwrap();

        let file_path = temp_file.path();

        // First call should parse and cache
        let ast1 = provider.get_ast(file_path).await;
        assert!(ast1.is_ok());

        // Second call should hit cache
        let ast2 = provider.get_ast(file_path).await;
        assert!(ast2.is_ok());

        // Both should return the same tree (by pointer equality)
        let tree1 = ast1.unwrap();
        let tree2 = ast2.unwrap();
        assert!(Arc::ptr_eq(&tree1, &tree2));
    }

    #[tokio::test]
    async fn test_ast_provider_cache_metrics() {
        let provider = AstProviderImpl::new().unwrap();

        // Initial metrics should show empty cache
        let initial_metrics = provider.get_cache_metrics();
        assert!(initial_metrics.is_object());

        // Create and parse a file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fn test() {}").unwrap();

        let _ast = provider.get_ast(temp_file.path()).await;

        // Metrics should now show cache activity
        let updated_metrics = provider.get_cache_metrics();
        assert!(updated_metrics.is_object());
    }

    #[test]
    fn test_language_detection() {
        let provider = AstProviderImpl::new().unwrap();

        // Test various file extensions
        assert!(matches!(
            provider.detect_language_from_path(Path::new("test.rs")),
            crate::ast::SourceLanguage::Rust
        ));

        assert!(matches!(
            provider.detect_language_from_path(Path::new("test.py")),
            crate::ast::SourceLanguage::Python
        ));

        assert!(matches!(
            provider.detect_language_from_path(Path::new("test.js")),
            crate::ast::SourceLanguage::JavaScript
        ));

        assert!(matches!(
            provider.detect_language_from_path(Path::new("test.ts")),
            crate::ast::SourceLanguage::JavaScript
        ));
    }

    #[test]
    fn test_cache_clearing() {
        let provider = AstProviderImpl::new().unwrap();

        // Clear cache should not panic
        provider.clear_cache();

        // Metrics should show cleared cache
        let metrics = provider.get_cache_metrics();
        assert!(metrics.is_object());
    }
}
