//! # JavaScript Parser
//!
//! Parser implementation for JavaScript.
//! Migrated from tree_sitter_impl.rs with language-specific logic.

use crate::ast::SourceLanguage;
use crate::engine::parsing::{LanguageParser, ParseError, Relation, RelationKind, Symbol, SymbolKind};

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Tree};
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Parser, Tree};

/// JavaScript-specific parser implementation
pub struct JavaScriptParser {
    parser: Parser,
}

impl JavaScriptParser {
    /// Create a new JavaScript parser
    pub fn new() -> Result<Self, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "javascript-lang")]
            {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_javascript::LANGUAGE.into())
                    .map_err(|e| ParseError::ParseFailed(format!("Failed to set JavaScript language: {}", e)))?;
                Ok(Self { parser })
            }
            #[cfg(not(feature = "javascript-lang"))]
            {
                Err(ParseError::ParseFailed("JavaScript language support not enabled".to_string()))
            }
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            let parser = Parser::new();
            Ok(Self { parser })
        }
    }
}

impl LanguageParser for JavaScriptParser {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::JavaScript
    }

    fn parse(&self, source: &str) -> Result<Tree, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            self.parser.parse(source, None)
                .ok_or_else(|| ParseError::ParseFailed("Failed to parse JavaScript source".to_string()))
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