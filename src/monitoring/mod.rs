//! Test infrastructure monitoring system (UV-219)
//!
//! Provides real-time test execution monitoring, failure categorization,
//! and performance tracking for the Uveddi static code analysis platform.

pub mod dashboard;
pub mod metrics;
pub mod websocket;
pub mod database;
pub mod classification;

pub use dashboard::MonitoringDashboard;
pub use metrics::{TestMetrics, TestExecution, TestResult};
pub use websocket::WebSocketManager;
pub use database::MonitoringDatabase;
pub use classification::FailureClassifier;
