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
pub mod baseline_collector;
#[cfg(feature = "prometheus")]
pub mod carbon_metrics;
pub mod config;
pub mod distribution;
pub mod enterprise_metrics;
pub mod memory_monitor;
pub mod parallel_metrics;
pub mod performance_metrics_collector;
pub mod reporting;
pub mod scheduler;

// Temporarily commented out exports with external dependencies
// pub use dashboard::MonitoringDashboard;
pub use metrics::{TestExecution, TestMetrics, TestResult};
// pub use websocket::WebSocketManager;
// pub use database::MonitoringDatabase;
// pub use classification::FailureClassifier;
pub use baseline_collector::{
    BaselineCollectionConfig, BaselineCollector, BaselineComparison, StoredBaseline,
};
#[cfg(feature = "prometheus")]
pub use carbon_metrics::{
    CarbonAwarenessCollector, CarbonAwarenessConfig, CarbonFootprintReport,
    EnergyConsumptionMetrics, WorkloadCarbonSummary, WorkloadType,
};
pub use config::{ConfigManager, ReportJobConfig, ReportingSystemConfig, SystemConfig};
pub use distribution::{DistributionChannel, DistributionManager, SlackWebhookConfig, SmtpConfig};
pub use enterprise_metrics::{
    BaselineData, EnterpriseMetricsCollector, MeasurementSnapshot, RegressionAnalysis,
};
pub use memory_monitor::MemoryMonitor;
pub use performance_metrics_collector::PerformanceMetricsCollector;
pub use reporting::{
    GeneratedReport, ReportConfiguration, ReportType, ReportingEngine, StakeholderRole,
};
pub use scheduler::{ReportScheduler, ScheduleConfig, ScheduleType, ScheduledJob};
