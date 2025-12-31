//! Main leak detection orchestrator for leaky abstraction analysis.
//!
//! This module coordinates all other analyzers to provide comprehensive
//! leak detection across multiple dimensions of architectural analysis.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::analyzers::{
    AbstractionValidator, ImplementationAnalyzer, InterfaceAnalyzer,
};
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig, LeakType,
};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use crate::database::models::ArchitecturalIssue;

/// Main coordinator for leak detection analysis.
pub struct LeakDetector {
    interface_analyzer: InterfaceAnalyzer,
    implementation_analyzer: ImplementationAnalyzer,
    abstraction_validator: AbstractionValidator,
}

impl LeakDetector {
    /// Creates a new leak detector with the given configuration.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self {
            interface_analyzer: InterfaceAnalyzer::new(),
            implementation_analyzer: ImplementationAnalyzer::new(),
            abstraction_validator: AbstractionValidator::new(config),
        }
    }

    /// Performs comprehensive leak detection on a parsed file.
    pub fn detect_leaks(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut all_issues = Vec::new();

        // Run interface analysis
        if let Ok(interface_results) = self
            .interface_analyzer
            .analyze_interface(parsed_file, context)
        {
            let interface_issues = self
                .interface_analyzer
                .convert_to_issues(&interface_results, context);
            all_issues.extend(interface_issues);
        }

        // Run implementation analysis
        if let Ok(implementation_results) = self
            .implementation_analyzer
            .analyze_implementation(parsed_file, context)
        {
            let implementation_issues = self
                .implementation_analyzer
                .convert_to_issues(&implementation_results, context);
            all_issues.extend(implementation_issues);
        }

        // Run abstraction validation
        if let Ok(validation_issues) = self
            .abstraction_validator
            .validate_abstractions(parsed_file, context)
        {
            all_issues.extend(validation_issues);
        }

        // Deduplicate and prioritize issues
        Ok(self.deduplicate_issues(all_issues))
    }

    /// Performs language-specific leak detection.
    pub fn detect_language_specific_leaks(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => self.detect_rust_leaks(parsed_file, context),
            crate::ast::SourceLanguage::Python => self.detect_python_leaks(parsed_file, context),
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                self.detect_js_leaks(parsed_file, context)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Detects Rust-specific leaks.
    fn detect_rust_leaks(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for specific Rust patterns that indicate leaks
        issues.extend(self.detect_rust_visibility_leaks(parsed_file, context)?);
        issues.extend(self.detect_rust_error_propagation(parsed_file, context)?);
        issues.extend(self.detect_rust_trait_leaks(parsed_file, context)?);

        Ok(issues)
    }

    /// Detects Python-specific leaks.
    fn detect_python_leaks(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for specific Python patterns that indicate leaks
        issues.extend(self.detect_python_private_access(parsed_file, context)?);
        issues.extend(self.detect_python_framework_coupling(parsed_file, context)?);
        issues.extend(self.detect_python_model_leaks(parsed_file, context)?);

        Ok(issues)
    }

    /// Detects JavaScript/TypeScript-specific leaks.
    fn detect_js_leaks(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for specific JS/TS patterns that indicate leaks
        issues.extend(self.detect_js_dom_leaks(parsed_file, context)?);
        issues.extend(self.detect_js_framework_coupling(parsed_file, context)?);
        issues.extend(self.detect_js_type_leaks(parsed_file, context)?);

        Ok(issues)
    }

    /// Detects Rust visibility-related leaks.
    fn detect_rust_visibility_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects Rust error propagation leaks.
    fn detect_rust_error_propagation(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects Rust trait-related leaks.
    fn detect_rust_trait_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects Python private access violations.
    fn detect_python_private_access(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects Python framework coupling.
    fn detect_python_framework_coupling(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects Python model leaks.
    fn detect_python_model_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects JavaScript DOM-related leaks.
    fn detect_js_dom_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects JavaScript framework coupling.
    fn detect_js_framework_coupling(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Detects JavaScript type leaks.
    fn detect_js_type_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Implementation would go here
        Ok(Vec::new())
    }

    /// Deduplicates similar issues to avoid noise.
    fn deduplicate_issues(&self, issues: Vec<ArchitecturalIssue>) -> Vec<ArchitecturalIssue> {
        let mut deduplicated = Vec::new();
        let mut seen_descriptions = std::collections::HashSet::new();

        for issue in issues {
            let key = format!("{}:{}", issue.file_path, issue.description);
            if !seen_descriptions.contains(&key) {
                seen_descriptions.insert(key);
                deduplicated.push(issue);
            }
        }

        deduplicated
    }

    /// Prioritizes issues based on severity and impact.
    pub fn prioritize_issues(
        &self,
        mut issues: Vec<ArchitecturalIssue>,
    ) -> Vec<ArchitecturalIssue> {
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

    /// Filters issues based on configuration or thresholds.
    pub fn filter_issues(
        &self,
        issues: Vec<ArchitecturalIssue>,
        min_severity: &str,
    ) -> Vec<ArchitecturalIssue> {
        let min_level = match min_severity {
            "high" => 3,
            "medium" => 2,
            "low" => 1,
            _ => 0,
        };

        issues
            .into_iter()
            .filter(|issue| {
                let issue_level = match issue.severity.as_str() {
                    "high" => 3,
                    "medium" => 2,
                    "low" => 1,
                    _ => 0,
                };
                issue_level >= min_level
            })
            .collect()
    }

    /// Helper function to create an architectural issue.
    fn create_issue(
        &self,
        context: &AnalysisContext,
        description: &str,
        line_number: u32,
        leak_type: LeakType,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            context.analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&leak_type),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "LeakDetector".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(line_number as i32);
        issue.end_line = Some(line_number as i32);
        issue
    }

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id`.
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
}

impl Clone for LeakDetector {
    fn clone(&self) -> Self {
        Self {
            interface_analyzer: self.interface_analyzer.clone(),
            implementation_analyzer: self.implementation_analyzer.clone(),
            abstraction_validator: self.abstraction_validator.clone(),
        }
    }
}
