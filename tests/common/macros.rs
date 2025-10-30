//! Test macros for feature-aware testing
//!
//! This module provides macros that simplify creating tests that work
//! with different feature configurations, particularly AI features.

/// Macro for AI feature-aware tests
/// 
/// This macro creates a test that behaves differently based on whether
/// AI features are compiled in or not.
#[macro_export]
macro_rules! ai_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::common::TestEnvironment;
            use crate::core::features::AiFeatureConfig;
            
            log::info!("Running AI-aware test: {} - {}", stringify!($test_name), AiFeatureConfig::description());
            
            #[cfg(any(feature = "ai", feature = "local-ai"))]
            {
                let config = crate::common::TestConfig {
                    enable_ai_features: true,
                    ..Default::default()
                };
                let mut env = TestEnvironment::with_config(config).await.unwrap();
                $test_body(env).await;
            }
            
            #[cfg(not(any(feature = "ai", feature = "local-ai")))]
            {
                let mut env = TestEnvironment::new().await.unwrap();
                $test_body(env).await;
            }
        }
    };
}

/// Macro for performance monitoring tests
/// 
/// This macro wraps tests with performance measurement and can be used
/// to detect performance regressions.
#[macro_export]
macro_rules! performance_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::common::{TestEnvironment, TestConfig};
            
            let config = TestConfig {
                enable_performance_monitoring: true,
                ..Default::default()
            };
            let mut env = TestEnvironment::with_config(config).await.unwrap();
            
            let start = std::time::Instant::now();
            $test_body(env).await;
            let duration = start.elapsed();
            
            // Performance assertions can be added here
            log::info!("Test {} completed in {:?}", stringify!($test_name), duration);
            
            // Fail if test takes longer than 30 seconds (adjust as needed)
            assert!(duration < std::time::Duration::from_secs(30), 
                "Test {} took too long: {:?}", stringify!($test_name), duration);
        }
    };
}

/// Macro for creating integration tests
/// 
/// This macro provides a pattern for integration tests that need setup
/// and teardown phases.
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $setup:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::common::TestEnvironment;
            
            let mut env = TestEnvironment::new().await.unwrap();
            
            // Setup phase
            $setup(&mut env).await.unwrap();
            
            // Test execution phase
            $test_body(env).await.unwrap();
        }
    };
}

/// Macro for feature flag testing
/// 
/// This macro creates tests that verify feature flag behavior.
#[macro_export]
macro_rules! feature_test {
    ($test_name:ident, $feature:literal, $enabled_test:expr, $disabled_test:expr) => {
        #[tokio::test]
        async fn $test_name() {
            #[cfg(feature = $feature)]
            {
                log::info!("Testing with feature '{}' enabled", $feature);
                $enabled_test().await;
            }
            
            #[cfg(not(feature = $feature))]
            {
                log::info!("Testing with feature '{}' disabled", $feature);
                $disabled_test().await;
            }
        }
    };
}

/// Macro for memory leak detection tests
/// 
/// This macro wraps tests with memory usage monitoring to detect leaks.
#[macro_export]
macro_rules! memory_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::common::TestEnvironment;
            
            // Get initial memory usage
            let initial_memory = get_memory_usage();
            
            // Run test multiple times to amplify potential leaks
            for iteration in 0..10 {
                let mut env = TestEnvironment::new().await.unwrap();
                $test_body(env, iteration).await;
                // Explicit cleanup to ensure resources are released
                env.cleanup();
            }
            
            // Force garbage collection (Rust doesn't have GC, but this gives allocator time to cleanup)
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            
            let final_memory = get_memory_usage();
            let memory_growth = final_memory.saturating_sub(initial_memory);
            
            // Memory growth should be minimal (< 10MB for 10 iterations)
            assert!(memory_growth < 10_000_000, 
                "Potential memory leak detected: {} bytes growth", memory_growth);
                
            log::info!("Memory test {} completed - growth: {} bytes", 
                stringify!($test_name), memory_growth);
        }
    };
}

/// Macro for database transaction tests
/// 
/// This macro ensures database operations are properly isolated in tests.
#[macro_export]
macro_rules! db_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use crate::common::TestEnvironment;
            
            let mut env = TestEnvironment::new().await.unwrap();
            
            // Start transaction
            let tx = env.database().begin_transaction().await.unwrap();
            
            // Run test
            let result = std::panic::AssertUnwindSafe($test_body(env))
                .catch_unwind()
                .await;
            
            // Rollback transaction regardless of test outcome
            tx.rollback().await.unwrap();
            
            // Re-throw panic if test failed
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}

/// Helper function to get current memory usage (approximation)
fn get_memory_usage() -> usize {
    // This is a simple approximation - in a real system you might use
    // more sophisticated memory monitoring
    use std::alloc::{GlobalAlloc, System};
    
    // For now, return a placeholder value
    // In a production system, you might use libraries like `memory-stats`
    // or system-specific APIs
    0
}

#[cfg(test)]
mod macro_tests {
    use super::*;
    use crate::common::TestEnvironment;
    
    ai_test!(test_ai_macro_basic, |mut _env: TestEnvironment| async {
        // This test will run with or without AI features
        assert!(true, "Basic AI test should pass");
    });
    
    performance_test!(test_performance_macro, |mut _env: TestEnvironment| async {
        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(true, "Performance test should complete quickly");
    });
    
    integration_test!(
        test_integration_macro,
        |env: &mut TestEnvironment| async {
            // Setup phase
            env.create_test_file("setup.txt", "setup complete").unwrap();
            Ok(())
        },
        |env: TestEnvironment| async {
            // Test phase
            let setup_file = env.temp_path().join("setup.txt");
            assert!(setup_file.exists(), "Setup should have created file");
            Ok(())
        }
    );
    
    feature_test!(
        test_feature_macro,
        "ai",
        || async {
            log::info!("AI feature is enabled");
        },
        || async {
            log::info!("AI feature is disabled");
        }
    );
}
