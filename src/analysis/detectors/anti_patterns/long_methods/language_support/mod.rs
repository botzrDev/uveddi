//! Language-specific support for method analysis

pub mod python;
pub mod rust;
pub mod typescript;

pub use python::PythonMethodAnalyzer;
pub use rust::RustMethodAnalyzer;
pub use typescript::TypeScriptMethodAnalyzer;