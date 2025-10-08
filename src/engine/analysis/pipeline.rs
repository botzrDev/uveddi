//! # Analysis Pipeline
//!
//! Orchestrates detector execution with the new analysis context.
//! Provides clear separation between business logic and AST parsing.

use super::performance::{AnalysisInstrumentation, AnalysisMetrics};
use super::AnalysisContext;
use crate::database::models::ArchitecturalIssue;
use crate::engine::analysis::context::{
    DependencySource, FileInfo, ProjectContext, ProjectDependency,
};
use crate::engine::parsing::{AstBuilder, LanguageParser, ParseResult};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

// Cache imports when feature is enabled
#[cfg(feature = "analysis-cache")]
use crate::engine::analysis::context::CacheHandles;
#[cfg(feature = "analysis-cache")]
use crate::engine::cache::{AnalysisCache, AstCache, CacheServiceManager};
#[cfg(feature = "analysis-cache")]
use std::sync::Mutex;

/// Analysis pipeline error types
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Context building failed: {0}")]
    ContextBuildError(String),

    #[error("Detector execution failed: {0}")]
    DetectorError(String),

    #[error("Result aggregation failed: {0}")]
    AggregationError(String),
}

/// Result of analysis pipeline execution
#[derive(Debug)]
pub struct AnalysisResult {
    pub issues: Vec<ArchitecturalIssue>,
    pub context: AnalysisContext,
    pub execution_time: std::time::Duration,
    pub performance_metrics: Option<AnalysisMetrics>,
}

/// Analysis pipeline that orchestrates detector execution
pub struct AnalysisPipeline {
    /// Registered detectors
    detectors: Vec<Box<dyn Detector>>,

    /// Context builder for preparing analysis context
    context_builder: Arc<ContextBuilder>,

    /// Performance instrumentation enabled
    performance_enabled: bool,

    /// Cache handles (when caching is enabled)
    #[cfg(feature = "analysis-cache")]
    cache_handles: Option<CacheHandles>,

    /// Cache service manager (when caching is enabled)
    #[cfg(feature = "analysis-cache")]
    cache_service_manager: Option<std::sync::Arc<std::sync::Mutex<CacheServiceManager>>>,
}

/// Trait for analysis detectors using the new context
pub trait Detector: Send + Sync {
    /// Detect issues using the analysis context
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError>;

    /// Get detector name
    fn name(&self) -> &str;

    /// Check if detector supports the given language
    fn supports_language(&self, language: &crate::ast::SourceLanguage) -> bool;
}

/// Builds analysis context from parse results
pub struct ContextBuilder {
    ast_builder: Arc<AstBuilder>,

    /// Cache handles (when caching is enabled)
    #[cfg(feature = "analysis-cache")]
    cache_handles: Option<CacheHandles>,
}

impl ContextBuilder {
    /// Create new context builder
    pub fn new(ast_builder: Arc<AstBuilder>) -> Self {
        Self {
            ast_builder,
            #[cfg(feature = "analysis-cache")]
            cache_handles: None,
        }
    }

    /// Create context builder with cache handles
    #[cfg(feature = "analysis-cache")]
    pub fn with_caches(ast_builder: Arc<AstBuilder>, cache_handles: CacheHandles) -> Self {
        Self {
            ast_builder,
            cache_handles: Some(cache_handles),
        }
    }

    /// Build analysis context from file path
    pub fn build_context(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisContext, PipelineError> {
        // Try to get AST from cache first if caching is enabled
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache_handles) = self.cache_handles {
            if let Ok(mut ast_cache) = cache_handles.ast_cache.lock() {
                if let Some(cached_entry) = ast_cache.get(&file_path.to_path_buf()) {
                    // Use cached AST - language detection from file extension
                    let language = crate::ast::SourceLanguage::from_path(file_path)
                        .ok_or_else(|| PipelineError::ContextBuildError(
                            format!("Unsupported file extension for '{}'", file_path.display())
                        ))?;

                    let file_info = FileInfo {
                        path: file_path.to_path_buf(),
                        language,
                        lines_of_code: cached_entry.source.lines().count(),
                        size_bytes: cached_entry.source.len(),
                        modified_at: cached_entry.modified_time,
                    };

                    return Ok(AnalysisContext::with_caches(
                        file_info,
                        Some(cached_entry.tree.clone()),
                        cached_entry.source.clone(),
                        Vec::new(), // TODO: Cache symbols and relations
                        Vec::new(),
                        project_context,
                        cache_handles.clone(),
                    ));
                }
            }
        }

        // Parse file using AstBuilder (cache miss or no caching)
        let parse_result = self
            .ast_builder
            .parse_file(file_path)
            .map_err(|e| PipelineError::ContextBuildError(format!("Parse failed: {}", e)))?;

        // Build file info
        let file_info = FileInfo {
            path: file_path.to_path_buf(),
            language: parse_result.language.clone(),
            lines_of_code: parse_result.source.lines().count(),
            size_bytes: parse_result.source.len(),
            modified_at: std::time::SystemTime::now(),
        };

        // Cache the AST if caching is enabled
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache_handles) = self.cache_handles {
            if let Ok(mut ast_cache) = cache_handles.ast_cache.lock() {
                if let Some(ref tree) = parse_result.tree {
                    let _ = ast_cache.put(
                        file_path.to_path_buf(),
                        tree.clone(),
                        parse_result.source.to_string(),
                    );
                }
            }
        }

        // Create analysis context
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache_handles) = self.cache_handles {
            Ok(AnalysisContext::with_caches(
                file_info,
                parse_result.tree,
                parse_result.source.to_string(),
                parse_result.symbols,
                parse_result.relations,
                project_context,
                cache_handles.clone(),
            ))
        } else {
            Ok(AnalysisContext::new(
                file_info,
                parse_result.tree,
                parse_result.source.to_string(),
                parse_result.symbols,
                parse_result.relations,
                project_context,
            ))
        }

        #[cfg(not(feature = "analysis-cache"))]
        Ok(AnalysisContext::new(
            file_info,
            parse_result.tree,
            parse_result.source.to_string(),
            parse_result.symbols,
            parse_result.relations,
            project_context,
        ))
    }

    /// Build context from existing parse result
    pub fn build_from_parse_result(
        &self,
        parse_result: ParseResult,
        project_context: ProjectContext,
    ) -> AnalysisContext {
        let file_info = FileInfo {
            path: parse_result.path().to_path_buf(),
            language: parse_result.language.clone(),
            lines_of_code: parse_result.source.lines().count(),
            size_bytes: parse_result.source.len(),
            modified_at: parse_result.modified_at,
        };

        // Use caches if available
        #[cfg(feature = "analysis-cache")]
        if let Some(ref cache_handles) = self.cache_handles {
            return AnalysisContext::with_caches(
                file_info,
                parse_result.tree,
                parse_result.source.to_string(),
                parse_result.symbols,
                parse_result.relations,
                project_context,
                cache_handles.clone(),
            );
        }

        AnalysisContext::new(
            file_info,
            parse_result.tree,
            parse_result.source.to_string(),
            parse_result.symbols,
            parse_result.relations,
            project_context,
        )
    }
}

impl AnalysisPipeline {
    /// Create a new analysis pipeline
    pub fn new(ast_builder: Arc<AstBuilder>) -> Self {
        Self {
            detectors: Vec::new(),
            context_builder: Arc::new(ContextBuilder::new(ast_builder)),
            performance_enabled: false,
            #[cfg(feature = "analysis-cache")]
            cache_handles: None,
            #[cfg(feature = "analysis-cache")]
            cache_service_manager: None,
        }
    }

    /// Create analysis pipeline with cache support
    #[cfg(feature = "analysis-cache")]
    pub fn with_caches(ast_builder: Arc<AstBuilder>, cache_handles: CacheHandles) -> Self {
        let context_builder = Arc::new(ContextBuilder::with_caches(
            ast_builder,
            cache_handles.clone(),
        ));

        // Create and start cache services
        let service_manager = CacheServiceManager::new(cache_handles.clone());
        let service_manager_arc = std::sync::Arc::new(std::sync::Mutex::new(service_manager));

        Self {
            detectors: Vec::new(),
            context_builder,
            performance_enabled: false,
            cache_handles: Some(cache_handles),
            cache_service_manager: Some(service_manager_arc),
        }
    }

    /// Enable performance instrumentation
    pub fn with_performance_instrumentation(mut self, enabled: bool) -> Self {
        self.performance_enabled = enabled;
        self
    }

    /// Add a detector to the pipeline
    pub fn with_detector(mut self, detector: Box<dyn Detector>) -> Self {
        self.detectors.push(detector);
        self
    }

    /// Analyze a file using the pipeline
    pub fn analyze(&self, context: AnalysisContext) -> Result<AnalysisResult, PipelineError> {
        let start_time = std::time::Instant::now();
        let mut all_issues = Vec::new();
        let mut instrumentation = AnalysisInstrumentation::new(self.performance_enabled);

        // Start detection phase
        instrumentation.start_phase("detection");

        // Run all applicable detectors
        for detector in &self.detectors {
            if detector.supports_language(&context.file_info.language) {
                let detector_start = Instant::now();

                match detector.detect(&context) {
                    Ok(mut issues) => {
                        let issue_count = issues.len();
                        all_issues.append(&mut issues);
                        instrumentation.record_issues_found(issue_count);
                    }
                    Err(e) => {
                        eprintln!("Detector {} failed: {}", detector.name(), e);
                        // Continue with other detectors
                    }
                }

                let detector_duration = detector_start.elapsed();
                instrumentation.record_detector_time(detector.name(), detector_duration);
            }
        }

        instrumentation.record_file_processed();
        let metrics = if self.performance_enabled {
            Some(instrumentation.finalize())
        } else {
            None
        };

        Ok(AnalysisResult {
            issues: all_issues,
            context,
            execution_time: start_time.elapsed(),
            performance_metrics: metrics,
        })
    }

    /// Analyze a file by path, building context automatically
    pub fn analyze_file(
        &self,
        file_path: &Path,
        project_context: ProjectContext,
    ) -> Result<AnalysisResult, PipelineError> {
        let context = self
            .context_builder
            .build_context(file_path, project_context)?;
        self.analyze(context)
    }

    /// Get registered detector count
    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }

    /// Start cache services (file watcher and metrics collector)
    #[cfg(feature = "analysis-cache")]
    pub fn start_cache_services(&self, watch_paths: Vec<std::path::PathBuf>) -> Result<(), PipelineError> {
        if let Some(ref manager_arc) = self.cache_service_manager {
            if let Ok(mut manager) = manager_arc.lock() {
                manager.start_services(watch_paths)
                    .map_err(|e| PipelineError::ContextBuildError(format!("Failed to start cache services: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Stop cache services gracefully
    #[cfg(feature = "analysis-cache")]
    pub fn stop_cache_services(&self) {
        if let Some(ref manager_arc) = self.cache_service_manager {
            if let Ok(mut manager) = manager_arc.lock() {
                manager.stop_services();
            }
        }
    }

    /// Check if cache services are running
    #[cfg(feature = "analysis-cache")]
    pub fn cache_services_running(&self) -> bool {
        if let Some(ref manager_arc) = self.cache_service_manager {
            if let Ok(manager) = manager_arc.lock() {
                return manager.is_running();
            }
        }
        false
    }
}
