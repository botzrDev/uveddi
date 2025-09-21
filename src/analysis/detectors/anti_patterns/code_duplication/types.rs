//! Type definitions for code duplication detection

use crate::ast::tree_sitter_impl::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a contiguous block of code extracted for duplication analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// The absolute path to the file containing this code block
    pub file_path: String,
    /// The starting line number of the block (1-based)
    pub start_line: u32,
    /// The ending line number of the block (1-based)
    pub end_line: u32,
    /// The starting byte offset of the block within the file
    pub start_byte: usize,
    /// The ending byte offset of the block within the file
    pub end_byte: usize,
    /// The raw source code of the block
    pub source: String,
    /// A sequence of normalized tokens used for structural comparison
    pub normalized_tokens: Vec<String>,
    /// A SHA-256 hash of the normalized token sequence
    pub structural_hash: String,
    /// The name of the function or method, if the block represents one
    pub function_name: Option<String>,
    /// The programming language of the source code
    pub language: SourceLanguage,
    /// Control Flow Graph representation (if enabled)
    pub cfg: Option<crate::analysis::cfg::ControlFlowGraph>,
    /// Extracted semantic features for advanced analysis
    pub semantic_features: Option<crate::analysis::semantic::SemanticFeatures>,
    /// Hash of the CFG structure for quick comparison
    pub cfg_hash: Option<String>,
}

impl CodeBlock {
    /// Creates a new code block
    pub fn new(
        file_path: String,
        start_line: u32,
        end_line: u32,
        start_byte: usize,
        end_byte: usize,
        source: String,
        language: SourceLanguage,
    ) -> Self {
        Self {
            file_path,
            start_line,
            end_line,
            start_byte,
            end_byte,
            source,
            normalized_tokens: Vec::new(),
            structural_hash: String::new(),
            function_name: None,
            language,
            cfg: None,
            semantic_features: None,
            cfg_hash: None,
        }
    }

    /// Returns the size of the code block in lines
    pub fn line_count(&self) -> u32 {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Returns the size of the code block in bytes
    pub fn byte_count(&self) -> usize {
        self.end_byte.saturating_sub(self.start_byte)
    }

    /// Returns the number of tokens in this block
    pub fn token_count(&self) -> usize {
        self.normalized_tokens.len()
    }

    /// Sets the normalized tokens and updates the structural hash
    pub fn set_normalized_tokens(&mut self, tokens: Vec<String>) {
        self.normalized_tokens = tokens;
        self.update_structural_hash();
    }

    /// Updates the structural hash based on current normalized tokens
    pub fn update_structural_hash(&mut self) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for token in &self.normalized_tokens {
            hasher.update(token.as_bytes());
            hasher.update(b"|"); // Separator
        }
        self.structural_hash = format!("{:x}", hasher.finalize());
    }
}

/// Represents a pair of code blocks identified as duplicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonePair {
    /// The first code block in the clone pair
    pub block1: CodeBlock,
    /// The second code block in the clone pair
    pub block2: CodeBlock,
    /// Similarity score between 0.0 and 1.0
    pub similarity: f64,
    /// The classification of the clone type
    pub clone_type: CloneType,
    /// The number of shared fingerprints between the blocks
    pub shared_fingerprints: usize,
    /// Additional metrics about the clone pair
    pub metrics: ClonePairMetrics,
}

impl ClonePair {
    /// Creates a new clone pair
    pub fn new(
        block1: CodeBlock,
        block2: CodeBlock,
        similarity: f64,
        clone_type: CloneType,
        shared_fingerprints: usize,
    ) -> Self {
        Self {
            block1,
            block2,
            similarity,
            clone_type,
            shared_fingerprints,
            metrics: ClonePairMetrics::default(),
        }
    }

    /// Returns true if this clone pair exceeds the given similarity threshold
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.similarity >= threshold
    }

    /// Returns the total lines of code involved in this clone pair
    pub fn total_lines(&self) -> u32 {
        self.block1.line_count() + self.block2.line_count()
    }
}

/// Types of code clones based on structural and syntactic similarity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloneType {
    /// Type-1 (Exact Clone): Identical except whitespace and comments
    Type1,
    /// Type-2 (Renamed Clone): Identical structure, different identifiers
    Type2,
    /// Type-3 (Near-Miss Clone): Similar structure with minor modifications
    Type3,
    /// Type-4 (Semantic Clone): Functionally equivalent but different syntax
    Type4,
}

impl CloneType {
    /// Returns the severity level for this clone type
    pub fn severity(&self) -> crate::analysis::detectors::base::Severity {
        use crate::analysis::detectors::base::Severity;
        match self {
            CloneType::Type1 => Severity::High,
            CloneType::Type2 => Severity::High,
            CloneType::Type3 => Severity::Medium,
            CloneType::Type4 => Severity::Low,
        }
    }

    /// Returns a human-readable description of this clone type
    pub fn description(&self) -> &'static str {
        match self {
            CloneType::Type1 => "Exact duplicate code blocks",
            CloneType::Type2 => "Structurally identical with renamed identifiers",
            CloneType::Type3 => "Similar structure with minor modifications",
            CloneType::Type4 => "Functionally equivalent but different implementation",
        }
    }
}

impl std::fmt::Display for CloneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneType::Type1 => write!(f, "Type-1"),
            CloneType::Type2 => write!(f, "Type-2"),
            CloneType::Type3 => write!(f, "Type-3"),
            CloneType::Type4 => write!(f, "Type-4"),
        }
    }
}

/// Additional metrics for clone pairs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClonePairMetrics {
    /// Token-level similarity score
    pub token_similarity: f64,
    /// AST-level similarity score
    pub ast_similarity: f64,
    /// CFG-level similarity score (if available)
    pub cfg_similarity: Option<f64>,
    /// Semantic similarity score (if available)
    pub semantic_similarity: Option<f64>,
    /// Number of identical tokens
    pub identical_tokens: usize,
    /// Number of different tokens
    pub different_tokens: usize,
}

/// A fingerprint used for fast clone candidate identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint {
    /// The hash value of the token sequence
    pub hash: String,
    /// The starting position in the token sequence
    pub start_position: usize,
    /// The length of the token sequence
    pub length: usize,
}

impl Fingerprint {
    /// Creates a new fingerprint
    pub fn new(hash: String, start_position: usize, length: usize) -> Self {
        Self {
            hash,
            start_position,
            length,
        }
    }
}

/// Results of the duplication detection analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationResults {
    /// All clone pairs found
    pub clone_pairs: Vec<ClonePair>,
    /// Summary statistics
    pub summary: DuplicationSummary,
    /// Detection metrics
    pub metrics: super::super::super::base::DetectionMetrics,
}

impl DuplicationResults {
    /// Creates new duplication results
    pub fn new(clone_pairs: Vec<ClonePair>) -> Self {
        let summary = DuplicationSummary::from_clone_pairs(&clone_pairs);
        Self {
            clone_pairs,
            summary,
            metrics: Default::default(),
        }
    }

    /// Filters clone pairs by minimum similarity threshold
    pub fn filter_by_similarity(mut self, threshold: f64) -> Self {
        self.clone_pairs.retain(|pair| pair.similarity >= threshold);
        self.summary = DuplicationSummary::from_clone_pairs(&self.clone_pairs);
        self
    }

    /// Filters clone pairs by clone type
    pub fn filter_by_type(mut self, clone_types: &[CloneType]) -> Self {
        self.clone_pairs
            .retain(|pair| clone_types.contains(&pair.clone_type));
        self.summary = DuplicationSummary::from_clone_pairs(&self.clone_pairs);
        self
    }
}

/// Summary statistics for duplication detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationSummary {
    /// Total number of clone pairs found
    pub total_pairs: usize,
    /// Breakdown by clone type
    pub by_type: HashMap<CloneType, usize>,
    /// Total duplicated lines of code
    pub total_duplicated_lines: u32,
    /// Files with duplications
    pub affected_files: Vec<String>,
    /// Average similarity score
    pub average_similarity: f64,
}

impl DuplicationSummary {
    /// Creates a summary from clone pairs
    pub fn from_clone_pairs(clone_pairs: &[ClonePair]) -> Self {
        let total_pairs = clone_pairs.len();
        let mut by_type = HashMap::new();
        let mut total_duplicated_lines = 0u32;
        let mut affected_files = std::collections::HashSet::new();
        let mut total_similarity = 0.0;

        for pair in clone_pairs {
            *by_type.entry(pair.clone_type.clone()).or_insert(0) += 1;
            total_duplicated_lines += pair.total_lines();
            affected_files.insert(pair.block1.file_path.clone());
            affected_files.insert(pair.block2.file_path.clone());
            total_similarity += pair.similarity;
        }

        let average_similarity = if total_pairs > 0 {
            total_similarity / total_pairs as f64
        } else {
            0.0
        };

        Self {
            total_pairs,
            by_type,
            total_duplicated_lines,
            affected_files: affected_files.into_iter().collect(),
            average_similarity,
        }
    }
}
