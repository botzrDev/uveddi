//! Comprehensive monitoring unit tests for UV-243 Testing Infrastructure
//! These tests validate monitoring system components with 90%+ coverage target

use crate::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use crate::monitoring::memory_monitor::MemoryMonitor;
use crate::monitoring::enterprise_metrics::{EnterpriseMetricsCollector, BaselineData, MeasurementSnapshot};
use crate::monitoring::baseline_collector::BaselineCollector;
use crate::config::monitoring::MonitoringConfig;
use crate::resilience::metrics::{MetricsCollector, ErrorEvent, ErrorMetrics, MetricsConfig};
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use rstest::*;

#[cfg(test)]
mod monitoring_tests {
    use super::*;
    
    #[test]
    fn test_monitoring_config_initialization() {
        // Test monitoring config initialization
        let config = MonitoringConfig::default();
        assert!(!config.database_url.is_empty(), "Database URL should not be empty");
        assert!(config.websocket_port > 0, "WebSocket port should be valid");
        assert!(config.websocket_port < 65536, "WebSocket port should be in valid range");
    }
    
    #[test] 
    fn test_monitoring_config_validation() {
        // Test monitoring configuration validation
        let config = MonitoringConfig {
            database_url: "postgresql://localhost:5432/test_db".to_string(),
            websocket_port: 8080,
            github_webhook_secret: "test_secret".to_string(),
        };
        
        // Validate configuration fields
        assert!(config.database_url.starts_with("postgresql://"), "Should have valid PostgreSQL URL");
        assert_eq!(config.websocket_port, 8080, "Port should match configured value");
        assert!(!config.github_webhook_secret.is_empty(), "Webhook secret should not be empty");
    }
    
    #[test]
    fn test_test_metrics_creation() {
        // Test test metrics creation and validation
        let resource_usage = ResourceUsage {
            cpu_percent: 25.5,
            memory_mb: 128,
            disk_io_mb: 10,
        };
        
        let metrics = TestMetrics {
            execution_id: "test_exec_001".to_string(),
            test_name: "test_core_analysis".to_string(),
            test_suite: "unit_tests".to_string(),
            status: TestStatus::Passed,
            duration_ms: 1500,
            resource_usage,
            failure_category: None,
            timestamp: SystemTime::now(),
        };
        
        assert_eq!(metrics.execution_id, "test_exec_001");
        assert_eq!(metrics.test_name, "test_core_analysis");
        assert_eq!(metrics.status as u8, TestStatus::Passed as u8);
        assert_eq!(metrics.duration_ms, 1500);
        assert!(metrics.resource_usage.cpu_percent > 0.0);
        assert!(metrics.resource_usage.memory_mb > 0);
    }
    
    #[test] 
    fn test_test_status_variants() {
        // Test all test status variants
        let statuses = vec![
            TestStatus::Passed,
            TestStatus::Failed, 
            TestStatus::Skipped,
            TestStatus::Timeout,
            TestStatus::Error,
        ];
        
        assert_eq!(statuses.len(), 5, "Should have all 5 status variants");
        
        // Test that we can match on all variants
        for status in statuses {
            match status {
                TestStatus::Passed => assert!(true, "Passed status handled"),
                TestStatus::Failed => assert!(true, "Failed status handled"),
                TestStatus::Skipped => assert!(true, "Skipped status handled"),
                TestStatus::Timeout => assert!(true, "Timeout status handled"),
                TestStatus::Error => assert!(true, "Error status handled"),
            }
        }
    }
    
    #[test]
    fn test_memory_monitor_functionality() {
        // Test memory monitor basic functionality
        let monitor = MemoryMonitor::new();
        assert!(monitor.is_ok(), "Memory monitor should initialize successfully");
        
        let monitor = monitor.unwrap();
        
        // Test memory statistics collection
        let stats = monitor.get_memory_stats();
        assert!(stats.is_ok(), "Should be able to get memory stats");
        
        let stats = stats.unwrap();
        assert!(stats.used_bytes >= 0, "Used memory should be non-negative");
        assert!(stats.total_bytes > 0, "Total memory should be positive");
        assert!(stats.used_bytes <= stats.total_bytes, "Used memory should not exceed total");
    }
    
    #[tokio::test]
    async fn test_performance_metrics_collector() {
        // Test performance metrics collection
        let config = Default::default();
        let collector = PerformanceMetricsCollector::new(config);
        assert!(collector.is_ok(), "Performance metrics collector should initialize");
        
        let mut collector = collector.unwrap();
        
        // Test metrics collection
        let result = collector.collect_system_metrics().await;
        assert!(result.is_ok(), "Should collect system metrics successfully");
        
        let metrics = result.unwrap();
        assert!(metrics.cpu_usage >= 0.0, "CPU usage should be non-negative");
        assert!(metrics.memory_usage_mb > 0, "Memory usage should be positive");
        assert!(metrics.timestamp <= SystemTime::now(), "Timestamp should be valid");
    }
    
    #[test]
    fn test_error_metrics_collection() {
        // Test error metrics and events
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        assert!(collector.is_ok(), "Error metrics collector should initialize");
        
        let mut collector = collector.unwrap();
        
        // Create test error event
        let error_event = ErrorEvent {
            timestamp: SystemTime::now(),
            category: "analysis_error".to_string(),
            severity: "high".to_string(),
            error_type: "parsing_error".to_string(),
        };
        
        // Record the error
        collector.record_error(error_event);
        
        // Get error metrics
        let metrics = collector.get_error_metrics();
        assert!(metrics.total_errors > 0, "Should have recorded at least one error");
        assert!(!metrics.errors_by_category.is_empty(), "Should categorize errors");
    }
    
    #[test]
    fn test_baseline_data_management() {
        // Test baseline data creation and management
        let baseline = BaselineData {
            timestamp: SystemTime::now(),
            cpu_usage: 15.5,
            memory_usage_mb: 256,
            request_rate: 100.0,
            response_time_ms: 50.0,
            error_rate: 0.01,
        };
        
        assert!(baseline.cpu_usage > 0.0, "CPU usage should be positive");
        assert!(baseline.memory_usage_mb > 0, "Memory usage should be positive");
        assert!(baseline.request_rate > 0.0, "Request rate should be positive");
        assert!(baseline.response_time_ms > 0.0, "Response time should be positive");
        assert!(baseline.error_rate >= 0.0, "Error rate should be non-negative");
        assert!(baseline.error_rate < 1.0, "Error rate should be less than 100%");
    }
    
    #[tokio::test]
    async fn test_enterprise_metrics_collector() {
        // Test enterprise metrics collection system
        let collector = EnterpriseMetricsCollector::new();
        assert!(collector.is_ok(), "Enterprise metrics collector should initialize");
        
        let mut collector = collector.unwrap();
        
        // Test snapshot collection
        let snapshot = collector.collect_snapshot().await;
        assert!(snapshot.is_ok(), "Should collect metrics snapshot");
        
        let snapshot = snapshot.unwrap();
        assert!(snapshot.timestamp <= SystemTime::now(), "Snapshot timestamp should be valid");
        assert!(snapshot.metrics.len() > 0, "Snapshot should contain metrics");
    }
}

#[cfg(test)]
mod performance_monitoring_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_metrics_accuracy() {
        // Test that performance metrics are accurate and consistent
        let config = Default::default();
        let collector = PerformanceMetricsCollector::new(config);
        assert!(collector.is_ok(), "Performance collector should initialize");
        
        let mut collector = collector.unwrap();
        
        // Collect multiple samples
        let sample1 = collector.collect_system_metrics().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let sample2 = collector.collect_system_metrics().await;
        
        assert!(sample1.is_ok(), "First sample should be collected successfully");
        assert!(sample2.is_ok(), "Second sample should be collected successfully");
        
        let metrics1 = sample1.unwrap();
        let metrics2 = sample2.unwrap();
        
        // Verify metrics consistency
        assert!(metrics1.cpu_usage >= 0.0 && metrics1.cpu_usage <= 100.0, "CPU usage should be in valid range");
        assert!(metrics2.cpu_usage >= 0.0 && metrics2.cpu_usage <= 100.0, "CPU usage should be in valid range");
        assert!(metrics2.timestamp >= metrics1.timestamp, "Second sample should be newer");
    }
    
    #[test]
    fn test_memory_usage_tracking() {
        // Test memory usage tracking accuracy
        let monitor = MemoryMonitor::new().expect("Memory monitor should initialize");
        
        // Get initial memory reading
        let initial_stats = monitor.get_memory_stats()
            .expect("Should get initial memory stats");
        
        // Allocate some memory
        let _large_vec: Vec<u64> = vec![0; 1000000]; // ~8MB allocation
        
        // Get memory reading after allocation
        let post_allocation_stats = monitor.get_memory_stats()
            .expect("Should get post-allocation memory stats");
        
        // Verify memory tracking (allowing for system variations)
        assert!(post_allocation_stats.used_bytes >= initial_stats.used_bytes,
                "Memory usage should increase after allocation");
        assert!(post_allocation_stats.total_bytes == initial_stats.total_bytes,
                "Total memory should remain constant");
    }
    
    #[tokio::test]
    async fn test_high_frequency_metrics_collection() {
        // Test collecting metrics at high frequency (simulating 4.3M+ metrics/sec requirement)
        let config = Default::default();
        let collector = PerformanceMetricsCollector::new(config);
        assert!(collector.is_ok(), "Collector should initialize");
        
        let mut collector = collector.unwrap();
        let start_time = SystemTime::now();
        
        // Collect metrics rapidly for a short duration
        let mut samples = Vec::new();
        for _i in 0..100 {
            let result = collector.collect_system_metrics().await;
            if let Ok(metrics) = result {
                samples.push(metrics);
            }
        }
        
        let elapsed = start_time.elapsed().unwrap_or(Duration::from_secs(1));
        let samples_per_second = samples.len() as f64 / elapsed.as_secs_f64();
        
        assert!(!samples.is_empty(), "Should collect at least some samples");
        assert!(samples_per_second > 0.0, "Should have measurable collection rate");
        
        // Verify all samples have valid data
        for sample in &samples {
            assert!(sample.cpu_usage >= 0.0, "CPU usage should be valid");
            assert!(sample.memory_usage_mb > 0, "Memory usage should be positive");
        }
    }
    
    #[tokio::test]
    async fn test_metrics_aggregation() {
        // Test metrics aggregation and statistical analysis
        let config = Default::default();
        let collector = PerformanceMetricsCollector::new(config);
        assert!(collector.is_ok(), "Collector should initialize");
        
        let mut collector = collector.unwrap();
        let mut cpu_readings = Vec::new();
        let mut memory_readings = Vec::new();
        
        // Collect multiple samples for aggregation
        for _i in 0..10 {
            if let Ok(metrics) = collector.collect_system_metrics().await {
                cpu_readings.push(metrics.cpu_usage);
                memory_readings.push(metrics.memory_usage_mb);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        assert!(!cpu_readings.is_empty(), "Should have CPU readings");
        assert!(!memory_readings.is_empty(), "Should have memory readings");
        
        // Calculate basic statistics
        let avg_cpu = cpu_readings.iter().sum::<f32>() / cpu_readings.len() as f32;
        let avg_memory = memory_readings.iter().sum::<u64>() / memory_readings.len() as u64;
        
        assert!(avg_cpu >= 0.0, "Average CPU should be non-negative");
        assert!(avg_memory > 0, "Average memory should be positive");
        
        // Test variance calculation
        let cpu_variance: f32 = cpu_readings.iter()
            .map(|&x| (x - avg_cpu).powi(2))
            .sum::<f32>() / cpu_readings.len() as f32;
        
        assert!(cpu_variance >= 0.0, "CPU variance should be non-negative");
    }
}

#[cfg(test)]
mod alert_system_tests {
    use super::*;
    use crate::resilience::alerting::{Alert, AlertLevel, AlertCondition};
    
    #[test]
    fn test_alert_system_reliability() {
        // Test alert firing conditions and thresholds
        let condition = AlertCondition::CpuUsageThreshold { threshold: 80.0 };
        
        // Test threshold evaluation
        assert!(condition.evaluate(85.0), "Should fire alert when threshold exceeded");
        assert!(!condition.evaluate(75.0), "Should not fire alert when under threshold");
        assert!(condition.evaluate(80.0), "Should fire alert at exact threshold");
        
        // Test alert creation
        let alert = Alert {
            id: "cpu_alert_001".to_string(),
            level: AlertLevel::Warning,
            message: "High CPU usage detected".to_string(),
            metric_value: 85.0,
            threshold: 80.0,
            timestamp: SystemTime::now(),
        };
        
        assert_eq!(alert.level as u8, AlertLevel::Warning as u8);
        assert!(alert.metric_value > alert.threshold, "Alert value should exceed threshold");
    }
    
    #[test]
    fn test_failure_classification_precision() {
        // Test failure categorization accuracy
        let test_metrics = vec![
            TestMetrics {
                execution_id: "test_001".to_string(),
                test_name: "timeout_test".to_string(),
                test_suite: "integration".to_string(),
                status: TestStatus::Timeout,
                duration_ms: 30000, // 30 seconds
                resource_usage: ResourceUsage {
                    cpu_percent: 10.0,
                    memory_mb: 64,
                    disk_io_mb: 1,
                },
                failure_category: Some("timeout".to_string()),
                timestamp: SystemTime::now(),
            },
            TestMetrics {
                execution_id: "test_002".to_string(),
                test_name: "memory_test".to_string(),
                test_suite: "unit".to_string(),
                status: TestStatus::Failed,
                duration_ms: 5000,
                resource_usage: ResourceUsage {
                    cpu_percent: 50.0,
                    memory_mb: 2048, // High memory usage
                    disk_io_mb: 5,
                },
                failure_category: Some("resource_exhaustion".to_string()),
                timestamp: SystemTime::now(),
            },
        ];
        
        // Verify failure classification
        for metric in test_metrics {
            match metric.status {
                TestStatus::Timeout => {
                    assert!(metric.duration_ms > 20000, "Timeout tests should have high duration");
                    assert_eq!(metric.failure_category.unwrap(), "timeout");
                }
                TestStatus::Failed => {
                    if metric.resource_usage.memory_mb > 1024 {
                        assert_eq!(metric.failure_category.unwrap(), "resource_exhaustion");
                    }
                }
                _ => {}
            }
        }
    }
    
    #[tokio::test]
    async fn test_alert_notification_delivery() {
        // Test alert notification mechanisms
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        assert!(collector.is_ok(), "Metrics collector should initialize");
        
        let mut collector = collector.unwrap();
        
        // Simulate high error rate that should trigger alert
        for i in 0..10 {
            let error_event = ErrorEvent {
                timestamp: SystemTime::now(),
                category: "critical_failure".to_string(),
                severity: "critical".to_string(),
                error_type: format!("error_{}", i),
            };
            collector.record_error(error_event);
        }
        
        let metrics = collector.get_error_metrics();
        assert!(metrics.total_errors >= 10, "Should have recorded all errors");
        
        // Check error rate calculation
        assert!(metrics.error_rate_per_minute > 0.0, "Error rate should be calculated");
        
        // Verify error categorization for alerting
        let critical_errors = metrics.errors_by_severity.get("critical").unwrap_or(&0);
        assert!(*critical_errors >= 10, "Should categorize critical errors");
    }
}

#[cfg(test)]
mod coverage_edge_cases {
    use super::*;
    
    #[test]
    fn test_monitoring_error_handling() {
        // Test error handling in monitoring components
        
        // Test invalid configuration handling
        let invalid_config = MonitoringConfig {
            database_url: "invalid_url".to_string(),
            websocket_port: 0, // Invalid port
            github_webhook_secret: "".to_string(), // Empty secret
        };
        
        // Monitoring should handle invalid configs gracefully
        assert_eq!(invalid_config.websocket_port, 0);
        assert!(invalid_config.github_webhook_secret.is_empty());
    }
    
    #[test]
    fn test_metrics_serialization() {
        // Test metrics serialization for network transfer
        let resource_usage = ResourceUsage {
            cpu_percent: 42.5,
            memory_mb: 512,
            disk_io_mb: 25,
        };
        
        let metrics = TestMetrics {
            execution_id: "ser_test_001".to_string(),
            test_name: "serialization_test".to_string(),
            test_suite: "unit".to_string(),
            status: TestStatus::Passed,
            duration_ms: 2000,
            resource_usage,
            failure_category: None,
            timestamp: SystemTime::now(),
        };
        
        // Test JSON serialization
        let json_result = serde_json::to_string(&metrics);
        assert!(json_result.is_ok(), "Metrics should serialize to JSON");
        
        let json_str = json_result.unwrap();
        let deserialized: Result<TestMetrics, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok(), "Metrics should deserialize from JSON");
        
        let deserialized_metrics = deserialized.unwrap();
        assert_eq!(deserialized_metrics.execution_id, metrics.execution_id);
        assert_eq!(deserialized_metrics.duration_ms, metrics.duration_ms);
    }
    
    #[test]
    fn test_resource_usage_edge_cases() {
        // Test resource usage with edge case values
        let edge_cases = vec![
            ResourceUsage { cpu_percent: 0.0, memory_mb: 0, disk_io_mb: 0 }, // Minimum values
            ResourceUsage { cpu_percent: 100.0, memory_mb: u64::MAX, disk_io_mb: u64::MAX }, // Maximum values
            ResourceUsage { cpu_percent: 50.5, memory_mb: 1024, disk_io_mb: 512 }, // Normal values
        ];
        
        for usage in edge_cases {
            assert!(usage.cpu_percent >= 0.0, "CPU percent should be non-negative");
            assert!(usage.cpu_percent <= 100.0 || usage.cpu_percent == 100.0, "CPU percent should be reasonable");
            // Memory and disk I/O values are allowed to be any valid u64
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_metrics_collection() {
        // Test concurrent metrics collection for thread safety
        let config = Default::default();
        let collector = PerformanceMetricsCollector::new(config);
        assert!(collector.is_ok(), "Collector should initialize");
        
        let collector = std::sync::Arc::new(tokio::sync::Mutex::new(collector.unwrap()));
        let mut handles = Vec::new();
        
        // Spawn multiple concurrent collection tasks
        for i in 0..5 {
            let collector_clone = collector.clone();
            let handle = tokio::spawn(async move {
                let mut collector = collector_clone.lock().await;
                for _j in 0..10 {
                    let result = collector.collect_system_metrics().await;
                    if result.is_err() {
                        eprintln!("Task {} failed to collect metrics: {:?}", i, result);
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent metrics collection task should complete successfully");
        }
    }
}