//! Core type definitions for dead code detection

use std::path::PathBuf;

/// Represents a symbol (e.g., function, variable, class) identified in the source code.
///
/// A `Symbol` captures essential information about a code element, including its name,
/// type, location, and visibility. This information is used during dead code analysis
/// to determine if the symbol is ever used.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The name of the symbol (e.g., `my_function`).
    pub name: String,
    /// The type of the symbol (e.g., `Function`, `Class`).
    pub symbol_type: SymbolType,
    /// The absolute path to the file where the symbol is defined.
    pub path: PathBuf,
    /// The line number where the symbol's definition begins.
    pub line_number: u32,
    /// Indicates whether the symbol is public or exported, making it an entry point.
    pub is_exported: bool,
    /// A flag used during reachability analysis to mark the symbol as used.
    pub is_live: bool,
    /// A score from 0.0 to 1.0 indicating the confidence that this symbol is dead code.
    ///
    /// - **High (0.8-1.0)**: Private/internal symbols with no references.
    /// - **Medium (0.5-0.7)**: Exported symbols in applications with no apparent usage.
    /// - **Low (0.2-0.4)**: Symbols in files with dynamic features (e.g., reflection).
    pub confidence: f64,
    /// A snippet of the source code where the symbol is defined.
    pub code_snippet: String,
}

/// Enumerates the different types of symbols that can be analyzed for dead code.
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Variable,
    Class,
    Struct,
    Enum,
    Constant,
    Module,
    Import,
    Trait,
    Macro,
    Decorator,
    Closure,
    AsyncFunction,
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "function"),
            SymbolType::Variable => write!(f, "variable"),
            SymbolType::Class => write!(f, "class"),
            SymbolType::Struct => write!(f, "struct"),
            SymbolType::Enum => write!(f, "enum"),
            SymbolType::Constant => write!(f, "constant"),
            SymbolType::Module => write!(f, "module"),
            SymbolType::Import => write!(f, "import"),
            SymbolType::Trait => write!(f, "trait"),
            SymbolType::Macro => write!(f, "macro"),
            SymbolType::Decorator => write!(f, "decorator"),
            SymbolType::Closure => write!(f, "closure"),
            SymbolType::AsyncFunction => write!(f, "async function"),
        }
    }
}

/// Represents a dead code issue found during analysis
#[derive(Debug, Clone)]
pub struct DeadCodeIssue {
    pub symbol: Symbol,
    pub severity: Severity,
    pub removal_safe: bool,
    pub related_symbols: Vec<String>,
}

/// Severity levels for dead code issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}