//! Code Duplication anti-pattern detector
//!
//! This detector implements a two-stage approach for detecting code clones:
//! 1. **Fast Candidate Generation**: Uses token-based hashing (Karp-Rabin algorithm) to quickly identify potential duplicates
//! 2. **AST-based Verification**: Performs structural comparison to confirm clones and classify their types
//!
//! ## Clone Types Detected:
//! - **Type-1**: Exact clones (identical except for whitespace and comments)
//! - **Type-2**: Renamed clones (identical structure, different identifiers/literals)
//! - **Type-3**: Near-miss clones (similar structure with minor modifications)
//!
//! ## Architecture:
//! - Uses Tree-sitter queries to extract function/method boundaries
//! - Implements rolling hash for efficient fingerprinting
//! - Maintains a global index of code blocks across all files
//! - Provides configurable similarity thresholds

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{Query, QueryCursor};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::error::UveddiError;
use async_trait::async_trait;
use crate::core::logging::{debug, info, warn};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
#[cfg(feature = "tree-sitter")]
use tree_sitter::{StreamingIterator, TreeCursor};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{StreamingIterator, TreeCursor};

/// Represents a contiguous block of code extracted for duplication analysis.
///
/// This struct holds all relevant information about a code snippet, including its
/// location, source code, and various representations used for comparison, such as
/// normalized tokens and structural hashes.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// The absolute path to the file containing this code block.
    pub file_path: String,
    /// The starting line number of the block (1-based).
    pub start_line: u32,
    /// The ending line number of the block (1-based).
    pub end_line: u32,
    /// The starting byte offset of the block within the file.
    pub start_byte: usize,
    /// The ending byte offset of the block within the file.
    pub end_byte: usize,
    /// The raw source code of the block.
    pub source: String,
    /// A sequence of normalized tokens used for structural comparison.
    /// In this representation, identifiers and literals may be replaced with placeholders.
    pub normalized_tokens: Vec<String>,
    /// A SHA-256 hash of the normalized token sequence, used for fast equality checks.
    pub structural_hash: String,
    /// The name of the function or method, if the block represents one.
    pub function_name: Option<String>,
    /// The programming language of the source code.
    pub language: SourceLanguage,

    // NEW: Semantic analysis data
    /// Control Flow Graph representation of the code block
    pub cfg: Option<crate::analysis::cfg::ControlFlowGraph>,
    /// Extracted semantic features for advanced analysis
    pub semantic_features: Option<crate::analysis::semantic::SemanticFeatures>,
    /// Hash of the CFG structure for quick comparison
    pub cfg_hash: Option<String>,
}

/// Represents a pair of code blocks that have been identified as duplicates.
///
/// This struct contains the two cloned blocks, their similarity score, and the
/// type of clone detected (e.g., exact, renamed, or near-miss).
#[derive(Debug, Clone)]
pub struct ClonePair {
    /// The first code block in the clone pair.
    pub block1: CodeBlock,
    /// The second code block in the clone pair.
    pub block2: CodeBlock,
    /// A similarity score between 0.0 and 1.0, where 1.0 indicates a perfect match.
    pub similarity: f64,
    /// The classification of the clone type.
    pub clone_type: CloneType,
    /// The number of shared fingerprints between the two blocks, used as a similarity metric.
    pub shared_fingerprints: usize,
}

/// Enumerates the types of code clones based on their structural and syntactic similarity.
#[derive(Debug, Clone, PartialEq)]
pub enum CloneType {
    /// **Type-1 (Exact Clone):** Identical code fragments, except for variations in
    /// whitespace, layout, and comments.
    Type1,
    /// **Type-2 (Renamed Clone):** Structurally and syntactically identical fragments,
    /// except for changes in identifier names and literal values.
    Type2,
    /// **Type-3 (Near-Miss Clone):** Code fragments with further modifications, such as
    /// changed, added, or removed statements, in addition to variations in identifiers,
    /// literals, and layout.
    Type3,
    /// **Type-4 (Semantic Clone):** Code fragments that are functionally equivalent
    /// but may have different syntax and structure. Detected using semantic analysis.
    Type4,
}

/// Configuration for the code duplication detector.
///
/// This struct allows for fine-tuning the sensitivity and performance of the
/// detection algorithm.
///
/// # Examples
///
/// ```
/// use uveddi::analysis::detectors::anti_patterns::code_duplication::DuplicationConfig;
///
/// let config = DuplicationConfig {
///     min_tokens: 30,
///     similarity_threshold: 0.9,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct DuplicationConfig {
    /// The minimum number of tokens a code block must have to be considered for analysis.
    pub min_tokens: usize,
    /// The minimum number of lines a code block must have to be considered.
    pub min_lines: usize,
    /// The similarity threshold (0.0 to 1.0) for classifying Type-3 (near-miss) clones.
    pub similarity_threshold: f64,
    /// The length of the token window used for generating rolling hash fingerprints.
    pub fingerprint_length: usize,
    /// If true, identifiers are replaced with a placeholder during normalization,
    /// enabling the detection of Type-2 clones.
    pub ignore_identifiers: bool,
    /// If true, literals (strings, numbers) are replaced with a placeholder,
    /// also contributing to Type-2 clone detection.
    pub ignore_literals: bool,

    // NEW: Semantic analysis settings
    /// Enable Control Flow Graph (CFG) analysis for semantic clone detection
    pub enable_cfg_analysis: bool,
    /// Enable semantic feature extraction for Type-4 clone detection
    pub enable_semantic_features: bool,
    /// Weight for CFG similarity in the overall similarity score
    pub cfg_similarity_weight: f64,
    /// Threshold for semantic similarity (Type-4 clone detection)
    pub semantic_similarity_threshold: f64,
    /// Number of iterations for Weisfeiler-Lehman kernel
    pub wl_kernel_iterations: usize,
    /// Maximum number of CFG nodes to process (performance limit)
    pub max_cfg_nodes: usize,

    // NEW: Performance Controls for Large Codebases
    /// Maximum number of clone pairs to detect per file (prevents timeout)
    pub max_clone_pairs_per_file: usize,
    /// Maximum number of blocks to process per file
    pub max_blocks_per_file: usize,
    /// Enable fast mode for large codebases (disables expensive analysis)
    pub enable_fast_mode: bool,
    /// Maximum processing time per file in seconds
    pub max_processing_time_seconds: u64,
}

impl Default for DuplicationConfig {
    /// Creates a default configuration optimized for general code duplication detection
    /// with performance controls for production use
    ///
    /// Default values:
    /// - `min_tokens`: 50 (avoid flagging very small code blocks)
    /// - `min_lines`: 5 (minimum 5 lines for meaningful duplication)
    /// - `similarity_threshold`: 0.8 (80% similarity for Type-3 clones)
    /// - `fingerprint_length`: 7 (good balance between precision and recall)
    /// - `ignore_identifiers`: true (enable Type-2 detection)
    /// - `ignore_literals`: true (enable Type-2 detection)
    /// - `enable_cfg_analysis`: false (disabled by default for performance)
    /// - `enable_semantic_features`: false (disabled by default for performance)
    /// - `cfg_similarity_weight`: 0.3 (30% weight for CFG similarity)
    /// - `semantic_similarity_threshold`: 0.75 (75% threshold for Type-4 clones)
    /// - `wl_kernel_iterations`: 3 (3 iterations for WL kernel)
    /// - `max_cfg_nodes`: 1000 (limit CFG size for performance)
    /// - `max_clone_pairs_per_file`: 50 (prevent runaway detection)
    /// - `max_blocks_per_file`: 100 (prevent analysis timeout)
    /// - `enable_fast_mode`: false (enable for large codebases)
    /// - `max_processing_time_seconds`: 30 (timeout per file)
    fn default() -> Self {
        Self {
            min_tokens: 50, // Minimum number of tokens for a code block to be considered
            min_lines: 5,   // Minimum number of lines for a code block
            similarity_threshold: 0.8, // Threshold for Type-3 clone detection
            fingerprint_length: 7, // Length of rolling hash window
            ignore_identifiers: true, // Normalize identifiers for Type-2 detection
            ignore_literals: true, // Normalize literals for Type-2 detection

            // NEW: Conservative defaults for semantic analysis (disabled for performance)
            enable_cfg_analysis: false,  // CHANGED: Disabled by default
            enable_semantic_features: false,  // CHANGED: Disabled by default
            cfg_similarity_weight: 0.3,
            semantic_similarity_threshold: 0.75,
            wl_kernel_iterations: 3,
            max_cfg_nodes: 1000,

            // NEW: Performance controls for production use
            max_clone_pairs_per_file: 50,  // Limit clone pairs per file
            max_blocks_per_file: 100,      // Limit blocks per file
            enable_fast_mode: false,       // Enable for large codebases
            max_processing_time_seconds: 30, // 30 second timeout per file
        }
    }
}

/// Tree-sitter queries for extracting function/method boundaries
const RUST_FUNCTION_QUERY: &str = r#"
(function_item) @function
"#;

/// Tree-sitter query for extracting methods from impl blocks
/// Currently unused but reserved for future impl method extraction
#[allow(dead_code)]
const RUST_IMPL_METHOD_QUERY: &str = r#"
(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name
      body: (block) @body
    ) @function
  )
)
"#;

const PYTHON_FUNCTION_QUERY: &str = r#"
(function_definition
  name: (identifier) @name
  body: (block) @body
) @function
"#;

const JAVASCRIPT_FUNCTION_QUERY: &str = r#"
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body
) @function
"#;

/// A detector for finding duplicated code using a two-stage hybrid approach.
///
/// This detector combines fast, fingerprint-based candidate generation with a more
/// precise AST-based verification to efficiently identify code clones across large
/// codebases. It can detect exact copies, renamed clones, and near-misses.
///
/// # Examples
///
/// ```
/// use uveddi::analysis::detectors::anti_patterns::code_duplication::CodeDuplicationDetector;
///
/// let detector = CodeDuplicationDetector::new();
/// // The detector can then be used with the `AnalysisDetector` trait methods.
/// ```
pub struct CodeDuplicationDetector {
    /// Configuration parameters for tuning the detector's sensitivity.
    pub config: DuplicationConfig,
    /// A global index mapping fingerprints to the code blocks that contain them,
    /// enabling fast lookup of potential clone candidates.
    fingerprint_index: Arc<Mutex<HashMap<String, Vec<CodeBlock>>>>,
    /// A collection of all code blocks analyzed so far, used for cross-file comparison.
    all_blocks: Arc<Mutex<Vec<CodeBlock>>>,
    /// Cache for CFG computations to avoid recomputing expensive control flow graphs
    cfg_cache: Arc<Mutex<HashMap<String, crate::analysis::cfg::ControlFlowGraph>>>,
    /// Cache for semantic features to avoid recomputing expensive feature extraction
    semantic_cache: Arc<Mutex<HashMap<String, crate::analysis::semantic::SemanticFeatures>>>,
}

impl Default for CodeDuplicationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeDuplicationDetector {
    /// Creates a new code duplication detector with the default configuration.
    pub fn new() -> Self {
        Self::with_config(DuplicationConfig::default())
    }

    /// Creates a new code duplication detector with a custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The `DuplicationConfig` to use for the analysis.
    pub fn with_config(config: DuplicationConfig) -> Self {
        Self {
            config,
            fingerprint_index: Arc::new(Mutex::new(HashMap::new())),
            all_blocks: Arc::new(Mutex::new(Vec::new())),
            cfg_cache: Arc::new(Mutex::new(HashMap::new())),
            semantic_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Extracts function and method bodies as `CodeBlock`s from a parsed file.
    ///
    /// This function uses language-specific Tree-sitter queries to identify function
    /// boundaries. It then processes each function, normalizing its tokens and
    /// computing a structural hash for later comparison.
    fn extract_code_blocks(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<CodeBlock>, AnalysisError> {
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(
                "No AST available for file".to_string(),
            ))
        })?;

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_QUERY,
            SourceLanguage::TypeScript => JAVASCRIPT_FUNCTION_QUERY, // UV-XXX: Reuse JavaScript queries for TypeScript
        };

        let query = Query::new(&tree.language(), query_str).map_err(|e| {
            AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(format!(
                "Failed to create query: {e}"
            )))
        })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), parsed_file.source.as_bytes());

        let mut blocks = Vec::new();

        while let Some(match_) = matches.next() {
            if let Some(function_capture) = match_.captures.iter().find(|c| c.index == 0) {
                let function_node = function_capture.node;
                let start_line = function_node.start_position().row as u32 + 1;
                let end_line = function_node.end_position().row as u32 + 1;

                // Skip blocks that are too small
                if end_line - start_line < self.config.min_lines as u32 {
                    continue;
                }

                let src_bytes = parsed_file.source.as_bytes();
                if function_node.end_byte() > src_bytes.len() {
                    warn!(
                        "Function node end_byte ({}) exceeds source length ({}); skipping block",
                        function_node.end_byte(),
                        src_bytes.len()
                    );
                    continue;
                }
                let source = function_node
                    .utf8_text(src_bytes)
                    .map_err(|e| {
                        AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(
                            format!("Failed to extract source: {e}"),
                        ))
                    })?
                    .to_string();

                // Extract function name by finding the identifier child
                let mut function_name = None;
                let mut cursor = function_node.walk();
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        if child.kind() == "identifier" {
                            if let Ok(name) = child.utf8_text(parsed_file.source.as_bytes()) {
                                function_name = Some(name.to_string());
                                break;
                            }
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }

                let normalized_tokens =
                    self.normalize_tokens(&source, parsed_file.language.clone())?;

                // Skip blocks with too few tokens
                if normalized_tokens.len() < self.config.min_tokens {
                    continue;
                }

                let structural_hash = self.compute_structural_hash(&normalized_tokens);

                let block = CodeBlock {
                    file_path: parsed_file.file_path.display().to_string(),
                    start_line,
                    end_line,
                    start_byte: function_node.start_byte(),
                    end_byte: function_node.end_byte(),
                    source,
                    normalized_tokens,
                    structural_hash,
                    function_name,
                    language: parsed_file.language.clone(),

                    // NEW: Initialize semantic analysis fields (will be populated later)
                    cfg: None,
                    semantic_features: None,
                    cfg_hash: None,
                };

                blocks.push(block);
            }
        }

        Ok(blocks)
    }

    /// Normalizes a sequence of tokens for Type-2 clone detection.
    ///
    /// This method performs a simple tokenization of the source code and normalizes
    /// identifiers and literals based on the configuration. This allows the detector
    /// to identify structurally identical code where variable names or values have
    /// been changed.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to tokenize.
    /// * `_language` - The programming language of the source (currently unused).
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of normalized tokens, or an `AnalysisError`.
    fn normalize_tokens(
        &self,
        source: &str,
        _language: SourceLanguage,
    ) -> Result<Vec<String>, AnalysisError> {
        // Simple tokenization - in a production system, this would use a proper lexer
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_string = false;
        let mut in_comment = false;

        for ch in source.chars() {
            match ch {
                // Handle string literals
                '"' | '\'' if !in_comment => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(&current_token));
                        current_token.clear();
                    }
                    in_string = !in_string;
                    // Replace string contents with placeholder if configured
                    if self.config.ignore_literals && in_string {
                        tokens.push("_LIT_".to_string());
                    }
                }
                // Simple comment detection for C-style languages
                '/' if !in_string && !in_comment => {
                    in_comment = true;
                }
                // End of line comment
                '\n' if in_comment => {
                    in_comment = false;
                }
                // Skip content inside strings and comments
                _ if in_string || in_comment => {
                    continue;
                }
                // Token boundary - whitespace
                c if c.is_whitespace() => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(&current_token));
                        current_token.clear();
                    }
                }
                // Part of an identifier or keyword
                c if c.is_alphanumeric() || c == '_' => {
                    current_token.push(c);
                }
                // Operator or punctuation - treat as separate token
                _ => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(&current_token));
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
            }
        }

        // Don't forget the last token
        if !current_token.is_empty() {
            tokens.push(self.normalize_token(&current_token));
        }

        Ok(tokens)
    }

    /// Normalizes a single token based on the detector's configuration.
    ///
    /// This helper function applies normalization rules to an individual token:
    /// - Numeric literals are replaced with `_LIT_` if `ignore_literals` is true.
    /// - Identifiers are replaced with `_ID_` if `ignore_identifiers` is true.
    /// - Other tokens (keywords, operators) are returned as-is.
    fn normalize_token(&self, token: &str) -> String {
        // Check if it's a literal (number)
        if token.chars().all(|c| c.is_numeric() || c == '.') && self.config.ignore_literals {
            return "_LIT_".to_string();
        }

        // Check if it's an identifier (starts with letter or underscore)
        if token
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && self.config.ignore_identifiers
        {
            return "_ID_".to_string();
        }

        token.to_string()
    }

    /// Computes a structural hash for a sequence of normalized tokens.
    ///
    /// This function uses SHA-256 to create a hash of the entire token sequence.
    /// This hash serves as a unique identifier for the structure of a code block,
    /// allowing for fast equality checks of Type-1 and Type-2 clones.
    fn compute_structural_hash(&self, tokens: &[String]) -> String {
        let mut hasher = Sha256::new();
        for token in tokens {
            hasher.update(token.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Generates a series of rolling hash fingerprints for a token sequence.
    ///
    /// This function implements a sliding window (Karp-Rabin) approach to create
    /// multiple fingerprints for a single code block. Each fingerprint represents a
    /// small, contiguous subsequence of tokens, enabling efficient partial matching
    /// and the detection of near-miss (Type-3) clones.
    fn generate_fingerprints(&self, tokens: &[String]) -> Vec<String> {
        let mut fingerprints = Vec::new();
        let window_size = self.config.fingerprint_length;

        if tokens.len() < window_size {
            return fingerprints;
        }

        for i in 0..=tokens.len() - window_size {
            let window = &tokens[i..i + window_size];
            let mut hasher = Sha256::new();
            for token in window {
                hasher.update(token.as_bytes());
            }
            fingerprints.push(format!("{:x}", hasher.finalize()));
        }

        fingerprints
    }

    /// Indexes code blocks by their fingerprints for fast candidate lookup.
    ///
    /// This method builds an inverted index that maps each fingerprint to a list of
    /// code blocks containing it. This index is crucial for the first stage of the
    /// detection process, allowing for efficient identification of potential clone
    /// candidates across the entire codebase.
    fn index_code_blocks(&self, blocks: &[CodeBlock]) -> Result<(), AnalysisError> {
        // UV-150: Use safe_lock_analysis_data for mutex lock error handling
        let mut index = safe_lock_analysis_data(&self.fingerprint_index, "index_code_blocks")?;
        let mut all_blocks = safe_lock_analysis_data(&self.all_blocks, "index_code_blocks")?;

        for block in blocks {
            // Add to global collection
            all_blocks.push(block.clone());

            // Generate fingerprints and add to index
            let fingerprints = self.generate_fingerprints(&block.normalized_tokens);
            for fingerprint in fingerprints {
                index.entry(fingerprint).or_default().push(block.clone());
            }
        }

        Ok(())
    }

    /// Finds potential clone candidates for a given code block using fingerprint matching.
    ///
    /// This function queries the fingerprint index to find other code blocks that share
    /// a significant number of fingerprints with the target block. This is the fast
    /// candidate generation stage of the two-stage detection approach.
    fn find_clone_candidates(&self, block: &CodeBlock) -> Vec<CodeBlock> {
        // UV-150: Use safe_lock_analysis_data for mutex lock error handling
        let index = match safe_lock_analysis_data(&self.fingerprint_index, "find_clone_candidates")
        {
            Ok(guard) => guard,
            Err(_) => return Vec::new(), // Graceful degradation: return no candidates if lock fails
        };
        let fingerprints = self.generate_fingerprints(&block.normalized_tokens);

        let mut candidates = HashMap::new();
        let mut candidate_scores = HashMap::new();

        for fingerprint in &fingerprints {
            if let Some(matching_blocks) = index.get(fingerprint) {
                for candidate in matching_blocks {
                    // Don't match against itself
                    if candidate.file_path == block.file_path
                        && candidate.start_line == block.start_line
                    {
                        continue;
                    }

                    let file_path = &candidate.file_path;
                    let start_line = candidate.start_line;
                    let key = format!("{file_path}:{start_line}");
                    candidates.insert(key.clone(), candidate.clone());
                    *candidate_scores.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Filter candidates based on minimum shared fingerprints
        let min_shared = (fingerprints.len() as f64 * self.config.similarity_threshold) as usize;
        candidates
            .into_iter()
            .filter(|(key, _)| candidate_scores.get(key).unwrap_or(&0) >= &min_shared)
            .map(|(_, candidate)| candidate)
            .collect()
    }

    /// Verifies if two code blocks are a true clone pair and classifies their type.
    ///
    /// This is the second, more precise stage of the detection process. It performs
    /// a detailed comparison of two candidate blocks to confirm if they are clones.
    /// - If their structural hashes match, they are a Type-1 or Type-2 clone.
    /// - Otherwise, their token similarity is calculated to check for Type-3 clones.
    fn verify_clone_pair(&self, block1: &CodeBlock, block2: &CodeBlock) -> Option<ClonePair> {
        // Exact structural match (Type-1 or Type-2)
        if block1.structural_hash == block2.structural_hash {
            let clone_type = if block1.source == block2.source {
                CloneType::Type1
            } else {
                CloneType::Type2
            };

            return Some(ClonePair {
                block1: block1.clone(),
                block2: block2.clone(),
                similarity: 1.0,
                clone_type,
                shared_fingerprints: self.count_shared_fingerprints(block1, block2),
            });
        }

        // Check for Type-3 clones using token similarity
        let similarity =
            self.calculate_token_similarity(&block1.normalized_tokens, &block2.normalized_tokens);
        if similarity >= self.config.similarity_threshold {
            return Some(ClonePair {
                block1: block1.clone(),
                block2: block2.clone(),
                similarity,
                clone_type: CloneType::Type3,
                shared_fingerprints: self.count_shared_fingerprints(block1, block2),
            });
        }

        None
    }

    /// Counts the number of shared fingerprints between two code blocks.
    ///
    /// This metric is used as part of the similarity assessment to help quantify
    /// how much two blocks overlap in structure.
    fn count_shared_fingerprints(&self, block1: &CodeBlock, block2: &CodeBlock) -> usize {
        let fp1: HashSet<_> = self
            .generate_fingerprints(&block1.normalized_tokens)
            .into_iter()
            .collect();
        let fp2: HashSet<_> = self
            .generate_fingerprints(&block2.normalized_tokens)
            .into_iter()
            .collect();
        fp1.intersection(&fp2).count()
    }

    /// Calculates the token-based similarity of two blocks using the Jaccard coefficient.
    ///
    /// This function computes the Jaccard similarity between two sets of tokens, which
    /// is defined as the size of the intersection divided by the size of the union.
    /// It is used to score the similarity of potential Type-3 (near-miss) clones.
    fn calculate_token_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        let set1: HashSet<_> = tokens1.iter().collect();
        let set2: HashSet<_> = tokens2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Converts a list of `ClonePair`s into a list of `ArchitecturalIssue`s.
    ///
    /// This function transforms the raw detection results into the standard format
    /// expected by the analysis engine, assigning severity based on the clone type
    /// and creating a detailed description for each issue.
    fn clone_pairs_to_issues(&self, clone_pairs: &[ClonePair]) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();

        for pair in clone_pairs {
            let severity = match pair.clone_type {
                CloneType::Type1 => "high",
                CloneType::Type2 => "medium",
                CloneType::Type3 => "low",
                CloneType::Type4 => "medium", // Semantic clones are medium priority
            };

            let description = format!(
                "Code duplication detected ({:?}): {:.1}% similarity between {}:{}-{} and {}:{}-{}",
                pair.clone_type,
                pair.similarity * 100.0,
                pair.block1.file_path,
                pair.block1.start_line,
                pair.block1.end_line,
                pair.block2.file_path,
                pair.block2.start_line,
                pair.block2.end_line
            );

            // Extract code snippet for the issue
            let code_snippet = if pair.block1.source.len() > 200 {
                {
                    let source = &pair.block1.source[..200];
                    format!("{source}...")
                }
            } else {
                pair.block1.source.clone()
            };

            let ai_explanation = Some(format!(
                "This code block appears to be duplicated in another location. Consider extracting the common logic into a shared function or module to improve maintainability. The duplicate is located at {}:{}-{} with {:.1}% similarity.",
                pair.block2.file_path,
                pair.block2.start_line,
                pair.block2.end_line,
                pair.similarity * 100.0
            ));

            let mut issue = ArchitecturalIssue::new(
                0,      // analysis_run_id - Will be set by the engine
                1,      // anti_pattern_type_id - Code duplication type ID  
                pair.block1.file_path.clone(), // file_path
                Some(pair.block1.start_line as i32), // line_number
                format!(
                    "Code duplication detected: {} similar lines",
                    pair.block1.end_line - pair.block1.start_line + 1
                ), // message
                "CodeDuplicationDetector".to_string(), // detector_name
                severity.to_string(), // severity
                description, // description
            );
            
            // Set additional fields
            issue.start_line = Some(pair.block1.start_line as i32);
            issue.end_line = Some(pair.block1.end_line as i32);
            issue.code_snippet = Some(code_snippet);
            issue.ai_explanation = ai_explanation;

            issues.push(issue);
        }

        issues
    }
}

/// Helper for safe mutex locking with error propagation (UV-150)
///
/// This function ensures consistent handling of mutex poison errors in analysis operations.
/// Returns a Result with crate::analysis::errors::AnalysisError::ConcurrencyFailure if the lock is poisoned.
fn safe_lock_analysis_data<'a, T>(
    mutex: &'a std::sync::Mutex<T>,
    operation: &'static str,
) -> Result<std::sync::MutexGuard<'a, T>, crate::analysis::errors::AnalysisError> {
    mutex.lock().map_err(|_| {
        crate::analysis::errors::AnalysisError::SymbolResolutionError(format!(
            "Concurrency failure during {} (mutex poisoned). See UV-150 error handling policy.",
            operation
        ))
    })
}

#[async_trait]
impl AnalysisDetector for CodeDuplicationDetector {
    /// Returns the unique name of this detector.
    fn get_detector_name(&self) -> &'static str {
        "CodeDuplicationDetector"
    }

    /// Returns a list of all anti-pattern types this detector can identify.
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Code Duplication".to_string(),
                description: "Identical or similar code blocks that should be refactored into reusable components".to_string(),
                category: "structural".to_string(),
            }
        ]
    }

    /// Detects code duplication issues in a single parsed file.
    ///
    /// This is the main entry point for the detector. It performs the following steps:
    /// 1. Extracts all function-level code blocks from the file.
    /// 2. Enhances blocks with CFG and semantic analysis (if enabled).
    /// 3. Indexes these blocks by their fingerprints for cross-file comparison.
    /// 4. For each block, finds potential clone candidates from the global index.
    /// 5. Verifies each candidate pair to confirm if they are a true clone.
    /// 6. Converts the verified clone pairs into `ArchitecturalIssue`s.
    ///
    /// # Arguments
    ///
    /// * `parsed_file` - The file to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ArchitecturalIssue`s or an `AnalysisError`.
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        use std::time::{Duration, Instant};
        let start_time = Instant::now();
        let timeout = Duration::from_secs(self.config.max_processing_time_seconds);

        info!(
            "Analyzing file for code duplication: {} (timeout: {}s)",
            parsed_file.file_path.display(),
            self.config.max_processing_time_seconds
        );

        // Phase 1: Extract code blocks from the current file
        let mut blocks = self.extract_code_blocks(parsed_file)?;

        // PERFORMANCE: Early exit if too many blocks
        if blocks.len() > self.config.max_blocks_per_file {
            warn!(
                "File has {} blocks, exceeding limit of {}. Using fast mode with subset.",
                blocks.len(),
                self.config.max_blocks_per_file
            );
            blocks.truncate(self.config.max_blocks_per_file);
        }

        // PERFORMANCE: Check timeout before expensive operations
        if start_time.elapsed() > timeout {
            warn!("Code duplication analysis timed out during block extraction");
            return Ok(vec![]);
        }

        // Phase 2: Enhance blocks with CFG and semantic analysis (if enabled and time permits)
        if !self.config.enable_fast_mode && start_time.elapsed() < timeout / 2 {
            // Use parallel processing for large numbers of blocks
            if blocks.len() > 10
                && (self.config.enable_cfg_analysis || self.config.enable_semantic_features)
            {
                debug!("Using parallel processing for {} blocks", blocks.len());
                self.process_blocks_parallel(&mut blocks, parsed_file)?;
            } else {
                // Use sequential processing for small numbers of blocks
                if self.config.enable_cfg_analysis {
                    self.enhance_blocks_with_cfg(&mut blocks, parsed_file)?;
                }
                if self.config.enable_semantic_features {
                    self.enhance_blocks_with_semantic_features(&mut blocks, parsed_file)?;
                }
            }
        } else {
            debug!("Skipping expensive CFG/semantic analysis due to fast mode or time constraints");
        }

        debug!(
            "Extracted {} code blocks from file: {}",
            blocks.len(),
            parsed_file.file_path.display()
        );
        for (i, block) in blocks.iter().enumerate() {
            debug!(
                "Block {}: lines {}-{}, {} tokens, function: {:?}",
                i + 1,
                block.start_line,
                block.end_line,
                block.normalized_tokens.len(),
                block.function_name
            );
        }

        if blocks.is_empty() {
            debug!(
                "No code blocks found in file: {}",
                parsed_file.file_path.display()
            );
            return Ok(vec![]);
        }

        // Index the blocks for future comparisons
        self.index_code_blocks(&blocks)?;

        // PERFORMANCE: Check timeout before clone detection
        if start_time.elapsed() > timeout {
            warn!("Code duplication analysis timed out before clone detection");
            return Ok(vec![]);
        }

        // Find clone pairs - with timeout protection and early exit
        let mut clone_pairs = if blocks.len() > 20 && !self.config.enable_fast_mode {
            debug!("Using parallel clone detection for {} blocks", blocks.len());
            self.find_clone_candidates_parallel_with_timeout(&blocks, timeout - start_time.elapsed())
        } else {
            debug!(
                "Using sequential clone detection for {} blocks",
                blocks.len()
            );
            let mut clone_pairs = Vec::new();
            for block in &blocks {
                // PERFORMANCE: Check timeout during processing
                if start_time.elapsed() > timeout {
                    warn!("Code duplication analysis timed out during clone detection");
                    break;
                }

                // PERFORMANCE: Early exit if we have enough clone pairs
                if clone_pairs.len() >= self.config.max_clone_pairs_per_file {
                    warn!(
                        "Reached maximum clone pairs limit ({}), stopping detection early",
                        self.config.max_clone_pairs_per_file
                    );
                    break;
                }

                let candidates = self.find_clone_candidates(block);
                debug!(
                    "Found {} candidates for block at line {} (pairs found so far: {})",
                    candidates.len(),
                    block.start_line,
                    clone_pairs.len()
                );

                // PERFORMANCE: Limit candidate processing
                let max_candidates = if self.config.enable_fast_mode { 5 } else { candidates.len() };
                for candidate in candidates.into_iter().take(max_candidates) {
                    // Enhanced verification with semantic analysis
                    if let Some(clone_pair) = self.verify_clone_pair_enhanced(block, &candidate) {
                        debug!("Verified clone pair: {} similarity", clone_pair.similarity);
                        clone_pairs.push(clone_pair);
                        
                        // PERFORMANCE: Early exit if we have enough
                        if clone_pairs.len() >= self.config.max_clone_pairs_per_file {
                            break;
                        }
                    }
                }
            }
            clone_pairs
        };

        // Remove duplicate pairs (A-B and B-A)
        // Sort pairs by first block location for consistent ordering
        clone_pairs.sort_by(|a, b| {
            let key1 = format!("{}:{}", a.block1.file_path, a.block1.start_line);
            let key2 = format!("{}:{}", b.block1.file_path, b.block1.start_line);
            key1.cmp(&key2)
        });
        // Remove pairs that represent the same clone relationship
        clone_pairs.dedup_by(|a, b| {
            // Check if this is the same pair in reverse order (A-B vs B-A)
            (a.block1.file_path == b.block2.file_path && a.block1.start_line == b.block2.start_line &&
             a.block2.file_path == b.block1.file_path && a.block2.start_line == b.block1.start_line) ||
            // Check if this is an exact duplicate
            (a.block1.file_path == b.block1.file_path && a.block1.start_line == b.block1.start_line &&
             a.block2.file_path == b.block2.file_path && a.block2.start_line == b.block2.end_line)
        });

        info!(
            "Found {} clone pairs in file: {}",
            clone_pairs.len(),
            parsed_file.file_path.display()
        );

        // Convert to architectural issues
        let issues = self.clone_pairs_to_issues(&clone_pairs);

        Ok(issues)
    }
}

impl CodeDuplicationDetector {
    /// Enhance code blocks with CFG analysis
    fn enhance_blocks_with_cfg(
        &self,
        blocks: &mut [CodeBlock],
        parsed_file: &ParsedFile,
    ) -> Result<(), AnalysisError> {
        use crate::analysis::cfg::CfgBuilder;

        debug!("Enhancing {} blocks with CFG analysis", blocks.len());

        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(
                "No AST available for CFG generation".to_string(),
            ))
        })?;

        for block in blocks.iter_mut() {
            // Skip if CFG nodes would exceed limit
            if block.normalized_tokens.len() > self.config.max_cfg_nodes {
                warn!(
                    "Skipping CFG generation for large block ({}+ tokens)",
                    block.normalized_tokens.len()
                );
                continue;
            }

            // Find the AST node for this block
            if let Some(function_node) =
                self.find_ast_node_for_block(tree.root_node(), block, parsed_file.source.as_bytes())
            {
                let mut cfg_builder = CfgBuilder::new();

                match cfg_builder.build_from_ast(
                    function_node,
                    parsed_file.source.as_str(),
                    parsed_file.language.clone(),
                ) {
                    Ok(cfg) => {
                        // Compute CFG hash for quick comparison
                        let cfg_hash = cfg.compute_structural_hash();
                        block.cfg = Some(cfg);
                        block.cfg_hash = Some(cfg_hash);
                        if let Some(cfg) = block.cfg.as_ref() {
                            debug!(
                                "Generated CFG for block at line {} with {} nodes",
                                block.start_line,
                                cfg.node_count()
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to generate CFG for block at line {}: {}",
                            block.start_line, e
                        );
                        // Continue without CFG - graceful degradation
                    }
                }
            }
        }

        Ok(())
    }

    /// Enhance code blocks with semantic feature extraction
    fn enhance_blocks_with_semantic_features(
        &self,
        blocks: &mut [CodeBlock],
        parsed_file: &ParsedFile,
    ) -> Result<(), AnalysisError> {
        use crate::analysis::semantic::SemanticAnalyzer;

        debug!("Enhancing {} blocks with semantic features", blocks.len());

        let analyzer = SemanticAnalyzer::new(parsed_file.language.clone());
        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(
                "No AST available for semantic analysis".to_string(),
            ))
        })?;

        for block in blocks.iter_mut() {
            // Only extract features if we have a CFG
            if let Some(ref cfg) = block.cfg {
                if let Some(function_node) = self.find_ast_node_for_block(
                    tree.root_node(),
                    block,
                    parsed_file.source.as_bytes(),
                ) {
                    match analyzer.extract_features(cfg, &function_node, parsed_file.source.as_str()) {
                        Ok(features) => {
                            block.semantic_features = Some(features);
                            debug!(
                                "Extracted semantic features for block at line {}",
                                block.start_line
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to extract semantic features for block at line {}: {}",
                                block.start_line, e
                            );
                            // Continue without semantic features - graceful degradation
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Find the AST node corresponding to a code block
    fn find_ast_node_for_block<'a>(
        &self,
        root_node: crate::ast::tree_sitter::Node<'a>,
        block: &CodeBlock,
        source: &[u8],
    ) -> Option<crate::ast::tree_sitter::Node<'a>> {
        let mut cursor = root_node.walk();

        // Simple approach: find node that matches the byte range
        fn find_matching_node<'a>(
            cursor: &mut TreeCursor<'a>,
            start_byte: usize,
            end_byte: usize,
        ) -> Option<crate::ast::tree_sitter::Node<'a>> {
            let node = cursor.node();

            // Check if this node matches our range
            if node.start_byte() == start_byte && node.end_byte() == end_byte {
                return Some(node);
            }

            // Check if this node contains our range
            if node.start_byte() <= start_byte && node.end_byte() >= end_byte {
                // Search children
                if cursor.goto_first_child() {
                    loop {
                        if let Some(found) = find_matching_node(cursor, start_byte, end_byte) {
                            return Some(found);
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }

                // If no child matches exactly, return this node as best match
                return Some(node);
            }

            None
        }

        find_matching_node(&mut cursor, block.start_byte, block.end_byte)
    }

    /// Enhanced clone pair verification with semantic analysis
    fn verify_clone_pair_enhanced(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> Option<ClonePair> {
        // First try existing verification
        if let Some(mut clone_pair) = self.verify_clone_pair(block1, block2) {
            // If we have semantic features, enhance the classification
            if let (Some(ref features1), Some(ref features2)) =
                (&block1.semantic_features, &block2.semantic_features)
            {
                // Use semantic similarity for Type-4 detection
                let semantic_similarity = self.compute_semantic_similarity(features1, features2);

                if semantic_similarity >= self.config.semantic_similarity_threshold {
                    // Check if this should be classified as Type-4
                    if clone_pair.clone_type == CloneType::Type3 && semantic_similarity > 0.8 {
                        clone_pair.clone_type = CloneType::Type4;
                        clone_pair.similarity = semantic_similarity;
                    } else if semantic_similarity > clone_pair.similarity {
                        // Update similarity if semantic analysis gives higher score
                        clone_pair.similarity = semantic_similarity;
                    }
                }
            }

            return Some(clone_pair);
        }

        // If traditional verification failed, try semantic-only detection
        if self.config.enable_semantic_features {
            if let (Some(ref features1), Some(ref features2)) =
                (&block1.semantic_features, &block2.semantic_features)
            {
                let semantic_similarity = self.compute_semantic_similarity(features1, features2);

                if semantic_similarity >= self.config.semantic_similarity_threshold {
                    return Some(ClonePair {
                        block1: block1.clone(),
                        block2: block2.clone(),
                        similarity: semantic_similarity,
                        clone_type: CloneType::Type4,
                        shared_fingerprints: self.count_shared_fingerprints(block1, block2),
                    });
                }
            }
        }

        None
    }

    /// Compute semantic similarity between two feature sets
    fn compute_semantic_similarity(
        &self,
        features1: &crate::analysis::semantic::SemanticFeatures,
        features2: &crate::analysis::semantic::SemanticFeatures,
    ) -> f64 {
        use crate::analysis::semantic::{HybridSimilarityScorer, SimilarityWeights};

        // Create a hybrid scorer with weights from config
        let weights = SimilarityWeights {
            structural: 0.3,
            cfg: self.config.cfg_similarity_weight,
            semantic: 0.25,
            ast_pattern: 0.15,
        };

        // For now, compute a simple feature vector similarity
        // In a full implementation, this would use the HybridSimilarityScorer
        let vec1 = &features1.feature_vector;
        let vec2 = &features2.feature_vector;

        // Compute cosine similarity
        let dot_product = vec1.dot(vec2);
        let norm1 = vec1.dot(vec1).sqrt();
        let norm2 = vec2.dot(vec2).sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Process blocks in parallel for CFG and semantic analysis
    fn process_blocks_parallel(
        &self,
        blocks: &mut [CodeBlock],
        parsed_file: &ParsedFile,
    ) -> Result<(), AnalysisError> {
        if !self.config.enable_cfg_analysis && !self.config.enable_semantic_features {
            return Ok(()); // Nothing to do
        }

        let tree = parsed_file.tree.as_ref().ok_or_else(|| {
            AnalysisError::AntiPatternDetectionError(
                "AST tree missing for parallel processing".to_string(),
            )
        })?;

        // Use a custom parallel iterator that works with mutable references
        blocks
            .par_iter_mut()
            .try_for_each(|block| -> Result<(), AnalysisError> {
                // Find function node for this block
                let function_node =
                    self.find_function_node_for_block_simple(tree, block, parsed_file)?;

                if let Some(node) = function_node {
                    // Process CFG if enabled
                    if self.config.enable_cfg_analysis {
                        if let Ok(Some(cfg)) = self.get_or_compute_cfg(
                            &node,
                            parsed_file.source.as_bytes(),
                            &parsed_file.language,
                        ) {
                            // Generate CFG hash
                            let cfg_hash = cfg.compute_structural_hash();
                            block.cfg = Some(cfg);
                            block.cfg_hash = Some(cfg_hash);
                        }
                    }

                    // Process semantic features if enabled and we have a CFG
                    if self.config.enable_semantic_features && block.cfg.is_some() {
                        if let Some(ref cfg) = block.cfg {
                            if let Ok(Some(features)) = self.get_or_compute_semantic_features(
                                cfg,
                                &node,
                                parsed_file.source.as_str(),
                                &parsed_file.language,
                            ) {
                                block.semantic_features = Some(features);
                            }
                        }
                    }
                }

                Ok(())
            })?;

        Ok(())
    }

    /// Simplified function node finder for parallel processing
    fn find_function_node_for_block_simple<'a>(
        &self,
        tree: &'a crate::ast::tree_sitter::Tree,
        block: &CodeBlock,
        parsed_file: &'a ParsedFile,
    ) -> Result<Option<crate::ast::tree_sitter::Node<'a>>, AnalysisError> {
        #[cfg(feature = "tree-sitter")]
        use crate::ast::tree_sitter::{Query, QueryCursor};

        let language = tree.language();
        let source = parsed_file.source.as_bytes();

        // Use the appropriate query based on language
        let query_str = match parsed_file.language {
            SourceLanguage::Rust => "(function_item name: (identifier) @name body: (block) @body) @function",
            SourceLanguage::Python => "(function_definition name: (identifier) @name body: (block) @body) @function",
            SourceLanguage::JavaScript => "(function_declaration name: (identifier) @name body: (statement_block) @body) @function",
            _ => return Ok(None), // Unsupported language
        };

        let query = Query::new(&language, query_str).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to create function query: {}",
                e
            ))
        })?;

        let mut cursor = QueryCursor::new();

        let mut matches = cursor.matches(&query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            if mat.captures.len() >= 3 {
                let function_node = mat.captures[0].node; // @function
                let start_line = function_node.start_position().row + 1;
                let end_line = function_node.end_position().row + 1;

                // Check if this function overlaps with our block
                if start_line <= block.end_line as usize && end_line >= block.start_line as usize {
                    return Ok(Some(function_node));
                }
            }
        }

        Ok(None)
    }

    /// Process clone candidate verification in parallel
    fn find_clone_candidates_parallel(&self, blocks: &[CodeBlock]) -> Vec<ClonePair> {
        blocks
            .par_iter()
            .flat_map(|block| {
                let candidates = self.find_clone_candidates(block);
                candidates
                    .into_par_iter()
                    .filter_map(|candidate| self.verify_clone_pair_enhanced(block, &candidate))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Process clone candidate verification in parallel with timeout
    fn find_clone_candidates_parallel_with_timeout(&self, blocks: &[CodeBlock], remaining_time: std::time::Duration) -> Vec<ClonePair> {
        use std::time::Instant;
        let start = Instant::now();
        
        // Limit the number of blocks processed in parallel to prevent timeout
        let max_blocks = if remaining_time.as_secs() < 10 { 
            10  // Very limited processing for low time
        } else { 
            blocks.len().min(50)  // Cap at 50 blocks for parallel processing
        };
        
        let limited_blocks = &blocks[..max_blocks.min(blocks.len())];
        
        // Use sequential processing with manual timeout checking for better control
        let mut results = Vec::new();
        for block in limited_blocks {
            // Check timeout before processing each block
            if start.elapsed() >= remaining_time {
                warn!("Parallel clone detection reached timeout with {} pairs found", results.len());
                break;
            }

            let candidates = self.find_clone_candidates(block);
            let limited_candidates = if remaining_time.as_secs() < 5 {
                candidates.into_iter().take(3).collect::<Vec<_>>()  // Very limited for low time
            } else {
                candidates.into_iter().take(10).collect::<Vec<_>>() // Normal limit
            };
            
            let mut block_pairs = 0;
            for candidate in limited_candidates {
                if let Some(clone_pair) = self.verify_clone_pair_enhanced(block, &candidate) {
                    results.push(clone_pair);
                    block_pairs += 1;
                    
                    // Limit pairs per block
                    if block_pairs >= 5 {
                        break;
                    }
                    
                    // Global limit
                    if results.len() >= self.config.max_clone_pairs_per_file {
                        break;
                    }
                }
            }
            
            // Global limit check
            if results.len() >= self.config.max_clone_pairs_per_file {
                break;
            }
        }
            
        if start.elapsed() >= remaining_time {
            warn!("Clone detection reached timeout with {} pairs found", results.len());
        } else {
            debug!("Clone detection completed in {}ms with {} pairs", 
                   start.elapsed().as_millis(), results.len());
        }
        
        results
    }

    /// Generate a cache key for CFG based on function content and language
    fn generate_cfg_cache_key(&self, source: &str, language: &SourceLanguage) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        hasher.update(format!("{:?}", language).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a cache key for semantic features based on CFG hash and function content
    fn generate_semantic_cache_key(&self, cfg_hash: &str, source: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cfg_hash.as_bytes());
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get or compute CFG for a function, using cache when possible
    fn get_or_compute_cfg(
        &self,
        function_node: &crate::ast::tree_sitter::Node,
        source: &[u8],
        language: &SourceLanguage,
    ) -> Result<Option<crate::analysis::cfg::ControlFlowGraph>, AnalysisError> {
        if function_node.end_byte() > source.len() {
            warn!(
                "Function node end_byte ({}) exceeds source length ({}); skipping CFG",
                function_node.end_byte(),
                source.len()
            );
            return Ok(None);
        }
        let function_source = function_node.utf8_text(source).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to extract function text: {}",
                e
            ))
        })?;

        let cache_key = self.generate_cfg_cache_key(function_source, language);

        // Try to get from cache first
        if let Ok(cache) = self.cfg_cache.lock() {
            if let Some(cached_cfg) = cache.get(&cache_key) {
                debug!("CFG cache hit for key: {}", cache_key);
                return Ok(Some(cached_cfg.clone()));
            }
        }

        // Cache miss - compute CFG
        debug!("CFG cache miss for key: {}", cache_key);
        let mut cfg_builder = crate::analysis::cfg::CfgBuilder::new();

        // Use the full file source for CFG building to ensure node byte ranges are valid
        let file_source = std::str::from_utf8(source).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Source not valid UTF-8 for CFG build: {}",
                e
            ))
        })?;

        match cfg_builder.build_from_ast(*function_node, file_source, language.clone()) {
            Ok(cfg) => {
                // Store in cache
                if let Ok(mut cache) = self.cfg_cache.lock() {
                    cache.insert(cache_key, cfg.clone());

                    // Limit cache size to prevent memory issues
                    if cache.len() > 1000 {
                        // Remove oldest entries (simple strategy)
                        let keys_to_remove: Vec<_> = cache.keys().take(100).cloned().collect();
                        for key in keys_to_remove {
                            cache.remove(&key);
                        }
                    }
                }
                Ok(Some(cfg))
            }
            Err(e) => {
                warn!("Failed to build CFG: {}", e);
                Ok(None)
            }
        }
    }

    /// Get or compute semantic features for a function, using cache when possible
    fn get_or_compute_semantic_features(
        &self,
        cfg: &crate::analysis::cfg::ControlFlowGraph,
        function_node: &crate::ast::tree_sitter::Node,
        source: &str,
        language: &SourceLanguage,
    ) -> Result<Option<crate::analysis::semantic::SemanticFeatures>, AnalysisError> {
        let cfg_hash = cfg.compute_structural_hash();
        let function_source = function_node.utf8_text(source.as_bytes()).map_err(|e| {
            AnalysisError::AntiPatternDetectionError(format!(
                "Failed to extract function text: {}",
                e
            ))
        })?;

        let cache_key = self.generate_semantic_cache_key(&cfg_hash, function_source);

        // Try to get from cache first
        if let Ok(cache) = self.semantic_cache.lock() {
            if let Some(cached_features) = cache.get(&cache_key) {
                debug!("Semantic cache hit for key: {}", cache_key);
                return Ok(Some(cached_features.clone()));
            }
        }

        // Cache miss - compute semantic features
        debug!("Semantic cache miss for key: {}", cache_key);
        let analyzer = crate::analysis::semantic::SemanticAnalyzer::new(language.clone());

        match analyzer.extract_features(cfg, function_node, source) {
            Ok(features) => {
                // Store in cache
                if let Ok(mut cache) = self.semantic_cache.lock() {
                    cache.insert(cache_key, features.clone());

                    // Limit cache size to prevent memory issues
                    if cache.len() > 1000 {
                        // Remove oldest entries (simple strategy)
                        let keys_to_remove: Vec<_> = cache.keys().take(100).cloned().collect();
                        for key in keys_to_remove {
                            cache.remove(&key);
                        }
                    }
                }
                Ok(Some(features))
            }
            Err(e) => {
                warn!("Failed to extract semantic features: {}", e);
                Ok(None)
            }
        }
    }

    /// Clear all caches to free memory
    pub fn clear_caches(&self) -> Result<(), AnalysisError> {
        if let Ok(mut cache) = self.cfg_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.semantic_cache.lock() {
            cache.clear();
        }
        debug!("Cleared CFG and semantic caches");
        Ok(())
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cfg_size = self.cfg_cache.lock().map(|c| c.len()).unwrap_or(0);
        let semantic_size = self.semantic_cache.lock().map(|c| c.len()).unwrap_or(0);
        (cfg_size, semantic_size)
    }
}
