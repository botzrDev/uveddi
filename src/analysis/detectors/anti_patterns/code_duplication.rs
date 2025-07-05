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
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tree_sitter::{Query, QueryCursor};

/// Represents a code block extracted for duplication analysis
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Path to the file containing this code block
    pub file_path: String,
    /// Starting line number (1-based)
    pub start_line: u32,
    /// Ending line number (1-based)
    pub end_line: u32,
    /// Starting byte offset in the file
    pub start_byte: usize,
    /// Ending byte offset in the file
    pub end_byte: usize,
    /// Raw source code of the block
    pub source: String,
    /// Normalized tokens for comparison (identifiers/literals may be replaced)
    pub normalized_tokens: Vec<String>,
    /// SHA-256 hash of the normalized token sequence
    pub structural_hash: String,
    /// Name of the function/method if available
    pub function_name: Option<String>,
    /// Programming language of the source code
    pub language: SourceLanguage,
}

/// Represents a detected clone pair
#[derive(Debug, Clone)]
pub struct ClonePair {
    /// First code block in the clone pair
    pub block1: CodeBlock,
    /// Second code block in the clone pair
    pub block2: CodeBlock,
    /// Similarity score between the blocks (0.0 to 1.0)
    pub similarity: f64,
    /// Classification of the clone type
    pub clone_type: CloneType,
    /// Number of shared fingerprints between the blocks
    pub shared_fingerprints: usize,
}

/// Types of code clones based on their structural similarity
#[derive(Debug, Clone, PartialEq)]
pub enum CloneType {
    /// Exact clones - identical code except for whitespace and comments
    Type1,
    /// Renamed clones - identical structure with different identifiers/literals
    Type2,
    /// Near-miss clones - similar structure with minor modifications
    Type3,
}

/// Configuration for the code duplication detector
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
    /// Minimum number of tokens for a code block to be considered for analysis
    pub min_tokens: usize,
    /// Minimum number of lines for a code block to be considered for analysis
    pub min_lines: usize,
    /// Similarity threshold for Type-3 clone detection (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Length of the rolling hash window for fingerprinting
    pub fingerprint_length: usize,
    /// Whether to normalize identifiers (enables Type-2 detection)
    pub ignore_identifiers: bool,
    /// Whether to normalize literals (enables Type-2 detection)
    pub ignore_literals: bool,
}

impl Default for DuplicationConfig {
    /// Creates a default configuration optimized for general code duplication detection
    ///
    /// Default values:
    /// - `min_tokens`: 50 (avoid flagging very small code blocks)
    /// - `min_lines`: 5 (minimum 5 lines for meaningful duplication)
    /// - `similarity_threshold`: 0.8 (80% similarity for Type-3 clones)
    /// - `fingerprint_length`: 7 (good balance between precision and recall)
    /// - `ignore_identifiers`: true (enable Type-2 detection)
    /// - `ignore_literals`: true (enable Type-2 detection)
    fn default() -> Self {
        Self {
            min_tokens: 50, // Minimum number of tokens for a code block to be considered
            min_lines: 5,   // Minimum number of lines for a code block
            similarity_threshold: 0.8, // Threshold for Type-3 clone detection
            fingerprint_length: 7, // Length of rolling hash window
            ignore_identifiers: true, // Normalize identifiers for Type-2 detection
            ignore_literals: true, // Normalize literals for Type-2 detection
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

/// Code Duplication Detector using two-stage hybrid approach
///
/// This detector combines fast fingerprint-based candidate generation with
/// precise AST-based verification to efficiently detect code clones across
/// large codebases.
///
/// # Examples
///
/// ```
/// use uveddi::analysis::detectors::anti_patterns::code_duplication::CodeDuplicationDetector;
///
/// let detector = CodeDuplicationDetector::new();
/// // Use with AnalysisDetector trait methods
/// ```
pub struct CodeDuplicationDetector {
    /// Configuration parameters for the detector
    config: DuplicationConfig,
    /// Global index of fingerprints to code blocks for fast candidate lookup
    fingerprint_index: Arc<Mutex<HashMap<String, Vec<CodeBlock>>>>,
    /// All analyzed code blocks for cross-file comparison
    all_blocks: Arc<Mutex<Vec<CodeBlock>>>,
}

impl Default for CodeDuplicationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeDuplicationDetector {
    /// Creates a new code duplication detector with default configuration
    pub fn new() -> Self {
        Self::with_config(DuplicationConfig::default())
    }

    /// Creates a new code duplication detector with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for the detector
    pub fn with_config(config: DuplicationConfig) -> Self {
        Self {
            config,
            fingerprint_index: Arc::new(Mutex::new(HashMap::new())),
            all_blocks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Extract code blocks from a parsed file using Tree-sitter queries
    fn extract_code_blocks(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<CodeBlock>, AnalysisError> {
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| AnalysisError::Analysis("No AST available for file".to_string()))?;

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_QUERY,
        };

        let query = Query::new(&tree.language(), query_str)
            .map_err(|e| AnalysisError::Analysis(format!("Failed to create query: {e}")))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), parsed_file.source.as_bytes());

        let mut blocks = Vec::new();

        for match_ in matches {
            if let Some(function_capture) = match_.captures.iter().find(|c| c.index == 0) {
                let function_node = function_capture.node;
                let start_line = function_node.start_position().row as u32 + 1;
                let end_line = function_node.end_position().row as u32 + 1;

                // Skip blocks that are too small
                if end_line - start_line < self.config.min_lines as u32 {
                    continue;
                }

                let source = function_node
                    .utf8_text(parsed_file.source.as_bytes())
                    .map_err(|e| {
                        AnalysisError::Analysis(format!("Failed to extract source: {e}"))
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
                    file_path: parsed_file.path.to_string_lossy().to_string(),
                    start_line,
                    end_line,
                    start_byte: function_node.start_byte(),
                    end_byte: function_node.end_byte(),
                    source,
                    normalized_tokens,
                    structural_hash,
                    function_name,
                    language: parsed_file.language.clone(),
                };

                blocks.push(block);
            }
        }

        Ok(blocks)
    }

    /// Normalize tokens for Type-2 clone detection
    ///
    /// This method performs a simple tokenization of the source code and normalizes
    /// identifiers and literals based on the configuration. In a production system,
    /// this would use a proper lexer for more accurate tokenization.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to tokenize
    /// * `_language` - The programming language (currently unused)
    ///
    /// # Returns
    ///
    /// A vector of normalized tokens
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

    /// Normalize a single token based on configuration
    ///
    /// This method applies normalization rules to individual tokens:
    /// - Numeric literals are replaced with "_LIT_" if `ignore_literals` is true
    /// - Identifiers are replaced with "_ID_" if `ignore_identifiers` is true
    /// - Other tokens are returned as-is
    ///
    /// # Arguments
    ///
    /// * `token` - The token to normalize
    ///
    /// # Returns
    ///
    /// The normalized token string
    fn normalize_token(&self, token: &str) -> String {
        // Check if it's a literal (number)
        if token.chars().all(|c| c.is_numeric() || c == '.')
            && self.config.ignore_literals {
                return "_LIT_".to_string();
            }

        // Check if it's an identifier (starts with letter or underscore)
        if token
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && self.config.ignore_identifiers {
                return "_ID_".to_string();
            }

        token.to_string()
    }

    /// Compute structural hash for a normalized token sequence
    ///
    /// Uses SHA-256 to create a hash of the entire token sequence. This is used
    /// for exact matching of Type-1 and Type-2 clones.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The normalized token sequence
    ///
    /// # Returns
    ///
    /// A hexadecimal string representation of the hash
    fn compute_structural_hash(&self, tokens: &[String]) -> String {
        let mut hasher = Sha256::new();
        for token in tokens {
            hasher.update(token.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Generate rolling hash fingerprints for a token sequence (Karp-Rabin algorithm)
    ///
    /// Creates a series of hash fingerprints using a sliding window approach.
    /// Each fingerprint represents a subsequence of tokens, enabling efficient
    /// partial matching for clone detection.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The token sequence to fingerprint
    ///
    /// # Returns
    ///
    /// A vector of fingerprint hashes as hexadecimal strings
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

    /// Index code blocks by their fingerprints for fast candidate generation
    ///
    /// Builds an inverted index mapping fingerprints to code blocks, enabling
    /// efficient lookup of potential clone candidates during analysis.
    ///
    /// # Arguments
    ///
    /// * `blocks` - The code blocks to index
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of the indexing operation
    fn index_code_blocks(&self, blocks: &[CodeBlock]) -> Result<(), AnalysisError> {
        let mut index = self.fingerprint_index.lock().unwrap();
        let mut all_blocks = self.all_blocks.lock().unwrap();

        for block in blocks {
            // Add to global collection
            all_blocks.push(block.clone());

            // Generate fingerprints and add to index
            let fingerprints = self.generate_fingerprints(&block.normalized_tokens);
            for fingerprint in fingerprints {
                index
                    .entry(fingerprint)
                    .or_default()
                    .push(block.clone());
            }
        }

        Ok(())
    }

    /// Find clone candidates using fingerprint matching
    ///
    /// Searches the fingerprint index for code blocks that share a significant
    /// number of fingerprints with the given block. This is the fast candidate
    /// generation stage of the two-stage approach.
    ///
    /// # Arguments
    ///
    /// * `block` - The code block to find candidates for
    ///
    /// # Returns
    ///
    /// A vector of candidate code blocks for clone verification
    fn find_clone_candidates(&self, block: &CodeBlock) -> Vec<CodeBlock> {
        let index = self.fingerprint_index.lock().unwrap();
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

    /// Verify clone candidates using AST structural comparison
    ///
    /// Performs detailed comparison of two code blocks to determine if they
    /// are true clones and classify their type. This is the verification stage
    /// of the two-stage approach.
    ///
    /// # Arguments
    ///
    /// * `block1` - First code block to compare
    /// * `block2` - Second code block to compare
    ///
    /// # Returns
    ///
    /// Some(ClonePair) if the blocks are clones, None otherwise
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

    /// Count shared fingerprints between two blocks
    ///
    /// Calculates the number of common fingerprints between two code blocks,
    /// which is used as a metric for similarity assessment.
    ///
    /// # Arguments
    ///
    /// * `block1` - First code block
    /// * `block2` - Second code block
    ///
    /// # Returns
    ///
    /// The number of shared fingerprints
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

    /// Calculate token-based similarity using Jaccard coefficient
    ///
    /// Computes the Jaccard similarity coefficient between two token sequences,
    /// which is the ratio of intersection to union of the token sets.
    ///
    /// # Arguments
    ///
    /// * `tokens1` - First token sequence
    /// * `tokens2` - Second token sequence
    ///
    /// # Returns
    ///
    /// Similarity score between 0.0 and 1.0
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

    /// Convert clone pairs to architectural issues
    ///
    /// Transforms detected clone pairs into the standard ArchitecturalIssue
    /// format used by the analysis framework.
    ///
    /// # Arguments
    ///
    /// * `clone_pairs` - The detected clone pairs to convert
    ///
    /// # Returns
    ///
    /// A vector of architectural issues representing the clone pairs
    fn clone_pairs_to_issues(&self, clone_pairs: &[ClonePair]) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();

        for pair in clone_pairs {
            let severity = match pair.clone_type {
                CloneType::Type1 => "high",
                CloneType::Type2 => "medium",
                CloneType::Type3 => "low",
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

            let issue = ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0,      // Will be set by the engine
                anti_pattern_type_id: 1, // Code duplication type ID
                file_path: pair.block1.file_path.clone(),
                start_line: Some(pair.block1.start_line as i32),
                end_line: Some(pair.block1.end_line as i32),
                severity: severity.to_string(),
                description,
                code_snippet: Some(code_snippet),
                ai_explanation,
            };

            issues.push(issue);
        }

        issues
    }
}

impl AnalysisDetector for CodeDuplicationDetector {
    fn get_detector_name(&self) -> &'static str {
        "CodeDuplicationDetector"
    }

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

    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        info!(
            "Analyzing file for code duplication: {}",
            parsed_file.path.display()
        );

        // Extract code blocks from the current file
        let blocks = self.extract_code_blocks(parsed_file)?;

        debug!(
            "Extracted {} code blocks from file: {}",
            blocks.len(),
            parsed_file.path.display()
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
                parsed_file.path.display()
            );
            return Ok(vec![]);
        }

        // Index the blocks for future comparisons
        self.index_code_blocks(&blocks)?;

        // Find clone pairs
        let mut clone_pairs = Vec::new();

        for block in &blocks {
            let candidates = self.find_clone_candidates(block);
            debug!(
                "Found {} candidates for block at line {}",
                candidates.len(),
                block.start_line
            );

            for candidate in candidates {
                if let Some(clone_pair) = self.verify_clone_pair(block, &candidate) {
                    debug!("Verified clone pair: {} similarity", clone_pair.similarity);
                    clone_pairs.push(clone_pair);
                }
            }
        }

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
             a.block2.file_path == b.block2.file_path && a.block2.start_line == b.block2.start_line)
        });

        info!(
            "Found {} clone pairs in file: {}",
            clone_pairs.len(),
            parsed_file.path.display()
        );

        // Convert to architectural issues
        let issues = self.clone_pairs_to_issues(&clone_pairs);

        Ok(issues)
    }
}
