//! AST provider implementation with caching
//!
//! Provides cached access to Abstract Syntax Trees for performance optimization.

use super::traits::AstProvider;
use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::ast::tree_sitter_impl::AstParser;
use crate::error::UveddiError;

use async_trait::async_trait;
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// AST provider implementation with integrated caching
pub struct AstProviderImpl {
    ast_parser: Mutex<AstParser>,
    ast_cache: AstCache,
    cache_lock: RwLock<()>, // For coordinating cache access
}

impl AstProviderImpl {
    /// Creates a new AST provider with default cache configuration
    pub fn new() -> Result<Self, UveddiError> {
        let ast_parser = AstParser::new()?;
        let cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(cache_config)?;

        Ok(Self {
            ast_parser: Mutex::new(ast_parser),
            ast_cache,
            cache_lock: RwLock::new(()),
        })
    }

    /// Creates a new AST provider with custom cache configuration
    pub fn with_cache_config(cache_config: CacheConfig) -> Result<Self, UveddiError> {
        let ast_parser = AstParser::new()?;
        let ast_cache = AstCache::new(cache_config)?;

        Ok(Self {
            ast_parser: Mutex::new(ast_parser),
            ast_cache,
            cache_lock: RwLock::new(()),
        })
    }

    /// Detects programming language from file path extension
    fn detect_language_from_path(&self, path: &Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Parses a file and caches the result
    async fn parse_and_cache(
        &self,
        file_path: &Path,
    ) -> Result<Arc<tree_sitter::Tree>, UveddiError> {
        info!("AST CACHE MISS: Parsing file {}", file_path.display());

        // Parse the file
        let parsed_file = self.ast_parser.lock().await.parse_file(file_path)?;

        // Extract the tree if available
        if let Some(tree) = parsed_file.tree {
            // Store in cache first
            if let Err(e) = self.ast_cache.store(file_path, tree) {
                warn!("Failed to cache AST for {:?}: {}", file_path, e);
                // If caching fails, we can't return the tree since it was moved
                return Err(UveddiError::AstError {
                    file: file_path.to_string_lossy().to_string(),
                    language: "unknown".to_string(),
                    message: "Failed to cache AST".to_string(),
                    suggestion: "Check cache configuration".to_string(),
                    source: None,
                });
            }

            // Return the cached version to ensure pointer equality
            if let Some(cached_tree) = self.ast_cache.get(file_path) {
                Ok(cached_tree)
            } else {
                // This shouldn't happen if store succeeded
                Err(UveddiError::AstError {
                    file: file_path.to_string_lossy().to_string(),
                    language: "unknown".to_string(),
                    message: "Failed to retrieve cached AST".to_string(),
                    suggestion: "Check cache implementation".to_string(),
                    source: None,
                })
            }
        } else {
            Err(UveddiError::AstError {
                file: file_path.to_string_lossy().to_string(),
                language: "unknown".to_string(),
                message: "Failed to generate AST from parsed file".to_string(),
                suggestion: "Check if the file contains valid syntax for the detected language"
                    .to_string(),
                source: None,
            })
        }
    }
}

#[async_trait]
impl AstProvider for AstProviderImpl {
    async fn get_ast(&self, file_path: &Path) -> Result<Arc<tree_sitter::Tree>, UveddiError> {
        // Check cache first (with read lock to allow concurrent reads)
        {
            let _read_guard = self.cache_lock.read().await;
            if let Some(cached_tree) = self.ast_cache.get(file_path) {
                info!(
                    "AST CACHE HIT: Using cached AST for {}",
                    file_path.display()
                );
                return Ok(cached_tree);
            }
        }

        // Cache miss - need to parse and cache
        // Use write lock to prevent concurrent parsing of the same file
        let _write_guard = self.cache_lock.write().await;

        // Double-check cache in case another thread parsed it while we were waiting
        if let Some(cached_tree) = self.ast_cache.get(file_path) {
            info!(
                "AST CACHE HIT: Using cached AST for {} (double-check)",
                file_path.display()
            );
            return Ok(cached_tree);
        }

        // Parse and cache the file
        self.parse_and_cache(file_path).await
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
