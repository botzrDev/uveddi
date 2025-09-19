//! Taint sink detection modules

pub mod file_sinks;
pub mod network_sinks;
pub mod command_sinks;

pub use file_sinks::FileSinkDetector;
pub use network_sinks::NetworkSinkDetector;
pub use command_sinks::CommandSinkDetector;

use crate::analysis::detectors::security::taint_analysis::types::{TaintSink, SourceLocation};
use crate::ast::{ParsedFile, SourceLanguage};
use crate::analysis::AnalysisError;

/// Trait for taint sink detection
pub trait TaintSinkDetector {
    /// Detect taint sinks in a parsed file
    fn detect_sinks(&self, file: &ParsedFile) -> Result<Vec<TaintSink>, AnalysisError>;

    /// Get language-specific sink patterns
    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String>;
}

/// Unified sink detector that combines all sink types
pub struct UnifiedSinkDetector {
    file_detector: FileSinkDetector,
    network_detector: NetworkSinkDetector,
    command_detector: CommandSinkDetector,
}

impl UnifiedSinkDetector {
    pub fn new() -> Self {
        Self {
            file_detector: FileSinkDetector::new(),
            network_detector: NetworkSinkDetector::new(),
            command_detector: CommandSinkDetector::new(),
        }
    }

    pub fn detect_all_sinks(&self, file: &ParsedFile) -> Result<Vec<TaintSink>, AnalysisError> {
        let mut sinks = Vec::new();

        sinks.extend(self.file_detector.detect_sinks(file)?);
        sinks.extend(self.network_detector.detect_sinks(file)?);
        sinks.extend(self.command_detector.detect_sinks(file)?);

        Ok(sinks)
    }
}

impl Default for UnifiedSinkDetector {
    fn default() -> Self {
        Self::new()
    }
}