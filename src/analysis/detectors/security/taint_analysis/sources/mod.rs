//! Taint source detection modules

pub mod external_sources;
pub mod input_sources;
pub mod user_sources;

pub use external_sources::ExternalSourceDetector;
pub use input_sources::InputSourceDetector;
pub use user_sources::UserSourceDetector;

use crate::analysis::detectors::security::taint_analysis::types::{SourceLocation, TaintSource};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};

/// Trait for taint source detection
pub trait TaintSourceDetector {
    /// Detect taint sources in a parsed file
    fn detect_sources(&self, file: &ParsedFile) -> Result<Vec<TaintSource>, AnalysisError>;

    /// Get language-specific source patterns
    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String>;
}

/// Unified source detector that combines all source types
pub struct UnifiedSourceDetector {
    input_detector: InputSourceDetector,
    external_detector: ExternalSourceDetector,
    user_detector: UserSourceDetector,
}

impl UnifiedSourceDetector {
    pub fn new() -> Self {
        Self {
            input_detector: InputSourceDetector::new(),
            external_detector: ExternalSourceDetector::new(),
            user_detector: UserSourceDetector::new(),
        }
    }

    pub fn detect_all_sources(&self, file: &ParsedFile) -> Result<Vec<TaintSource>, AnalysisError> {
        let mut sources = Vec::new();

        sources.extend(self.input_detector.detect_sources(file)?);
        sources.extend(self.external_detector.detect_sources(file)?);
        sources.extend(self.user_detector.detect_sources(file)?);

        Ok(sources)
    }
}

impl Default for UnifiedSourceDetector {
    fn default() -> Self {
        Self::new()
    }
}
