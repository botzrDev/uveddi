//! Main dead code detector implementation

use async_trait::async_trait;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::analysis::memory::{PooledObject, DETECTOR_POOLS};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::core::logging::{debug, info};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};

use super::analysis::AnalysisOrchestrator;
use super::config::DeadCodeConfig;
use super::language_support::{LanguageAnalyzer, LanguageAnalyzerFactory};
use super::patterns::PatternDetector;
use super::types::{DeadCodeIssue, Severity, Symbol};
use super::validation::ValidationOrchestrator;

/// A detector for identifying unused (dead) code in a codebase.
pub struct DeadCodeDetector {
    config: DeadCodeConfig,
    language_analyzers: std::collections::HashMap<SourceLanguage, Box<dyn LanguageAnalyzer>>,
}

impl DeadCodeDetector {
    /// Creates a new `DeadCodeDetector` with the given configuration.
    pub fn new(config: DeadCodeConfig) -> Self {
        let mut language_analyzers = std::collections::HashMap::new();

        // Pre-create analyzers for all supported languages
        language_analyzers.insert(
            SourceLanguage::Rust,
            LanguageAnalyzerFactory::create(SourceLanguage::Rust),
        );
        language_analyzers.insert(
            SourceLanguage::Python,
            LanguageAnalyzerFactory::create(SourceLanguage::Python),
        );
        language_analyzers.insert(
            SourceLanguage::JavaScript,
            LanguageAnalyzerFactory::create(SourceLanguage::JavaScript),
        );
        language_analyzers.insert(
            SourceLanguage::TypeScript,
            LanguageAnalyzerFactory::create(SourceLanguage::TypeScript),
        );

        Self {
            config,
            language_analyzers,
        }
    }

    /// Creates a new `DeadCodeDetector` with a pooled configuration.
    pub fn with_pooled_config() -> (Self, PooledObject<DeadCodeConfig>) {
        let mut pooled_config = DETECTOR_POOLS.dead_code_configs.get();
        pooled_config.reset();
        let detector = Self::new(pooled_config.clone());
        (detector, pooled_config)
    }

    /// Creates a new `DeadCodeDetector` with default configuration.
    pub fn with_default_config() -> Self {
        Self::new(DeadCodeConfig::default())
    }

    /// Detects dead code in a parsed file
    pub async fn detect_dead_code(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<DeadCodeIssue>, AnalysisError> {
        debug!(
            "Running dead code detection on: {}",
            parsed_file.file_path.display()
        );

        // Get language-specific analyzer
        let analyzer = self
            .language_analyzers
            .get(&parsed_file.language)
            .ok_or_else(|| {
                AnalysisError::DetectionError(format!(
                    "Unsupported language: {:?}",
                    parsed_file.language
                ))
            })?;

        // Extract symbols and references
        let mut symbols = analyzer.extract_symbols(parsed_file)?;
        let references = analyzer.extract_references(parsed_file)?;

        // Mark reachable symbols
        let entry_points = analyzer.identify_entry_points(&symbols);
        AnalysisOrchestrator::mark_reachable_symbols(&mut symbols, &references, entry_points);

        // Perform analysis
        let issues =
            AnalysisOrchestrator::analyze(symbols.clone(), references.clone(), parsed_file.language).await?;

        // Validate issues
        let validated_issues = ValidationOrchestrator::validate(issues, &symbols).await?;

        Ok(validated_issues)
    }

    /// Extract symbols from a parsed file (public for testing)
    pub fn extract_symbols(&self, parsed_file: &ParsedFile) -> Result<Vec<Symbol>, AnalysisError> {
        let analyzer = self
            .language_analyzers
            .get(&parsed_file.language)
            .ok_or_else(|| {
                AnalysisError::DetectionError(format!(
                    "Unsupported language: {:?}",
                    parsed_file.language
                ))
            })?;

        analyzer.extract_symbols(parsed_file)
    }

    /// Extract references from a parsed file (public for testing)
    pub fn extract_references(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<HashSet<String>, AnalysisError> {
        let analyzer = self
            .language_analyzers
            .get(&parsed_file.language)
            .ok_or_else(|| {
                AnalysisError::DetectionError(format!(
                    "Unsupported language: {:?}",
                    parsed_file.language
                ))
            })?;

        analyzer.extract_references(parsed_file)
    }

    /// Check if a symbol should be kept alive based on patterns
    fn should_keep_alive(&self, symbol: &Symbol) -> bool {
        for pattern in &self.config.keep_alive_patterns {
            if symbol.name.contains(pattern) {
                return true;
            }
        }
        false
    }
}

#[async_trait]
impl AnalysisDetector for DeadCodeDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running Dead Code detection on: {}",
            parsed_file.file_path.display()
        );

        let dead_code_issues = self.detect_dead_code(parsed_file).await?;
        let mut architectural_issues = Vec::new();

        for issue in dead_code_issues {
            let symbol = &issue.symbol;

            // Skip if should be kept alive
            if self.should_keep_alive(symbol) {
                continue;
            }

            // Apply configuration filters
            if self.config.library_mode && !symbol.is_exported {
                continue;
            }

            if symbol.confidence < self.config.min_confidence {
                continue;
            }

            let severity_str = issue.severity.to_string();

            let mut architectural_issue = ArchitecturalIssue::new(
                0, // analysis_run_id will be set by the caller
                2, // anti_pattern_type_id for Dead Code
                symbol.path.display().to_string(),
                Some(symbol.line_number as i32),
                format!(
                    "Potentially dead code: {} '{}' is not used (confidence: {:.1}%)",
                    symbol.symbol_type,
                    symbol.name,
                    symbol.confidence * 100.0
                ),
                "DeadCodeDetector".to_string(),
                severity_str,
                format!(
                    "Potentially dead code: {} '{}' is not used (confidence: {:.1}%)",
                    symbol.symbol_type,
                    symbol.name,
                    symbol.confidence * 100.0
                ),
            );

            architectural_issue.start_line = Some(symbol.line_number as i32);
            architectural_issue.end_line = Some(symbol.line_number as i32);
            architectural_issue.code_snippet = Some(symbol.code_snippet.clone());

            architectural_issues.push(architectural_issue);
        }

        if architectural_issues.is_empty() {
            debug!(
                "No dead code issues found in {}",
                parsed_file.file_path.display()
            );
        } else {
            info!(
                "Found {} potential dead code issues in {}",
                architectural_issues.len(),
                parsed_file.file_path.display()
            );
        }

        Ok(architectural_issues)
    }

    fn get_detector_name(&self) -> &'static str {
        "DeadCodeDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(2),
            name: "Dead Code".to_string(),
            description: "Code that is defined but never used, including unused functions, variables, classes, and modules.".to_string(),
            category: "Maintainability".to_string(),
        }]
    }
}