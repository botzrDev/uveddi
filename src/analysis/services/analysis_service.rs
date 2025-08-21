//! Core Analysis Service
//!
//! This service handles core analysis coordination, replacing the analysis coordination
//! logic from the monolithic AnalysisEngine. It manages detector scheduling, result
//! aggregation, and analysis workflow orchestration.

use crate::analysis::components::traits::{
    AnalysisAggregator as AnalysisAggregatorTrait, DetectorScheduler as DetectorSchedulerTrait,
    PluginManagerHandle as PluginManagerHandleTrait,
};
use crate::analysis::components::{
    AnalysisAggregator, ConfigurationService, DetectorScheduler, PluginManagerHandle,
};
use crate::analysis::detector_factory::DetectorFactory;
use crate::analysis::file_discovery::{FileDiscovery, SourceFile};
use crate::analysis::symbols::GlobalSymbolTable;
use crate::analysis::workspace::{WorkspaceDetector, WorkspaceInfo, WorkspaceType};
use crate::ast::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::ingestion::AsyncWalker;

use super::{AnalysisResult, ServiceConfiguration};

use crate::core::logging::{debug, info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Statistics about analysis execution
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    pub total_files_analyzed: usize,
    pub total_issues_found: usize,
    pub analysis_duration_ms: u64,
    pub issues_by_type: std::collections::HashMap<AntiPatternType, usize>,
}

/// Information about available detectors
#[derive(Debug, Clone)]
pub struct DetectorInfo {
    pub name: String,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub enabled: bool,
}

/// Core analysis service responsible for coordinating analysis operations
pub struct AnalysisService {
    config_service: Arc<ConfigurationService>,
    detector_scheduler: Arc<DetectorScheduler>,
    aggregator: Arc<AnalysisAggregator>,
    plugin_manager: Option<Arc<PluginManagerHandle>>,
    detector_factory: Arc<DetectorFactory>,
    workspace_detector: WorkspaceDetector,
    file_discovery: FileDiscovery,
    stats: Arc<Mutex<AnalysisStats>>,
}

impl AnalysisService {
    /// Create new analysis service with injected dependencies
    pub fn new(
        config_service: Arc<ConfigurationService>,
        detector_scheduler: Arc<DetectorScheduler>,
        aggregator: Arc<AnalysisAggregator>,
        plugin_manager: Option<Arc<PluginManagerHandle>>,
        detector_factory: Arc<DetectorFactory>,
    ) -> Self {
        Self {
            config_service,
            detector_scheduler,
            aggregator,
            plugin_manager,
            detector_factory,
            workspace_detector: WorkspaceDetector,
            file_discovery: FileDiscovery::new(),
            stats: Arc::new(Mutex::new(AnalysisStats::default())),
        }
    }

    /// Run comprehensive analysis on a path
    pub async fn run_analysis(&self, path: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting comprehensive analysis for path: {}",
            path.display()
        );

        // Detect workspace information
        let workspace_info = self.detect_workspace(path).await?;
        info!("Detected workspace: {:?}", workspace_info.manifest_paths);

        // Discover files to analyze
        let source_files = self.discover_source_files(path).await?;
        info!(
            "Discovered {} source files for analysis",
            source_files.len()
        );

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_files_analyzed = source_files.len();
        }

        // Record files processed in aggregator for accurate reporting
        for _ in 0..source_files.len() {
            self.aggregator.record_file_processed();
        }

        // Run analysis based on path type
        let issues = if path.is_file() {
            self.analyze_single_file(path).await?
        } else {
            self.analyze_directory(&source_files).await?
        };

        // Record findings in aggregator
        self.aggregator.record_findings(issues.clone());

        // Update final stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_issues_found = issues.len();
            stats.analysis_duration_ms = start_time.elapsed().as_millis() as u64;

            // Count issues by type
            for issue in &issues {
                let anti_pattern_type = AntiPatternType {
                    anti_pattern_type_id: Some(issue.anti_pattern_type_id),
                    name: format!("Type {}", issue.anti_pattern_type_id),
                    description: "Auto-generated type for stats".to_string(),
                    category: "unknown".to_string(),
                };
                *stats.issues_by_type.entry(anti_pattern_type).or_insert(0) += 1;
            }
        }

        info!(
            "Analysis completed: {} issues found in {}ms",
            issues.len(),
            start_time.elapsed().as_millis()
        );
        Ok(issues)
    }

    /// Run analysis on a single file
    pub async fn analyze_single_file(
        &self,
        file_path: &Path,
    ) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        debug!("Analyzing single file: {}", file_path.display());

        // Create source file representation
        let source_file = SourceFile::new(file_path.to_path_buf())?;

        // Schedule analysis through detector scheduler
        self.detector_scheduler
            .schedule_file_analysis(&source_file)
            .await
            .map_err(Into::into)
    }

    /// Run analysis on multiple files in a directory
    async fn analyze_directory(
        &self,
        source_files: &[SourceFile],
    ) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        debug!("Analyzing {} files in directory", source_files.len());

        // Schedule batch analysis through detector scheduler
        // Run analysis on each file in the directory
        let mut all_issues = Vec::new();
        for source_file in source_files {
            match self
                .detector_scheduler
                .schedule_file_analysis(source_file)
                .await
            {
                Ok(mut issues) => all_issues.append(&mut issues),
                Err(e) => warn!("Failed to analyze file {:?}: {}", source_file.path, e),
            }
        }
        Ok(all_issues)
    }

    /// Discover source files in the given path
    async fn discover_source_files(&self, path: &Path) -> AnalysisResult<Vec<SourceFile>> {
        if path.is_file() {
            return Ok(vec![SourceFile::new(path.to_path_buf())?]);
        }

        let workspace_type = WorkspaceDetector::detect_composite_workspace_type(path);
        // Use new filtered discovery
        let files = self
            .file_discovery
            .discover_files_for_workspace_type(path, &workspace_type)?;
        Ok(files)
    }

    /// Detect workspace information for the given path
    async fn detect_workspace(&self, path: &Path) -> AnalysisResult<WorkspaceInfo> {
        match WorkspaceDetector::detect_workspace(path).await {
            Ok(Some(workspace)) => Ok(workspace),
            Ok(None) => Err(
                crate::analysis::errors::AnalysisError::workspace_discovery_error(
                    path.display().to_string(),
                    "No workspace or crate detected",
                )
                .into(),
            ),
            Err(e) => Err(e.into()),
        }
    }

    /// Get supported detector information
    pub fn get_supported_detectors(&self) -> Vec<DetectorInfo> {
        self.detector_factory
            .available_detectors()
            .into_iter()
            .map(|name| DetectorInfo {
                name: name.clone(),
                description: format!("Detector: {}", name),
                supported_languages: vec!["Rust".to_string(), "Python".to_string()], // Default supported languages
                enabled: true, // Default enabled state
            })
            .collect()
    }

    /// Get analysis statistics
    pub async fn get_analysis_stats(&self) -> AnalysisStats {
        self.stats.lock().await.clone()
    }

    /// Reset analysis statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = AnalysisStats::default();
    }

    /// Check if plugins are enabled and available
    pub fn has_plugin_support(&self) -> bool {
        self.plugin_manager.is_some()
    }

    /// Get plugin manager handle if available
    pub fn get_plugin_manager(&self) -> Option<Arc<PluginManagerHandle>> {
        self.plugin_manager.clone()
    }

    /// Run analysis with custom detector configuration
    pub async fn run_analysis_with_detectors(
        &self,
        path: &Path,
        enabled_detectors: &[String],
    ) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        info!(
            "Running analysis with custom detectors: {:?}",
            enabled_detectors
        );

        // Configure detector scheduler with specific detectors
        self.detector_scheduler
            .configure_enabled_detectors(enabled_detectors.to_vec())
            .await?;

        // Run standard analysis
        self.run_analysis(path).await
    }

    /// Analyze with symbol table integration
    pub async fn run_analysis_with_symbols(
        &self,
        path: &Path,
        symbol_table: Arc<GlobalSymbolTable>,
    ) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        info!("Running analysis with symbol table integration");

        // Configure detector scheduler with symbol table
        self.detector_scheduler
            .set_symbol_table(symbol_table)
            .await?;

        // Run standard analysis
        self.run_analysis(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::components::ast_provider::AstProviderImpl;
    use crate::analysis::components::{
        analysis_aggregator::AnalysisAggregator, config_service::ConfigurationService,
        dependency_graph_builder::DependencyGraphBuilderImpl,
        detector_scheduler::DetectorScheduler,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_analysis_service() -> AnalysisService {
        let config_service = Arc::new(ConfigurationService::new_with_defaults());
        let aggregator = Arc::new(AnalysisAggregator::new());
        let ast_provider_impl = Arc::new(AstProviderImpl::new().unwrap());
        let ast_provider_trait: Arc<dyn crate::analysis::components::traits::AstProvider> =
            ast_provider_impl.clone();
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider_trait,
            None,
            aggregator.clone(),
            DetectorFactory::create_default_detectors(),
        ));
        let detector_factory = Arc::new(DetectorFactory::new());

        AnalysisService::new(
            config_service,
            detector_scheduler,
            aggregator,
            None, // No plugin manager for tests
            detector_factory,
        )
    }

    #[tokio::test]
    async fn test_analysis_service_creation() {
        let service = create_test_analysis_service();
        assert!(!service.has_plugin_support());
        assert!(service.get_plugin_manager().is_none());
    }

    #[tokio::test]
    async fn test_analysis_stats_initialization() {
        let service = create_test_analysis_service();
        let stats = service.get_analysis_stats().await;

        assert_eq!(stats.total_files_analyzed, 0);
        assert_eq!(stats.total_issues_found, 0);
        assert_eq!(stats.analysis_duration_ms, 0);
        assert!(stats.issues_by_type.is_empty());
    }

    #[tokio::test]
    async fn test_supported_detectors() {
        let service = create_test_analysis_service();
        let detectors = service.get_supported_detectors();

        // Should have some detectors available
        assert!(!detectors.is_empty());

        // Each detector should have required fields
        for detector in detectors {
            assert!(!detector.name.is_empty());
            assert!(!detector.description.is_empty());
        }
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let service = create_test_analysis_service();

        // Modify stats manually for testing
        {
            let mut stats = service.stats.lock().await;
            stats.total_files_analyzed = 5;
            stats.total_issues_found = 10;
        }

        // Reset stats
        service.reset_stats().await;

        // Verify reset
        let stats = service.get_analysis_stats().await;
        assert_eq!(stats.total_files_analyzed, 0);
        assert_eq!(stats.total_issues_found, 0);
    }
}
