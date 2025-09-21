//! Language-specific God Object detection implementations

pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

pub use javascript::JavaScriptGodObjectAnalyzer;
pub use python::PythonGodObjectAnalyzer;
pub use rust::RustGodObjectAnalyzer;
pub use typescript::TypeScriptGodObjectAnalyzer;
