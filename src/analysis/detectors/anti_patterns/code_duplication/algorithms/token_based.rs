//! Token-based clone detection using rolling hashes

use super::CloneDetectionAlgorithm;
use crate::analysis::detectors::anti_patterns::code_duplication::types::{
    ClonePair, CloneType, CodeBlock, Fingerprint,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::SourceLanguage;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Token-based clone detector using Karp-Rabin rolling hash algorithm
pub struct TokenBasedDetector {
    /// Length of token windows for fingerprinting
    window_size: usize,
    /// Minimum similarity threshold for Type-3 clones
    similarity_threshold: f64,
    /// Whether to normalize identifiers
    normalize_identifiers: bool,
    /// Whether to normalize literals
    normalize_literals: bool,
}

impl TokenBasedDetector {
    /// Creates a new token-based detector
    pub fn new(window_size: usize, similarity_threshold: f64) -> Self {
        Self {
            window_size,
            similarity_threshold,
            normalize_identifiers: true,
            normalize_literals: true,
        }
    }

    /// Creates a detector with custom normalization settings
    pub fn with_normalization(
        window_size: usize,
        similarity_threshold: f64,
        normalize_identifiers: bool,
        normalize_literals: bool,
    ) -> Self {
        Self {
            window_size,
            similarity_threshold,
            normalize_identifiers,
            normalize_literals,
        }
    }

    /// Tokenizes source code into normalized tokens
    pub fn tokenize(&self, source: &str, language: &SourceLanguage) -> Vec<String> {
        // This is a simplified tokenizer - in reality you'd use language-specific parsing
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_string = false;
        let mut in_comment = false;

        for ch in source.chars() {
            match ch {
                '"' | '\'' if !in_comment => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(current_token.clone(), language));
                        current_token.clear();
                    }
                    in_string = !in_string;
                    if self.normalize_literals && in_string {
                        tokens.push("STRING_LITERAL".to_string());
                    }
                }
                '/' if !in_string => {
                    // Simple comment detection
                    in_comment = true;
                }
                '\n' => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(current_token.clone(), language));
                        current_token.clear();
                    }
                    in_comment = false;
                }
                _ if in_string || in_comment => {
                    // Skip string contents and comments
                }
                _ if ch.is_whitespace() => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(current_token.clone(), language));
                        current_token.clear();
                    }
                }
                _ if ch.is_alphanumeric() || ch == '_' => {
                    current_token.push(ch);
                }
                _ => {
                    if !current_token.is_empty() {
                        tokens.push(self.normalize_token(current_token.clone(), language));
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(self.normalize_token(current_token, language));
        }

        tokens
    }

    /// Normalizes a single token based on configuration
    fn normalize_token(&self, token: String, language: &SourceLanguage) -> String {
        // Check if it's a keyword
        if self.is_keyword(&token, language) {
            return token;
        }

        // Check if it's a literal
        if self.is_literal(&token) && self.normalize_literals {
            return "LITERAL".to_string();
        }

        // Check if it's an identifier
        if self.is_identifier(&token) && self.normalize_identifiers {
            return "IDENTIFIER".to_string();
        }

        token
    }

    /// Checks if a token is a language keyword
    fn is_keyword(&self, token: &str, language: &SourceLanguage) -> bool {
        match language {
            SourceLanguage::Rust => {
                matches!(
                    token,
                    "fn" | "let"
                        | "mut"
                        | "if"
                        | "else"
                        | "while"
                        | "for"
                        | "match"
                        | "struct"
                        | "enum"
                        | "impl"
                        | "trait"
                        | "pub"
                        | "use"
                        | "mod"
                )
            }
            SourceLanguage::Python => {
                matches!(
                    token,
                    "def"
                        | "class"
                        | "if"
                        | "else"
                        | "elif"
                        | "while"
                        | "for"
                        | "try"
                        | "except"
                        | "finally"
                        | "with"
                        | "import"
                        | "from"
                        | "as"
                )
            }
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                matches!(
                    token,
                    "function"
                        | "var"
                        | "let"
                        | "const"
                        | "if"
                        | "else"
                        | "while"
                        | "for"
                        | "try"
                        | "catch"
                        | "finally"
                        | "class"
                        | "extends"
                        | "import"
                        | "export"
                )
            }
        }
    }

    /// Checks if a token is a literal value
    fn is_literal(&self, token: &str) -> bool {
        token.parse::<f64>().is_ok()
            || token.starts_with('"')
            || token.starts_with('\'')
            || matches!(
                token,
                "true" | "false" | "null" | "undefined" | "None" | "True" | "False"
            )
    }

    /// Checks if a token is an identifier
    fn is_identifier(&self, token: &str) -> bool {
        !token.is_empty()
            && (token.chars().next().unwrap().is_alphabetic() || token.starts_with('_'))
            && token.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Generates rolling hash fingerprints for a token sequence
    pub fn generate_fingerprints(&self, tokens: &[String]) -> Vec<Fingerprint> {
        if tokens.len() < self.window_size {
            return Vec::new();
        }

        let mut fingerprints = Vec::new();

        for i in 0..=tokens.len() - self.window_size {
            let window = &tokens[i..i + self.window_size];
            let hash = self.hash_token_window(window);
            fingerprints.push(Fingerprint::new(hash, i, self.window_size));
        }

        fingerprints
    }

    /// Computes hash for a token window
    fn hash_token_window(&self, tokens: &[String]) -> String {
        let mut hasher = Sha256::new();
        for token in tokens {
            hasher.update(token.as_bytes());
            hasher.update(b"|");
        }
        format!("{:x}", hasher.finalize())
    }

    /// Calculates similarity between two token sequences
    pub fn calculate_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() && tokens2.is_empty() {
            return 1.0;
        }
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

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

    /// Finds common fingerprints between two sets
    pub fn find_common_fingerprints(
        &self,
        fingerprints1: &[Fingerprint],
        fingerprints2: &[Fingerprint],
    ) -> usize {
        let set1: HashSet<_> = fingerprints1.iter().map(|f| &f.hash).collect();
        let set2: HashSet<_> = fingerprints2.iter().map(|f| &f.hash).collect();
        set1.intersection(&set2).count()
    }
}

impl CloneDetectionAlgorithm for TokenBasedDetector {
    fn name(&self) -> &'static str {
        "Token-based (Karp-Rabin)"
    }

    fn detect_clones(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> Result<Option<ClonePair>, crate::analysis::AnalysisError> {
        // Skip if same file and overlapping regions
        if block1.file_path == block2.file_path
            && !(block1.end_line < block2.start_line || block2.end_line < block1.start_line)
        {
            return Ok(None);
        }

        // Tokenize both blocks
        let tokens1 = self.tokenize(&block1.source, &block1.language);
        let tokens2 = self.tokenize(&block2.source, &block2.language);

        // Check minimum size requirements
        if tokens1.len() < self.window_size || tokens2.len() < self.window_size {
            return Ok(None);
        }

        // Generate fingerprints
        let fingerprints1 = self.generate_fingerprints(&tokens1);
        let fingerprints2 = self.generate_fingerprints(&tokens2);

        if fingerprints1.is_empty() || fingerprints2.is_empty() {
            return Ok(None);
        }

        // Find common fingerprints
        let shared_fingerprints = self.find_common_fingerprints(&fingerprints1, &fingerprints2);

        if shared_fingerprints == 0 {
            return Ok(None);
        }

        // Calculate similarity
        let similarity = self.calculate_similarity(&tokens1, &tokens2);

        if similarity < self.similarity_threshold {
            return Ok(None);
        }

        // Determine clone type based on similarity and normalization
        let clone_type = if similarity >= 0.98 {
            if self.normalize_identifiers || self.normalize_literals {
                CloneType::Type2
            } else {
                CloneType::Type1
            }
        } else {
            CloneType::Type3
        };

        Ok(Some(ClonePair::new(
            block1.clone(),
            block2.clone(),
            similarity,
            clone_type,
            shared_fingerprints,
        )))
    }

    fn supports_language(&self, _language: &SourceLanguage) -> bool {
        true // Token-based detection works for all languages
    }

    fn confidence_level(&self) -> f64 {
        0.85 // High confidence for token-based detection
    }
}
