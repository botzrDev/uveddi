//! Language-specific support for code duplication detection

pub mod python;
pub mod rust;
pub mod typescript;

pub use python::PythonLanguageSupport;
pub use rust::RustLanguageSupport;
pub use typescript::TypeScriptLanguageSupport;

use crate::analysis::detectors::anti_patterns::code_duplication::types::CodeBlock;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};

/// Trait for language-specific code duplication analysis
pub trait LanguageSupport: Send + Sync {
    /// Returns the language this support handles
    fn language(&self) -> SourceLanguage;

    /// Extracts code blocks from a parsed file
    fn extract_code_blocks(&self, file: &ParsedFile) -> Result<Vec<CodeBlock>, AnalysisError>;

    /// Normalizes tokens for the specific language
    fn normalize_tokens(&self, tokens: &[String]) -> Vec<String>;

    /// Checks if a token is a language keyword
    fn is_keyword(&self, token: &str) -> bool;

    /// Checks if a token is a built-in type
    fn is_builtin_type(&self, token: &str) -> bool;

    /// Returns language-specific patterns to ignore
    fn get_ignore_patterns(&self) -> Vec<String>;

    /// Extracts function/method signatures for comparison
    fn extract_signatures(&self, source: &str) -> Vec<FunctionSignature>;

    /// Gets the comment syntax for this language
    fn comment_syntax(&self) -> CommentSyntax;
}

/// Represents a function or method signature
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Parameter types (if available)
    pub parameter_types: Vec<String>,
    /// Return type (if available)
    pub return_type: Option<String>,
    /// Visibility modifier
    pub visibility: Option<String>,
    /// Whether it's a method (has self parameter)
    pub is_method: bool,
    /// Generic parameters
    pub generics: Vec<String>,
}

/// Comment syntax for a language
#[derive(Debug, Clone)]
pub struct CommentSyntax {
    /// Single-line comment prefix
    pub single_line: String,
    /// Multi-line comment start
    pub multi_line_start: Option<String>,
    /// Multi-line comment end
    pub multi_line_end: Option<String>,
    /// Documentation comment prefix
    pub doc_comment: Option<String>,
}

/// Factory for creating language support instances
pub struct LanguageSupportFactory;

impl LanguageSupportFactory {
    /// Creates appropriate language support for the given language
    pub fn create_support(language: &SourceLanguage) -> Box<dyn LanguageSupport> {
        match language {
            SourceLanguage::Rust => Box::new(RustLanguageSupport::new()),
            SourceLanguage::Python => Box::new(PythonLanguageSupport::new()),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                Box::new(TypeScriptLanguageSupport::new())
            }
            _ => Box::new(GenericLanguageSupport::new(*language)),
        }
    }

    /// Returns all supported languages
    pub fn supported_languages() -> Vec<SourceLanguage> {
        vec![
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::JavaScript,
            SourceLanguage::TypeScript,
        ]
    }
}

/// Generic language support for unsupported languages
pub struct GenericLanguageSupport {
    language: SourceLanguage,
}

impl GenericLanguageSupport {
    fn new(language: SourceLanguage) -> Self {
        Self { language }
    }
}

impl LanguageSupport for GenericLanguageSupport {
    fn language(&self) -> SourceLanguage {
        self.language
    }

    fn extract_code_blocks(&self, file: &ParsedFile) -> Result<Vec<CodeBlock>, AnalysisError> {
        // Generic extraction - treat entire file as one block
        let content = std::fs::read_to_string(file.path())
            .map_err(|e| AnalysisError::file_system_error(file.path().display().to_string(), e))?;

        let lines: Vec<_> = content.lines().collect();
        let mut blocks = Vec::new();

        // Split into blocks based on empty lines
        let mut current_start = 1;
        let mut current_content = String::new();

        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() && !current_content.trim().is_empty() {
                blocks.push(CodeBlock::new(
                    file.path().to_string_lossy().to_string(),
                    current_start,
                    i as u32,
                    0,
                    current_content.len(),
                    current_content.clone(),
                    self.language,
                ));
                current_start = i as u32 + 2;
                current_content.clear();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Add final block if any
        if !current_content.trim().is_empty() {
            blocks.push(CodeBlock::new(
                file.path().to_string_lossy().to_string(),
                current_start,
                lines.len() as u32,
                0,
                current_content.len(),
                current_content,
                self.language,
            ));
        }

        Ok(blocks)
    }

    fn normalize_tokens(&self, tokens: &[String]) -> Vec<String> {
        // Generic normalization - just return as-is
        tokens.to_vec()
    }

    fn is_keyword(&self, _token: &str) -> bool {
        false // No language-specific knowledge
    }

    fn is_builtin_type(&self, _token: &str) -> bool {
        false // No language-specific knowledge
    }

    fn get_ignore_patterns(&self) -> Vec<String> {
        vec![] // No specific patterns to ignore
    }

    fn extract_signatures(&self, _source: &str) -> Vec<FunctionSignature> {
        vec![] // Generic support doesn't extract signatures
    }

    fn comment_syntax(&self) -> CommentSyntax {
        CommentSyntax {
            single_line: "//".to_string(),
            multi_line_start: Some("/*".to_string()),
            multi_line_end: Some("*/".to_string()),
            doc_comment: None,
        }
    }
}
