pub use uveddi_plugin_api::models::DependencyGraph;

/// Analysis results container
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    pub cycles: Vec<Cycle>,
    pub total_modules: usize,
    pub total_dependencies: usize,
    pub analysis_duration: std::time::Duration,
}

pub use uveddi_plugin_api::models::{Cycle, CycleSeverity};
