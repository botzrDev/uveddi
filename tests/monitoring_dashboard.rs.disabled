//! Unit tests for MonitoringDashboard (UV-219)

use uveddi::monitoring::dashboard::MonitoringDashboard;
use uveddi::config::monitoring::MonitoringConfig;
use uveddi::monitoring::database::MonitoringDatabase;
use uveddi::monitoring::websocket::WebSocketManager;

#[tokio::test]
async fn test_monitoring_dashboard_initialization() {
    let config = MonitoringConfig::default();
    let dashboard = MonitoringDashboard::new(config).await;
    assert!(dashboard.is_ok(), "MonitoringDashboard should initialize successfully");
}

#[tokio::test]
async fn test_monitoring_dashboard_start() {
    let config = MonitoringConfig::default();
    let dashboard = MonitoringDashboard::new(config).await.unwrap();
    let result = dashboard.start().await;
    assert!(result.is_ok(), "MonitoringDashboard should start successfully");
}

#[tokio::test]
async fn test_monitoring_database_initialization() {
    let config = MonitoringConfig::default();
    let db = MonitoringDatabase::new(&config.database_url).await;
    assert!(db.is_ok(), "MonitoringDatabase should initialize successfully");
}

#[tokio::test]
async fn test_websocket_manager_broadcast_event() {
    use std::time::SystemTime;
    use uveddi::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
    use uveddi::monitoring::websocket::{TestEvent, TestEventType, WebSocketManager};

    let ws_manager = WebSocketManager::new();
    let metrics = TestMetrics {
        execution_id: "test-exec-1".to_string(),
        test_name: "sample_test".to_string(),
        test_suite: "suite_a".to_string(),
        status: TestStatus::Passed,
        duration_ms: 123,
        resource_usage: ResourceUsage {
            cpu_percent: 10.0,
            memory_mb: 50,
            disk_io_mb: 1,
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
    // No assertion: just ensure no panic
}
