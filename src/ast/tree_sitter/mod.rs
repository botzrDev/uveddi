use tree_sitter::{Parser, Tree};
use std::path::Path;

pub mod queries;


/// Multi-language AST parser following ERD specifications
pub struct AstParser {
    rust_parser: Parser,
    python_parser: Parser,
    javascript_parser: Parser,
}

impl AstParser {
    /// Initialize parsers for supported languages (ER-F-002)
    pub fn new() -> Result<Self, AstError> {
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language())?;
        
        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language())?;
        
        let mut javascript_parser = Parser::new();
        javascript_parser.set_language(tree_sitter_javascript::language())?;
        
        Ok(AstParser {
            rust_parser,
            python_parser,
            javascript_parser,
        })
    }

    /// Parse file and extract dependencies (ER-F-003)
    pub fn parse_file(&mut self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let source = std::fs::read_to_string(file_path)?;
        let language = self.detect_language(file_path)?;
        
        let parser = match language {
            SourceLanguage::Rust => &mut self.rust_parser,
            SourceLanguage::Python => &mut self.python_parser,
            SourceLanguage::JavaScript => &mut self.javascript_parser,
        };
        
        let tree = parser.parse(&source, None)
            .ok_or(AstError::ParseFailed)?;
            
        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            language,
            tree,
            source,
        })
    }

    fn detect_language(&self, file_path: &Path) -> Result<SourceLanguage, AstError> {
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .ok_or(AstError::UnsupportedLanguage(format!("No file extension for {:?}", file_path)))?;

        match extension {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "jsx" | "ts" | "tsx" => Ok(SourceLanguage::JavaScript),
            _ => Err(AstError::UnsupportedLanguage(extension.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct ParsedFile {
    pub path: std::path::PathBuf,
    pub language: SourceLanguage,
    pub tree: Tree,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),
    #[error("AST parsing failed")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}
