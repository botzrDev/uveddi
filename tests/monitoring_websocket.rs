//! Unit tests for WebSocketManager and TestEvent (UV-219)

use uveddi::monitoring::websocket::{WebSocketManager, TestEvent, TestEventType};
use uveddi::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
use std::time::SystemTime;

#[tokio::test]
async fn test_websocket_manager_new_and_broadcast() {
    let ws_manager = WebSocketManager::new();
    let metrics = TestMetrics {
        execution_id: "exec-1".to_string(),
        test_name: "test".to_string(),
        test_suite: "suite".to_string(),
        status: TestStatus::Passed,
        duration_ms: 42,
        resource_usage: ResourceUsage {
            cpu_percent: 1.0,
            memory_mb: 10,
            disk_io_mb: 0,
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
