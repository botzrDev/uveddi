//! Rust-specific God Object detection

mod implementations;
mod patterns;
mod queries;
mod structs;

pub use implementations::RustImplAnalyzer;
pub use patterns::RustPatternDetector;
pub use structs::RustStructAnalyzer;

use super::super::config::GodObjectConfig;
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use tracing::debug;

/// Rust-specific God Object analyzer
pub struct RustGodObjectAnalyzer<'a> {
    config: &'a GodObjectConfig,
    struct_analyzer: RustStructAnalyzer<'a>,
    impl_analyzer: RustImplAnalyzer<'a>,
    pattern_detector: RustPatternDetector<'a>,
}

impl<'a> RustGodObjectAnalyzer<'a> {
    pub fn new(config: &'a GodObjectConfig) -> Self {
        Self {
            config,
            struct_analyzer: RustStructAnalyzer::new(config),
            impl_analyzer: RustImplAnalyzer::new(config),
            pattern_detector: RustPatternDetector::new(config),
        }
    }

    /// Analyze a Rust file for God Objects
    pub fn analyze(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Stage 1: Framework Detection
        let detected_frameworks = self.pattern_detector.analyze_imports(parsed_file)?;
        debug!("Detected Rust frameworks: {:?}", detected_frameworks);

        // Detect derive macros for DTO patterns
        let derive_attributes = self.pattern_detector.analyze_derive_macros(parsed_file)?;

        // 1. Find all impl blocks and count their methods
        let impl_method_counts = self.impl_analyzer.count_impl_methods(parsed_file)?;

        // 2. Analyze structs with enhanced pattern detection
        issues.extend(self.struct_analyzer.analyze_structs(
            parsed_file,
            &impl_method_counts,
            &detected_frameworks,
            &derive_attributes,
        )?);

        Ok(issues)
    }
}
