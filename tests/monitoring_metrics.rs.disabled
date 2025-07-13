//! Unit tests for TestMetrics and related types (UV-219)

use uveddi::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
use std::time::SystemTime;

#[test]
fn test_test_metrics_struct_creation() {
    let metrics = TestMetrics {
        execution_id: "exec-123".to_string(),
        test_name: "test_name".to_string(),
        test_suite: "suite_a".to_string(),
        status: TestStatus::Passed,
        duration_ms: 100,
        resource_usage: ResourceUsage {
            cpu_percent: 5.0,
            memory_mb: 32,
            disk_io_mb: 2,
        },
        failure_category: None,
        timestamp: SystemTime::now(),
    };
    assert_eq!(metrics.status, TestStatus::Passed);
    assert_eq!(metrics.resource_usage.memory_mb, 32);
}

#[test]
fn test_resource_usage_struct() {
    let usage = ResourceUsage {
        cpu_percent: 12.5,
        memory_mb: 128,
        disk_io_mb: 10,
    };
    assert!(usage.cpu_percent > 0.0);
    assert_eq!(usage.memory_mb, 128);
}
