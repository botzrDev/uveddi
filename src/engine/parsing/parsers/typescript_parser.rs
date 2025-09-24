//! # TypeScript Parser
//!
//! Parser implementation for TypeScript.
//! Migrated from tree_sitter_impl.rs with enhanced TypeScript-specific logic.

use crate::ast::SourceLanguage;
use crate::engine::parsing::{
    LanguageParser, ParseError, Relation, RelationKind, Symbol, SymbolKind,
};
use std::sync::Mutex;

#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Parser, Tree};
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Tree};

/// TypeScript-specific parser implementation
pub struct TypeScriptParser {
    parser: Mutex<Parser>,
}

impl TypeScriptParser {
    /// Create a new TypeScript parser
    pub fn new() -> Result<Self, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "typescript-lang")]
            {
                let mut parser = Parser::new();
                // Try TSX first for broader compatibility, fallback to TypeScript
                let tsx_result = parser.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into());
                if tsx_result.is_err() {
                    parser
                        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                        .map_err(|e| {
                            ParseError::ParseFailed(format!(
                                "Failed to set TypeScript language: {}",
                                e
                            ))
                        })?;
                }
                Ok(Self {
                    parser: Mutex::new(parser),
                })
            }
            #[cfg(not(feature = "typescript-lang"))]
            {
                Err(ParseError::ParseFailed(
                    "TypeScript language support not enabled".to_string(),
                ))
            }
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            let parser = Parser::new();
            Ok(Self {
                parser: Mutex::new(parser),
            })
        }
    }
}

impl LanguageParser for TypeScriptParser {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::TypeScript
    }

    fn parse(&self, source: &str) -> Result<Tree, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            let mut parser = self.parser.lock().map_err(|_| {
                ParseError::ParseFailed("TypeScript parser lock poisoned".to_string())
            })?;
            parser.parse(source, None).ok_or_else(|| {
                ParseError::ParseFailed("Failed to parse TypeScript source".to_string())
            })
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            Ok(Tree::default())
        }
    }

    fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
        {
            let mut cursor = root.walk();
            for child in root.children(&mut cursor) {
                match child.kind() {
                    "function_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Function,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "class_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Class,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "interface_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Type,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "type_alias_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Type,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "enum_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Type,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        symbols
    }

    fn build_relations(&self, tree: &Tree, source: &str) -> Vec<Relation> {
        let mut relations = Vec::new();
        let root = tree.root_node();

        #[cfg(feature = "tree-sitter")]
        {
            let mut cursor = root.walk();
            for child in root.children(&mut cursor) {
                if child.kind() == "import_statement" {
                    if let Some(source_node) = child.child_by_field_name("source") {
                        if let Ok(import_path) = source_node.utf8_text(source.as_bytes()) {
                            relations.push(Relation {
                                from: "current_module".to_string(),
                                to: import_path.to_string(),
                                kind: RelationKind::Imports,
                            });
                        }
                    }
                }
            }
        }

        relations
    }
}
