//! Traits for dependency injection in the analysis engine
//!
//! These traits enable constructor injection and testing with mock implementations.
//! They provide abstractions over the concrete implementations used by AnalysisEngine.

use crate::analysis::detectors::dependency::Dependency;
use crate::analysis::errors::AnalysisError;
use crate::ast::ParsedFile;
use std::path::Path;

/// Trait for AST parsing functionality
/// 
/// Enables dependency injection of different AST parser implementations
/// for better testability and modularity.
pub trait AstParserTrait: Send + Sync {
    /// Parse a file into an AST representation
    /// 
    /// # Arguments
    /// * `path` - Path to the file to parse
    /// 
    /// # Returns
    /// Parsed file with AST representation
    /// 
    /// # Errors
    /// Returns AnalysisError if parsing fails
    fn parse_file(&self, path: &Path) -> Result<ParsedFile, AnalysisError>;
    
    /// Check if the parser is properly initialized
    /// 
    /// # Returns
    /// True if parser is ready for use, false otherwise
    fn is_initialized(&self) -> bool;
    
    /// Get supported languages by this parser
    fn supported_languages(&self) -> Vec<crate::ast::SourceLanguage>;
}

/// Trait for dependency extraction functionality
/// 
/// Enables dependency injection of different dependency extractors
/// for better testability and modularity.
pub trait DependencyExtractorTrait: Send + Sync {
    /// Extract dependencies from a parsed file
    /// 
    /// # Arguments
    /// * `parsed_file` - The parsed file to analyze
    /// 
    /// # Returns
    /// Vector of dependencies found in the file
    /// 
    /// # Errors
    /// Returns AnalysisError if extraction fails
    fn extract_from_ast(&self, parsed_file: &ParsedFile) -> Result<Vec<Dependency>, AnalysisError>;
    
    /// Check if the extractor supports the given language
    /// 
    /// # Arguments
    /// * `language` - The source language to check
    /// 
    /// # Returns
    /// True if language is supported, false otherwise
    fn supports_language(&self, language: &crate::ast::SourceLanguage) -> bool;
}

/// Trait for result caching functionality
/// 
/// Enables dependency injection of different cache implementations
/// for better testability and modularity. Uses string-based keys and JSON values
/// for dyn compatibility.
pub trait ResultCacheTrait: Send + Sync {
    /// Get a cached result by key as JSON string
    /// 
    /// # Arguments
    /// * `key` - The cache key to look up
    /// 
    /// # Returns
    /// Optional cached result as JSON string
    /// 
    /// # Errors
    /// Returns AnalysisError if cache access fails
    fn get_json(&self, key: &str) -> Result<Option<String>, AnalysisError>;
    
    /// Set a cached result by key as JSON string
    /// 
    /// # Arguments
    /// * `key` - The cache key to store under
    /// * `value` - The JSON value to cache
    /// 
    /// # Errors
    /// Returns AnalysisError if cache storage fails
    fn set_json(&self, key: &str, value: &str) -> Result<(), AnalysisError>;
    
    /// Clear all cached results
    fn clear(&self);
    
    /// Get cache statistics for monitoring
    fn get_stats(&self) -> CacheStats;
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub memory_usage: usize,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            entries: 0,
            memory_usage: 0,
        }
    }
}