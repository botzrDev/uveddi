//! Python-specific language support for code duplication detection

use super::{CommentSyntax, FunctionSignature, LanguageSupport};
use crate::analysis::detectors::anti_patterns::code_duplication::types::CodeBlock;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};

/// Python language support for code duplication detection
pub struct PythonLanguageSupport;

impl PythonLanguageSupport {
    pub fn new() -> Self {
        Self
    }

    const KEYWORDS: &'static [&'static str] = &[
        "def", "class", "if", "elif", "else", "while", "for", "try", "except", "finally", "with",
        "as", "import", "from", "return", "yield", "break", "continue", "pass", "lambda", "and",
        "or", "not", "in", "is", "None", "True", "False", "async", "await",
    ];

    const BUILTIN_TYPES: &'static [&'static str] = &[
        "int", "float", "str", "list", "dict", "tuple", "set", "bool", "bytes", "object", "type",
        "callable", "any", "union", "optional", "List", "Dict", "Tuple", "Set",
    ];
}

impl LanguageSupport for PythonLanguageSupport {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }

    fn extract_code_blocks(&self, file: &ParsedFile) -> Result<Vec<CodeBlock>, AnalysisError> {
        let content = std::fs::read_to_string(file.path()).map_err(|e| {
            AnalysisError::file_system_error(file.path().display().to_string(), e)
        })?;

        let mut blocks = Vec::new();
        let lines: Vec<_> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            if line.starts_with("def ") || line.starts_with("class ") {
                let start_line = i + 1;
                let indent_level = lines[i].len() - lines[i].trim_start().len();

                // Find the end of the function/class by looking for the next item at the same or lesser indentation
                let mut end_line = i + 1;
                for j in (i + 1)..lines.len() {
                    let current_line = lines[j];
                    if current_line.trim().is_empty() {
                        continue; // Skip empty lines
                    }

                    let current_indent = current_line.len() - current_line.trim_start().len();
                    if current_indent <= indent_level {
                        end_line = j;
                        break;
                    }
                    end_line = j + 1;
                }

                if end_line > start_line + 2 {
                    // Only include substantial blocks
                    let block_content: String = lines[i..end_line].join("\n");
                    blocks.push(CodeBlock::new(
                        file.path().to_string_lossy().to_string(),
                        start_line as u32,
                        end_line as u32,
                        0,
                        block_content.len(),
                        block_content,
                        SourceLanguage::Python,
                    ));
                }

                i = end_line;
            } else {
                i += 1;
            }
        }

        Ok(blocks)
    }

    fn normalize_tokens(&self, tokens: &[String]) -> Vec<String> {
        tokens
            .iter()
            .map(|token| {
                if self.is_keyword(token) || self.is_builtin_type(token) {
                    token.clone()
                } else if token.chars().all(|c| c.is_numeric() || c == '.') {
                    "NUMBER".to_string()
                } else if (token.starts_with('"') && token.ends_with('"'))
                    || (token.starts_with('\'') && token.ends_with('\''))
                    || token.starts_with("f\"")
                    || token.starts_with("r\"")
                {
                    "STRING".to_string()
                } else if token
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_alphabetic() || c == '_')
                    && token.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    "IDENTIFIER".to_string()
                } else {
                    token.clone()
                }
            })
            .collect()
    }

    fn is_keyword(&self, token: &str) -> bool {
        Self::KEYWORDS.contains(&token)
    }

    fn is_builtin_type(&self, token: &str) -> bool {
        Self::BUILTIN_TYPES.contains(&token)
    }

    fn get_ignore_patterns(&self) -> Vec<String> {
        vec![
            "# ".to_string(),
            "\"\"\"".to_string(),
            "'''".to_string(),
            "import ".to_string(),
            "from ".to_string(),
        ]
    }

    fn extract_signatures(&self, source: &str) -> Vec<FunctionSignature> {
        let mut signatures = Vec::new();
        let lines: Vec<_> = source.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") {
                if let Some(sig) = self.parse_function_signature(line) {
                    signatures.push(sig);
                }
            }
        }

        signatures
    }

    fn comment_syntax(&self) -> CommentSyntax {
        CommentSyntax {
            single_line: "#".to_string(),
            multi_line_start: Some("\"\"\"".to_string()),
            multi_line_end: Some("\"\"\"".to_string()),
            doc_comment: Some("\"\"\"".to_string()),
        }
    }
}

impl PythonLanguageSupport {
    fn parse_function_signature(&self, line: &str) -> Option<FunctionSignature> {
        let trimmed = line.trim();
        if !trimmed.starts_with("def ") {
            return None;
        }

        let name_start = 4; // After "def "
        let paren_pos = trimmed.find('(')?;
        let name = trimmed[name_start..paren_pos].trim().to_string();

        // Check if it's a method by looking for 'self' parameter
        let colon_pos = trimmed.find(':')?;
        let params_str = &trimmed[paren_pos + 1..colon_pos];
        let close_paren = params_str.rfind(')')?;
        let params = &params_str[..close_paren];

        let is_method = params.trim_start().starts_with("self");

        Some(FunctionSignature {
            name,
            parameter_types: Vec::new(), // Python doesn't always have type hints
            return_type: None,           // Would need to parse type hints
            visibility: None,            // Python doesn't have explicit visibility
            is_method,
            generics: Vec::new(), // Python doesn't have generics in the same way
        })
    }
}

impl Default for PythonLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}
