//! Language-specific God Object detection implementations

pub mod rust;
pub mod python;
pub mod javascript;
pub mod typescript;

pub use rust::RustGodObjectAnalyzer;
pub use python::PythonGodObjectAnalyzer;
pub use javascript::JavaScriptGodObjectAnalyzer;
pub use typescript::TypeScriptGodObjectAnalyzer;