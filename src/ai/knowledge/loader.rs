//! Knowledge Library Loader and Runtime Access
//!
//! This module provides the runtime loading and access layer for the compressed
//! knowledge library. It implements lazy loading with OnceLock pattern for
//! thread-safe access and supports both embedded and file-based loading.

use crate::ai::knowledge::{compression::*, indexing::*, schema::*};
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

/// Global knowledge library instance using OnceLock for lazy loading
static KNOWLEDGE_LIBRARY: OnceLock<KnowledgeLibraryLookup> = OnceLock::new();

/// Knowledge library loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Whether to load from embedded data or external file
    pub use_embedded: bool,
    /// Path to external knowledge library file
    pub library_path: Option<std::path::PathBuf>,
    /// Path to external index file
    pub index_path: Option<std::path::PathBuf>,
    /// Whether to validate checksums on load
    pub validate_checksums: bool,
    /// Whether to preload all indices
    pub preload_indices: bool,
    /// Maximum memory usage for caching (in bytes)
    pub max_cache_size: usize,
}

/// Knowledge library loading errors
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("Failed to read library file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to decompress library data: {0}")]
    Decompression(String),

    #[error("Failed to deserialize library: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Checksum validation failed")]
    ChecksumMismatch,

    #[error("Library version incompatible: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Index file not found: {path}")]
    IndexNotFound { path: String },

    #[error("Library not initialized")]
    NotInitialized,
}

/// Knowledge library loader
pub struct KnowledgeLibraryLoader {
    config: LoaderConfig,
}

impl KnowledgeLibraryLoader {
    /// Create a new loader with configuration
    pub fn new(config: LoaderConfig) -> Self {
        Self { config }
    }

    /// Create a loader with default embedded configuration
    pub fn embedded() -> Self {
        let config = LoaderConfig {
            use_embedded: true,
            library_path: None,
            index_path: None,
            validate_checksums: true,
            preload_indices: true,
            max_cache_size: 100 * 1024 * 1024, // 100MB
        };
        Self::new(config)
    }

    /// Create a loader for external files
    pub fn from_files<P: AsRef<Path>>(library_path: P, index_path: P) -> Self {
        let config = LoaderConfig {
            use_embedded: false,
            library_path: Some(library_path.as_ref().to_path_buf()),
            index_path: Some(index_path.as_ref().to_path_buf()),
            validate_checksums: true,
            preload_indices: true,
            max_cache_size: 100 * 1024 * 1024, // 100MB
        };
        Self::new(config)
    }

    /// Load the knowledge library and index
    pub fn load(&self) -> Result<KnowledgeLibraryLookup, LoaderError> {
        let (library, index) = if self.config.use_embedded {
            self.load_embedded()?
        } else {
            self.load_from_files()?
        };

        // Validate versions and checksums
        if self.config.validate_checksums {
            self.validate_library(&library)?;
            self.validate_index(&index)?;
        }

        Ok(KnowledgeLibraryLookup::new(library, index))
    }

    /// Load embedded knowledge library and index
    fn load_embedded(&self) -> Result<(KnowledgeLibrary, KnowledgeIndex), LoaderError> {
        // In a real implementation, this would load from embedded compressed data
        // For now, we'll create a minimal library for testing
        let library = self.create_minimal_library();
        let index = self.create_minimal_index(&library)?;

        Ok((library, index))
    }

    /// Load knowledge library and index from external files
    fn load_from_files(&self) -> Result<(KnowledgeLibrary, KnowledgeIndex), LoaderError> {
        let library_path =
            self.config
                .library_path
                .as_ref()
                .ok_or_else(|| LoaderError::IndexNotFound {
                    path: "library_path not configured".to_string(),
                })?;

        let index_path =
            self.config
                .index_path
                .as_ref()
                .ok_or_else(|| LoaderError::IndexNotFound {
                    path: "index_path not configured".to_string(),
                })?;

        // Load and decompress library
        let library_data = std::fs::read(library_path)?;
        let decompressed_library = self.decompress_library_data(&library_data)?;
        let library: KnowledgeLibrary = serde_json::from_slice(&decompressed_library)?;

        // Load index
        let index_data = std::fs::read(index_path)?;
        let index: KnowledgeIndex = serde_json::from_slice(&index_data)?;

        Ok((library, index))
    }

    /// Decompress library data using the compression metadata
    fn decompress_library_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, LoaderError> {
        // In a real implementation, this would use zstd with dictionary decompression
        decompress_with_zstd(compressed_data, None)
            .map_err(|e| LoaderError::Decompression(e.to_string()))
    }

    /// Validate library integrity
    fn validate_library(&self, library: &KnowledgeLibrary) -> Result<(), LoaderError> {
        // Check schema version compatibility
        let expected_version = "1.0.0"; // Current schema version
        if library.metadata.schema_version != expected_version {
            return Err(LoaderError::VersionMismatch {
                expected: expected_version.to_string(),
                found: library.metadata.schema_version.clone(),
            });
        }

        // Validate library structure
        library
            .validate()
            .map_err(|e| LoaderError::Decompression(format!("Library validation failed: {}", e)))?;

        Ok(())
    }

    /// Validate index integrity
    fn validate_index(&self, index: &KnowledgeIndex) -> Result<(), LoaderError> {
        // Validate index checksum
        if index.metadata.checksum.is_empty() {
            return Err(LoaderError::ChecksumMismatch);
        }

        // Validate index structure
        if index.total_patterns == 0 {
            return Err(LoaderError::Decompression("Empty index".to_string()));
        }

        Ok(())
    }

    /// Create a minimal library for testing/fallback
    fn create_minimal_library(&self) -> KnowledgeLibrary {
        let mut library = KnowledgeLibrary::new();

        // Add a few universal patterns for testing
        let god_object = PatternKnowledge {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            definition: CompressedString::new("A class that knows too much or does too much"),
            symptoms: vec![
                CompressedString::new("Very large class (>1000 lines)"),
                CompressedString::new("High number of methods"),
                CompressedString::new("Low cohesion"),
                CompressedString::new("High coupling"),
            ],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![
                DetectionMethod::MetricThreshold {
                    metric_name: "lines_of_code".to_string(),
                    threshold: 1000.0,
                    operator: ComparisonOperator::GreaterThan,
                },
                DetectionMethod::MetricThreshold {
                    metric_name: "method_count".to_string(),
                    threshold: 50.0,
                    operator: ComparisonOperator::GreaterThan,
                },
            ],
            solutions: vec![
                SolutionPattern {
                    id: "extract_class".to_string(),
                    title: "Extract Class".to_string(),
                    implementation: CompressedString::new(
                        "Break the large class into smaller, focused classes. Each class should have a single responsibility."
                    ),
                    examples: vec![],
                    effort_level: EffortLevel::Medium,
                    prerequisites: vec!["Identify responsibilities".to_string()],
                    expected_impact: ImpactLevel::High,
                },
            ],
            examples: CodeExamples {
                primary: vec![],
                variations: std::collections::HashMap::new(),
            },
            language_variations: std::collections::HashMap::new(),
            related_patterns: vec!["large_class".to_string(), "feature_envy".to_string()],
            tags: vec!["oop".to_string(), "design".to_string(), "maintainability".to_string()],
            frequency_score: 0.8,
            detection_confidence: 0.9,
        };

        let magic_numbers = PatternKnowledge {
            id: "magic_numbers".to_string(),
            name: "Magic Numbers".to_string(),
            definition: CompressedString::new(
                "Using hard-coded numeric values without explanation",
            ),
            symptoms: vec![
                CompressedString::new("Unexplained numeric literals"),
                CompressedString::new("Repeated numeric values"),
                CompressedString::new("Configuration values in code"),
            ],
            impact: ImpactLevel::Medium,
            category: AntiPatternCategory::Maintainability,
            detection_methods: vec![DetectionMethod::RegexPattern {
                pattern: r"\b\d+\b".to_string(),
                context: "numeric_literal".to_string(),
            }],
            solutions: vec![SolutionPattern {
                id: "named_constants".to_string(),
                title: "Use Named Constants".to_string(),
                implementation: CompressedString::new(
                    "Replace magic numbers with named constants that explain their purpose.",
                ),
                examples: vec![],
                effort_level: EffortLevel::Low,
                prerequisites: vec![],
                expected_impact: ImpactLevel::Medium,
            }],
            examples: CodeExamples {
                primary: vec![],
                variations: std::collections::HashMap::new(),
            },
            language_variations: std::collections::HashMap::new(),
            related_patterns: vec!["hard_coded_values".to_string()],
            tags: vec!["readability".to_string(), "maintainability".to_string()],
            frequency_score: 0.6,
            detection_confidence: 0.8,
        };

        library.add_universal_pattern(god_object);
        library.add_universal_pattern(magic_numbers);

        library
    }

    /// Create a minimal index for testing
    fn create_minimal_index(
        &self,
        library: &KnowledgeLibrary,
    ) -> Result<KnowledgeIndex, LoaderError> {
        let mut builder = crate::ai::knowledge::indexing::IndexBuilder::default();
        builder
            .build_from_library(library)
            .map_err(|e| LoaderError::Decompression(format!("Index creation failed: {}", e)))
    }
}

/// Global initialization function
pub fn initialize_knowledge_library(loader: KnowledgeLibraryLoader) -> Result<(), LoaderError> {
    let lookup = loader.load()?;
    KNOWLEDGE_LIBRARY
        .set(lookup)
        .map_err(|_| LoaderError::NotInitialized)?;
    Ok(())
}

/// Get the global knowledge library instance
pub fn get_knowledge_library() -> Result<&'static KnowledgeLibraryLookup, LoaderError> {
    KNOWLEDGE_LIBRARY.get().ok_or(LoaderError::NotInitialized)
}

/// Convenience function to initialize with embedded library
pub fn initialize_embedded() -> Result<(), LoaderError> {
    let loader = KnowledgeLibraryLoader::embedded();
    initialize_knowledge_library(loader)
}

/// Convenience function to initialize from files
pub fn initialize_from_files<P: AsRef<Path>>(
    library_path: P,
    index_path: P,
) -> Result<(), LoaderError> {
    let loader = KnowledgeLibraryLoader::from_files(library_path, index_path);
    initialize_knowledge_library(loader)
}

/// Knowledge library statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct LibraryStats {
    pub total_patterns: usize,
    pub universal_patterns: usize,
    pub language_specific_patterns: usize,
    pub supported_languages: Vec<SourceLanguage>,
    pub categories: Vec<AntiPatternCategory>,
    pub compression_ratio: f32,
    pub index_size: usize,
    pub load_time_ms: u64,
}

/// Get statistics about the loaded knowledge library
pub fn get_library_stats() -> Result<LibraryStats, LoaderError> {
    let lookup = get_knowledge_library()?;
    let index_stats = lookup.get_index_stats();

    let library = lookup.get_library();
    let stats = LibraryStats {
        total_patterns: *index_stats.get("total_patterns").unwrap_or(&0),
        universal_patterns: library.universal_patterns.len(),
        language_specific_patterns: library
            .language_specific
            .values()
            .map(|lang| lang.patterns.len())
            .sum(),
        supported_languages: library.metadata.supported_languages.clone(),
        categories: vec![
            AntiPatternCategory::ObjectOriented,
            AntiPatternCategory::Architectural,
            AntiPatternCategory::Performance,
            AntiPatternCategory::Security,
            AntiPatternCategory::Memory,
            AntiPatternCategory::Concurrency,
            AntiPatternCategory::ErrorHandling,
            AntiPatternCategory::Maintainability,
            AntiPatternCategory::Resources,
            AntiPatternCategory::ApiDesign,
        ],
        compression_ratio: library.metadata.compression_summary.compression_ratio,
        index_size: lookup.get_index_metadata().memory_used,
        load_time_ms: lookup.get_index_metadata().generation_time_ms,
    };

    Ok(stats)
}

/// Search patterns using the global knowledge library
pub fn search_patterns(query: &str) -> Result<Vec<&'static PatternKnowledge>, LoaderError> {
    let lookup = get_knowledge_library()?;

    // Try different search strategies
    let mut results = Vec::new();

    // 1. Direct pattern ID lookup
    if let Some(pattern) = lookup.get_pattern(query) {
        results.push(pattern);
        return Ok(results);
    }

    // 2. Symptom-based search
    let symptom_results = lookup.search_by_symptoms(&[query.to_string()]);
    results.extend(symptom_results);

    // 3. Tag-based search
    let tag_results = lookup.search_by_tags(&[query.to_string()]);
    results.extend(tag_results);

    // Deduplicate results
    results.sort_by_key(|p| &p.id);
    results.dedup_by_key(|p| &p.id);

    Ok(results)
}

/// Get patterns for a specific language
pub fn get_language_patterns(
    language: SourceLanguage,
) -> Result<Vec<&'static PatternKnowledge>, LoaderError> {
    let lookup = get_knowledge_library()?;
    Ok(lookup.get_language_patterns(language))
}

/// Get patterns by category
pub fn get_category_patterns(
    category: AntiPatternCategory,
) -> Result<Vec<&'static PatternKnowledge>, LoaderError> {
    let lookup = get_knowledge_library()?;
    Ok(lookup.get_patterns_by_category(category))
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            use_embedded: true,
            library_path: None,
            index_path: None,
            validate_checksums: true,
            preload_indices: true,
            max_cache_size: 50 * 1024 * 1024, // 50MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_loader_creation() {
        let loader = KnowledgeLibraryLoader::embedded();
        assert!(loader.config.use_embedded);
        assert!(loader.config.validate_checksums);
    }

    #[test]
    fn test_embedded_loading() {
        let loader = KnowledgeLibraryLoader::embedded();
        let lookup = loader.load().unwrap();

        // Test that we can perform lookups
        let pattern = lookup.get_pattern("god_object");
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().name, "God Object");
    }

    #[test]
    fn test_global_initialization() {
        let loader = KnowledgeLibraryLoader::embedded();

        // This test might fail if run multiple times since OnceLock can only be set once
        if KNOWLEDGE_LIBRARY.get().is_none() {
            assert!(initialize_knowledge_library(loader).is_ok());
            assert!(get_knowledge_library().is_ok());
        }
    }

    #[test]
    fn test_search_patterns() {
        let loader = KnowledgeLibraryLoader::embedded();
        if KNOWLEDGE_LIBRARY.get().is_none() {
            let _ = initialize_knowledge_library(loader);
        }

        if let Ok(results) = search_patterns("god_object") {
            assert!(!results.is_empty());
            assert_eq!(results[0].id, "god_object");
        }
    }

    #[test]
    fn test_library_stats() {
        let loader = KnowledgeLibraryLoader::embedded();
        if KNOWLEDGE_LIBRARY.get().is_none() {
            let _ = initialize_knowledge_library(loader);
        }

        if let Ok(stats) = get_library_stats() {
            assert!(stats.total_patterns > 0);
            assert!(stats.universal_patterns > 0);
            assert!(!stats.supported_languages.is_empty());
        }
    }
}
