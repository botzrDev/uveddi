//! Entropy Analysis

use crate::analysis::detectors::security::crypto::types::{CryptoFinding};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;

pub struct EntropyAnalyzer;

impl EntropyAnalyzer {
    pub fn new() -> Self { Self }
    pub fn analyze(&self, _content: &str, _language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        Ok(Vec::new())
    }
}