//! Simple AST parser implementation

use super::types::{AstNode, NodeType, Token};
use crate::analysis::AnalysisError;

/// Simple AST parser
pub struct AstParser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> AstParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<AstNode, AnalysisError> {
        self.parse_block()
    }

    fn parse_block(&mut self) -> Result<AstNode, AnalysisError> {
        let mut block = AstNode::new(NodeType::Block, "block".to_string());

        while self.position < self.tokens.len() {
            if let Ok(node) = self.parse_expression() {
                block.add_child(node);
            } else {
                self.position += 1; // Skip unparseable tokens
            }
        }

        Ok(block)
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

        Ok(AstNode::new(node_type, value))
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token) = self.current_token() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, expected: Token, error_msg: &str) -> Result<(), AnalysisError> {
        if self.match_token(&expected) {
            Ok(())
        } else {
            Err(AnalysisError::ParseError(error_msg.to_string()))
        }
    }
}

/// Tokenizer for creating tokens from source code
pub struct Tokenizer {
    ignore_identifiers: bool,
    ignore_literals: bool,
}

impl Tokenizer {
    pub fn new(ignore_identifiers: bool, ignore_literals: bool) -> Self {
        Self {
            ignore_identifiers,
            ignore_literals,
        }
    }

    pub fn tokenize(&self, source: &str) -> Vec<Token> {
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

    fn is_keyword(&self, token: &str) -> bool {
        matches!(
            token,
            "function" | "class" | "if" | "else" | "while" | "for" | "return" | "var" | "let" | "const"
        )
    }

    fn is_literal(&self, token: &str) -> bool {
        token.parse::<f64>().is_ok() ||
        token.starts_with('"') ||
        token.starts_with('\'') ||
        matches!(token, "true" | "false" | "null")
    }

    fn is_identifier(&self, token: &str) -> bool {
        !token.is_empty() &&
        token.chars().next().unwrap().is_alphabetic() &&
        token.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}