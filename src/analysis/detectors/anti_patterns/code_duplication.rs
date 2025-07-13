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
use crate::error::UveddiError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use log::{debug, info};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tree_sitter::{Query, QueryCursor};

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
    config: DuplicationConfig,
    /// A global index mapping fingerprints to the code blocks that contain them,
    /// enabling fast lookup of potential clone candidates.
    fingerprint_index: Arc<Mutex<HashMap<String, Vec<CodeBlock>>>>,
    /// A collection of all code blocks analyzed so far, used for cross-file comparison.
    all_blocks: Arc<Mutex<Vec<CodeBlock>>>,
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
        let tree = parsed_file
            .tree
            .as_ref()
            .ok_or_else(|| {
                UveddiError::AstError(crate::ast::tree_sitter_impl::AstError::Other(
                    "No AST available for file".to_string(),
                ))
            })?;

        let query_str = match parsed_file.language {
            SourceLanguage::Rust => RUST_FUNCTION_QUERY,
            SourceLanguage::Python => PYTHON_FUNCTION_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_FUNCTION_QUERY,
        };

        let query = Query::new(&tree.language(), query_str).map_err(|e| {
            UveddiError::AstError(crate::ast::tree_sitter_impl::AstError::Other(format!(
                "Failed to create query: {e}"
            )))
        })?;

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
                        UveddiError::AstError(crate::ast::tree_sitter_impl::AstError::Other(format!(
                            "Failed to extract source: {e}"
                        )))
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
                file_path: pair.block1.file_path.clone(), // UV-222: Now O(1) Arc<PathBuf> clone
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
    /// 2. Indexes these blocks by their fingerprints for cross-file comparison.
    /// 3. For each block, finds potential clone candidates from the global index.
    /// 4. Verifies each candidate pair to confirm if they are a true clone.
    /// 5. Converts the verified clone pairs into `ArchitecturalIssue`s.
    ///
    /// # Arguments
    ///
    /// * `parsed_file` - The file to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ArchitecturalIssue`s or an `AnalysisError`.
    fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        info!(
            "Analyzing file for code duplication: {}",
            parsed_file.file_path.display()
        );

        // Extract code blocks from the current file
        let blocks = self.extract_code_blocks(parsed_file)?;

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
            debug!("No code blocks found in file: {}", parsed_file.file_path.display());
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
