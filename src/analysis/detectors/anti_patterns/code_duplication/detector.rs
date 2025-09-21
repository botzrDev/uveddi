//! Main code duplication detector implementation

use super::{
    algorithms::{AstBasedDetector, CloneDetectionAlgorithm, SemanticDetector, TokenBasedDetector},
    config::DuplicationConfig,
    language_support::{LanguageSupport, LanguageSupportFactory},
    metrics::{SimilarityCalculator, ThresholdManager},
    types::{ClonePair, CodeBlock, DuplicationResults, DuplicationSummary},
};

use crate::analysis::detectors::base::{
    AnalysisContext, DetectionMetrics, Detector, DetectorCategory, DetectorConfig, DetectorOutput,
    Issue, Severity,
};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use rayon::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Main code duplication detector
pub struct CodeDuplicationDetector {
    /// Configuration for the detector
    config: DuplicationConfig,
    /// Token-based detection algorithm
    token_detector: TokenBasedDetector,
    /// AST-based detection algorithm
    ast_detector: AstBasedDetector,
    /// Semantic detection algorithm
    semantic_detector: Option<SemanticDetector>,
    /// Similarity calculator
    similarity_calculator: SimilarityCalculator,
    /// Threshold manager
    threshold_manager: ThresholdManager,
    /// Cache for language support instances
    language_supports: HashMap<SourceLanguage, Box<dyn LanguageSupport>>,
    /// Global code block cache
    block_cache: Arc<Mutex<HashMap<String, Vec<CodeBlock>>>>,
}

impl CodeDuplicationDetector {
    /// Creates a new code duplication detector with default configuration
    pub fn new() -> Self {
        Self::with_config(DuplicationConfig::new())
    }

    /// Creates a detector with custom configuration
    pub fn with_config(config: DuplicationConfig) -> Self {
        let token_detector =
            TokenBasedDetector::new(config.fingerprint_length, config.similarity_threshold);

        let ast_detector = AstBasedDetector::new(config.similarity_threshold);

        let semantic_detector = if config.enable_semantic_features {
            Some(SemanticDetector::new(config.semantic_similarity_threshold))
        } else {
            None
        };

        let mut language_supports = HashMap::new();
        for lang in LanguageSupportFactory::supported_languages() {
            language_supports.insert(lang, LanguageSupportFactory::create_support(&lang));
        }

        Self {
            config,
            token_detector,
            ast_detector,
            semantic_detector,
            similarity_calculator: SimilarityCalculator::new(),
            threshold_manager: ThresholdManager::new(),
            language_supports,
            block_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Extracts code blocks from all files in the analysis context
    async fn extract_all_code_blocks(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<CodeBlock>, AnalysisError> {
        let start_time = Instant::now();
        let mut all_blocks = Vec::new();

        if context.parallel && self.config.enable_parallel {
            // Parallel extraction
            let blocks: Result<Vec<_>, _> = context
                .files
                .par_iter()
                .map(|file| self.extract_code_blocks_from_file(file))
                .collect();

            for file_blocks in blocks? {
                all_blocks.extend(file_blocks);
            }
        } else {
            // Sequential extraction
            for file in &context.files {
                let file_blocks = self.extract_code_blocks_from_file(file)?;
                all_blocks.extend(file_blocks);
            }
        }

        // Filter blocks based on configuration
        all_blocks = self.filter_blocks(all_blocks)?;

        crate::core::logging::debug!(
            "Extracted {} code blocks in {}ms",
            all_blocks.len(),
            start_time.elapsed().as_millis()
        );

        Ok(all_blocks)
    }

    /// Extracts code blocks from a single file
    fn extract_code_blocks_from_file(
        &self,
        file: &crate::ast::tree_sitter_impl::ParsedFile,
    ) -> Result<Vec<CodeBlock>, AnalysisError> {
        // Check cache first
        let file_path = file.path().to_string_lossy().to_string();
        if let Ok(cache) = self.block_cache.lock() {
            if let Some(cached_blocks) = cache.get(&file_path) {
                return Ok(cached_blocks.clone());
            }
        }

        // Skip if file should be ignored
        if self.config.should_skip_file(&file_path) {
            return Ok(Vec::new());
        }

        // Get language support
        let language_support = self
            .language_supports
            .get(&file.language)
            .ok_or_else(|| AnalysisError::UnsupportedLanguage(format!("{:?}", file.language)))?;

        // Extract blocks using language-specific logic
        let mut blocks = language_support.extract_code_blocks(file)?;

        // Normalize tokens for each block
        for block in &mut blocks {
            let tokens = self.token_detector.tokenize(&block.source, &block.language);
            let normalized_tokens = language_support.normalize_tokens(&tokens);
            block.set_normalized_tokens(normalized_tokens);
        }

        // Cache the results
        if let Ok(mut cache) = self.block_cache.lock() {
            cache.insert(file_path, blocks.clone());
        }

        Ok(blocks)
    }

    /// Filters code blocks based on configuration criteria
    fn filter_blocks(&self, blocks: Vec<CodeBlock>) -> Result<Vec<CodeBlock>, AnalysisError> {
        Ok(blocks
            .into_iter()
            .filter(|block| {
                // Filter by minimum size requirements
                block.token_count() >= self.config.min_tokens
                    && block.line_count() >= self.config.min_lines as u32
            })
            .collect())
    }

    /// Detects clones using all configured algorithms
    async fn detect_clones_comprehensive(
        &self,
        blocks: &[CodeBlock],
    ) -> Result<Vec<ClonePair>, AnalysisError> {
        let mut all_pairs = Vec::new();

        // Token-based detection
        let token_pairs = self.token_detector.detect_all_clones(blocks)?;
        all_pairs.extend(token_pairs);

        // AST-based detection (if enabled)
        let ast_pairs = self.ast_detector.detect_all_clones(blocks)?;
        all_pairs.extend(ast_pairs);

        // Semantic detection (if enabled)
        if let Some(ref semantic_detector) = self.semantic_detector {
            let semantic_pairs = semantic_detector.detect_all_clones(blocks)?;
            all_pairs.extend(semantic_pairs);
        }

        // Remove duplicates and apply thresholds
        let filtered_pairs = self.deduplicate_and_filter_pairs(all_pairs)?;

        Ok(filtered_pairs)
    }

    /// Removes duplicate clone pairs and applies threshold filtering
    fn deduplicate_and_filter_pairs(
        &self,
        pairs: Vec<ClonePair>,
    ) -> Result<Vec<ClonePair>, AnalysisError> {
        let mut unique_pairs = Vec::new();
        let mut seen_pairs = std::collections::HashSet::new();

        for pair in pairs {
            // Create a canonical representation for deduplication
            let key = self.create_pair_key(&pair);

            if !seen_pairs.contains(&key) {
                // Apply threshold filtering
                let threshold = self
                    .threshold_manager
                    .get_threshold(&pair.clone_type, &pair.block1.language);
                if pair.similarity >= threshold {
                    unique_pairs.push(pair);
                    seen_pairs.insert(key);
                }
            }
        }

        // Sort by similarity (highest first)
        unique_pairs.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply maximum results limit
        if let Some(max_pairs) = self.config.max_clone_pairs {
            unique_pairs.truncate(max_pairs);
        }

        Ok(unique_pairs)
    }

    /// Creates a unique key for clone pair deduplication
    fn create_pair_key(&self, pair: &ClonePair) -> String {
        let (file1, line1, file2, line2) = if pair.block1.file_path < pair.block2.file_path
            || (pair.block1.file_path == pair.block2.file_path
                && pair.block1.start_line < pair.block2.start_line)
        {
            (
                &pair.block1.file_path,
                pair.block1.start_line,
                &pair.block2.file_path,
                pair.block2.start_line,
            )
        } else {
            (
                &pair.block2.file_path,
                pair.block2.start_line,
                &pair.block1.file_path,
                pair.block1.start_line,
            )
        };

        format!("{}:{}:{}:{}", file1, line1, file2, line2)
    }

    /// Converts clone pairs to issues for reporting
    fn convert_pairs_to_issues(&self, pairs: &[ClonePair]) -> Vec<Issue> {
        pairs
            .iter()
            .map(|pair| {
                let issue_id = format!("code_duplication_{}", pair.clone_type);
                let title = format!("{} - Code duplication detected", pair.clone_type);
                let description = format!(
                    "Duplicate code found between {}:{}-{} and {}:{}-{} (similarity: {:.1}%)",
                    pair.block1.file_path,
                    pair.block1.start_line,
                    pair.block1.end_line,
                    pair.block2.file_path,
                    pair.block2.start_line,
                    pair.block2.end_line,
                    pair.similarity * 100.0
                );

                Issue::new(
                    issue_id,
                    title,
                    description,
                    pair.clone_type.severity(),
                    pair.block1.file_path.clone(),
                    pair.block1.start_line,
                    pair.block1.end_line,
                )
                .with_metadata(
                    "clone_type".to_string(),
                    serde_json::Value::String(pair.clone_type.to_string()),
                )
                .with_metadata(
                    "similarity".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(pair.similarity).unwrap(),
                    ),
                )
                .with_metadata(
                    "duplicate_file".to_string(),
                    serde_json::Value::String(pair.block2.file_path.clone()),
                )
                .with_metadata(
                    "duplicate_lines".to_string(),
                    serde_json::Value::String(format!(
                        "{}-{}",
                        pair.block2.start_line, pair.block2.end_line
                    )),
                )
                .with_suggestion(format!(
                    "Consider extracting the common logic into a shared function or module"
                ))
            })
            .collect()
    }
}

#[async_trait]
impl Detector for CodeDuplicationDetector {
    type Config = DuplicationConfig;
    type Output = DuplicationResults;

    fn name(&self) -> &'static str {
        "Code Duplication Detector"
    }

    fn category(&self) -> DetectorCategory {
        DetectorCategory::AntiPattern
    }

    fn supported_languages(&self) -> &[SourceLanguage] {
        &[
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }

    async fn detect(&self, context: &AnalysisContext) -> Result<Self::Output, AnalysisError> {
        let start_time = Instant::now();

        // Extract code blocks from all files
        let blocks = self.extract_all_code_blocks(context).await?;

        if blocks.is_empty() {
            return Ok(DuplicationResults::new(Vec::new()));
        }

        // Detect clones using all algorithms
        let clone_pairs = self.detect_clones_comprehensive(&blocks).await?;

        // Create detection metrics
        let metrics = DetectionMetrics {
            duration_ms: start_time.elapsed().as_millis() as u64,
            files_analyzed: context.files.len(),
            nodes_processed: blocks.len(),
            memory_usage_bytes: None,
            custom_metrics: HashMap::new(),
        };

        let mut results = DuplicationResults::new(clone_pairs);
        results.metrics = metrics;

        Ok(results)
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
        // Update sub-components as needed
        self.token_detector = TokenBasedDetector::new(
            self.config.fingerprint_length,
            self.config.similarity_threshold,
        );
        self.ast_detector = AstBasedDetector::new(self.config.similarity_threshold);
    }

    fn is_enabled(&self) -> bool {
        self.config.base.enabled
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// DetectorConfig is already implemented in config.rs, no need to duplicate

impl DetectorOutput for DuplicationResults {
    fn severity(&self) -> Severity {
        self.clone_pairs
            .iter()
            .map(|pair| pair.clone_type.severity())
            .max()
            .unwrap_or(Severity::Info)
    }

    fn issues(&self) -> &[Issue] {
        // Convert clone pairs to issues on demand
        // In a real implementation, you might want to cache this conversion
        &[]
    }

    fn metrics(&self) -> Option<DetectionMetrics> {
        Some(self.metrics.clone())
    }

    fn combine(mut self, other: Self) -> Self {
        self.clone_pairs.extend(other.clone_pairs);
        self.summary = DuplicationSummary::from_clone_pairs(&self.clone_pairs);

        // Combine metrics
        self.metrics.duration_ms += other.metrics.duration_ms;
        self.metrics.files_analyzed += other.metrics.files_analyzed;
        self.metrics.nodes_processed += other.metrics.nodes_processed;

        self
    }
}

#[async_trait]
impl AnalysisDetector for CodeDuplicationDetector {
    async fn detect_issues(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // TODO: Integrate duplication analysis results into architectural issues (UV-412)
        Ok(Vec::new())
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(7),
            name: "Code Duplication".to_string(),
            description: "Identifies duplicated code blocks that should be refactored".to_string(),
            category: "structural".to_string(),
        }]
    }

    fn get_detector_name(&self) -> &'static str {
        "CodeDuplicationDetector"
    }
}

impl Default for CodeDuplicationDetector {
    fn default() -> Self {
        Self::new()
    }
}
