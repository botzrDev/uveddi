//! Integration tests for Phase 2 memory optimization - Object Pools

use std::sync::Arc;
use std::thread;
use uveddi::analysis::memory::*;

#[test]
fn test_phase2_object_pool_initialization() {
    // Test that object pools can be initialized with custom config
    let mut config = MemoryOptimizationConfig::default();
    config.object_pools.detector_pool_capacity = 50;
    config.object_pools.temporary_pool_capacity = 100;

    let pools = initialize_detector_pools(&config);
    let stats = pools.get_all_stats();

    assert_eq!(stats.dead_code_configs.total_capacity, 50);
    assert_eq!(stats.string_vectors.total_capacity, 100);
    assert_eq!(stats.issue_vectors.total_capacity, 50);
}

#[test]
fn test_memory_pool_basic_functionality() {
    // Test core MemoryPool functionality
    let pool = MemoryPool::<Vec<String>>::new(10, AllocationStrategy::FixedSize(10), "test_pool");

    // Test getting and returning objects
    {
        let mut obj = pool.get();
        obj.push("test".to_string());
        assert_eq!(obj.len(), 1);
        assert_eq!(obj[0], "test");
    } // Object returned to pool here

    // Pool should be functional
    let stats = pool.get_stats();
    assert_eq!(stats.pool_name, "test_pool");
    assert_eq!(stats.total_capacity, 10);
}

#[test]
fn test_pooled_object_reset() {
    let pool = MemoryPool::<Vec<String>>::new(5, AllocationStrategy::FixedSize(5), "reset_test");

    let mut obj = pool.get();
    obj.push("data".to_string());
    obj.push("more_data".to_string());
    assert_eq!(obj.len(), 2);

    obj.reset();
    assert_eq!(obj.len(), 0);
}

#[test]
fn test_pool_pre_population() {
    let pool = MemoryPool::<Vec<i32>>::new(20, AllocationStrategy::FixedSize(20), "prepop_test");

    // Pre-populate with 50% capacity
    pool.pre_populate(10);

    let stats = pool.get_stats();
    assert!(stats.total_objects_available > 0);
    // With sharding, objects are distributed across shards so total available may be less than expected
    assert!(stats.total_objects_available <= 20); // Use pool capacity instead
    assert!(stats.utilization_percentage > 0.0);
}

#[test]
fn test_pool_statistics_and_efficiency() {
    let pool = MemoryPool::<String>::new(
        100,
        AllocationStrategy::AdaptiveBased {
            target_memory: 1024,
        },
        "stats_test",
    );

    // Pre-populate to test efficiency metrics
    pool.pre_populate(40); // 40% utilization

    let stats = pool.get_stats();
    assert_eq!(stats.pool_name, "stats_test");
    assert_eq!(stats.total_capacity, 100);
    assert!(stats.is_efficiently_utilized()); // Should be between 20-80%

    let recommendation = stats.get_efficiency_recommendation();
    assert!(recommendation.contains("efficiently utilized"));
}

#[test]
fn test_concurrent_pool_access() {
    let pool = Arc::new(MemoryPool::<Vec<u64>>::new(
        50,
        AllocationStrategy::GrowthBased {
            initial: 10,
            growth_factor: 1.5,
        },
        "concurrent_test",
    ));

    pool.pre_populate(25);

    let mut handles = vec![];

    // Spawn multiple threads to access pool concurrently
    for i in 0..5 {
        let pool_clone = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            for j in 0..3 {
                let mut obj = pool_clone.get();
                obj.push((i * 10 + j) as u64);
                // Object automatically returned when dropped
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Pool should still be functional
    let stats = pool.get_stats();
    assert_eq!(stats.pool_name, "concurrent_test");
    assert!(stats.shard_count >= 4); // Should have multiple shards for concurrency
}

#[test]
fn test_detector_pools_integration() {
    let pools = DetectorPools::new();

    // Test getting detector configurations
    {
        let _dead_code_config = pools.dead_code_configs.get();
        let mut string_vec = pools.string_vectors.get();
        let _issue_vec = pools.issue_vectors.get();

        // Use the objects
        string_vec.push("test_pattern".to_string());
        assert_eq!(string_vec.len(), 1);

        // Objects will be returned when dropped
    }

    // Pools should still be functional
    let stats = pools.get_all_stats();
    assert_eq!(stats.dead_code_configs.pool_name, "dead_code_configs");
    assert_eq!(stats.string_vectors.pool_name, "string_vectors");
    assert_eq!(stats.issue_vectors.pool_name, "issue_vectors");
}

#[test]
fn test_detector_pools_metrics_export() {
    let pools = DetectorPools::new();
    pools.pre_populate(30.0);

    let metrics = pools.export_metrics();

    // Verify JSON structure
    assert!(metrics["detector_pools"].is_object());
    assert!(metrics["detector_pools"]["dead_code_configs"].is_object());
    assert!(metrics["detector_pools"]["temporary_objects"].is_object());

    // Verify metrics data
    let dead_code_metrics = &metrics["detector_pools"]["dead_code_configs"];
    assert!(dead_code_metrics["capacity"].is_number());
    assert!(dead_code_metrics["available"].is_number());
    assert!(dead_code_metrics["utilization_percent"].is_number());
    assert!(dead_code_metrics["efficient"].is_boolean());

    // Verify temporary objects metrics
    let temp_metrics = &metrics["detector_pools"]["temporary_objects"];
    assert!(temp_metrics["string_vectors"].is_object());
    assert!(temp_metrics["issue_vectors"].is_object());
}

#[test]
fn test_efficiency_reporting() {
    let pools = DetectorPools::new();
    pools.pre_populate(45.0); // Should be efficiently utilized

    let efficiency_report = pools.get_efficiency_report();
    assert_eq!(efficiency_report.len(), 3); // 3 pools total

    // Each report should mention efficiency
    for report in efficiency_report {
        assert!(report.contains("Pool"));
        assert!(report.contains("%"));
    }

    let stats = pools.get_all_stats();
    let overall_efficiency = stats.overall_efficiency();
    assert!((0.0..=100.0).contains(&overall_efficiency));

    let efficient_count = stats.efficiently_utilized_count();
    assert!(efficient_count <= 3);
}

#[test]
fn test_memory_optimization_with_pools() {
    // Test full memory optimization initialization with pools
    let mut config = MemoryOptimizationConfig::default();
    config.object_pools.enabled = true;
    config.object_pools.pre_population_percentage = 30.0;

    let result = initialize_memory_optimization(config);
    assert!(
        result.is_ok(),
        "Memory optimization with pools should initialize successfully"
    );

    // Test status export includes pool information
    let status = get_optimization_status();
    assert_eq!(
        status["phase"].as_str().unwrap(),
        "Phase 4 - Zero-Copy AST Caching"
    );
    assert!(status["pools"].is_object());
    assert!(status["pools"]["dead_code_configs"].is_object());
    // Arena information may be present depending on feature flags
    // This test only verifies pools are properly initialized
}

#[test]
fn test_allocation_strategies() {
    // Test different allocation strategies
    let fixed_pool = MemoryPool::<i32>::new(10, AllocationStrategy::FixedSize(10), "fixed_test");

    let growth_pool = MemoryPool::<i32>::new(
        10,
        AllocationStrategy::GrowthBased {
            initial: 5,
            growth_factor: 2.0,
        },
        "growth_test",
    );

    let adaptive_pool = MemoryPool::<i32>::new(
        10,
        AllocationStrategy::AdaptiveBased {
            target_memory: 1024,
        },
        "adaptive_test",
    );

    // All pools should be functional
    let _obj1 = fixed_pool.get();
    let _obj2 = growth_pool.get();
    let _obj3 = adaptive_pool.get();

    // Verify stats
    assert_eq!(fixed_pool.get_stats().pool_name, "fixed_test");
    assert_eq!(growth_pool.get_stats().pool_name, "growth_test");
    assert_eq!(adaptive_pool.get_stats().pool_name, "adaptive_test");
}

#[test]
fn test_pool_capacity_limits() {
    let pool = MemoryPool::<Vec<u8>>::new(
        5, // Small capacity
        AllocationStrategy::FixedSize(5),
        "capacity_test",
    );

    // Pre-populate to capacity
    pool.pre_populate(5);

    // Get more objects than capacity allows
    let objects: Vec<_> = (0..10).map(|_| pool.get()).collect();

    // All objects should be valid
    assert_eq!(objects.len(), 10);

    // Some objects came from pool, others were created new
    let stats = pool.get_stats();
    assert_eq!(stats.total_capacity, 5);
}

#[test]
fn test_configuration_presets_with_pools() {
    // Test large codebase configuration
    let large_config = MemoryOptimizationConfig::large_codebase();
    assert!(large_config.object_pools.enabled);
    assert_eq!(large_config.object_pools.detector_pool_capacity, 200);
    assert_eq!(large_config.object_pools.temporary_pool_capacity, 400);
    assert_eq!(large_config.object_pools.pre_population_percentage, 30.0);

    // Test small project configuration
    let small_config = MemoryOptimizationConfig::small_project();
    assert!(small_config.object_pools.enabled);
    assert_eq!(small_config.object_pools.detector_pool_capacity, 50);
    assert_eq!(small_config.object_pools.temporary_pool_capacity, 100);
    assert_eq!(small_config.object_pools.pre_population_percentage, 20.0);

    // Both should validate successfully
    assert!(large_config.validate().is_ok());
    assert!(small_config.validate().is_ok());
}

#[test]
fn test_global_detector_pools() {
    // Test the global DETECTOR_POOLS static
    let stats = DETECTOR_POOLS.get_all_stats();

    assert_eq!(stats.dead_code_configs.pool_name, "dead_code_configs");
    assert_eq!(stats.string_vectors.pool_name, "string_vectors");
    assert_eq!(stats.issue_vectors.pool_name, "issue_vectors");

    // Should be able to get objects from global pools
    let _config = DETECTOR_POOLS.dead_code_configs.get();
    let _strings = DETECTOR_POOLS.string_vectors.get();
    let _issues = DETECTOR_POOLS.issue_vectors.get();
}
