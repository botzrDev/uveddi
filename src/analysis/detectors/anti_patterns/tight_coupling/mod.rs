pub mod analysis;
pub mod config;
pub mod detector;
pub mod issue_evaluator;
pub mod language_support;
pub mod metrics;
pub mod report;
pub mod types;
pub mod visualization;

#[cfg(test)]
mod tests;

// Re-export main components for easy access
pub use config::TightCouplingConfig;
pub use detector::{CrossFileAnalysisDetector, TightCouplingDetector};
pub use issue_evaluator::IssueEvaluator;
pub use report::{CouplingAnalysisReport, ReportGenerator};
pub use types::{CouplingMetrics, CouplingThresholds, Dependency, DependencyStrength};

// Re-export analysis components
pub use analysis::{CouplingCalculator, DependencyAnalyzer, ImportAnalyzer, InterfaceAnalyzer};

// Re-export language support
pub use language_support::{LanguageAnalyzer, PythonAnalyzer, RustAnalyzer, TypeScriptAnalyzer};

// Re-export metrics calculators
pub use metrics::{AfferentCouplingCalculator, EfferentCouplingCalculator, InstabilityCalculator};

// Re-export visualization components
pub use visualization::{CouplingMatrixGenerator, DependencyGraphVisualizer};