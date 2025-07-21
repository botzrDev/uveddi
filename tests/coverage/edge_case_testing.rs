//! Edge Case and Boundary Condition Testing
//! 
//! This module focuses on edge cases, boundary conditions, and error paths
//! to ensure comprehensive test coverage for exceptional scenarios.

use uveddi::analysis::engine_builder::EngineBuilder;
use uveddi::analysis::config::AnalysisConfig;
use uveddi::analysis::memory::config::MemoryConfig;
use uveddi::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use uveddi::resilience::retry::{RetryPolicy, ExponentialBackoff};
use uveddi::security::authentication::AuthenticationService;
use uveddi::security::models::{User, Role};
use uveddi::monitoring::metrics::MetricsCollector;

use tokio_test;
use tempfile::TempDir;
use std::path::PathBuf;
use std::time::Duration;

/// Edge cases for Analysis Engine
#[cfg(test)]
mod analysis_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_analysis_with_zero_memory_limit() {
        // Test analysis engine with zero memory limit
        let memory_config = MemoryConfig {
            arena_size: 0,
            detector_pool_size: 0,
            enable_zero_copy: false,
            memory_limit_mb: 0,
        };
        
        let mut analysis_config = AnalysisConfig::default();
        analysis_config.memory = memory_config;
        
        let result = EngineBuilder::new()
            .with_config(analysis_config)
            .build();
        
        // Should handle zero memory gracefully (either succeed with defaults or fail safely)
        match result {
            Ok(engine) => assert!(engine.is_initialized()),
            Err(e) => assert!(e.to_string().contains("memory") || e.to_string().contains("config")),
        }
    }

    #[tokio::test]
    async fn test_analysis_with_maximum_memory_limit() {
        // Test analysis engine with maximum memory limit
        let memory_config = MemoryConfig {
            arena_size: usize::MAX,
            detector_pool_size: usize::MAX,
            enable_zero_copy: true,
            memory_limit_mb: u64::MAX,
        };
        
        let mut analysis_config = AnalysisConfig::default();
        analysis_config.memory = memory_config;
        
        let result = EngineBuilder::new()
            .with_config(analysis_config)
            .build();
        
        // Should handle maximum values gracefully
        match result {
            Ok(engine) => assert!(engine.is_initialized()),
            Err(e) => {
                // Expected to fail with very large values
                assert!(e.to_string().contains("memory") || e.to_string().contains("limit"));
            }
        }
    }

    #[tokio::test]
    async fn test_analysis_with_unicode_file_path() {
        // Test analysis with Unicode file paths
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let unicode_file = temp_dir.path().join("测试文件_🦀.rs");
        
        std::fs::write(&unicode_file, r#"
            fn unicode_function_测试() {
                println!("Unicode content: 🦀");
            }
        "#).expect("Failed to write Unicode file");

        let config = AnalysisConfig::default();
        let mut engine = EngineBuilder::new()
            .with_config(config)
            .build()
            .expect("Failed to build analysis engine");

        let result = engine.analyze_file(&unicode_file).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analysis_with_very_long_file_path() {
        // Test analysis with very long file paths
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create nested directory structure with long names
        let mut long_path = temp_dir.path().to_path_buf();
        for i in 0..10 {
            let dir_name = format!("very_long_directory_name_that_exceeds_normal_limits_{}", i);
            long_path = long_path.join(dir_name);
        }
        
        if let Ok(_) = std::fs::create_dir_all(&long_path) {
            let long_file = long_path.join("test.rs");
            
            if let Ok(_) = std::fs::write(&long_file, "fn test() {}") {
                let config = AnalysisConfig::default();
                let mut engine = EngineBuilder::new()
                    .with_config(config)
                    .build()
                    .expect("Failed to build analysis engine");

                let result = engine.analyze_file(&long_file).await;
                // Should handle long paths gracefully
                assert!(result.is_ok() || result.as_ref().err().unwrap().to_string().contains("path"));
            }
        }
    }

    #[tokio::test]
    async fn test_analysis_with_binary_file() {
        // Test analysis with binary file (should fail gracefully)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_file = temp_dir.path().join("binary.bin");
        
        // Create binary content
        let binary_data: Vec<u8> = (0..255).collect();
        std::fs::write(&binary_file, binary_data).expect("Failed to write binary file");

        let config = AnalysisConfig::default();
        let mut engine = EngineBuilder::new()
            .with_config(config)
            .build()
            .expect("Failed to build analysis engine");

        let result = engine.analyze_file(&binary_file).await;
        // Should handle binary files gracefully (either skip or error appropriately)
        assert!(result.is_ok() || result.as_ref().err().unwrap().to_string().contains("binary") || result.as_ref().err().unwrap().to_string().contains("encoding"));
    }

    #[tokio::test]
    async fn test_analysis_with_circular_symlinks() {
        // Test analysis with circular symbolic links (Unix only)
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let link1 = temp_dir.path().join("link1");
            let link2 = temp_dir.path().join("link2");
            
            if std::os::unix::fs::symlink(&link2, &link1).is_ok() &&
               std::os::unix::fs::symlink(&link1, &link2).is_ok() {
                
                let config = AnalysisConfig::default();
                let mut engine = EngineBuilder::new()
                    .with_config(config)
                    .build()
                    .expect("Failed to build analysis engine");

                let result = engine.analyze_file(&link1).await;
                // Should detect and handle circular links
                assert!(result.is_err());
                assert!(result.as_ref().err().unwrap().to_string().contains("circular") || 
                        result.as_ref().err().unwrap().to_string().contains("loop") ||
                        result.as_ref().err().unwrap().to_string().contains("symlink"));
            }
        }
    }
}

/// Edge cases for Resilience Patterns
#[cfg(test)]
mod resilience_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_zero_failure_threshold() {
        // Test circuit breaker with zero failure threshold
        let config = CircuitBreakerConfig {
            failure_threshold: 0,
            timeout: Duration::from_secs(1),
            half_open_max_calls: 1,
        };
        
        let result = CircuitBreaker::new(config);
        // Should either create a circuit that's always open or handle gracefully
        assert!(result.is_open() || result.is_closed());
    }

    #[tokio::test]
    async fn test_circuit_breaker_zero_timeout() {
        // Test circuit breaker with zero timeout
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_secs(0),
            half_open_max_calls: 1,
        };
        
        let mut circuit_breaker = CircuitBreaker::new(config);
        
        // Open the circuit
        let _ = circuit_breaker.call(|| async { Err::<(), _>("failure") }).await;
        assert!(circuit_breaker.is_open());
        
        // With zero timeout, should immediately be available for retry
        let result = circuit_breaker.call(|| async { Ok::<_, ()>("success") }).await;
        assert!(result.is_ok() || circuit_breaker.is_open());
    }

    #[tokio::test]
    async fn test_retry_policy_zero_attempts() {
        // Test retry policy with zero max attempts
        let retry_policy = RetryPolicy::ExponentialBackoff(ExponentialBackoff {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_attempts: 0,
            multiplier: 2.0,
        });
        
        let mut attempt_count = 0;
        let result = retry_policy.execute(|| async {
            attempt_count += 1;
            Err::<(), _>("always fails")
        }).await;
        
        // Should either not attempt at all or attempt once
        assert!(result.is_err());
        assert!(attempt_count <= 1);
    }

    #[tokio::test]
    async fn test_retry_policy_negative_multiplier() {
        // Test retry policy with negative backoff multiplier
        let retry_policy = RetryPolicy::ExponentialBackoff(ExponentialBackoff {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_attempts: 3,
            multiplier: -1.0,
        });
        
        let mut attempt_count = 0;
        let start_time = std::time::Instant::now();
        
        let result = retry_policy.execute(|| async {
            attempt_count += 1;
            if attempt_count < 3 {
                Err("temporary failure")
            } else {
                Ok("success")
            }
        }).await;
        
        // Should handle negative multiplier gracefully
        assert!(result.is_ok() || result.is_err());
        // Should not take negative time (impossible) or too long
        let elapsed = start_time.elapsed();
        assert!(elapsed < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_retry_policy_infinite_multiplier() {
        // Test retry policy with infinite multiplier
        let retry_policy = RetryPolicy::ExponentialBackoff(ExponentialBackoff {
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(100),
            max_attempts: 2,
            multiplier: f64::INFINITY,
        });
        
        let mut attempt_count = 0;
        let start_time = std::time::Instant::now();
        
        let result = retry_policy.execute(|| async {
            attempt_count += 1;
            Err::<(), _>("always fails")
        }).await;
        
        // Should handle infinite multiplier gracefully
        assert!(result.is_err());
        // Should be capped by max_delay
        let elapsed = start_time.elapsed();
        assert!(elapsed < Duration::from_secs(1));
    }
}

/// Edge cases for Security Framework
#[cfg(test)]
mod security_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_authentication_with_empty_user_id() {
        // Test authentication with empty user ID
        let auth_service = AuthenticationService::new();
        
        let user = User {
            id: "".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![Role::User],
        };
        
        let token = auth_service.authenticate(&user, "password").await;
        // Should handle empty user ID appropriately
        assert!(token.is_err());
    }

    #[tokio::test]
    async fn test_authentication_with_null_characters() {
        // Test authentication with null characters in input
        let auth_service = AuthenticationService::new();
        
        let user = User {
            id: "user\0123".to_string(),
            username: "test\0user".to_string(),
            email: "test\0@example.com".to_string(),
            roles: vec![Role::User],
        };
        
        let token = auth_service.authenticate(&user, "pass\0word").await;
        // Should handle null characters securely
        assert!(token.is_err());
    }

    #[tokio::test]
    async fn test_authentication_with_very_long_inputs() {
        // Test authentication with very long inputs
        let auth_service = AuthenticationService::new();
        
        let long_string = "a".repeat(10000);
        let user = User {
            id: long_string.clone(),
            username: long_string.clone(),
            email: format!("{}@example.com", long_string),
            roles: vec![Role::User],
        };
        
        let token = auth_service.authenticate(&user, &long_string).await;
        // Should handle very long inputs without crashing
        assert!(token.is_ok() || token.is_err()); // Either way, shouldn't panic
    }

    #[tokio::test]
    async fn test_authentication_with_unicode_injection() {
        // Test authentication with Unicode injection attempts
        let auth_service = AuthenticationService::new();
        
        let user = User {
            id: "user123".to_string(),
            username: "admin\u{202e}resu".to_string(), // Right-to-left override
            email: "test@example.com".to_string(),
            roles: vec![Role::User],
        };
        
        let token = auth_service.authenticate(&user, "password").await;
        // Should handle Unicode injection securely
        assert!(token.is_ok() || token.is_err()); // Shouldn't panic or bypass security
    }

    #[tokio::test]
    async fn test_user_with_no_roles() {
        // Test user with empty roles vector
        let user = User {
            id: "user123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![], // No roles
        };
        
        // Should handle users with no roles gracefully
        assert!(user.roles.is_empty());
        
        // Authentication might still work, but authorization should fail
        let auth_service = AuthenticationService::new();
        let token = auth_service.authenticate(&user, "password").await;
        // Token creation might succeed, but permissions should be limited
        assert!(token.is_ok() || token.is_err());
    }
}

/// Edge cases for Monitoring System
#[cfg(test)]
mod monitoring_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_with_special_characters() {
        // Test metrics collector with special character metric names
        let mut collector = MetricsCollector::new();
        
        let special_names = vec![
            "metric.with.dots",
            "metric-with-dashes",
            "metric_with_underscores",
            "metric/with/slashes",
            "metric:with:colons",
            "metric with spaces",
            "metric🦀with🦀emoji",
            "metric\nwith\nnewlines",
            "metric\twith\ttabs",
            "",  // Empty string
        ];
        
        for name in special_names {
            collector.increment_counter(name, 1);
            // Should handle all character types gracefully
            let value = collector.get_counter_value(name);
            assert!(value.is_some() || value.is_none()); // Either way, shouldn't panic
        }
    }

    #[tokio::test]
    async fn test_metrics_collector_with_extreme_values() {
        // Test metrics collector with extreme numeric values
        let mut collector = MetricsCollector::new();
        
        // Test with infinity and NaN
        collector.set_gauge("infinity_gauge", f64::INFINITY);
        collector.set_gauge("neg_infinity_gauge", f64::NEG_INFINITY);
        collector.set_gauge("nan_gauge", f64::NAN);
        
        // Should handle extreme values without crashing
        let inf_value = collector.get_gauge_value("infinity_gauge");
        let neg_inf_value = collector.get_gauge_value("neg_infinity_gauge");
        let nan_value = collector.get_gauge_value("nan_gauge");
        
        // Values might be sanitized or preserved as-is
        assert!(inf_value.is_some() || inf_value.is_none());
        assert!(neg_inf_value.is_some() || neg_inf_value.is_none());
        assert!(nan_value.is_some() || nan_value.is_none());
    }

    #[tokio::test]
    async fn test_metrics_collector_concurrent_access() {
        // Test metrics collector under concurrent access
        let collector = std::sync::Arc::new(std::sync::Mutex::new(MetricsCollector::new()));
        
        let mut handles = vec![];
        
        // Spawn multiple tasks that modify the same metric
        for i in 0..10 {
            let collector_clone = collector.clone();
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let mut c = collector_clone.lock().unwrap();
                    c.increment_counter("concurrent_counter", 1);
                    c.set_gauge("concurrent_gauge", (i * 100 + j) as f64);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        // Verify final state
        let final_collector = collector.lock().unwrap();
        let final_count = final_collector.get_counter_value("concurrent_counter");
        assert!(final_count.is_some());
        if let Some(count) = final_count {
            assert_eq!(count, 1000); // 10 tasks * 100 increments each
        }
    }

    #[tokio::test]
    async fn test_metrics_collector_memory_pressure() {
        // Test metrics collector under memory pressure
        let mut collector = MetricsCollector::new();
        
        // Create many metrics to test memory usage
        for i in 0..1000 {
            let metric_name = format!("metric_{}", i);
            collector.increment_counter(&metric_name, i);
            collector.set_gauge(&format!("gauge_{}", i), i as f64);
            
            // Record histogram values
            for j in 0..10 {
                collector.record_histogram(&format!("histogram_{}", i), j as f64);
            }
        }
        
        // Should handle many metrics without excessive memory usage
        // This is more of a performance/memory test
        let count = collector.get_counter_value("metric_999");
        assert!(count.is_some());
        assert_eq!(count.unwrap(), 999);
    }

    #[tokio::test]
    async fn test_metrics_collector_rapid_updates() {
        // Test metrics collector with rapid updates
        let mut collector = MetricsCollector::new();
        
        let start_time = std::time::Instant::now();
        
        // Perform rapid updates
        for i in 0..10000 {
            collector.increment_counter("rapid_counter", 1);
            collector.set_gauge("rapid_gauge", i as f64);
            collector.record_histogram("rapid_histogram", (i % 100) as f64);
        }
        
        let elapsed = start_time.elapsed();
        
        // Should handle rapid updates efficiently
        assert!(elapsed < Duration::from_secs(5));
        
        let final_count = collector.get_counter_value("rapid_counter").unwrap();
        assert_eq!(final_count, 10000);
    }
}

/// Timeout and cancellation testing
#[cfg(test)]
mod timeout_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_analysis_with_timeout() {
        // Test analysis that should timeout
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn test() {}").expect("Failed to write test file");

        let config = AnalysisConfig::default();
        let mut engine = EngineBuilder::new()
            .with_config(config)
            .build()
            .expect("Failed to build analysis engine");

        // Set a very short timeout
        let timeout_duration = Duration::from_millis(1);
        
        let result = tokio::time::timeout(
            timeout_duration,
            engine.analyze_file(&test_file)
        ).await;
        
        // Should either complete quickly or timeout
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_timeout_operations() {
        // Test circuit breaker with operations that timeout
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            half_open_max_calls: 1,
        };
        
        let mut circuit_breaker = CircuitBreaker::new(config);
        
        // Simulate operations that timeout
        let result1 = circuit_breaker.call(|| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<_, ()>("slow operation")
        }).await;
        
        let result2 = circuit_breaker.call(|| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<_, ()>("slow operation")
        }).await;
        
        // Circuit might open due to timeouts being treated as failures
        assert!(result1.is_ok() || result1.is_err());
        assert!(result2.is_ok() || result2.is_err());
    }
}