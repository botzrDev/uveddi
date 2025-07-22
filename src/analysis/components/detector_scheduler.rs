//! Detector scheduler implementation
//!
//! Orchestrates the execution of analysis detectors across the codebase.

use super::plugin_manager::PluginManagerHandle;
use super::traits::{
    AnalysisAggregator, AstProvider, ConfigurationService,
    DetectorScheduler as DetectorSchedulerTrait, PluginManagerHandle as PluginManagerHandleTrait,
};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::{detectors::cycle::CycleDetector, AnalysisDetector};
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use crate::ingestion::AsyncWalker;

use async_trait::async_trait;
use log::{info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

/// Detector scheduler implementation
pub struct DetectorScheduler {
    config_service: Arc<dyn ConfigurationService>,
    ast_provider: Arc<dyn AstProvider>,
    plugin_manager: Option<PluginManagerHandle>,
    aggregator: Arc<dyn AnalysisAggregator>,
    file_detectors: Arc<RwLock<Vec<Box<dyn AnalysisDetector + Send + Sync>>>>,
    cycle_detector: CycleDetector,
}

impl DetectorScheduler {
    /// Creates a new detector scheduler
    pub fn new(
        config_service: Arc<dyn ConfigurationService>,
        ast_provider: Arc<dyn AstProvider>,
        plugin_manager: Option<PluginManagerHandle>,
        aggregator: Arc<dyn AnalysisAggregator>,
        file_detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    ) -> Self {
        Self {
            config_service,
            ast_provider,
            plugin_manager,
            aggregator,
            file_detectors: Arc::new(RwLock::new(file_detectors)),
            cycle_detector: CycleDetector::new(),
        }
    }

    /// Adds a detector to the scheduler
    pub async fn add_detector(
        &self,
        detector: Box<dyn AnalysisDetector + Send + Sync>,
    ) -> Result<(), UveddiError> {
        let mut detectors = self.file_detectors.write().await;
        detectors.push(detector);
        Ok(())
    }

    /// Analyzes a single file with all enabled detectors
    async fn analyze_file(&self, file_path: &Path) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        info!("Analyzing file: {}", file_path.display());

        // Get AST for the file
        let ast = self.ast_provider.get_ast(file_path).await?;

        // Create ParsedFile for detector execution
        let file_content =
            std::fs::read_to_string(file_path).map_err(|e| UveddiError::IoError {
                operation: "reading file for analysis".to_string(),
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
                suggestion: "Ensure the file exists and is readable".to_string(),
                source: Some(e),
            })?;

        let parsed_file = crate::ast::ParsedFile {
            file_path: Arc::new(file_path.to_path_buf()),
            language: self.detect_language_from_path(file_path),
            tree: Some((*ast).clone()),
            source: Arc::new(file_content),
            custom_ast: Arc::new(None),
            modified_at: std::fs::metadata(file_path)?.modified()?,
        };

        let mut all_issues = Vec::new();

        // Run file-level detectors
        {
            let detectors = self.file_detectors.read().await;
            for detector in detectors.iter() {
                let detector_name = detector.get_detector_name();

                // Check if detector is enabled
                if !self.config_service.is_detector_enabled(&detector_name) {
                    continue;
                }

                match detector.detect_issues(&parsed_file).await {
                    Ok(mut issues) => {
                        info!(
                            "Detector {} found {} issues in {}",
                            detector_name,
                            issues.len(),
                            file_path.display()
                        );
                        all_issues.append(&mut issues);
                    }
                    Err(e) => {
                        warn!(
                            "Error running detector {} on {}: {}",
                            detector_name,
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Run plugin detectors if available
        if let Some(ref plugin_manager) = self.plugin_manager {
            if let Err(e) = self
                .run_plugin_detectors(plugin_manager, file_path, ast)
                .await
            {
                warn!(
                    "Error running plugin detectors on {}: {}",
                    file_path.display(),
                    e
                );
            }
        }

        // Record findings in aggregator
        self.aggregator.record_findings(all_issues.clone());

        Ok(all_issues)
    }

    /// Runs plugin detectors on a file
    async fn run_plugin_detectors(
        &self,
        plugin_manager: &PluginManagerHandle,
        file_path: &Path,
        ast: Arc<tree_sitter::Tree>,
    ) -> Result<(), UveddiError> {
        // Get plugin configuration
        let plugin_config = self.config_service.get_plugin_config();

        for (plugin_id, _config) in plugin_config {
            match PluginManagerHandleTrait::execute_plugin(
                plugin_manager,
                plugin_id.clone(),
                file_path.to_path_buf(),
                ast.clone(),
            )
            .await
            {
                Ok(plugin_issues) => {
                    if !plugin_issues.is_empty() {
                        info!(
                            "Plugin {} found {} issues in {}",
                            plugin_id,
                            plugin_issues.len(),
                            file_path.display()
                        );
                        self.aggregator.record_findings(plugin_issues);
                    }
                }
                Err(e) => {
                    warn!(
                        "Plugin {} failed on {}: {}",
                        plugin_id,
                        file_path.display(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Detects programming language from file path extension
    fn detect_language_from_path(&self, path: &Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Checks if a file should be analyzed based on configuration
    fn should_analyze_file(&self, file_path: &Path) -> bool {
        // Basic check - ensure it's a source file we can handle
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            matches!(extension, "rs" | "py" | "js" | "ts" | "jsx" | "tsx")
        } else {
            false
        }
    }
}

#[async_trait]
impl DetectorSchedulerTrait for DetectorScheduler {
    async fn schedule_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        if !self.should_analyze_file(file_path) {
            return Ok(Vec::new());
        }

        self.analyze_file(file_path).await
    }

    async fn schedule_directory(
        &self,
        dir_path: &Path,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        info!("Scheduling analysis for directory: {}", dir_path.display());

        let mut all_issues = Vec::new();
        let walker = AsyncWalker::for_source_code();
        let mut file_stream = walker.walk(dir_path);
        let mut files_processed = 0;

        while let Some(file_result) = file_stream.next().await {
            match file_result {
                Ok(file_path) => {
                    if self.should_analyze_file(&file_path) {
                        match self.analyze_file(&file_path).await {
                            Ok(mut file_issues) => {
                                all_issues.append(&mut file_issues);
                                files_processed += 1;

                                // Record that we processed this file
                                self.aggregator.record_file_processed();
                            }
                            Err(e) => {
                                warn!("Failed to analyze file {}: {}", file_path.display(), e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }

        info!(
            "Completed directory analysis: {} files processed, {} issues found",
            files_processed,
            all_issues.len()
        );

        Ok(all_issues)
    }

    async fn schedule_graph_analysis(
        &self,
        graph: &LocalDependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        info!(
            "Running graph-level analysis on dependency graph with {} nodes",
            graph.node_count()
        );

        let mut graph_issues = Vec::new();

        // Run cycle detection
        if self.config_service.is_detector_enabled("cyclic_dependency") {
            let cycle_issues = self.cycle_detector.detect_cycles(graph, 0); // analysis_run_id = 0 for now
            info!("Cycle detector found {} issues", cycle_issues.len());
            graph_issues.extend(cycle_issues);
        }

        // Run other graph-based detectors
        {
            let detectors = self.file_detectors.read().await;
            for detector in detectors.iter() {
                let detector_name = detector.get_detector_name();

                if !self.config_service.is_detector_enabled(&detector_name) {
                    continue;
                }

                // Some detectors can also work on graphs
                let graph_detector_issues = detector.detect(graph);
                if !graph_detector_issues.is_empty() {
                    info!(
                        "Graph detector {} found {} issues",
                        detector_name,
                        graph_detector_issues.len()
                    );
                    graph_issues.extend(graph_detector_issues);
                }
            }
        }

        // Record graph-level findings
        self.aggregator.record_findings(graph_issues.clone());

        info!(
            "Graph analysis completed: {} total graph-level issues found",
            graph_issues.len()
        );

        Ok(graph_issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::components::traits::{
        AnalysisAggregator as AnalysisAggregatorTrait, AstProvider,
        ConfigurationService as ConfigurationServiceTrait,
    };
    use crate::analysis::components::{AnalysisAggregator, AstProviderImpl, ConfigurationService};
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    async fn create_test_scheduler() -> DetectorScheduler {
        let config_service =
            Arc::new(ConfigurationService::new()) as Arc<dyn ConfigurationServiceTrait>;
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap()) as Arc<dyn AstProvider>;
        let aggregator = Arc::new(AnalysisAggregator::new()) as Arc<dyn AnalysisAggregatorTrait>;

        let detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>> =
            vec![Box::new(GodObjectDetector::new(10, 15))];

        DetectorScheduler::new(
            config_service,
            ast_provider,
            None, // No plugin manager for tests
            aggregator,
            detectors,
        )
    }

    #[tokio::test]
    async fn test_detector_scheduler_creation() {
        let scheduler = create_test_scheduler().await;

        // Scheduler should be created successfully
        let detectors = scheduler.file_detectors.read().await;
        assert_eq!(detectors.len(), 1);
    }

    #[tokio::test]
    async fn test_schedule_file_analysis() {
        let scheduler = create_test_scheduler().await;

        // Create a temporary Rust file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        let mut file = fs::File::create(&test_file).unwrap();
        writeln!(file, "fn main() {{ println!(\"Hello, world!\"); }}").unwrap();

        let result = scheduler.schedule_file(&test_file).await;
        assert!(result.is_ok());

        // Should have analyzed the file without errors
        let issues = result.unwrap();
        // Exact number of issues depends on detector implementation
    }

    #[tokio::test]
    async fn test_schedule_directory_analysis() {
        let scheduler = create_test_scheduler().await;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create test files
        let main_rs = src_dir.join("main.rs");
        let mut file = fs::File::create(&main_rs).unwrap();
        writeln!(file, "fn main() {{ }}").unwrap();

        let lib_rs = src_dir.join("lib.rs");
        let mut file = fs::File::create(&lib_rs).unwrap();
        writeln!(file, "pub fn test() {{ }}").unwrap();

        let result = scheduler.schedule_directory(&src_dir).await;
        assert!(result.is_ok());

        // Should have processed files in the directory
        let issues = result.unwrap();
        // Exact results depend on detector behavior
    }

    #[tokio::test]
    async fn test_schedule_graph_analysis() {
        let scheduler = create_test_scheduler().await;

        // Create a simple dependency graph
        let graph = LocalDependencyGraph::new();

        let result = scheduler.schedule_graph_analysis(&graph).await;
        assert!(result.is_ok());

        // Should complete without errors
        let issues = result.unwrap();
        // Empty graph should not have cycle issues
    }

    #[test]
    fn test_should_analyze_file() {
        let scheduler = tokio_test::block_on(create_test_scheduler());

        // Test various file types
        assert!(scheduler.should_analyze_file(Path::new("test.rs")));
        assert!(scheduler.should_analyze_file(Path::new("test.py")));
        assert!(scheduler.should_analyze_file(Path::new("test.js")));
        assert!(scheduler.should_analyze_file(Path::new("test.ts")));

        // Should not analyze non-source files
        assert!(!scheduler.should_analyze_file(Path::new("test.txt")));
        assert!(!scheduler.should_analyze_file(Path::new("README.md")));
        assert!(!scheduler.should_analyze_file(Path::new("Cargo.toml")));
    }

    #[test]
    fn test_language_detection() {
        let scheduler = tokio_test::block_on(create_test_scheduler());

        assert!(matches!(
            scheduler.detect_language_from_path(Path::new("test.rs")),
            crate::ast::SourceLanguage::Rust
        ));

        assert!(matches!(
            scheduler.detect_language_from_path(Path::new("test.py")),
            crate::ast::SourceLanguage::Python
        ));

        assert!(matches!(
            scheduler.detect_language_from_path(Path::new("test.js")),
            crate::ast::SourceLanguage::JavaScript
        ));
    }
}
