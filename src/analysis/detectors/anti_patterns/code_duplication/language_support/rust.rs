//! Rust-specific language support for code duplication detection

use super::{LanguageSupport, FunctionSignature, CommentSyntax};
use crate::analysis::detectors::anti_patterns::code_duplication::types::CodeBlock;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};

/// Rust language support for code duplication detection
pub struct RustLanguageSupport;

impl RustLanguageSupport {
    pub fn new() -> Self {
        Self
    }

    /// Rust keywords that should not be normalized
    const KEYWORDS: &'static [&'static str] = &[
        "fn", "let", "mut", "const", "static", "if", "else", "while", "for", "loop", "break",
        "continue", "return", "match", "struct", "enum", "impl", "trait", "pub", "use", "mod",
        "crate", "super", "self", "Self", "where", "async", "await", "move", "ref", "type",
        "union", "unsafe", "extern", "as", "dyn", "Box", "Vec", "Option", "Result",
    ];

    /// Rust built-in types
    const BUILTIN_TYPES: &'static [&'static str] = &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc",
        "Arc", "RefCell", "Cell", "Mutex", "RwLock",
    ];
}

impl LanguageSupport for RustLanguageSupport {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }

    fn extract_code_blocks(&self, file: &ParsedFile) -> Result<Vec<CodeBlock>, AnalysisError> {
        let content = std::fs::read_to_string(&file.path)
            .map_err(|e| AnalysisError::IoError(format!("Failed to read file: {}", e)))?;

        let mut blocks = Vec::new();
        let lines: Vec<_> = content.lines().collect();

        // Simple function extraction - look for "fn " patterns
        let mut in_function = false;
        let mut current_function_start = 0;
        let mut brace_count = 0;
        let mut function_name = String::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("fn ") && !in_function {
                in_function = true;
                current_function_start = i + 1;
                brace_count = 0;

                // Extract function name
                if let Some(name_end) = trimmed.find('(') {
                    function_name = trimmed[3..name_end].trim().to_string();
                }
            }

            if in_function {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                if brace_count == 0 && line.contains('}') {
                    // End of function
                    let function_content: String = lines[current_function_start - 1..=i]
                        .join("\n");

                    if function_content.lines().count() >= 5 {
                        // Only include substantial functions
                        blocks.push(CodeBlock::new(
                            file.path.to_string_lossy().to_string(),
                            current_function_start as u32,
                            (i + 1) as u32,
                            0,
                            function_content.len(),
                            function_content,
                            SourceLanguage::Rust,
                        ));
                    }

                    in_function = false;
                    function_name.clear();
                }
            }
        }

        Ok(blocks)
    }

    fn normalize_tokens(&self, tokens: &[String]) -> Vec<String> {
        tokens
            .iter()
            .map(|token| {
                if self.is_keyword(token) || self.is_builtin_type(token) {
                    token.clone() // Keep keywords and types as-is
                } else if token.chars().all(|c| c.is_numeric() || c == '.') {
                    "NUMBER".to_string() // Normalize numbers
                } else if (token.starts_with('"') && token.ends_with('"')) ||
                          (token.starts_with('\'') && token.ends_with('\'')) {
                    "STRING".to_string() // Normalize string literals
                } else if token.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') &&
                         token.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    "IDENTIFIER".to_string() // Normalize identifiers
                } else {
                    token.clone() // Keep operators and punctuation
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
            "// ".to_string(),
            "/// ".to_string(),
            "//! ".to_string(),
            "#[".to_string(),
            "use ".to_string(),
            "mod ".to_string(),
        ]
    }

    fn extract_signatures(&self, source: &str) -> Vec<FunctionSignature> {
        let mut signatures = Vec::new();
        let lines: Vec<_> = source.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                if let Some(sig) = self.parse_function_signature(line) {
                    signatures.push(sig);
                }
            }
        }

        signatures
    }

    fn comment_syntax(&self) -> CommentSyntax {
        CommentSyntax {
            single_line: "//".to_string(),
            multi_line_start: Some("/*".to_string()),
            multi_line_end: Some("*/".to_string()),
            doc_comment: Some("///".to_string()),
        }
    }
}

impl RustLanguageSupport {
    fn parse_function_signature(&self, line: &str) -> Option<FunctionSignature> {
        let trimmed = line.trim();
        let is_public = trimmed.starts_with("pub ");

        // Find function name
        let fn_start = trimmed.find("fn ")?;
        let name_start = fn_start + 3;
        let paren_pos = trimmed.find('(')?;

        let name = trimmed[name_start..paren_pos].trim();

        // Extract generics if present
        let (clean_name, generics) = if let Some(generic_start) = name.find('<') {
            let generic_end = name.find('>')?;
            let generics_str = &name[generic_start + 1..generic_end];
            let generics: Vec<String> = generics_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            (name[..generic_start].to_string(), generics)
        } else {
            (name.to_string(), Vec::new())
        };

        Some(FunctionSignature {
            name: clean_name,
            parameter_types: Vec::new(), // Would need more parsing for full parameter analysis
            return_type: None,           // Would need more parsing for return type
            visibility: if is_public { Some("pub".to_string()) } else { None },
            is_method: false,            // Would need context to determine
            generics,
        })
    }
}

impl Default for RustLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}