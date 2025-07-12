//! Integration test for MonitoringDashboard and WebSocketManager (UV-219)

use uveddi::monitoring::dashboard::MonitoringDashboard;
use uveddi::config::monitoring::MonitoringConfig;
use uveddi::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
use uveddi::monitoring::websocket::{TestEvent, TestEventType, WebSocketManager};
use std::time::SystemTime;

#[tokio::test]
async fn test_monitoring_dashboard_and_websocket_integration() {
    let config = MonitoringConfig::default();
    let dashboard = MonitoringDashboard::new(config).await.unwrap();
    dashboard.start().await.unwrap();

    let ws_manager = WebSocketManager::new();
    let metrics = TestMetrics {
        execution_id: "integration-exec-1".to_string(),
        test_name: "integration_test".to_string(),
        test_suite: "integration_suite".to_string(),
        status: TestStatus::Passed,
        duration_ms: 200,
        resource_usage: ResourceUsage {
            cpu_percent: 15.0,
            memory_mb: 64,
            disk_io_mb: 3,
        },
        failure_category: None,
        timestamp: SystemTime::now(),
    };
    let event = TestEvent {
        event_type: TestEventType::TestCompleted,
        test_metrics: metrics,
        timestamp: SystemTime::now(),
    };
    ws_manager.broadcast_test_event(event).await;
    // No assertion: just ensure integration does not panic
}
