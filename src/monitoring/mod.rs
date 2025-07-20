//! Test infrastructure monitoring system (UV-219)
//!
//! Provides real-time test execution monitoring, failure categorization,
//! and performance tracking for the Uveddi static code analysis platform.

// Temporarily commented out modules with external dependencies
// pub mod dashboard;
pub mod metrics;
// pub mod websocket;
// pub mod database;
// pub mod classification;
pub mod memory_monitor;
pub mod performance_metrics_collector;
pub mod parallel_metrics;
pub mod enterprise_metrics;
pub mod baseline_collector;

// Temporarily commented out exports with external dependencies
// pub use dashboard::MonitoringDashboard;
pub use metrics::{TestExecution, TestMetrics, TestResult};
// pub use websocket::WebSocketManager;
// pub use database::MonitoringDatabase;
// pub use classification::FailureClassifier;
pub use memory_monitor::MemoryMonitor;
pub use performance_metrics_collector::PerformanceMetricsCollector;
pub use enterprise_metrics::{EnterpriseMetricsCollector, BaselineData, MeasurementSnapshot, RegressionAnalysis};
pub use baseline_collector::{BaselineCollector, BaselineCollectionConfig, BaselineComparison, StoredBaseline};
