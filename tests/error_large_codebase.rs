use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use uveddi::error::large_codebase::{
    ErrorAggregator, ErrorContext, LargeCodebaseError, LargeCodebaseErrorHandler,
    NotificationSystem, ProgressState, ProgressTracker, RecoveryStrategies,
};
use uveddi::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use uveddi::database::models::PerformanceMetricsConfig;
use uveddi::resilience::circuit_breaker::CircuitBreaker;
use uveddi::resilience::retry::{RetryClient, RetryConfig};

fn create_test_context(files_processed: usize, total_files: usize) -> ErrorContext {
    ErrorContext {
        timestamp: Instant::now(),
        file_count_processed: files_processed,
        total_file_count: total_files,
        memory_usage_mb: 256.0,
        processing_duration: Duration::from_secs(10),
    }
}

#[tokio::test]
async fn test_error_aggregator_basic_functionality() {
    let aggregator = ErrorAggregator::new(100, 0.1);
    let context = create_test_context(10, 100);

    let error = LargeCodebaseError::FileAccessError {
        path: PathBuf::from("/test/file.rs"),
        cause: "File not found".to_string(),
        retry_count: 1,
    };

    let should_abort = aggregator.add_error(error.clone(), context.clone()).await;
    assert!(!should_abort, "Should not abort with single error below threshold");

    let errors = aggregator.get_error_summary().await;
    assert_eq!(errors.len(), 1, "Should have one error recorded");
}

#[tokio::test]
async fn test_error_aggregator_threshold_exceeded() {
    let aggregator = ErrorAggregator::new(100, 0.05); // 5% threshold
    
    for i in 0..10 {
        let context = create_test_context(i, 100);
        let error = LargeCodebaseError::FileAccessError {
            path: PathBuf::from(format!("/test/file_{}.rs", i)),
            cause: "Access denied".to_string(),
            retry_count: 2,
        };

        let should_abort = aggregator.add_error(error, context).await;
        
        if i >= 5 { // 6/100 = 6% > 5% threshold
            assert!(should_abort, "Should abort when threshold exceeded at iteration {}", i);
            break;
        } else {
            assert!(!should_abort, "Should not abort below threshold at iteration {}", i);
        }
    }
}

#[tokio::test]
async fn test_error_aggregator_max_errors_limit() {
    let aggregator = ErrorAggregator::new(5, 1.0); // High threshold, low max errors
    
    for i in 0..10 {
        let context = create_test_context(i, 1000); // Low error rate
        let error = LargeCodebaseError::MemoryPressure {
            current: 1024 * 1024 * 512, // 512MB
            limit: 1024 * 1024 * 256,   // 256MB
            affected_files: vec![PathBuf::from(format!("/test/file_{}.rs", i))],
        };

        aggregator.add_error(error, context).await;
    }

    let errors = aggregator.get_error_summary().await;
    assert_eq!(errors.len(), 5, "Should maintain max errors limit");
}

#[tokio::test]
async fn test_recovery_strategies_initialization() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let strategies = RecoveryStrategies::new(retry_client, circuit_breaker);

    assert!(strategies.get_strategy("file_access").is_some());
    assert!(strategies.get_strategy("memory_pressure").is_some());
    assert!(strategies.get_strategy("parsing_timeout").is_some());
    assert!(strategies.get_strategy("dependency_cycle").is_some());
    assert!(strategies.get_strategy("batch_failure").is_some());
    assert!(strategies.get_strategy("resource_exhaustion").is_some());
    assert!(strategies.get_strategy("unknown_error").is_none());
}

#[tokio::test]
async fn test_recovery_strategy_application() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let strategies = RecoveryStrategies::new(retry_client, circuit_breaker);

    let error = LargeCodebaseError::ParsingTimeout {
        file: PathBuf::from("/test/large_file.rs"),
        duration: Duration::from_secs(30),
        partial_results: Some("partial AST".to_string()),
    };

    if let Some(strategy) = strategies.get_strategy("parsing_timeout") {
        let result = strategies.apply_strategy(strategy, &error).await;
        assert!(result.is_ok(), "Strategy application should succeed");
        assert!(result.unwrap(), "Strategy should indicate successful recovery");
    } else {
        panic!("parsing_timeout strategy should exist");
    }
}

#[tokio::test]
async fn test_progress_tracker_basic_operations() {
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));
    let tracker = ProgressTracker::new(1000, Duration::from_millis(100), metrics_collector);

    tracker.update_progress(100, 5).await;
    let progress = tracker.get_progress().await;

    assert_eq!(progress.files_processed, 100);
    assert_eq!(progress.files_total, 1000);
    assert_eq!(progress.files_failed, 5);
}

#[tokio::test]
async fn test_progress_tracker_batch_operations() {
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));
    let tracker = ProgressTracker::new(500, Duration::from_millis(50), metrics_collector);

    tracker.set_current_batch(Some("batch_001".to_string())).await;
    let progress = tracker.get_progress().await;
    assert_eq!(progress.current_batch, Some("batch_001".to_string()));

    tracker.set_current_batch(None).await;
    let progress = tracker.get_progress().await;
    assert_eq!(progress.current_batch, None);
}

#[tokio::test]
async fn test_notification_system() {
    let (notification_system, mut error_receiver, mut progress_receiver) = NotificationSystem::new();

    let error = LargeCodebaseError::DependencyResolution {
        cycle: vec![
            PathBuf::from("/src/a.rs"),
            PathBuf::from("/src/b.rs"),
            PathBuf::from("/src/a.rs"),
        ],
        depth: 3,
    };
    let context = create_test_context(50, 100);

    notification_system.notify_error(error.clone(), context.clone());

    let received = timeout(Duration::from_millis(100), error_receiver.recv()).await;
    assert!(received.is_ok(), "Should receive error notification");
    
    if let Ok(Some((_received_error, received_context))) = received {
        assert_eq!(received_context.file_count_processed, 50);
        assert_eq!(received_context.total_file_count, 100);
    }

    let progress = ProgressState {
        files_processed: 75,
        files_total: 100,
        files_failed: 2,
        current_batch: Some("test_batch".to_string()),
        estimated_completion: None,
        last_checkpoint: Instant::now(),
    };

    notification_system.notify_progress(progress.clone());

    let received = timeout(Duration::from_millis(100), progress_receiver.recv()).await;
    assert!(received.is_ok(), "Should receive progress notification");
    
    if let Ok(Some(received_progress)) = received {
        assert_eq!(received_progress.files_processed, 75);
        assert_eq!(received_progress.files_total, 100);
        assert_eq!(received_progress.files_failed, 2);
    }
}

#[tokio::test]
async fn test_large_codebase_error_handler_creation() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let (handler, _error_receiver, _progress_receiver) = LargeCodebaseErrorHandler::new(
        10000,                          // total_files
        1024 * 1024 * 512,             // memory_limit (512MB)
        Duration::from_secs(30),        // timeout_duration
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    assert_eq!(handler.get_memory_limit(), 1024 * 1024 * 512);
    assert_eq!(handler.get_timeout_duration(), Duration::from_secs(30));
}

#[tokio::test]
async fn test_large_codebase_error_handler_error_handling() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let (handler, mut error_receiver, _progress_receiver) = LargeCodebaseErrorHandler::new(
        1000,
        1024 * 1024 * 256,
        Duration::from_secs(15),
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    let error = LargeCodebaseError::BatchProcessingFailure {
        batch_id: "batch_123".to_string(),
        failed_files: vec![PathBuf::from("/test/failed.rs")],
        success_count: 95,
        failure_count: 5,
    };
    let context = create_test_context(100, 1000);

    let result = handler.handle_error(error.clone(), context.clone()).await;
    assert!(result.is_ok(), "Error handling should succeed");

    // Verify notification was sent
    let received = timeout(Duration::from_millis(100), error_receiver.recv()).await;
    assert!(received.is_ok(), "Should receive error notification");
}

#[tokio::test]
async fn test_large_codebase_error_handler_progress_updates() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let (handler, _error_receiver, mut progress_receiver) = LargeCodebaseErrorHandler::new(
        500,
        1024 * 1024 * 128,
        Duration::from_secs(10),
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    handler.update_progress(250, 10).await;

    // Verify progress notification was sent
    let received = timeout(Duration::from_millis(100), progress_receiver.recv()).await;
    assert!(received.is_ok(), "Should receive progress notification");
    
    if let Ok(Some(progress)) = received {
        assert_eq!(progress.files_processed, 250);
        assert_eq!(progress.files_failed, 10);
        assert_eq!(progress.files_total, 500);
    }
}

#[tokio::test]
async fn test_memory_pressure_detection() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let memory_limit = 1024 * 1024 * 256; // 256MB
    let (handler, _error_receiver, _progress_receiver) = LargeCodebaseErrorHandler::new(
        1000,
        memory_limit,
        Duration::from_secs(30),
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    // Test below limit
    let pressure = handler.check_memory_pressure(memory_limit / 2).await;
    assert!(pressure.is_none(), "Should not detect pressure below limit");

    // Test above limit
    let pressure = handler.check_memory_pressure(memory_limit * 2).await;
    assert!(pressure.is_some(), "Should detect pressure above limit");

    if let Some(LargeCodebaseError::MemoryPressure { current, limit, .. }) = pressure {
        assert_eq!(current, memory_limit * 2);
        assert_eq!(limit, memory_limit);
    } else {
        panic!("Expected MemoryPressure error");
    }
}

#[tokio::test]
async fn test_error_conversion_to_uveddi_error() {
    use uveddi::error::UveddiError;

    let errors = vec![
        LargeCodebaseError::FileAccessError {
            path: PathBuf::from("/test.rs"),
            cause: "Not found".to_string(),
            retry_count: 3,
        },
        LargeCodebaseError::MemoryPressure {
            current: 1024 * 1024 * 512,
            limit: 1024 * 1024 * 256,
            affected_files: vec![PathBuf::from("/large.rs")],
        },
        LargeCodebaseError::ParsingTimeout {
            file: PathBuf::from("/complex.rs"),
            duration: Duration::from_secs(45),
            partial_results: None,
        },
        LargeCodebaseError::DependencyResolution {
            cycle: vec![PathBuf::from("/a.rs"), PathBuf::from("/b.rs")],
            depth: 2,
        },
        LargeCodebaseError::BatchProcessingFailure {
            batch_id: "test_batch".to_string(),
            failed_files: vec![PathBuf::from("/failed.rs")],
            success_count: 90,
            failure_count: 10,
        },
        LargeCodebaseError::ResourceExhaustion {
            resource_type: "CPU".to_string(),
            current_usage: 0.95,
            threshold: 0.8,
        },
    ];

    for error in errors {
        let uveddi_error: UveddiError = error.into();
        // Just verify conversion succeeds without panicking
        let error_string = format!("{:?}", uveddi_error);
        assert!(!error_string.is_empty(), "Error should convert to non-empty string");
    }
}

#[tokio::test]
async fn test_checkpoint_operations() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let (handler, _error_receiver, _progress_receiver) = LargeCodebaseErrorHandler::new(
        2000,
        1024 * 1024 * 512,
        Duration::from_secs(20),
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    handler.update_progress(500, 25).await;
    
    let result = handler.save_progress_checkpoint().await;
    assert!(result.is_ok(), "Checkpoint save should succeed");

    handler.set_current_batch(Some("checkpoint_batch".to_string())).await;
    // No direct way to verify batch was set, but operation should complete without error
}

#[tokio::test]
async fn test_error_summary_retrieval() {
    let retry_client = Arc::new(RetryClient::new(RetryConfig::default()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(10)));
    let metrics_collector = Arc::new(PerformanceMetricsCollector::new(PerformanceMetricsConfig::default(), 1000));

    let (handler, _error_receiver, _progress_receiver) = LargeCodebaseErrorHandler::new(
        100,
        1024 * 1024 * 64,
        Duration::from_secs(5),
        retry_client,
        circuit_breaker,
        metrics_collector,
    );

    // Add some errors
    for i in 0..3 {
        let error = LargeCodebaseError::FileAccessError {
            path: PathBuf::from(format!("/test_{}.rs", i)),
            cause: "Permission denied".to_string(),
            retry_count: 1,
        };
        let context = create_test_context(i * 10, 100);
        
        handler.handle_error(error, context).await.unwrap();
    }

    let summary = handler.get_error_summary().await;
    assert_eq!(summary.len(), 3, "Should have recorded all errors");
}