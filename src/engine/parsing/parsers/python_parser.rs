//! # Python Parser
//!
//! Parser implementation for Python.
//! Migrated from tree_sitter_impl.rs with language-specific logic.

use crate::ast::SourceLanguage;
use crate::engine::parsing::{
    LanguageParser, ParseError, Relation, RelationKind, Symbol, SymbolKind,
};
use std::sync::Mutex;

#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Parser, Tree};
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Tree};

/// Python-specific parser implementation
pub struct PythonParser {
    parser: Mutex<Parser>,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new() -> Result<Self, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            #[cfg(feature = "python-lang")]
            {
                let mut parser = Parser::new();
                parser
                    .set_language(&tree_sitter_python::LANGUAGE.into())
                    .map_err(|e| {
                        ParseError::ParseFailed(format!("Failed to set Python language: {}", e))
                    })?;
                Ok(Self {
                    parser: Mutex::new(parser),
                })
            }
            #[cfg(not(feature = "python-lang"))]
            {
                Err(ParseError::ParseFailed(
                    "Python language support not enabled".to_string(),
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

impl LanguageParser for PythonParser {
    fn language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }

    fn parse(&self, source: &str) -> Result<Tree, ParseError> {
        #[cfg(feature = "tree-sitter")]
        {
            let mut parser = self
                .parser
                .lock()
                .map_err(|_| ParseError::ParseFailed("Python parser lock poisoned".to_string()))?;
            parser
                .parse(source, None)
                .ok_or_else(|| ParseError::ParseFailed("Failed to parse Python source".to_string()))
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
                    "class_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Class,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                    end_line: name_node.end_position().row,
                                    end_column: name_node.end_position().column,
                                    parent: None,
                                });
                            }
                        }
                    }
                    "function_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                symbols.push(Symbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Function,
                                    line: name_node.start_position().row,
                                    column: name_node.start_position().column,
                                    end_line: name_node.end_position().row,
                                    end_column: name_node.end_position().column,
                                    parent: None,
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
                if child.kind() == "import_statement" || child.kind() == "import_from_statement" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Ok(import_name) = name_node.utf8_text(source.as_bytes()) {
                            relations.push(Relation {
                                from: "current_module".to_string(),
                                to: import_name.to_string(),
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
