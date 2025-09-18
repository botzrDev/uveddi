//! TypeScript/JavaScript-specific language support for code duplication detection

use super::{LanguageSupport, FunctionSignature, CommentSyntax};
use crate::analysis::detectors::anti_patterns::code_duplication::types::CodeBlock;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};

/// TypeScript/JavaScript language support for code duplication detection
pub struct TypeScriptLanguageSupport;

impl TypeScriptLanguageSupport {
    pub fn new() -> Self {
        Self
    }

    const KEYWORDS: &'static [&'static str] = &[
        "function", "class", "interface", "type", "enum", "namespace", "module", "import",
        "export", "default", "const", "let", "var", "if", "else", "switch", "case", "while",
        "for", "do", "try", "catch", "finally", "throw", "return", "break", "continue",
        "new", "this", "super", "extends", "implements", "static", "private", "protected",
        "public", "readonly", "abstract", "async", "await", "yield", "typeof", "instanceof",
    ];

    const BUILTIN_TYPES: &'static [&'static str] = &[
        "string", "number", "boolean", "object", "undefined", "null", "void", "any", "unknown",
        "never", "Array", "Object", "Function", "Promise", "Date", "RegExp", "Error", "Map",
        "Set", "WeakMap", "WeakSet", "Symbol", "BigInt", "Partial", "Required", "Readonly",
    ];
}

impl LanguageSupport for TypeScriptLanguageSupport {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::TypeScript
    }

    fn extract_code_blocks(&self, file: &ParsedFile) -> Result<Vec<CodeBlock>, AnalysisError> {
        let content = std::fs::read_to_string(&file.path)
            .map_err(|e| AnalysisError::IoError(format!("Failed to read file: {}", e)))?;

        let mut blocks = Vec::new();
        let lines: Vec<_> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Look for function declarations, arrow functions, and class methods
            if self.is_function_start(line) {
                let start_line = i + 1;
                let mut brace_count = 0;
                let mut in_function = false;
                let mut end_line = i;

                for j in i..lines.len() {
                    let current_line = lines[j];

                    if current_line.contains('{') {
                        in_function = true;
                    }

                    if in_function {
                        brace_count += current_line.matches('{').count() as i32;
                        brace_count -= current_line.matches('}').count() as i32;

                        if brace_count == 0 {
                            end_line = j + 1;
                            break;
                        }
                    }
                }

                if end_line > start_line + 2 {
                    let block_content: String = lines[i..end_line].join("\n");
                    blocks.push(CodeBlock::new(
                        file.path.to_string_lossy().to_string(),
                        start_line as u32,
                        end_line as u32,
                        0,
                        block_content.len(),
                        block_content,
                        SourceLanguage::TypeScript,
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
                } else if (token.starts_with('"') && token.ends_with('"')) ||
                          (token.starts_with('\'') && token.ends_with('\'')) ||
                          (token.starts_with('`') && token.ends_with('`')) {
                    "STRING".to_string()
                } else if token.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_' || c == '$') &&
                         token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
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
            "// ".to_string(),
            "/* ".to_string(),
            "* ".to_string(),
            "/** ".to_string(),
            "import ".to_string(),
            "export ".to_string(),
        ]
    }

    fn extract_signatures(&self, source: &str) -> Vec<FunctionSignature> {
        let mut signatures = Vec::new();
        let lines: Vec<_> = source.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if self.is_function_start(trimmed) {
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
            doc_comment: Some("/**".to_string()),
        }
    }
}

impl TypeScriptLanguageSupport {
    fn is_function_start(&self, line: &str) -> bool {
        line.contains("function ") ||
        line.contains(" => ") ||
        (line.contains("(") && line.contains(")") && line.contains("{")) ||
        line.trim_start().starts_with("async ")
    }

    fn parse_function_signature(&self, line: &str) -> Option<FunctionSignature> {
        let trimmed = line.trim();

        // Handle different function syntaxes
        if let Some(func_pos) = trimmed.find("function ") {
            // Traditional function declaration
            let name_start = func_pos + 9;
            if let Some(paren_pos) = trimmed[name_start..].find('(') {
                let name = trimmed[name_start..name_start + paren_pos].trim().to_string();
                return Some(FunctionSignature {
                    name,
                    parameter_types: Vec::new(),
                    return_type: None,
                    visibility: self.extract_visibility(trimmed),
                    is_method: false,
                    generics: Vec::new(),
                });
            }
        } else if trimmed.contains(" => ") {
            // Arrow function
            if let Some(arrow_pos) = trimmed.find(" => ") {
                let before_arrow = trimmed[..arrow_pos].trim();
                if let Some(eq_pos) = before_arrow.rfind('=') {
                    let name_part = before_arrow[..eq_pos].trim();
                    if let Some(name) = name_part.split_whitespace().last() {
                        return Some(FunctionSignature {
                            name: name.to_string(),
                            parameter_types: Vec::new(),
                            return_type: None,
                            visibility: self.extract_visibility(trimmed),
                            is_method: false,
                            generics: Vec::new(),
                        });
                    }
                }
            }
        }

        None
    }

    fn extract_visibility(&self, line: &str) -> Option<String> {
        if line.contains("private ") {
            Some("private".to_string())
        } else if line.contains("protected ") {
            Some("protected".to_string())
        } else if line.contains("public ") {
            Some("public".to_string())
        } else {
            None
        }
    }
}

impl Default for TypeScriptLanguageSupport {
    fn default() -> Self {
        Self::new()
    }
}