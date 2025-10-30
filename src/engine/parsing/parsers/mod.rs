//! # Language-specific Parsers
//!
//! Individual parser implementations for each supported language.
//! Each parser implements the LanguageParser trait.

pub mod javascript_parser;
pub mod python_parser;
pub mod rust_parser;
pub mod typescript_parser;

// Re-export parsers
pub use javascript_parser::JavaScriptParser;
pub use python_parser::PythonParser;
pub use rust_parser::RustParser;
pub use typescript_parser::TypeScriptParser;
