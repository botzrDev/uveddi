//! Adapter implementations for existing types to support dependency injection
//!
//! These adapters enable the existing concrete implementations to work with
//! the new trait-based dependency injection system while maintaining
//! backward compatibility.

use crate::analysis::{
    detectors::dependency::{Dependency, DependencyExtractor},
    errors::AnalysisError,
    traits::{AstParserTrait, CacheStats, DependencyExtractorTrait, ResultCacheTrait},
};
use crate::ast::{tree_sitter_impl::AstParser, ParsedFile, SourceLanguage};
use crate::cache::result_cache::ResultCache;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Adapter implementation for AstParser to implement AstParserTrait
pub struct AstParserAdapter {
    parser: Arc<Mutex<AstParser>>,
}

impl AstParserAdapter {
    /// Create a new adapter wrapping an existing AstParser
    pub fn new(parser: AstParser) -> Self {
        Self {
            parser: Arc::new(Mutex::new(parser)),
        }
    }

    /// Create a new adapter with a new AstParser instance
    pub fn new_default() -> Result<Self, AnalysisError> {
        let parser = AstParser::new().map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to initialize AST parser: {}", e))
        })?;
        Ok(Self {
            parser: Arc::new(Mutex::new(parser)),
        })
    }
}

impl AstParserTrait for AstParserAdapter {
    fn parse_file(&self, path: &Path) -> Result<ParsedFile, AnalysisError> {
        let mut parser = self
            .parser
            .lock()
            .map_err(|e| AnalysisError::DetectionError(format!("Parser lock failed: {}", e)))?;

        parser.parse_file(path).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "AST parsing failed for {}: {}",
                path.display(),
                e
            ))
        })
    }

    fn is_initialized(&self) -> bool {
        // The AstParser is considered initialized if it was created successfully
        true
    }

    fn supported_languages(&self) -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
        ]
    }
}

/// Adapter implementation for DependencyExtractor to implement DependencyExtractorTrait
pub struct DependencyExtractorAdapter {
    extractor: DependencyExtractor,
}

impl DependencyExtractorAdapter {
    /// Create a new adapter wrapping an existing DependencyExtractor
    pub fn new(extractor: DependencyExtractor) -> Self {
        Self { extractor }
    }

    /// Create a new adapter with a new DependencyExtractor instance
    pub fn new_default() -> Result<Self, AnalysisError> {
        let extractor = DependencyExtractor::new().map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to initialize dependency extractor: {}",
                e
            ))
        })?;
        Ok(Self { extractor })
    }
}

impl DependencyExtractorTrait for DependencyExtractorAdapter {
    fn extract_from_ast(&self, parsed_file: &ParsedFile) -> Result<Vec<Dependency>, AnalysisError> {
        self.extractor.extract_from_ast(parsed_file).map_err(|e| {
            AnalysisError::DetectionError(format!("Dependency extraction failed: {}", e))
        })
    }

    fn supports_language(&self, language: &SourceLanguage) -> bool {
        match language {
            SourceLanguage::Rust => true,
            SourceLanguage::Python => true,
            SourceLanguage::JavaScript => true,
            _ => false,
        }
    }
}

/// Adapter implementation for ResultCache to implement ResultCacheTrait
pub struct ResultCacheAdapter {
    cache: Arc<Mutex<ResultCache>>,
}

impl ResultCacheAdapter {
    /// Create a new adapter wrapping an existing ResultCache
    pub fn new(cache: ResultCache) -> Self {
        Self {
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    /// Create a new adapter with a new in-memory ResultCache
    pub fn new_memory() -> Result<Self, AnalysisError> {
        let cache = ResultCache::new_in_memory().map_err(|e| {
            AnalysisError::DetectionError(format!("Failed to initialize result cache: {}", e))
        })?;
        Ok(Self {
            cache: Arc::new(Mutex::new(cache)),
        })
    }

    /// Create a new adapter with a file-based ResultCache
    pub fn new_with_path(path: &Path) -> Result<Self, AnalysisError> {
        let cache = ResultCache::new(path).map_err(|e| {
            AnalysisError::DetectionError(format!(
                "Failed to initialize result cache at {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(Self {
            cache: Arc::new(Mutex::new(cache)),
        })
    }
}

impl ResultCacheTrait for ResultCacheAdapter {
    fn get_json(&self, key: &str) -> Result<Option<String>, AnalysisError> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| AnalysisError::DetectionError(format!("Cache lock failed: {}", e)))?;

        // Convert string key to PathBuf for cache operations
        let path_buf = std::path::PathBuf::from(key);

        // Try to get the value as a JSON string
        match cache.get::<_, String>(&path_buf) {
            Ok(opt) => Ok(opt),
            Err(e) => Err(AnalysisError::DetectionError(format!(
                "Cache get operation failed: {}",
                e
            ))),
        }
    }

    fn set_json(&self, key: &str, value: &str) -> Result<(), AnalysisError> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| AnalysisError::DetectionError(format!("Cache lock failed: {}", e)))?;

        // Convert string key to PathBuf for cache operations
        let path_buf = std::path::PathBuf::from(key);

        // Store the JSON string
        cache.set(&path_buf, &value.to_string()).map_err(|e| {
            AnalysisError::DetectionError(format!("Cache set operation failed: {}", e))
        })
    }

    fn clear(&self) {
        if let Ok(_cache) = self.cache.lock() {
            // The original ResultCache doesn't have a clear method
            // This is a no-op for now
        }
    }

    fn get_stats(&self) -> CacheStats {
        // The original ResultCache doesn't expose stats, so we return defaults
        CacheStats::default()
    }
}
