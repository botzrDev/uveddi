//! Language-specific support for knowledge graph construction

pub mod rust;
pub mod python;
pub mod javascript;

pub use rust::RustEntityProcessor;
pub use python::PythonEntityProcessor;
pub use javascript::JavaScriptEntityProcessor;

use crate::analysis::detectors::security::knowledge_graph::types::CodeEntity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::path::PathBuf;

/// Language-agnostic entity processor
pub struct LanguageEntityProcessor {
    rust_processor: RustEntityProcessor,
    python_processor: PythonEntityProcessor,
    js_processor: JavaScriptEntityProcessor,
}

impl LanguageEntityProcessor {
    pub fn new() -> Self {
        Self {
            rust_processor: RustEntityProcessor::new(),
            python_processor: PythonEntityProcessor::new(),
            js_processor: JavaScriptEntityProcessor::new(),
        }
    }

    /// Process entities based on file language
    pub async fn process_file_entities(
        &self,
        file_path: &PathBuf,
        content: &str,
        language: SourceLanguage,
    ) -> Result<Vec<CodeEntity>, AnalysisError> {
        match language {
            SourceLanguage::Rust => {
                self.rust_processor.process_rust_entities(file_path, content).await
            }
            SourceLanguage::Python => {
                self.python_processor.process_python_entities(file_path, content).await
            }
            SourceLanguage::JavaScript => {
                self.js_processor.process_js_entities(file_path, content).await
            }
            _ => {
                // For unsupported languages, return empty list
                Ok(Vec::new())
            }
        }
    }

    /// Determine language from file extension
    pub fn detect_language(&self, file_path: &PathBuf) -> SourceLanguage {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" => SourceLanguage::Rust,
                "py" => SourceLanguage::Python,
                "js" | "ts" | "jsx" | "tsx" => SourceLanguage::JavaScript,
                _ => SourceLanguage::Unknown,
            }
        } else {
            SourceLanguage::Unknown
        }
    }
}