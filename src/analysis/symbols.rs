//! Data structures for symbol resolution and management.
//!
//! This module will contain the building blocks for cross-file semantic analysis,
//! including the `GlobalSymbolTable` and the `CanonicalSymbol` representation
//! as described in the research documents.

use std::collections::HashMap;

/// A unique identifier for a symbol, potentially a combination of
/// file ID and location to ensure uniqueness across a project.
pub type SymbolId = u64;

/// Represents a canonical, language-agnostic symbol.
/// This structure is designed to be the common Intermediate Representation (IR)
/// for symbols from Rust, Python, and JavaScript/TypeScript.
#[derive(Debug, Clone)]
pub struct CanonicalSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub location: SourceLocation,
    // More fields like `visibility`, `type_info`, etc., will be added here.
}

/// The kind of entity a symbol represents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Trait,
    Interface,
    Module,
    Variable,
    Parameter,
    TypeAlias,
    Unknown,
}

/// Represents a location in the source code, storing owned data
/// instead of references to avoid lifetime issues.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// A project-wide table that maps unique symbol IDs to their definitions.
/// This will be populated by the first pass of the semantic analysis.
#[derive(Debug, Clone, Default)]
pub struct GlobalSymbolTable {
    symbols: HashMap<SymbolId, CanonicalSymbol>,
    // This will be expanded to include scope management and lookup logic.
}

impl GlobalSymbolTable {
    /// Creates a new, empty symbol table.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Adds a symbol to the table.
    pub fn add_symbol(&mut self, symbol: CanonicalSymbol) {
        self.symbols.insert(symbol.id, symbol);
    }

    /// Retrieves a symbol by its ID.
    pub fn get_symbol(&self, id: &SymbolId) -> Option<&CanonicalSymbol> {
        self.symbols.get(id)
    }
}

/// Placeholder documentation for public items
