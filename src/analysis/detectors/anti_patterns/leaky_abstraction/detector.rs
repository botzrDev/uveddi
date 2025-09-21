//! Main detector implementation for leaky abstraction analysis.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::ast::SourceLanguage;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;

use super::analyzers::LeakDetector;
use super::config::create_default_config;
use super::language_support::{
    PythonLanguageSupport, RustLanguageSupport, TypeScriptLanguageSupport,
};
use super::types::{AnalysisContext, ArchitecturalConfig, LeakType};

/// The primary detector for identifying leaky abstractions in a codebase.
///
/// This struct orchestrates the analysis by combining architectural configuration,
/// language-specific parsing, and a set of detection heuristics to find violations
/// of architectural boundaries and encapsulation.
pub struct LeakyAbstractionDetector {
    /// The architectural configuration that guides the analysis.
    pub config: ArchitecturalConfig,

    /// The main leak detection orchestrator.
    leak_detector: LeakDetector,

    /// Language-specific support modules.
    rust_support: RustLanguageSupport,
    python_support: PythonLanguageSupport,
    typescript_support: TypeScriptLanguageSupport,
}

impl Clone for LeakyAbstractionDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            leak_detector: self.leak_detector.clone(),
            rust_support: self.rust_support.clone(),
            python_support: self.python_support.clone(),
            typescript_support: self.typescript_support.clone(),
        }
    }
}

impl Default for LeakyAbstractionDetector {
    /// Creates a new `LeakyAbstractionDetector` with a default configuration.
    fn default() -> Self {
        Self::new()
    }
}

impl LeakyAbstractionDetector {
    /// Creates a new `LeakyAbstractionDetector` with a default configuration.
    ///
    /// The default configuration includes common file path patterns for architectural
    /// layers and a list of well-known infrastructure modules for Rust, Python, and JavaScript.
    pub fn new() -> Self {
        let config = create_default_config();
        let leak_detector = LeakDetector::new(config.clone());

        Self {
            config,
            leak_detector,
            rust_support: RustLanguageSupport::new(),
            python_support: PythonLanguageSupport::new(),
            typescript_support: TypeScriptLanguageSupport::new(),
        }
    }

    /// Creates a new detector with a custom `ArchitecturalConfig`.
    ///
    /// This allows for fine-tuning the analysis to match a project's specific
    /// architectural conventions and dependencies.
    ///
    /// # Arguments
    ///
    /// * `config` - The `ArchitecturalConfig` to use for analysis.
    pub fn with_config(config: ArchitecturalConfig) -> Self {
        let leak_detector = LeakDetector::new(config.clone());

        Self {
            config,
            leak_detector,
            rust_support: RustLanguageSupport::new(),
            python_support: PythonLanguageSupport::new(),
            typescript_support: TypeScriptLanguageSupport::new(),
        }
    }

    /// Analyzes a single file for leaky abstractions.
    pub fn analyze_file(
        &self,
        parsed_file: &ParsedFile,
        analysis_run_id: i64,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Create analysis context
        let file_path = parsed_file.file_path.display().to_string();
        let current_layer = self.get_layer_from_path(&file_path);

        let context = AnalysisContext {
            analysis_run_id,
            file_path,
            current_layer,
        };

        let mut all_issues = Vec::new();

        // Run main leak detection
        match self.leak_detector.detect_leaks(parsed_file, &context) {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => {
                tracing::warn!("Leak detector failed: {}", e);
            }
        }

        // Run language-specific analysis
        match self.analyze_language_specific(parsed_file, &context) {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => {
                tracing::warn!("Language-specific analysis failed: {}", e);
            }
        }

        // Deduplicate and prioritize issues
        Ok(self.post_process_issues(all_issues))
    }

    /// Performs language-specific analysis.
    fn analyze_language_specific(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        match parsed_file.language {
            SourceLanguage::Rust => self.rust_support.analyze_file(parsed_file, context),
            SourceLanguage::Python => self.python_support.analyze_file(parsed_file, context),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                self.typescript_support.analyze_file(parsed_file, context)
            }
        }
    }

    /// Post-processes issues by deduplicating and prioritizing them.
    fn post_process_issues(&self, issues: Vec<ArchitecturalIssue>) -> Vec<ArchitecturalIssue> {
        let deduplicated = self.deduplicate_issues(issues);
        self.prioritize_issues(deduplicated)
    }

    /// Deduplicates similar issues to avoid noise.
    fn deduplicate_issues(&self, issues: Vec<ArchitecturalIssue>) -> Vec<ArchitecturalIssue> {
        let mut deduplicated = Vec::new();
        let mut seen_descriptions = std::collections::HashSet::new();

        for issue in issues {
            let key = format!(
                "{}:{}:{}",
                issue.file_path,
                issue.start_line.unwrap_or(0),
                issue.description
            );
            if !seen_descriptions.contains(&key) {
                seen_descriptions.insert(key);
                deduplicated.push(issue);
            }
        }

        deduplicated
    }

    /// Prioritizes issues based on severity and impact.
    fn prioritize_issues(&self, mut issues: Vec<ArchitecturalIssue>) -> Vec<ArchitecturalIssue> {
        issues.sort_by(|a, b| {
            // Sort by severity (high > medium > low)
            let severity_order = |s: &str| match s {
                "high" => 3,
                "medium" => 2,
                "low" => 1,
                _ => 0,
            };

            let a_priority = severity_order(&a.severity);
            let b_priority = severity_order(&b.severity);

            b_priority.cmp(&a_priority)
        });

        issues
    }

    /// Determines the architectural layer of a given file path based on configured mappings.
    fn get_layer_from_path(&self, file_path: &str) -> Option<super::types::ArchitecturalLayer> {
        for (pattern, layer) in &self.config.layer_mappings {
            if self.matches_pattern(file_path, pattern) {
                return Some(layer.clone());
            }
        }
        None
    }

    /// A simple glob-like pattern matcher for file paths.
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len() - 3];
            path.contains(&format!("/{}/", middle)) || path.contains(&format!("\\{}/", middle))
        } else if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            path.ends_with(suffix)
        } else if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            path.starts_with(prefix)
        } else {
            path.contains(pattern)
        }
    }

    /// Helper function to create a new `ArchitecturalIssue` instance.
    fn create_issue(
        &self,
        analysis_run_id: i64,
        file_path: &str,
        line_number: u32,
        leak_type: LeakType,
        description: &str,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&leak_type),
            file_path.to_string(),
            Some(line_number as i32),
            description.to_string(),
            "LeakyAbstractionDetector".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(line_number as i32);
        issue.end_line = Some(line_number as i32);
        issue
    }

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id` for database storage.
    fn get_anti_pattern_id_for_leak_type(&self, leak_type: &LeakType) -> i64 {
        match leak_type {
            LeakType::VisibilityViolation => 1,
            LeakType::LayerViolation => 2,
            LeakType::ImplementationExposure => 3,
            LeakType::FrameworkCoupling => 4,
            LeakType::ErrorPropagation => 5,
            LeakType::PerformanceLeak => 6,
        }
    }

    /// Gets the detector configuration.
    pub fn get_config(&self) -> &ArchitecturalConfig {
        &self.config
    }

    /// Updates the detector configuration.
    pub fn set_config(&mut self, config: ArchitecturalConfig) {
        self.config = config.clone();
        self.leak_detector = LeakDetector::new(config);
    }
}

#[async_trait]
impl AnalysisDetector for LeakyAbstractionDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        #[cfg(not(feature = "tree-sitter"))]
        {
            tracing::debug!(
                "Tree-sitter feature not enabled, skipping leaky abstraction detection"
            );
            return Ok(Vec::new());
        }

        #[cfg(feature = "tree-sitter")]
        {
            let analysis_run_id = 1; // TODO: Get from context
            self.analyze_file(parsed_file, analysis_run_id)
        }
    }

    fn get_detector_name(&self) -> &'static str {
        "LeakyAbstractionDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![
            AntiPatternType {
                anti_pattern_type_id: Some(1),
                name: "Visibility Violation".to_string(),
                description: "Accessing private or internal components inappropriately".to_string(),
                category: "encapsulation".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(2),
                name: "Layer Violation".to_string(),
                description: "Inappropriate dependencies between architectural layers".to_string(),
                category: "architectural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(3),
                name: "Implementation Exposure".to_string(),
                description: "Internal implementation details exposed through public APIs"
                    .to_string(),
                category: "encapsulation".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(4),
                name: "Framework Coupling".to_string(),
                description: "Business logic tightly coupled to specific frameworks".to_string(),
                category: "coupling".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(5),
                name: "Error Propagation".to_string(),
                description: "Low-level errors propagated across abstraction boundaries"
                    .to_string(),
                category: "error_handling".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: Some(6),
                name: "Performance Leak".to_string(),
                description: "Abstraction causing unexpected performance degradation".to_string(),
                category: "behavioral".to_string(),
            },
        ]
    }
}
