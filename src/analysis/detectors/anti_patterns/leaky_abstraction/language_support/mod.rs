//! Language-specific support modules for leaky abstraction detection.

pub mod rust;
pub mod python;
pub mod typescript;

pub use rust::RustLanguageSupport;
pub use python::PythonLanguageSupport;
pub use typescript::TypeScriptLanguageSupport;