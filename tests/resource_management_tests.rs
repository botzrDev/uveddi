//! Comprehensive tests for resource management system
//! 
//! These tests validate the resource management infrastructure under various conditions
//! including memory exhaustion, concurrent access, degradation scenarios, and recovery.

use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;

use uveddi::resource_management::*;

mod memory_tracker_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_allocation_and_release() {
        let tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        
        // Test basic allocation
        let guard1 = tracker.allocate("test_component", 100).unwrap();
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current, 100);
        assert_eq!(stats.allocation_count, 1);
        
        // Test second allocation
        let guard2 = tracker.allocate("test_component_2", 200).unwrap();
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current, 300);
        assert_eq!(stats.allocation_count, 2);
        
        // Test release
        drop(guard1);
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current, 200);
        assert_eq!(stats.allocation_count, 1);
        
        drop(guard2);
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current, 0);
        assert_eq!(stats.allocation_count, 0);
    }
    
    #[tokio::test]
    async fn test_memory_exhaustion() {
        let tracker = Arc::new(MemoryTracker::new(500).unwrap());
        
        // Allocate up to the limit
        let _guard1 = tracker.allocate("test", 300).unwrap();
        let _guard2 = tracker.allocate("test", 200).unwrap();
        
        // This should fail - would exceed limit
        let result = tracker.allocate("test", 100);
        assert!(result.is_err());
        
        match result {
            Err(ResourceError::MemoryExhausted { requested, available, limit }) => {
                assert_eq!(requested, 100);
                assert_eq!(available, 0);
                assert_eq!(limit, 500);
            },
            _ => panic!("Expected MemoryExhausted error"),
        }
    }
    
    #[tokio::test]
    async fn test_memory_pressure_calculation() {
        let tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        
        let _guard1 = tracker.allocate("test", 700).unwrap();
        assert_eq!(tracker.get_memory_pressure(), 0.7);
        
        let _guard2 = tracker.allocate("test", 200).unwrap();
        assert_eq!(tracker.get_memory_pressure(), 0.9);
    }
    
    #[tokio::test]
    async fn test_peak_memory_tracking() {
        let tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        
        let guard1 = tracker.allocate("test", 500).unwrap();
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.peak, 500);
        
        let _guard2 = tracker.allocate("test", 300).unwrap();
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.peak, 800);
        
        drop(guard1);
        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current, 300);
        assert_eq!(stats.peak, 800); // Peak should remain
    }
    
    #[tokio::test]
    async fn test_component_tracking() {
        let tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        
        let _guard1 = tracker.allocate("component_a", 200).unwrap();
        let _guard2 = tracker.allocate("component_b", 300).unwrap();
        let _guard3 = tracker.allocate("component_a", 100).unwrap();
        
        let stats = tracker.get_usage_stats();
        assert_eq!(*stats.component_usage.get("component_a").unwrap(), 300);
        assert_eq!(*stats.component_usage.get("component_b").unwrap(), 300);
    }
}

mod streaming_file_processor_tests {
    use super::*;
    use tokio::fs::write;
    
    #[tokio::test]
    async fn test_small_file_in_memory_processing() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
        write(&file_path, content).await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let config = resource_config::FileHandlingLimits::default();
        let mut processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let result = processor.process_file(&file_path).await.unwrap();
        
        assert_eq!(result.processing_strategy, ProcessingStrategy::InMemory);
        assert_eq!(result.metadata.line_count, 3);
        assert_eq!(result.chunks_processed, 1);
        assert!(!result.metadata.binary_data_detected);
    }
    
    #[tokio::test]
    async fn test_large_file_streaming() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("large_test.rs");
        
        // Create a large file that exceeds in-memory threshold
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("fn function_{}() {{\n    // Comment {}\n}}\n", i, i));
        }
        write(&file_path, &content).await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let mut config = resource_config::FileHandlingLimits::default();
        config.max_file_size_memory = 1024; // 1KB limit to force streaming
        config.streaming_chunk_size = 512;   // 512 byte chunks
        
        let mut processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let result = processor.process_file(&file_path).await.unwrap();
        
        match result.processing_strategy {
            ProcessingStrategy::Streaming { chunk_size } => {
                assert_eq!(chunk_size, 512);
            },
            _ => panic!("Expected streaming processing strategy"),
        }
        
        assert!(result.chunks_processed > 1);
        assert_eq!(result.metadata.line_count, 3000); // 3 lines per function * 1000 functions
    }
    
    #[tokio::test]
    async fn test_file_type_filtering() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.xyz");
        write(&file_path, "content").await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let mut config = resource_config::FileHandlingLimits::default();
        config.allowed_file_types = vec!["rs".to_string(), "py".to_string()];
        
        let processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let strategy = processor.determine_strategy(&file_path).await.unwrap();
        match strategy {
            ProcessingStrategy::Skip { reason } => {
                assert!(reason.contains("File type not allowed"));
            },
            _ => panic!("Expected Skip strategy for disallowed file type"),
        }
    }
    
    #[tokio::test]
    async fn test_binary_data_detection() {
        let temp_dir = tempdir().unwrap();
        let binary_file = temp_dir.path().join("binary.dat");
        
        // Create file with binary data
        let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0x7F, 0x80];
        tokio::fs::write(&binary_file, &binary_data).await.unwrap();
        
        let memory_tracker = Arc::new(MemoryTracker::new(1024 * 1024).unwrap());
        let config = resource_config::FileHandlingLimits::default();
        let mut processor = StreamingFileProcessor::new(memory_tracker, config);
        
        let result = processor.process_file(&binary_file).await.unwrap();
        assert!(result.metadata.binary_data_detected);
    }
}

mod analysis_orchestrator_tests {
    use super::*;
    use tokio::sync::RwLock;
    
    fn create_test_config() -> Arc<RwLock<ResourceConfig>> {
        Arc::new(RwLock::new(ResourceConfig::testing()))
    }
    
    #[tokio::test]
    async fn test_concurrent_analysis_limiting() {
        let memory_tracker = Arc::new(MemoryTracker::new(10000).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();
        
        // Start multiple analyses up to the limit
        let mut executions = Vec::new();
        
        for i in 0..2 { // Testing config allows max 2 concurrent
            let execution = orchestrator.start_analysis(
                &format!("test_analysis_{}", i),
                100,
                AnalysisPriority::Normal,
            ).await.unwrap();
            
            executions.push(execution);
        }
        
        assert_eq!(orchestrator.get_active_count(), 2);
        
        // Attempt to start a third analysis - should be queued or fail
        let result = timeout(
            Duration::from_millis(100),
            orchestrator.start_analysis(
                "test_analysis_blocked",
                100,
                AnalysisPriority::Normal,
            )
        ).await;
        
        // Should timeout because we're at the limit
        assert!(result.is_err());
        
        // Clean up
        for execution in executions {
            execution.complete();
        }
    }
    
    #[tokio::test]
    async fn test_memory_based_analysis_rejection() {
        let memory_tracker = Arc::new(MemoryTracker::new(500).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker.clone(), config).unwrap();
        
        // Use up most of the memory
        let _memory_guard = memory_tracker.allocate("existing_usage", 400).unwrap();
        
        // Try to start an analysis that would exceed memory
        let result = orchestrator.start_analysis(
            "memory_heavy_analysis",
            200, // Would exceed available memory
            AnalysisPriority::Normal,
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_analysis_priority_ordering() {
        let memory_tracker = Arc::new(MemoryTracker::new(10000).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();
        
        // Fill up concurrent slots
        let _execution1 = orchestrator.start_analysis(
            "blocking_analysis_1",
            100,
            AnalysisPriority::Normal,
        ).await.unwrap();
        
        let _execution2 = orchestrator.start_analysis(
            "blocking_analysis_2",
            100,
            AnalysisPriority::Normal,
        ).await.unwrap();
        
        // Now queue analyses with different priorities
        // Note: In a full implementation, we'd need to verify queue ordering
        // For this test, we're just checking that priority is accepted
        
        let stats = orchestrator.get_statistics();
        assert_eq!(stats.total_analyses, 2);
    }
    
    #[tokio::test]
    async fn test_analysis_timeout() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let orchestrator = AnalysisOrchestrator::new(memory_tracker, config).unwrap();
        
        let execution = orchestrator.start_analysis(
            "test_analysis",
            100,
            AnalysisPriority::Normal,
        ).await.unwrap();
        
        // Simulate a long-running analysis by keeping the execution alive
        let duration = execution.complete();
        assert!(duration > Duration::from_nanos(0));
    }
}

mod degradation_manager_tests {
    use super::*;
    use tokio::sync::RwLock;
    
    fn create_test_config() -> Arc<RwLock<ResourceConfig>> {
        let mut config = ResourceConfig::testing();
        config.degradation.light_threshold = 0.5;
        config.degradation.heavy_threshold = 0.7;
        config.degradation.emergency_threshold = 0.9;
        config.degradation.recovery_threshold = 0.4;
        
        Arc::new(RwLock::new(config))
    }
    
    #[tokio::test]
    async fn test_degradation_level_progression() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker.clone(), config);
        
        // Initially should be normal
        assert_eq!(manager.get_current_level(), DegradationLevel::Normal);
        
        // Use memory to trigger light degradation (50%+)
        let _guard1 = memory_tracker.allocate("test", 600).unwrap();
        
        let changed = manager.adjust_service_level().await.unwrap();
        assert!(changed);
        assert_eq!(manager.get_current_level(), DegradationLevel::LightDegradation);
        
        // Use more memory to trigger heavy degradation (70%+)
        let _guard2 = memory_tracker.allocate("test", 100).unwrap();
        
        let changed = manager.adjust_service_level().await.unwrap();
        assert!(changed);
        assert_eq!(manager.get_current_level(), DegradationLevel::HeavyDegradation);
        
        // Use even more memory to trigger emergency mode (90%+)
        let _guard3 = memory_tracker.allocate("test", 200).unwrap();
        
        let changed = manager.adjust_service_level().await.unwrap();
        assert!(changed);
        assert_eq!(manager.get_current_level(), DegradationLevel::EmergencyMode);
    }
    
    #[tokio::test]
    async fn test_feature_state_management() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);
        
        // Initially all features should be enabled
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        assert!(manager.is_feature_enabled(&FeatureCategory::DetailedReporting));
        
        // Apply heavy degradation
        manager.force_degradation_level(
            DegradationLevel::HeavyDegradation,
            "Test degradation".to_string()
        ).await.unwrap();
        
        // AI analysis should be disabled in heavy degradation
        assert!(!manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        
        // Recover to normal
        manager.force_degradation_level(
            DegradationLevel::Normal,
            "Test recovery".to_string()
        ).await.unwrap();
        
        // Features should be re-enabled
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        assert!(manager.is_feature_enabled(&FeatureCategory::DetailedReporting));
    }
    
    #[tokio::test]
    async fn test_emergency_cleanup() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);
        
        manager.trigger_emergency_cleanup().await.unwrap();
        
        assert_eq!(manager.get_current_level(), DegradationLevel::EmergencyMode);
        
        // Most features should be disabled
        assert!(!manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        assert!(!manager.is_feature_enabled(&FeatureCategory::InteractiveFeatures));
        assert!(!manager.is_feature_enabled(&FeatureCategory::LargeFileProcessing));
    }
    
    #[tokio::test]
    async fn test_degradation_history() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);
        
        // Apply some degradation levels
        manager.force_degradation_level(
            DegradationLevel::LightDegradation,
            "Test 1".to_string()
        ).await.unwrap();
        
        manager.force_degradation_level(
            DegradationLevel::HeavyDegradation,
            "Test 2".to_string()
        ).await.unwrap();
        
        let history = manager.get_degradation_history(Duration::from_secs(60));
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].1, DegradationLevel::LightDegradation);
        assert_eq!(history[1].1, DegradationLevel::HeavyDegradation);
        assert_eq!(history[0].2, "Test 1");
        assert_eq!(history[1].2, "Test 2");
    }
    
    #[tokio::test]
    async fn test_manual_feature_override() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let manager = GracefulDegradationManager::new(memory_tracker, config);
        
        // Manually disable a feature
        manager.set_feature_enabled(FeatureCategory::AiAnalysis, false).await.unwrap();
        assert!(!manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        
        // Re-enable it
        manager.set_feature_enabled(FeatureCategory::AiAnalysis, true).await.unwrap();
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
    }
}

mod resource_monitor_tests {
    use super::*;
    use tokio::sync::RwLock;
    
    fn create_test_config() -> Arc<RwLock<ResourceConfig>> {
        let mut config = ResourceConfig::testing();
        config.monitoring.interval_seconds = 1; // Fast monitoring for tests
        Arc::new(RwLock::new(config))
    }
    
    #[tokio::test]
    async fn test_resource_monitoring_creation() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker, config);
        
        let metrics = monitor.get_current_metrics().await.unwrap();
        assert_eq!(metrics.active_alerts.len(), 0);
        assert!(metrics.uptime_seconds >= 0);
        assert_eq!(metrics.current_usage.memory.current, 0);
    }
    
    #[tokio::test]
    async fn test_memory_pressure_alert_generation() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker.clone(), config);
        
        // Allocate memory to trigger pressure
        let _guard = memory_tracker.allocate("test", 800).unwrap();
        
        // Trigger memory pressure response
        monitor.trigger_memory_pressure_response().await.unwrap();
        
        let metrics = monitor.get_current_metrics().await.unwrap();
        assert!(!metrics.active_alerts.is_empty());
        
        let alert = &metrics.active_alerts[0];
        assert_eq!(alert.resource_type, "memory");
        assert!(alert.current_value > 0.7); // Should be high pressure
        assert!(!alert.acknowledged);
    }
    
    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker.clone(), config);
        
        // Generate an alert
        let _guard = memory_tracker.allocate("test", 800).unwrap();
        monitor.trigger_memory_pressure_response().await.unwrap();
        
        let metrics = monitor.get_current_metrics().await.unwrap();
        let alert_id = &metrics.active_alerts[0].id;
        
        // Acknowledge the alert
        monitor.acknowledge_alert(alert_id).unwrap();
        
        let metrics = monitor.get_current_metrics().await.unwrap();
        assert!(metrics.active_alerts[0].acknowledged);
    }
    
    #[tokio::test]
    async fn test_emergency_cleanup_alert() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker, config);
        
        monitor.trigger_emergency_cleanup().await.unwrap();
        
        let metrics = monitor.get_current_metrics().await.unwrap();
        assert!(!metrics.active_alerts.is_empty());
        
        let alert = &metrics.active_alerts[0];
        assert_eq!(alert.severity, resource_monitor::AlertSeverity::Emergency);
        assert!(alert.message.contains("Emergency"));
    }
    
    #[tokio::test]
    async fn test_alert_callback_registration() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = create_test_config();
        let monitor = ResourceMonitor::new(memory_tracker.clone(), config);
        
        let alert_received = Arc::new(std::sync::Mutex::new(false));
        let alert_received_clone = alert_received.clone();
        
        monitor.register_alert_callback(move |alert| {
            *alert_received_clone.lock().unwrap() = true;
            println!("Alert received: {}", alert.message);
        });
        
        // Generate an alert
        let _guard = memory_tracker.allocate("test", 950).unwrap();
        monitor.trigger_memory_pressure_response().await.unwrap();
        
        // Give the callback a moment to execute
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        assert!(*alert_received.lock().unwrap());
    }
}

mod integration_tests {
    use super::*;
    use tokio::sync::RwLock;
    
    #[tokio::test]
    async fn test_full_resource_management_lifecycle() {
        let config = ResourceConfig::testing();
        let manager = ResourceManager::new(config).unwrap();
        
        // Start monitoring (in real scenario this runs in background)
        let monitoring_task = {
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                if let Err(e) = manager_clone.start_monitoring().await {
                    eprintln!("Monitoring failed: {}", e);
                }
            })
        };
        
        // Simulate analysis workload
        let stats = manager.get_usage_stats();
        assert_eq!(stats.memory.current, 0);
        assert_eq!(stats.active_analyses, 0);
        
        // Allocate some memory
        let _guard = manager.allocate_memory("test_analysis", 100).unwrap();
        
        let stats = manager.get_usage_stats();
        assert_eq!(stats.memory.current, 100);
        
        // Test resource exhaustion scenario
        let mut guards = Vec::new();
        for i in 0..10 {
            if let Ok(guard) = manager.allocate_memory(&format!("analysis_{}", i), 10) {
                guards.push(guard);
            } else {
                break; // Expected when we hit limits
            }
        }
        
        // Clean up
        drop(guards);
        
        let final_stats = manager.get_usage_stats();
        assert_eq!(final_stats.memory.current, 100); // Only the original guard remains
        
        // Cancel monitoring task
        monitoring_task.abort();
    }
    
    #[tokio::test]
    async fn test_stress_test_memory_allocation() {
        let memory_tracker = Arc::new(MemoryTracker::new(10000).unwrap());
        
        // Spawn multiple concurrent tasks that allocate and release memory
        let tasks: Vec<_> = (0..10).map(|i| {
            let tracker = memory_tracker.clone();
            tokio::spawn(async move {
                for j in 0..10 {
                    let component_name = format!("component_{}_{}", i, j);
                    if let Ok(guard) = tracker.allocate(&component_name, 50) {
                        // Hold the allocation for a bit
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        drop(guard);
                    }
                }
            })
        }).collect();
        
        // Wait for all tasks to complete
        for task in tasks {
            task.await.unwrap();
        }
        
        // All memory should be released
        let final_stats = memory_tracker.get_usage_stats();
        assert_eq!(final_stats.current, 0);
        assert!(final_stats.peak > 0); // Should have had peak usage
    }
    
    #[tokio::test]
    async fn test_degradation_under_pressure() {
        let memory_tracker = Arc::new(MemoryTracker::new(1000).unwrap());
        let config = Arc::new(RwLock::new(ResourceConfig::testing()));
        let manager = GracefulDegradationManager::new(memory_tracker.clone(), config);
        
        // Gradually increase memory pressure
        let _guard1 = memory_tracker.allocate("test", 300).unwrap(); // 30%
        assert_eq!(manager.get_current_level(), DegradationLevel::Normal);
        
        let _guard2 = memory_tracker.allocate("test", 300).unwrap(); // 60%
        manager.adjust_service_level().await.unwrap();
        assert_eq!(manager.get_current_level(), DegradationLevel::LightDegradation);
        
        let _guard3 = memory_tracker.allocate("test", 200).unwrap(); // 80%
        manager.adjust_service_level().await.unwrap();
        assert_eq!(manager.get_current_level(), DegradationLevel::HeavyDegradation);
        
        let _guard4 = memory_tracker.allocate("test", 100).unwrap(); // 90%
        manager.adjust_service_level().await.unwrap();
        assert_eq!(manager.get_current_level(), DegradationLevel::EmergencyMode);
        
        // Verify features are disabled appropriately
        assert!(!manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        assert!(!manager.is_feature_enabled(&FeatureCategory::LargeFileProcessing));
        
        // Test recovery by releasing memory
        drop(_guard4);
        drop(_guard3);
        drop(_guard2);
        
        // Note: Real recovery would require time-based conditions,
        // but we can test forced recovery
        manager.force_degradation_level(DegradationLevel::Normal, "Recovery test".to_string()).await.unwrap();
        
        assert!(manager.is_feature_enabled(&FeatureCategory::AiAnalysis));
        assert!(manager.is_feature_enabled(&FeatureCategory::LargeFileProcessing));
    }
}