//! AST-based clone detection using structural comparison

use super::CloneDetectionAlgorithm;
use crate::analysis::detectors::anti_patterns::code_duplication::types::{CodeBlock, ClonePair, CloneType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::HashMap;

/// AST-based clone detector using structural tree comparison
pub struct AstBasedDetector {
    /// Minimum similarity threshold for structural comparison
    similarity_threshold: f64,
    /// Whether to ignore identifier names in comparison
    ignore_identifiers: bool,
    /// Whether to ignore literal values in comparison
    ignore_literals: bool,
    /// Maximum tree depth to analyze (performance limit)
    max_depth: usize,
}

impl AstBasedDetector {
    /// Creates a new AST-based detector
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
            ignore_identifiers: true,
            ignore_literals: true,
            max_depth: 50,
        }
    }

    /// Creates a detector with custom settings
    pub fn with_settings(
        similarity_threshold: f64,
        ignore_identifiers: bool,
        ignore_literals: bool,
        max_depth: usize,
    ) -> Self {
        Self {
            similarity_threshold,
            ignore_identifiers,
            ignore_literals,
            max_depth,
        }
    }

    /// Parses source code and extracts AST structure
    pub fn parse_ast(&self, source: &str, language: &SourceLanguage) -> Result<AstNode, AnalysisError> {
        // Simplified AST parsing - in reality this would use tree-sitter
        let tokens = self.tokenize(source);
        self.build_ast(&tokens)
    }

    /// Simple tokenizer for AST construction
    fn tokenize(&self, source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for ch in source.chars() {
            match ch {
                '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' => {
                    if !current.is_empty() {
                        tokens.push(self.classify_token(current.clone()));
                        current.clear();
                    }
                    tokens.push(Token::Operator(ch.to_string()));
                }
                _ if ch.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push(self.classify_token(current.clone()));
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(self.classify_token(current));
        }

        tokens
    }

    /// Classifies a token based on its content
    fn classify_token(&self, token: String) -> Token {
        if self.is_keyword(&token) {
            Token::Keyword(token)
        } else if self.is_literal(&token) {
            if self.ignore_literals {
                Token::Literal("LITERAL".to_string())
            } else {
                Token::Literal(token)
            }
        } else if self.is_identifier(&token) {
            if self.ignore_identifiers {
                Token::Identifier("IDENTIFIER".to_string())
            } else {
                Token::Identifier(token)
            }
        } else {
            Token::Operator(token)
        }
    }

    /// Checks if token is a keyword
    fn is_keyword(&self, token: &str) -> bool {
        matches!(
            token,
            "function" | "class" | "if" | "else" | "while" | "for" | "return" | "var" | "let" | "const"
        )
    }

    /// Checks if token is a literal
    fn is_literal(&self, token: &str) -> bool {
        token.parse::<f64>().is_ok() ||
        token.starts_with('"') ||
        token.starts_with('\'') ||
        matches!(token, "true" | "false" | "null")
    }

    /// Checks if token is an identifier
    fn is_identifier(&self, token: &str) -> bool {
        !token.is_empty() &&
        token.chars().next().unwrap().is_alphabetic() &&
        token.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Builds AST from tokens
    fn build_ast(&self, tokens: &[Token]) -> Result<AstNode, AnalysisError> {
        let mut parser = AstParser::new(tokens);
        parser.parse()
    }

    /// Computes structural similarity between two AST nodes
    pub fn compute_structural_similarity(&self, node1: &AstNode, node2: &AstNode) -> f64 {
        self.compute_node_similarity(node1, node2, 0)
    }

    /// Recursively computes similarity between two nodes
    fn compute_node_similarity(&self, node1: &AstNode, node2: &AstNode, depth: usize) -> f64 {
        if depth > self.max_depth {
            return 0.5; // Default similarity for deep nodes
        }

        // Compare node types
        if std::mem::discriminant(&node1.node_type) != std::mem::discriminant(&node2.node_type) {
            return 0.0;
        }

        // Compare node values
        let value_similarity = match (&node1.node_type, &node2.node_type) {
            (NodeType::Keyword(k1), NodeType::Keyword(k2)) => {
                if k1 == k2 { 1.0 } else { 0.0 }
            }
            (NodeType::Identifier(_), NodeType::Identifier(_)) => {
                if self.ignore_identifiers { 1.0 } else {
                    if node1.value == node2.value { 1.0 } else { 0.0 }
                }
            }
            (NodeType::Literal(_), NodeType::Literal(_)) => {
                if self.ignore_literals { 1.0 } else {
                    if node1.value == node2.value { 1.0 } else { 0.0 }
                }
            }
            (NodeType::Operator(o1), NodeType::Operator(o2)) => {
                if o1 == o2 { 1.0 } else { 0.0 }
            }
            _ => 1.0,
        };

        if value_similarity == 0.0 {
            return 0.0;
        }

        // Compare children
        let child_similarity = if node1.children.is_empty() && node2.children.is_empty() {
            1.0
        } else {
            self.compute_children_similarity(&node1.children, &node2.children, depth + 1)
        };

        // Weighted average
        0.3 * value_similarity + 0.7 * child_similarity
    }

    /// Computes similarity between lists of child nodes
    fn compute_children_similarity(&self, children1: &[AstNode], children2: &[AstNode], depth: usize) -> f64 {
        if children1.is_empty() && children2.is_empty() {
            return 1.0;
        }

        let max_len = children1.len().max(children2.len());
        if max_len == 0 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let min_len = children1.len().min(children2.len());

        // Compare aligned children
        for i in 0..min_len {
            total_similarity += self.compute_node_similarity(&children1[i], &children2[i], depth);
        }

        // Penalty for different lengths
        let length_penalty = (max_len - min_len) as f64 / max_len as f64;

        (total_similarity / max_len as f64) * (1.0 - length_penalty * 0.5)
    }

    /// Extracts structural features from an AST
    pub fn extract_structural_features(&self, node: &AstNode) -> StructuralFeatures {
        let mut features = StructuralFeatures::new();
        self.collect_features(node, &mut features, 0);
        features
    }

    /// Recursively collects structural features
    fn collect_features(&self, node: &AstNode, features: &mut StructuralFeatures, depth: usize) {
        if depth > self.max_depth {
            return;
        }

        features.node_count += 1;
        features.max_depth = features.max_depth.max(depth);

        match &node.node_type {
            NodeType::Keyword(k) => {
                *features.keyword_counts.entry(k.clone()).or_insert(0) += 1;
            }
            NodeType::Operator(o) => {
                *features.operator_counts.entry(o.clone()).or_insert(0) += 1;
            }
            _ => {}
        }

        features.children_counts.push(node.children.len());

        for child in &node.children {
            self.collect_features(child, features, depth + 1);
        }
    }
}

impl CloneDetectionAlgorithm for AstBasedDetector {
    fn name(&self) -> &'static str {
        "AST-based Structural"
    }

    fn detect_clones(&self, block1: &CodeBlock, block2: &CodeBlock) -> Result<Option<ClonePair>, AnalysisError> {
        // Skip if same file and overlapping regions
        if block1.file_path == block2.file_path &&
           !(block1.end_line < block2.start_line || block2.end_line < block1.start_line) {
            return Ok(None);
        }

        // Parse ASTs
        let ast1 = self.parse_ast(&block1.source, &block1.language)?;
        let ast2 = self.parse_ast(&block2.source, &block2.language)?;

        // Compute structural similarity
        let similarity = self.compute_structural_similarity(&ast1, &ast2);

        if similarity < self.similarity_threshold {
            return Ok(None);
        }

        // Determine clone type based on similarity and settings
        let clone_type = if similarity >= 0.95 {
            if self.ignore_identifiers || self.ignore_literals {
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
            0, // No fingerprint concept for AST-based detection
        )))
    }

    fn supports_language(&self, _language: &SourceLanguage) -> bool {
        true // AST-based detection works for all languages with proper parsers
    }

    fn confidence_level(&self) -> f64 {
        0.9 // Very high confidence for structural comparison
    }
}

/// Token types for AST construction
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Operator(String),
}

/// AST node representation
#[derive(Debug, Clone)]
pub struct AstNode {
    node_type: NodeType,
    value: String,
    children: Vec<AstNode>,
}

#[derive(Debug, Clone)]
enum NodeType {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Operator(String),
    Block,
    Expression,
}

/// Simple AST parser
struct AstParser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> AstParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }

    fn parse(&mut self) -> Result<AstNode, AnalysisError> {
        self.parse_block()
    }

    fn parse_block(&mut self) -> Result<AstNode, AnalysisError> {
        let mut children = Vec::new();

        while self.position < self.tokens.len() {
            if let Ok(node) = self.parse_expression() {
                children.push(node);
            } else {
                self.position += 1; // Skip unparseable tokens
            }
        }

        Ok(AstNode {
            node_type: NodeType::Block,
            value: "block".to_string(),
            children,
        })
    }

    fn parse_expression(&mut self) -> Result<AstNode, AnalysisError> {
        if self.position >= self.tokens.len() {
            return Err(AnalysisError::ParseError("Unexpected end of input".to_string()));
        }

        let token = &self.tokens[self.position];
        self.position += 1;

        let (node_type, value) = match token {
            Token::Keyword(k) => (NodeType::Keyword(k.clone()), k.clone()),
            Token::Identifier(i) => (NodeType::Identifier(i.clone()), i.clone()),
            Token::Literal(l) => (NodeType::Literal(l.clone()), l.clone()),
            Token::Operator(o) => (NodeType::Operator(o.clone()), o.clone()),
        };

        Ok(AstNode {
            node_type,
            value,
            children: Vec::new(),
        })
    }
}

/// Structural features extracted from AST
#[derive(Debug, Clone)]
pub struct StructuralFeatures {
    pub node_count: usize,
    pub max_depth: usize,
    pub keyword_counts: HashMap<String, usize>,
    pub operator_counts: HashMap<String, usize>,
    pub children_counts: Vec<usize>,
}

impl StructuralFeatures {
    fn new() -> Self {
        Self {
            node_count: 0,
            max_depth: 0,
            keyword_counts: HashMap::new(),
            operator_counts: HashMap::new(),
            children_counts: Vec::new(),
        }
    }
}