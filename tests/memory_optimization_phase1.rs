//! Integration tests for Phase 1 memory optimization foundation

use uveddi::analysis::memory::*;

#[test]
fn test_phase1_foundation_setup() {
    // Test that memory optimization can be initialized
    let config = MemoryOptimizationConfig::default();
    let result = initialize_memory_optimization(config);
    assert!(result.is_ok(), "Memory optimization initialization should succeed");
}

#[test]
fn test_allocator_detection() {
    let allocator = get_allocator_info();
    assert!(allocator == "mimalloc" || allocator == "system");

    // Test consistency between functions
    let is_optimized = is_optimized_allocator();
    if cfg!(feature = "mimalloc") {
        assert_eq!(allocator, "mimalloc");
        assert!(is_optimized);
    } else {
        assert_eq!(allocator, "system");
        assert!(!is_optimized);
    }
}

#[test]
fn test_basic_metrics_collection() {
    // Test that metrics can be collected and exported
    BASIC_MEMORY_METRICS.update_memory_usage(1024 * 1024 * 1024); // 1GB

    let metrics = BASIC_MEMORY_METRICS.get_metrics();
    assert_eq!(metrics.current_memory_bytes, 1024 * 1024 * 1024);
    assert!(metrics.is_within_target()); // Should be within 8GB default target

    let json = BASIC_MEMORY_METRICS.export_json();
    assert!(json["memory_optimization_phase1"]["current_memory_gb"].as_f64().unwrap() > 0.0);
}

#[test]
fn test_configuration_presets() {
    // Test large codebase configuration
    let large_config = MemoryOptimizationConfig::large_codebase();
    assert!(large_config.validate().is_ok());
    assert_eq!(large_config.target_max_memory_bytes, 6 * 1024 * 1024 * 1024);

    // Test small project configuration
    let small_config = MemoryOptimizationConfig::small_project();
    assert!(small_config.validate().is_ok());
    assert_eq!(small_config.target_max_memory_bytes, 2 * 1024 * 1024 * 1024);
}

#[test]
fn test_optimization_status_export() {
    let status = get_optimization_status();

    // Verify required fields are present
    assert!(status["phase"].is_string());
    assert!(status["allocator"].is_string());
    assert!(status["optimized"].is_boolean());
    assert!(status["metrics"].is_object());

    // Verify phase information
    assert_eq!(status["phase"].as_str().unwrap(), "Phase 4 - Zero-Copy AST Caching");
}

#[cfg(feature = "mimalloc")]
#[test]
fn test_mimalloc_enabled() {
    assert!(is_optimized_allocator());
    assert_eq!(get_allocator_info(), "mimalloc");
}

#[cfg(not(feature = "mimalloc"))]
#[test]
fn test_system_allocator_fallback() {
    assert!(!is_optimized_allocator());
    assert_eq!(get_allocator_info(), "system");
}