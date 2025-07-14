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

    /// Creates a new symbol table with SQLite persistence (UV-2)
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - Symbol table instance or error
    pub fn new_with_persistence(db_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        // UV-2: Follow ResultCache pattern for SQLite persistence
        // Use rusqlite for SQLite integration
        use rusqlite::{Connection, params};
        let conn = Connection::open(db_path)?;
        // Create symbols table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                start_byte INTEGER,
                end_byte INTEGER
            )",
            [],
        )?;
        // Load symbols from DB
        let mut stmt = conn.prepare("SELECT id, name, kind, file_path, start_byte, end_byte FROM symbols")?;
        let symbol_iter = stmt.query_map([], |row| {
            Ok(CanonicalSymbol {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: match row.get::<_, String>(2)?.as_str() {
                    "Function" => SymbolKind::Function,
                    "Method" => SymbolKind::Method,
                    "Class" => SymbolKind::Class,
                    "Struct" => SymbolKind::Struct,
                    "Trait" => SymbolKind::Trait,
                    "Interface" => SymbolKind::Interface,
                    "Module" => SymbolKind::Module,
                    "Variable" => SymbolKind::Variable,
                    "Parameter" => SymbolKind::Parameter,
                    "TypeAlias" => SymbolKind::TypeAlias,
                    _ => SymbolKind::Unknown,
                },
                location: SourceLocation {
                    file_path: row.get(3)?,
                    start_byte: row.get(4)?,
                    end_byte: row.get(5)?,
                },
            })
        })?;
        let mut symbols = HashMap::new();
        for symbol in symbol_iter {
            let sym = symbol?;
            symbols.insert(sym.id, sym);
        }
        Ok(Self { symbols })
    }

    /// Adds a symbol to the table.
    pub fn add_symbol(&mut self, symbol: CanonicalSymbol) {
        self.symbols.insert(symbol.id, symbol);
    }

    /// Retrieves a symbol by its ID.
    pub fn get_symbol(&self, id: &SymbolId) -> Option<&CanonicalSymbol> {
        self.symbols.get(id)
    }

    /// Persists a symbol to SQLite (UV-2)
    pub fn persist_symbol(&self, db_path: &std::path::Path, symbol: &CanonicalSymbol) -> Result<(), Box<dyn std::error::Error>> {
        use rusqlite::{Connection, params};
        let conn = Connection::open(db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO symbols (id, name, kind, file_path, start_byte, end_byte) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                symbol.id,
                symbol.name,
                format!("{:?}", symbol.kind),
                symbol.location.file_path,
                symbol.location.start_byte,
                symbol.location.end_byte,
            ],
        )?;
        Ok(())
    }
}
