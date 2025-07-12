use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// TestMetrics captures metrics for a single test execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub execution_id: String,
    pub test_name: String,
    pub test_suite: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub resource_usage: ResourceUsage,
    pub failure_category: Option<String>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_io_mb: u64,
}

/// TestExecution and TestResult stubs for future extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult;
