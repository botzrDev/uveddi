//! Shared parser setup utilities for tests
//!
//! This module eliminates duplication in test parser initialization by providing
//! common utilities for creating parsed files and setting up AST parsers across
//! all detector tests.

use crate::ast::tree_sitter_impl::{AstParser, ParsedFile, SourceLanguage};
use std::path::PathBuf;

/// Error type for test parser operations
#[derive(Debug)]
pub enum TestParserError {
    ParserCreation(String),
    FileParsing(String),
}

impl std::fmt::Display for TestParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestParserError::ParserCreation(msg) => write!(f, "Parser creation failed: {}", msg),
            TestParserError::FileParsing(msg) => write!(f, "File parsing failed: {}", msg),
        }
    }
}

impl std::error::Error for TestParserError {}

/// Shared parser helper for consistent test setup
pub struct TestParserHelper {
    parser: AstParser,
}

impl TestParserHelper {
    /// Create a new test parser helper
    pub fn new() -> Result<Self, TestParserError> {
        let parser = AstParser::new()
            .map_err(|e| TestParserError::ParserCreation(format!("Failed to create parser: {}", e)))?;

        Ok(Self { parser })
    }

    /// Parse Rust code for testing
    pub fn parse_rust_code(&mut self, code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
        self.parse_content(code, filename, SourceLanguage::Rust)
    }

    /// Parse Python code for testing
    pub fn parse_python_code(&mut self, code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
        self.parse_content(code, filename, SourceLanguage::Python)
    }

    /// Parse JavaScript code for testing
    pub fn parse_javascript_code(&mut self, code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
        self.parse_content(code, filename, SourceLanguage::JavaScript)
    }

    /// Parse TypeScript code for testing
    pub fn parse_typescript_code(&mut self, code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
        self.parse_content(code, filename, SourceLanguage::TypeScript)
    }

    /// Parse content with specified language
    pub fn parse_content(&mut self, code: &str, filename: &str, language: SourceLanguage) -> Result<ParsedFile, TestParserError> {
        let file_path = PathBuf::from(filename);
        self.parser
            .parse_content(code, &file_path, language)
            .map_err(|e| TestParserError::FileParsing(format!("Failed to parse {}: {}", filename, e)))
    }
}

/// Convenience function to create a parser and parse Rust code in one call
pub fn create_parsed_rust_file(code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
    let mut helper = TestParserHelper::new()?;
    helper.parse_rust_code(code, filename)
}

/// Convenience function to create a parser and parse Python code in one call
pub fn create_parsed_python_file(code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
    let mut helper = TestParserHelper::new()?;
    helper.parse_python_code(code, filename)
}

/// Convenience function to create a parser and parse JavaScript code in one call
pub fn create_parsed_javascript_file(code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
    let mut helper = TestParserHelper::new()?;
    helper.parse_javascript_code(code, filename)
}

/// Convenience function to create a parser and parse TypeScript code in one call
pub fn create_parsed_typescript_file(code: &str, filename: &str) -> Result<ParsedFile, TestParserError> {
    let mut helper = TestParserHelper::new()?;
    helper.parse_typescript_code(code, filename)
}

/// Test macro for creating a parser with standard error handling
#[macro_export]
macro_rules! create_test_parser {
    () => {
        crate::test_utils::parser_helpers::TestParserHelper::new()
            .expect("Failed to create test parser")
    };
}

/// Test macro for parsing Rust code with standard error handling
#[macro_export]
macro_rules! parse_rust_test_code {
    ($code:expr, $filename:expr) => {
        crate::test_utils::parser_helpers::create_parsed_rust_file($code, $filename)
            .expect(&format!("Failed to parse Rust test code from {}", $filename))
    };
}

/// Test macro for parsing Python code with standard error handling
#[macro_export]
macro_rules! parse_python_test_code {
    ($code:expr, $filename:expr) => {
        crate::test_utils::parser_helpers::create_parsed_python_file($code, $filename)
            .expect(&format!("Failed to parse Python test code from {}", $filename))
    };
}

/// Test macro for parsing JavaScript code with standard error handling
#[macro_export]
macro_rules! parse_javascript_test_code {
    ($code:expr, $filename:expr) => {
        crate::test_utils::parser_helpers::create_parsed_javascript_file($code, $filename)
            .expect(&format!("Failed to parse JavaScript test code from {}", $filename))
    };
}

/// Test macro for parsing TypeScript code with standard error handling
#[macro_export]
macro_rules! parse_typescript_test_code {
    ($code:expr, $filename:expr) => {
        crate::test_utils::parser_helpers::create_parsed_typescript_file($code, $filename)
            .expect(&format!("Failed to parse TypeScript test code from {}", $filename))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_helper_creation() {
        let result = TestParserHelper::new();
        assert!(result.is_ok(), "Should be able to create parser helper");
    }

    #[test]
    fn test_rust_code_parsing() {
        let rust_code = r#"
            fn hello_world() {
                println!("Hello, world!");
            }
        "#;

        let result = create_parsed_rust_file(rust_code, "test.rs");
        assert!(result.is_ok(), "Should be able to parse valid Rust code");
    }

    #[test]
    fn test_python_code_parsing() {
        let python_code = r#"
def hello_world():
    print("Hello, world!")
        "#;

        let result = create_parsed_python_file(python_code, "test.py");
        assert!(result.is_ok(), "Should be able to parse valid Python code");
    }

    #[test]
    fn test_javascript_code_parsing() {
        let js_code = r#"
function helloWorld() {
    console.log("Hello, world!");
}
        "#;

        let result = create_parsed_javascript_file(js_code, "test.js");
        assert!(result.is_ok(), "Should be able to parse valid JavaScript code");
    }
}