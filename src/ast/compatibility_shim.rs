//! # Compatibility Shim
//!
//! Provides backward compatibility for existing detectors during the engine migration.
//! This module bridges the old `ParsedFile` interface to the new engine architecture.

use crate::analysis::cache::wrappers::ArchivableSystemTime;
use crate::ast::tree_sitter_impl::CustomAst;
use crate::ast::SourceLanguage;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Import types conditionally when engine module is available
#[cfg(feature = "engine-integration")]
use crate::database::models::ArchitecturalIssue;
#[cfg(feature = "engine-integration")]
use crate::engine::analysis::context::{
    AnalysisContext, DependencySource, FileInfo, ProjectContext, ProjectDependency,
};
#[cfg(feature = "engine-integration")]
use crate::engine::analysis::pipeline::{
    AnalysisPipeline, AnalysisResult, ContextBuilder, Detector, PipelineError,
};
#[cfg(feature = "engine-integration")]
use crate::engine::parsing::AstBuilder;

// Re-export tree-sitter types
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::Tree;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;

/// Compatibility wrapper for the old ParsedFile interface
#[derive(Debug, Clone)]
pub struct ParsedFileCompat {
    pub file_path: Arc<PathBuf>,
    pub language: SourceLanguage,
    pub tree: Option<Tree>,
    pub source: Arc<String>,
    pub custom_ast: Arc<Option<CustomAst>>,
    pub modified_at: ArchivableSystemTime,
}

impl ParsedFileCompat {
    /// Create from tree-sitter ParsedFile
    pub fn from_tree_sitter(parsed: crate::ast::tree_sitter_impl::ParsedFile) -> Self {
        Self {
            file_path: Arc::clone(&parsed.file_path),
            language: parsed.language.clone(),
            tree: None, // Tree cannot be cloned, must be omitted
            source: Arc::clone(&parsed.source),
            custom_ast: Arc::clone(&parsed.custom_ast),
            modified_at: parsed.modified_at.clone(),
        }
    }

    /// Convert to tree-sitter ParsedFile (tree will be None)
    pub fn to_tree_sitter(&self) -> crate::ast::tree_sitter_impl::ParsedFile {
        crate::ast::tree_sitter_impl::ParsedFile {
            file_path: Arc::clone(&self.file_path),
            language: self.language.clone(),
            tree: self.tree.clone(),
            source: Arc::clone(&self.source),
            custom_ast: Arc::clone(&self.custom_ast),
            modified_at: self.modified_at.clone(),
        }
    }

    /// Create a new compatibility wrapper (stub implementation)
    pub fn new(file_path: PathBuf, language: SourceLanguage, source: String) -> Self {
        let custom_ast = Arc::new(None);

        Self {
            file_path: Arc::new(file_path),
            language,
            tree: None, // Will be populated when engine is fully integrated
            source: Arc::new(source),
            custom_ast,
            modified_at: std::time::SystemTime::now().into(),
        }
    }

    /// Get the source code as a string slice
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// Create a parse summary (for compatibility)
    pub fn summary(&self) -> String {
        format!("Parsed {}: {:?}", self.file_path.display(), self.language)
    }

    /// Extract relevant code (stub implementation)
    pub fn extract_relevant_code(&self, issue_context: &str) -> Option<std::borrow::Cow<str>> {
        let lines: Vec<&str> = self.source.lines().collect();

        // Look for lines containing the issue context
        for (i, line) in lines.iter().enumerate() {
            if line.contains(issue_context) {
                // Return 3 lines of context around the match
                let start = i.saturating_sub(3);
                let end = std::cmp::min(i + 4, lines.len());
                let context_lines = lines[start..end].join("\n");
                return Some(std::borrow::Cow::Owned(context_lines));
            }
        }

        None
    }

    /// Extract code segment (stub implementation)
    pub fn extract_code_segment<'a>(
        &'a self,
        start: usize,
        end: usize,
    ) -> std::borrow::Cow<'a, str> {
        if start == 0 && end == self.source.len() {
            std::borrow::Cow::Borrowed(&*self.source)
        } else if start < end && end <= self.source.len() {
            std::borrow::Cow::Owned(self.source[start..end].to_string())
        } else {
            std::borrow::Cow::Borrowed("")
        }
    }

    /// Extract lines (stub implementation)
    pub fn extract_lines<'a>(
        &'a self,
        start_line: usize,
        end_line: usize,
    ) -> std::borrow::Cow<'a, str> {
        let lines: Vec<&str> = self.source.lines().collect();

        if start_line == 0 && end_line >= lines.len() {
            std::borrow::Cow::Borrowed(&*self.source)
        } else if start_line < end_line && end_line <= lines.len() {
            let extracted = lines[start_line..end_line].join("\n");
            std::borrow::Cow::Owned(extracted)
        } else {
            std::borrow::Cow::Borrowed("")
        }
    }

    /// Get context snippet (stub implementation)
    pub fn get_context_snippet<'a>(
        &'a self,
        line: usize,
        context_lines: usize,
    ) -> std::borrow::Cow<'a, str> {
        let lines: Vec<&str> = self.source.lines().collect();

        if lines.is_empty() {
            return std::borrow::Cow::Borrowed("");
        }

        let start = line.saturating_sub(context_lines);
        let end = std::cmp::min(line + context_lines + 1, lines.len());

        self.extract_lines(start, end)
    }
}

/// Compatibility wrapper for AstParser functionality
///
/// TODO: This will be fully implemented once the engine module is complete.
/// For now, this provides a stub implementation to maintain API compatibility.
pub struct AstParserCompat {
    // TODO: Replace with AstBuilder when engine integration is complete
}

#[cfg(feature = "engine-integration")]
/// Adapter that bridges old detector interface to new analysis context
pub struct DetectorAdapter {
    legacy_detector: Box<dyn crate::analysis::AnalysisDetector>,
    pipeline: Arc<AnalysisPipeline>,
}

#[cfg(feature = "engine-integration")]
impl DetectorAdapter {
    /// Create adapter for legacy detector
    pub fn new(
        legacy_detector: Box<dyn crate::analysis::AnalysisDetector>,
        pipeline: Arc<AnalysisPipeline>,
    ) -> Self {
        Self {
            legacy_detector,
            pipeline,
        }
    }
}

#[cfg(feature = "engine-integration")]
impl Detector for DetectorAdapter {
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError> {
        // Convert AnalysisContext to ParsedFileCompat for legacy detector
        let compat_file = ParsedFileCompat {
            file_path: Arc::new(context.file_info.path.clone()),
            language: context.file_info.language.clone(),
            tree: context.syntax_tree.clone(),
            source: Arc::new(context.source.clone()),
            custom_ast: Arc::new(None), // Legacy AST not available
            modified_at: context.file_info.modified_at.into(),
        };

        // Call legacy detector with compatibility wrapper
        // Use a more robust async execution approach
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| PipelineError::DetectorError("No tokio runtime available".to_string()))?;

        rt.block_on(async {
            self.legacy_detector
                .detect_issues(&compat_file)
                .await
                .map_err(|e| {
                    PipelineError::DetectorError(format!(
                        "Legacy detector '{}' failed: {}",
                        self.legacy_detector.get_detector_name(),
                        e
                    ))
                })
        })
    }

    fn name(&self) -> &str {
        self.legacy_detector.get_detector_name()
    }

    fn supports_language(&self, language: &SourceLanguage) -> bool {
        // Check if legacy detector has specific language support
        // For now, assume most legacy detectors support all languages
        // This could be enhanced by adding a language support method to AnalysisDetector
        match self.legacy_detector.get_detector_name() {
            "GodObjectDetector" | "CodeDuplicationDetector" | "DeadCodeDetector" => true,
            _ => true, // Default to supporting all languages for unknown detectors
        }
    }
}

#[cfg(feature = "engine-integration")]
/// Helper to convert AnalysisContext to ParsedFileCompat
impl From<&AnalysisContext> for ParsedFileCompat {
    fn from(context: &AnalysisContext) -> Self {
        Self {
            file_path: Arc::new(context.file_info.path.clone()),
            language: context.file_info.language.clone(),
            tree: context.syntax_tree.clone(),
            source: Arc::new(context.source.clone()),
            custom_ast: Arc::new(None),
            modified_at: context.file_info.modified_at.into(),
        }
    }
}

impl AstParserCompat {
    /// Create a new compatibility wrapper
    pub fn new() -> Result<Self, crate::ast::tree_sitter_impl::AstError> {
        Ok(Self {})
    }

    /// Parse a file using the new engine but return the old interface (stub)
    pub fn parse_file(
        &self,
        file_path: &Path,
    ) -> Result<ParsedFileCompat, crate::ast::tree_sitter_impl::AstError> {
        // TODO: Implement using AstBuilder when engine is complete
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| crate::ast::tree_sitter_impl::AstError::Other(e.to_string()))?;

        let language = SourceLanguage::from_path(file_path).ok_or_else(|| {
            crate::ast::tree_sitter_impl::AstError::UnsupportedLanguage("Unknown".to_string())
        })?;

        Ok(ParsedFileCompat::new(
            file_path.to_path_buf(),
            language,
            source,
        ))
    }

    /// Parse content directly (stub)
    pub fn parse_content(
        &self,
        content: &str,
        file_path: &Path,
        language: SourceLanguage,
    ) -> Result<ParsedFileCompat, crate::ast::tree_sitter_impl::AstError> {
        // TODO: Implement using AstBuilder when engine is complete
        Ok(ParsedFileCompat::new(
            file_path.to_path_buf(),
            language,
            content.to_string(),
        ))
    }
}
