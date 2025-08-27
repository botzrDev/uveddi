//! Analysis Orchestrator
//!
//! Main orchestrator coordinating analysis services using the Facade pattern.
//! This replaces the monolithic AnalysisEngine with a lightweight coordinator
//! that delegates to specialized services.

use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::services::performance_service::PerformanceReport;
use crate::analysis::services::{
    AnalysisResult, AnalysisService, DependencyAnalysisService, PerformanceAnalysisService,
};
use crate::database::models::ArchitecturalIssue;

// AI service imports (feature-gated)
#[cfg(feature = "ai")]
use crate::ai::engine::AiAnalysisEngine;
#[cfg(feature = "ai")]
use crate::ai::{AiInsight, AiService};

use crate::core::logging::{debug, info};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Enhanced analysis result with performance metrics and dependency graph
#[derive(Debug, Clone)]
pub struct EnhancedAnalysisResult {
    pub issues: Vec<ArchitecturalIssue>,
    pub dependency_graph: LocalDependencyGraph,
    pub performance_report: PerformanceReport,

    // AI insights are optional and only available when AI feature is enabled
    #[cfg(feature = "ai")]
    pub ai_insights: Option<Vec<AiInsight>>,
}

/// Analysis options for configuring orchestrator behavior
#[derive(Debug, Clone, Default)]
pub struct AnalysisOptions {
    pub enable_performance_monitoring: bool,
    pub enable_dependency_analysis: bool,
    pub enabled_detectors: Option<Vec<String>>,
    pub memory_limit_bytes: Option<usize>,
    pub parallel_execution: bool,

    #[cfg(feature = "ai")]
    pub enable_ai_analysis: bool,
}

/// Main orchestrator coordinating analysis services (Facade Pattern)
pub struct AnalysisOrchestrator {
    analysis_service: Arc<AnalysisService>,
    dependency_service: Arc<DependencyAnalysisService>,
    performance_service: Arc<PerformanceAnalysisService>,

    // Optional AI services (feature-gated)
    #[cfg(feature = "ai")]
    ai_service: Option<Arc<AiAnalysisEngine>>,
}

impl AnalysisOrchestrator {
    /// Create new orchestrator with service dependencies
    pub fn new(
        analysis_service: Arc<AnalysisService>,
        dependency_service: Arc<DependencyAnalysisService>,
        performance_service: Arc<PerformanceAnalysisService>,
    ) -> Self {
        Self {
            analysis_service,
            dependency_service,
            performance_service,
            #[cfg(feature = "ai")]
            ai_service: None,
        }
    }

    /// Create orchestrator with AI support
    #[cfg(feature = "ai")]
    pub fn new_with_ai(
        analysis_service: Arc<AnalysisService>,
        dependency_service: Arc<DependencyAnalysisService>,
        performance_service: Arc<PerformanceAnalysisService>,
        ai_service: Arc<AiAnalysisEngine>,
    ) -> Self {
        Self {
            analysis_service,
            dependency_service,
            performance_service,
            ai_service: Some(ai_service),
        }
    }

    /// Main analysis method - maintains backward compatibility
    pub async fn analyze(
        &self,
        path: &Path,
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        let options = AnalysisOptions::default();
        self.analyze_with_options(path, &options).await
    }

    /// Analysis with configurable options
    pub async fn analyze_with_options(
        &self,
        path: &Path,
        options: &AnalysisOptions,
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        let start_time = Instant::now();
        info!("Starting orchestrated analysis for: {}", path.display());

        let (issues, graph) = if options.parallel_execution {
            self.run_parallel_analysis(path, options).await?
        } else {
            self.run_sequential_analysis(path, options).await?
        };

        info!(
            "Orchestrated analysis completed in {}ms: {} issues, {} dependencies",
            start_time.elapsed().as_millis(),
            issues.len(),
            graph.node_count()
        );

        Ok((issues, graph))
    }

    /// Enhanced analysis with performance monitoring
    pub async fn analyze_with_performance_monitoring(
        &self,
        path: &Path,
    ) -> AnalysisResult<EnhancedAnalysisResult> {
        let monitoring_session = self.performance_service.start_monitoring().await;
        info!(
            "Starting enhanced analysis with performance monitoring for: {}",
            path.display()
        );

        // Run analysis with memory limits
        let (issues, dependency_graph) = self
            .performance_service
            .analyze_with_memory_limits(self.analyze(path))
            .await?;

        // Generate performance report
        let performance_report = self
            .performance_service
            .stop_monitoring(&monitoring_session)
            .await;

        info!(
            "Enhanced analysis completed with {} issues found",
            issues.len()
        );
        Ok(EnhancedAnalysisResult {
            issues,
            dependency_graph,
            performance_report,
            #[cfg(feature = "ai")]
            ai_insights: None,
        })
    }

    /// Memory-aware analysis with adaptive behavior
    pub async fn analyze_with_memory_limits(
        &self,
        path: &Path,
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        debug!("Running memory-aware analysis for: {}", path.display());

        self.performance_service
            .analyze_with_memory_limits(self.analyze(path))
            .await
    }

    /// Analysis with custom detector configuration
    pub async fn analyze_with_detectors(
        &self,
        path: &Path,
        enabled_detectors: &[String],
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        let mut options = AnalysisOptions::default();
        options.enabled_detectors = Some(enabled_detectors.to_vec());

        self.analyze_with_options(path, &options).await
    }

    /// AI-enhanced analysis (if AI features are enabled)
    #[cfg(feature = "ai")]
    pub async fn analyze_with_ai(&self, path: &Path) -> AnalysisResult<EnhancedAnalysisResult> {
        if let Some(ai_service) = &self.ai_service {
            info!("Running AI-enhanced analysis for: {}", path.display());

            // Run standard analysis first
            let mut enhanced_result = self.analyze_with_performance_monitoring(path).await?;

            // Enhance with AI analysis
            match ai_service.analyze_issues(&enhanced_result.issues).await {
                Ok(ai_insights) => {
                    info!(
                        "AI analysis completed with {} additional insights",
                        ai_insights.len()
                    );
                    enhanced_result.ai_insights = Some(ai_insights);
                }
                Err(e) => {
                    warn!(
                        "AI analysis failed: {}, continuing with standard analysis only",
                        e
                    );
                    enhanced_result.ai_insights = None;
                }
            }

            Ok(enhanced_result)
        } else {
            warn!("AI analysis requested but AI service not available");
            self.analyze_with_performance_monitoring(path).await
        }
    }

    /// Get orchestrator status and service health
    pub async fn get_status(&self) -> OrchestratorStatus {
        let analysis_stats = self.analysis_service.get_analysis_stats().await;
        let dependency_stats = self.dependency_service.get_dependency_stats().await;
        let current_memory = self.performance_service.get_current_memory_usage().await;
        let active_sessions = self.performance_service.get_active_sessions().await;

        OrchestratorStatus {
            analysis_stats,
            dependency_stats,
            current_memory_usage: current_memory,
            active_monitoring_sessions: active_sessions.len(),
            services_healthy: true, // TODO: Implement actual health checks
            #[cfg(feature = "ai")]
            ai_service_available: self.ai_service.is_some(),
        }
    }

    /// Clear all service caches
    pub async fn clear_caches(&self) -> AnalysisResult<()> {
        info!("Clearing all service caches");

        // Clear dependency cache
        self.dependency_service.clear_cache().await;

        // TODO: Clear other service caches as they become available

        info!("All service caches cleared");
        Ok(())
    }

    // Private helper methods

    async fn run_parallel_analysis(
        &self,
        path: &Path,
        options: &AnalysisOptions,
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        debug!("Running parallel analysis");

        // Execute analysis and dependency building concurrently
        let (issues, graph) = tokio::try_join!(
            self.run_core_analysis(path, options),
            self.run_dependency_analysis(path, options)
        )?;

        Ok((issues, graph))
    }

    async fn run_sequential_analysis(
        &self,
        path: &Path,
        options: &AnalysisOptions,
    ) -> AnalysisResult<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        debug!("Running sequential analysis");

        // Run analysis first
        let issues = self.run_core_analysis(path, options).await?;

        // Then build dependency graph
        let graph = self.run_dependency_analysis(path, options).await?;

        Ok((issues, graph))
    }

    async fn run_core_analysis(
        &self,
        path: &Path,
        options: &AnalysisOptions,
    ) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        if let Some(detectors) = &options.enabled_detectors {
            self.analysis_service
                .run_analysis_with_detectors(path, detectors)
                .await
        } else {
            self.analysis_service.run_analysis(path).await
        }
    }

    async fn run_dependency_analysis(
        &self,
        path: &Path,
        _options: &AnalysisOptions,
    ) -> AnalysisResult<LocalDependencyGraph> {
        self.dependency_service.build_dependency_graph(path).await
    }
}

/// Status information about the orchestrator and its services
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub analysis_stats: crate::analysis::services::analysis_service::AnalysisStats,
    pub dependency_stats: crate::analysis::services::dependency_service::DependencyStats,
    pub current_memory_usage: usize,
    pub active_monitoring_sessions: usize,
    pub services_healthy: bool,

    #[cfg(feature = "ai")]
    pub ai_service_available: bool,
}

/// Builder for creating AnalysisOrchestrator instances
pub struct AnalysisOrchestratorBuilder {
    analysis_service: Option<Arc<AnalysisService>>,
    dependency_service: Option<Arc<DependencyAnalysisService>>,
    performance_service: Option<Arc<PerformanceAnalysisService>>,

    #[cfg(feature = "ai")]
    ai_service: Option<Arc<AiAnalysisEngine>>,
}

impl AnalysisOrchestratorBuilder {
    pub fn new() -> Self {
        Self {
            analysis_service: None,
            dependency_service: None,
            performance_service: None,
            #[cfg(feature = "ai")]
            ai_service: None,
        }
    }

    pub fn with_analysis_service(mut self, service: Arc<AnalysisService>) -> Self {
        self.analysis_service = Some(service);
        self
    }

    pub fn with_dependency_service(mut self, service: Arc<DependencyAnalysisService>) -> Self {
        self.dependency_service = Some(service);
        self
    }

    pub fn with_performance_service(mut self, service: Arc<PerformanceAnalysisService>) -> Self {
        self.performance_service = Some(service);
        self
    }

    #[cfg(feature = "ai")]
    pub fn with_ai_service(mut self, service: Arc<AiAnalysisEngine>) -> Self {
        self.ai_service = Some(service);
        self
    }

    pub fn with_default_configuration(self) -> Self {
        // TODO: Implement default service creation
        self
    }

    pub fn build(self) -> AnalysisResult<AnalysisOrchestrator> {
        let analysis_service = self.analysis_service.ok_or_else(|| {
            crate::analysis::errors::AnalysisError::configuration_error(
                "analysis_service",
                "missing",
                "Analysis service must be configured",
            )
        })?;

        let dependency_service = self.dependency_service.ok_or_else(|| {
            crate::analysis::errors::AnalysisError::configuration_error(
                "dependency_service",
                "missing",
                "Dependency service must be configured",
            )
        })?;

        let performance_service = self.performance_service.ok_or_else(|| {
            crate::analysis::errors::AnalysisError::configuration_error(
                "performance_service",
                "missing",
                "Performance service must be configured",
            )
        })?;

        #[cfg(feature = "ai")]
        {
            if let Some(ai_service) = self.ai_service {
                Ok(AnalysisOrchestrator::new_with_ai(
                    analysis_service,
                    dependency_service,
                    performance_service,
                    ai_service,
                ))
            } else {
                Ok(AnalysisOrchestrator::new(
                    analysis_service,
                    dependency_service,
                    performance_service,
                ))
            }
        }

        #[cfg(not(feature = "ai"))]
        {
            Ok(AnalysisOrchestrator::new(
                analysis_service,
                dependency_service,
                performance_service,
            ))
        }
    }
}

impl Default for AnalysisOrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::components::ast_provider::AstProviderImpl;
    use crate::analysis::components::cache_manager::CacheManagerImpl;
    use crate::analysis::components::{
        analysis_aggregator::AnalysisAggregator, config_service::ConfigurationService,
        dependency_graph_builder::DependencyGraphBuilderImpl,
        detector_scheduler::DetectorScheduler,
    };
    use crate::analysis::services::performance_service::{
        MemoryConfig, PerformanceAnalysisService,
    };
    use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;

    fn create_test_orchestrator() -> AnalysisOrchestrator {
        // Create mock services for testing
        let config_service = Arc::new(ConfigurationService::new_with_defaults());
        let ast_provider_impl = Arc::new(AstProviderImpl::new().unwrap());
        let ast_provider =
            ast_provider_impl.clone() as Arc<dyn crate::analysis::components::traits::AstProvider>;
        let aggregator = Arc::new(AnalysisAggregator::new());
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            None,
            aggregator.clone(),
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors(),
        ));
        let detector_factory = Arc::new(crate::analysis::detector_factory::DetectorFactory::new());

        let analysis_service = Arc::new(AnalysisService::new(
            config_service,
            detector_scheduler,
            aggregator,
            None,
            detector_factory,
        ));

        let dependency_builder =
            Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone()).unwrap());
        let cache_manager = Arc::new(futures::executor::block_on(CacheManagerImpl::new()).unwrap());

        let dependency_service = Arc::new(DependencyAnalysisService::new(
            // Dependency service expects concrete types
            ast_provider_impl.clone(),
            dependency_builder,
            cache_manager,
        ));

        let metrics_collector = Arc::new(PerformanceMetricsCollector::new(
            crate::database::models::PerformanceMetricsConfig::default(),
            10,
        ));
        let performance_service = Arc::new(PerformanceAnalysisService::new(
            metrics_collector,
            MemoryConfig::default(),
        ));

        AnalysisOrchestrator::new(analysis_service, dependency_service, performance_service)
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = create_test_orchestrator();
        let status = orchestrator.get_status().await;

        assert!(status.services_healthy);
        assert_eq!(status.active_monitoring_sessions, 0);
    }

    #[tokio::test]
    async fn test_analysis_options_default() {
        let options = AnalysisOptions::default();

        assert!(!options.enable_performance_monitoring);
        assert!(!options.enable_dependency_analysis);
        assert!(options.enabled_detectors.is_none());
        assert!(options.memory_limit_bytes.is_none());
        assert!(!options.parallel_execution);
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let builder = AnalysisOrchestratorBuilder::new();

        // Builder should start empty
        assert!(builder.analysis_service.is_none());
        assert!(builder.dependency_service.is_none());
        assert!(builder.performance_service.is_none());
    }

    #[tokio::test]
    async fn test_clear_caches() {
        let orchestrator = create_test_orchestrator();

        // Should not panic
        let result = orchestrator.clear_caches().await;
        assert!(result.is_ok());
    }
}
