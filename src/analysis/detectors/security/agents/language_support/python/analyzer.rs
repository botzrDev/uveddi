//! Core Python security analyzer implementation
//!
//! This module implements the main PythonAgentAnalyzer struct and coordinates
//! Python-specific security analysis using pattern detection.

use super::super::{LanguageAgentAnalyzer, LanguagePatterns};
use super::patterns::{
    analyze_code_execution_patterns, analyze_dangerous_imports, analyze_obfuscation_patterns,
    analyze_persistence_patterns, get_python_patterns,
};
use crate::analysis::detectors::security::core::SecurityContext;
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use async_trait::async_trait;

/// Python-specific agent pattern analyzer
pub struct PythonAgentAnalyzer {
    patterns: LanguagePatterns,
}

impl PythonAgentAnalyzer {
    pub fn new() -> Self {
        let patterns = get_python_patterns();
        Self { patterns }
    }

    /// Analyze Python-specific agent patterns
    async fn analyze_python_patterns(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for Python-specific dangerous patterns
        issues.extend(analyze_dangerous_imports(context).await?);

        // Check for code execution patterns
        issues.extend(analyze_code_execution_patterns(context).await?);

        // Check for obfuscation patterns
        issues.extend(analyze_obfuscation_patterns(context).await?);

        // Check for persistence patterns
        issues.extend(analyze_persistence_patterns(context).await?);

        Ok(issues)
    }
}

impl Default for PythonAgentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAgentAnalyzer for PythonAgentAnalyzer {
    fn supported_language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }

    async fn analyze_language_specific(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        self.analyze_python_patterns(context).await
    }

    fn can_analyze(&self, context: &SecurityContext) -> bool {
        matches!(context.language, SourceLanguage::Python)
    }
}
