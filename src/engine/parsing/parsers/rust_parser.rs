//! # Rust Parser
//!
//! Parser implementation for Rust.
//! Migrated from tree_sitter_impl.rs with language-specific logic.

use crate::ast::SourceLanguage;
use crate::engine::parsing::{LanguageParser, ParseError, Relation, RelationKind, Symbol, SymbolKind};
use std::sync::Mutex;

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Tree};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Parser, Tree};

/// Rust-specific parser implementation
pub struct RustParser {
    parser: Mutex<Parser>,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new() -> Result<Self, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "rust-lang")]
            {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| ParseError::ParseFailed(format!("Failed to set Rust language: {}", e)))?;
                Ok(Self { parser: Mutex::new(parser) })
            }
            #[cfg(not(feature = "rust-lang"))]
            {
                Err(ParseError::ParseFailed("Rust language support not enabled".to_string()))
            }
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            // Use stub parser when tree-sitter feature is disabled
            let parser = Parser::new();
            Ok(Self { parser: Mutex::new(parser) })
        }
    }
}

impl LanguageParser for RustParser {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }

    fn parse(&self, source: &str) -> Result<Tree, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            let mut parser = self
                .parser
                .lock()
                .map_err(|_| ParseError::ParseFailed("Rust parser lock poisoned".to_string()))?;
            parser
                .parse(source, None)
                .ok_or_else(|| ParseError::ParseFailed("Failed to parse Rust source".to_string()))
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            // Return stub tree when feature is disabled
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
                    "struct_item" => {
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
                    "function_item" => {
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
                    "mod_item" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Module,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "const_item" | "static_item" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Constant,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                });
                            }
                        }
                    }
                    "type_item" => {
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
            // Extract use statements (imports)
            let mut cursor = root.walk();
            for child in root.children(&mut cursor) {
                if child.kind() == "use_declaration" {
                    if let Some(use_clause) = child.child_by_field_name("argument") {
                        if let Ok(import_path) = use_clause.utf8_text(source.as_bytes()) {
                            relations.push(Relation {
                                from: "current_module".to_string(),
                                to: import_path.to_string(),
                                kind: RelationKind::Imports,
                            });
                        }
                    }
                }
            }

            // Extract impl blocks (implements relation)
            for child in root.children(&mut cursor) {
                if child.kind() == "impl_item" {
                    if let Some(trait_node) = child.child_by_field_name("trait") {
                        if let Some(type_node) = child.child_by_field_name("type") {
                            if let (Ok(trait_name), Ok(type_name)) = (
                                trait_node.utf8_text(source.as_bytes()),
                                type_node.utf8_text(source.as_bytes())
                            ) {
                                relations.push(Relation {
                                    from: type_name.to_string(),
                                    to: trait_name.to_string(),
                                    kind: RelationKind::Implements,
                                });
                            }
                        }
                    }
                }
            }
        }

        relations
    }
}