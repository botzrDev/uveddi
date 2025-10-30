//! Language-Specific Cryptographic Analysis

pub mod rust;
pub mod python;
pub mod javascript;

pub use rust::RustCryptoAnalyzer;
pub use python::PythonCryptoAnalyzer;
pub use javascript::JavaScriptCryptoAnalyzer;

use crate::analysis::detectors::security::crypto::types::CryptoFinding;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;

/// Language analyzer coordinator
pub struct LanguageAnalyzerCoordinator {
    rust: RustCryptoAnalyzer,
    python: PythonCryptoAnalyzer,
    javascript: JavaScriptCryptoAnalyzer,
}

impl LanguageAnalyzerCoordinator {
    pub fn new() -> Self {
        Self {
            rust: RustCryptoAnalyzer::new(),
            python: PythonCryptoAnalyzer::new(),
            javascript: JavaScriptCryptoAnalyzer::new(),
        }
    }

    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        match language {
            SourceLanguage::Rust => self.rust.analyze_comprehensive(content),
            SourceLanguage::Python => self.python.analyze_comprehensive(content),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript =>
                self.javascript.analyze_comprehensive(content),
        }
    }
}