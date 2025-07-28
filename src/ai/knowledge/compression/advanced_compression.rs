//! Advanced Compression System for AI Knowledge Library
//!
//! This module implements sophisticated compression algorithms and optimization
//! techniques to achieve the epic's ambitious <10MB compressed size target
//! while maintaining fast decompression and lookup performance.
//!
//! Based on research findings from docs/compression_research.md:
//! - Zstandard provides optimal balance (70-75% compression, 1000+ MB/s decompression)
//! - Dictionary training enables 10-30% additional compression gains
//! - Level 9-12 provides optimal balance for CLI applications

use crate::ai::knowledge::schema::*;
use zstd::{Encoder, Decoder};
use std::collections::HashMap;
use std::io::{Read, Write};

/// Advanced compression system with multi-stage optimization
pub struct AdvancedCompressionSystem {
    /// Compression configuration
    config: CompressionConfig,
    /// Dictionary trainer for domain-specific compression
    dictionary_trainer: DomainDictionaryTrainer,
    /// Compression analytics
    analytics: CompressionAnalytics,
    /// Performance metrics
    metrics: CompressionMetrics,
}

/// Advanced compression configuration (research-backed)
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Zstandard compression level (9-12 per research recommendation)
    pub zstd_level: i32,
    /// Enable dictionary training (research: 10-30% additional gains)
    pub enable_dictionary_training: bool,
    /// Dictionary size in bytes (research-optimized)
    pub dictionary_size: usize,
    /// Enable content deduplication
    pub enable_deduplication: bool,
    /// Enable semantic compression
    pub enable_semantic_compression: bool,
    /// Enable pattern-specific optimization
    pub enable_pattern_optimization: bool,
    /// Target compression ratio (research: 70-75% achievable)
    pub target_compression_ratio: f32,
}

/// Domain-specific dictionary trainer (research-guided implementation)
pub struct DomainDictionaryTrainer {
    /// Programming language keywords (research: improves compression 2-6%)
    language_keywords: HashMap<SourceLanguage, Vec<String>>,
    /// Framework-specific terms
    framework_terms: HashMap<String, Vec<String>>,
    /// Anti-pattern terminology
    pattern_terminology: Vec<String>,
    /// Code pattern samples
    code_samples: Vec<String>,
}

impl AdvancedCompressionSystem {
    /// Create new advanced compression system
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            dictionary_trainer: DomainDictionaryTrainer::new(),
            analytics: CompressionAnalytics::new(),
            metrics: CompressionMetrics::new(),
        }
    }

    /// Compress knowledge library with advanced optimization
    /// Research target: 70-75% compression ratio, <10MB final size
    pub fn compress_knowledge_library(
        &mut self,
        library: &KnowledgeLibrary,
    ) -> Result<CompressedKnowledgeLibrary, CompressionError> {
        let start_time = std::time::Instant::now();

        // 1. Pre-compression optimization
        let optimized_library = self.optimize_for_compression(library)?;

        // 2. Train domain-specific dictionary (research: 10-30% additional gains)
        let dictionary = if self.config.enable_dictionary_training {
            Some(self.train_domain_dictionary(&optimized_library)?)
        } else {
            None
        };

        // 3. Apply semantic compression
        let semantically_compressed = if self.config.enable_semantic_compression {
            self.apply_semantic_compression(&optimized_library)?
        } else {
            optimized_library
        };

        // 4. Apply content deduplication
        let deduplicated = if self.config.enable_deduplication {
            self.apply_content_deduplication(&semantically_compressed)?
        } else {
            semantically_compressed
        };

        // 5. Apply zstd compression with dictionary (research-backed approach)
        let compressed_data = self.apply_zstd_compression(&deduplicated, dictionary.as_ref())?;

        // 6. Generate compression metadata
        let compression_metadata = self.generate_compression_metadata(
            library,
            &compressed_data,
            &dictionary,
        )?;

        // 7. Validate compression targets (epic requirements)
        self.validate_compression_targets(&compressed_data, &compression_metadata)?;

        // 8. Update metrics
        self.metrics.record_compression_time(start_time.elapsed());
        self.metrics.record_compression_ratio(
            library.calculate_uncompressed_size(),
            compressed_data.len(),
        );

        Ok(CompressedKnowledgeLibrary {
            compressed_data,
            compression_metadata,
            dictionary,
            decompression_indices: self.generate_decompression_indices(&deduplicated)?,
        })
    }

    /// Train domain-specific compression dictionary (research implementation)
    fn train_domain_dictionary(
        &mut self,
        library: &OptimizedKnowledgeLibrary,
    ) -> Result<Vec<u8>, CompressionError> {
        // 1. Extract training samples (research: programming keywords, API patterns)
        let mut training_samples = Vec::new();

        // Add programming language keywords
        for (language, keywords) in &self.dictionary_trainer.language_keywords {
            training_samples.extend(keywords.iter().map(|k| k.as_bytes().to_vec()));
        }

        // Add framework-specific terms
        for (framework, terms) in &self.dictionary_trainer.framework_terms {
            training_samples.extend(terms.iter().map(|t| t.as_bytes().to_vec()));
        }

        // Add anti-pattern terminology
        training_samples.extend(
            self.dictionary_trainer.pattern_terminology
                .iter()
                .map(|t| t.as_bytes().to_vec())
        );

        // Add actual content samples from library
        let content_samples = self.extract_content_samples(library)?;
        training_samples.extend(content_samples);

        // 2. Train zstd dictionary (research: custom dictionaries improve 2-6%)
        let dictionary = zstd::dict::from_samples(&training_samples, self.config.dictionary_size)
            .map_err(|e| CompressionError::DictionaryTraining(e.to_string()))?;

        // 3. Validate dictionary effectiveness
        self.validate_dictionary_effectiveness(&dictionary, library)?;

        Ok(dictionary)
    }

    /// Apply zstd compression with trained dictionary (research-backed)
    fn apply_zstd_compression(
        &self,
        library: &OptimizedKnowledgeLibrary,
        dictionary: Option<&[u8]>,
    ) -> Result<Vec<u8>, CompressionError> {
        let serialized = library.serialize_optimized()?;

        // Use research-recommended compression level (9-12 for balance)
        let mut encoder = if let Some(dict) = dictionary {
            Encoder::with_dictionary(Vec::new(), self.config.zstd_level, dict)
                .map_err(|e| CompressionError::Compression(e.to_string()))?
        } else {
            Encoder::new(Vec::new(), self.config.zstd_level)
                .map_err(|e| CompressionError::Compression(e.to_string()))?
        };

        encoder.write_all(&serialized)
            .map_err(|e| CompressionError::Compression(e.to_string()))?;

        let compressed = encoder.finish()
            .map_err(|e| CompressionError::Compression(e.to_string()))?;

        Ok(compressed)
    }

    /// Validate compression meets epic targets
    fn validate_compression_targets(
        &self,
        compressed_data: &[u8],
        metadata: &CompressionMetadata,
    ) -> Result<(), CompressionError> {
        // Epic target: <10MB compressed size
        if compressed_data.len() > 10 * 1024 * 1024 {
            return Err(CompressionError::TargetNotMet {
                target: "10MB compressed size".to_string(),
                actual: format!("{}MB", compressed_data.len() / (1024 * 1024)),
            });
        }

        // Research target: ≥70% compression ratio
        if metadata.compression_ratio < 0.70 {
            return Err(CompressionError::TargetNotMet {
                target: "70% compression ratio".to_string(),
                actual: format!("{:.1}%", metadata.compression_ratio * 100.0),
            });
        }

        Ok(())
    }
}

/// Compressed knowledge library with optimization metadata
#[derive(Debug, Clone)]
pub struct CompressedKnowledgeLibrary {
    /// Compressed binary data
    pub compressed_data: Vec<u8>,
    /// Compression metadata and statistics
    pub compression_metadata: CompressionMetadata,
    /// Trained compression dictionary
    pub dictionary: Option<Vec<u8>>,
    /// Indices for fast decompression (research: PHF for O(1) lookups)
    pub decompression_indices: DecompressionIndices,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            zstd_level: 9,                    // Research: optimal balance
            enable_dictionary_training: true, // Research: 10-30% additional gains
            dictionary_size: 64 * 1024,      // 64KB dictionary
            enable_deduplication: true,
            enable_semantic_compression: true,
            enable_pattern_optimization: true,
            target_compression_ratio: 0.75,  // Research: 70-75% achievable
        }
    }
}
