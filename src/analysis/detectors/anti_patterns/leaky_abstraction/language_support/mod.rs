//! Language-specific support modules for leaky abstraction detection.

pub mod python;
pub mod rust;
pub mod typescript;

pub use python::PythonLanguageSupport;
pub use rust::RustLanguageSupport;
pub use typescript::TypeScriptLanguageSupport;
