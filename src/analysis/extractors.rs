//! Logic for extracting symbols from an AST.

use crate::analysis::symbols::{CanonicalSymbol, GlobalSymbolTable, SymbolKind, SourceLocation};
use crate::ast::tree_sitter::ParsedFile;
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Query, QueryCursor};
use std::sync::atomic::{AtomicU64, Ordering};

/// A simple counter to generate unique symbol IDs.
static SYMBOL_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Extracts symbol declarations from a parsed file and adds them to a global symbol table.
pub struct SymbolExtractor {
    // In the future, this could hold language-specific queries.
}

impl SymbolExtractor {
    pub fn new() -> Self {
        Self {}
    }

    /// Extracts all top-level declarations from a file.
    #[cfg(feature = "tree-sitter")]
    pub fn extract_declarations(
        &self,
        file: &ParsedFile,
        symbol_table: &mut GlobalSymbolTable,
    ) -> Result<(), String> {
        let language = match file.language {
            crate::ast::tree_sitter::SourceLanguage::Rust => "rust",
            crate::ast::tree_sitter::SourceLanguage::Python => "python",
            crate::ast::tree_sitter::SourceLanguage::JavaScript => "javascript",
            // Add other languages as needed
        };

        let query_source = match language {
            "rust" => r#"
                (function_item name: (identifier) @name) @function
                (struct_item name: (type_identifier) @name) @struct
                (trait_item name: (type_identifier) @name) @trait
            "#,
            "python" => r#"
                (function_definition name: (identifier) @name) @function
                (class_definition name: (identifier) @name) @class
            "#,
            "javascript" | "typescript" => r#"
                (function_declaration name: (identifier) @name) @function
                (class_declaration name: (type_identifier) @name) @class
                (lexical_declaration (variable_declarator name: (identifier) @name)) @variable
            "#,
            _ => return Ok(()),
        };

        let tree = file.tree.as_ref().ok_or("Missing AST")?;
        let query = Query::new(&tree.language(), query_source)
            .map_err(|e| e.to_string())?;
        
        let mut cursor = QueryCursor::new();
        let captures = cursor.captures(&query, tree.root_node(), file.source.as_bytes());

        for (match_, _) in captures {
            if let Some(name_capture) = match_.captures.iter().find(|c| query.capture_names()[c.index as usize] == "name") {
                let node = name_capture.node;
                let kind_capture_index = match_.pattern_index;
                // Safely get the kind name
                let kind_name = query.capture_names().get(kind_capture_index).cloned().unwrap_or("unknown");

                let kind = match kind_name {
                    "function" => SymbolKind::Function,
                    "struct" | "class" => SymbolKind::Class,
                    "trait" => SymbolKind::Trait,
                    "variable" => SymbolKind::Variable,
                    _ => SymbolKind::Unknown,
                };
                
                let symbol_name = node.utf8_text(file.source.as_bytes()).unwrap_or("").to_string();
                let id = SYMBOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

                let symbol = CanonicalSymbol {
                    id,
                    name: symbol_name,
                    kind,
                    location: SourceLocation {
                        file_path: file.path.to_str().unwrap().to_string(),
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                    },
                };
                symbol_table.add_symbol(symbol);
            }
        }

        Ok(())
    }

    /// Stub implementation when tree-sitter is disabled
    #[cfg(not(feature = "tree-sitter"))]
    pub fn extract_declarations(
        &self,
        _file: &ParsedFile,
        _symbol_table: &mut GlobalSymbolTable,
    ) -> Result<(), String> {
        Err("Tree-sitter feature not enabled - symbol extraction unavailable".to_string())
    }
}
