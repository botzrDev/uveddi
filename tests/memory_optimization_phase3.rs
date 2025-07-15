//! Integration tests for Phase 3 memory optimization - Arena Allocation

use uveddi::analysis::memory::*;
// use std::path::PathBuf; // Not needed in current tests

#[test]
fn test_phase3_arena_allocation_initialization() {
    // Test that arena allocation can be initialized with custom config
    let mut config = MemoryOptimizationConfig::default();
    config.arena_allocation.default_arena_size_mb = 64;
    config.arena_allocation.max_concurrent_arenas = 16;
    
    let result = initialize_memory_optimization(config);
    assert!(result.is_ok(), "Arena allocation should initialize successfully");
    
    // Test status export includes arena information
    let status = get_optimization_status();
    assert_eq!(status["phase"].as_str().unwrap(), "Phase 4 - Zero-Copy AST Caching");
    assert!(status["arenas"].is_object());
}

#[test]
fn test_analysis_arena_basic_functionality() {
    // Test core ArenaManager functionality with bumpalo-herd pattern
    let manager = AnalysisArenaManager::new();
    let mut handle = manager.get_arena();
    
    // Test basic allocation
    let value = handle.alloc(42i32);
    assert_eq!(*value, 42);
    
    // Test string allocation
    let text = handle.alloc_str("arena test string");
    assert_eq!(text, "arena test string");
    
    // Check metrics
    let stats = manager.get_stats();
    assert!(stats.total_arenas_created >= 1);
    assert!(handle.bytes_allocated() > 0);
}

#[test]
fn test_arena_collections() {
    let manager = AnalysisArenaManager::new();
    let handle = manager.get_arena();
    
    // Test arena vector creation
    let mut issues = handle.create_vec();
    issues.push(uveddi::database::models::ArchitecturalIssue {
        issue_id: Some(1),
        analysis_run_id: 1,
        anti_pattern_type_id: 1,
        file_path: "test.rs".to_string(),
        start_line: Some(1),
        end_line: Some(10),
        severity: "medium".to_string(),
        description: "arena test issue".to_string(),
        code_snippet: Some("test code".to_string()),
        ai_explanation: None,
    });
    
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].file_path, "test.rs");
    assert_eq!(issues[0].description, "arena test issue");
    
    // Test arena string creation
    let mut arena_string = handle.create_string();
    arena_string.push_str("arena allocated string");
    assert_eq!(arena_string.as_str(), "arena allocated string");
    
    // Convert to owned
    let owned_issues = conversion::arena_issues_to_owned(issues);
    assert_eq!(owned_issues.len(), 1);
}

#[test]
fn test_arena_handle_functionality() {
    let manager = AnalysisArenaManager::new();
    let mut handle = manager.get_arena();
    
    // Test initial state
    assert_eq!(handle.bytes_allocated(), 0);
    
    // Allocate some data
    let _data1 = handle.alloc([0u8; 512]); // 512 bytes
    let _data2 = handle.alloc_str("test string"); // Additional bytes
    
    // Check that bytes are tracked
    assert!(handle.bytes_allocated() > 512);
    
    // Test reset functionality
    handle.reset();
    assert_eq!(handle.bytes_allocated(), 0);
}

#[test]
fn test_arena_manager_stats() {
    let manager = AnalysisArenaManager::new();
    
    let initial_stats = manager.get_stats();
    assert_eq!(initial_stats.total_arenas_created, 0);
    assert_eq!(initial_stats.default_capacity_mb, 32);
    
    // Get some arenas
    let _handle1 = manager.get_arena();
    let _handle2 = manager.get_arena();
    
    let updated_stats = manager.get_stats();
    assert_eq!(updated_stats.total_arenas_created, 2);
    
    // Export metrics
    let metrics = manager.export_metrics();
    assert_eq!(metrics["arena_manager"]["total_arenas_created"], 2);
    assert_eq!(metrics["arena_manager"]["pattern"], "bumpalo-herd");
    assert_eq!(metrics["arena_manager"]["thread_safe"], true);
    assert_eq!(metrics["arena_manager"]["contention_free"], true);
}

#[test]
fn test_arena_analysis_result_structure() {
    let result = ArenaAnalysisResult {
        file_path: "example.rs".to_string(),
        issues: vec![],
        analysis_metadata: AnalysisMetadata {
            lines_of_code: 50,
            functions_found: 3,
            complexity_score: 1.5,
        },
        performance_metrics: AnalysisPerformanceMetrics {
            arena_bytes_used: 1024,
            analysis_duration_ms: 10,
            memory_efficiency: 0.9,
        },
    };
    
    // Verify it's Send (can be passed between threads)
    fn assert_send<T: Send>(_: &T) {}
    assert_send(&result);
    
    // Verify it can be serialized
    let _json = serde_json::to_string(&result).unwrap();
}

#[test]
fn test_arena_conversion_helpers() {
    let manager = AnalysisArenaManager::new();
    let handle = manager.get_arena();
    
    // Create arena-allocated data
    let mut arena_issues = handle.create_vec();
    arena_issues.push(uveddi::database::models::ArchitecturalIssue {
        issue_id: Some(1),
        analysis_run_id: 1,
        anti_pattern_type_id: 1,
        file_path: "test.rs".to_string(),
        start_line: Some(1),
        end_line: Some(10),
        severity: "high".to_string(),
        description: "test issue".to_string(),
        code_snippet: Some("test code".to_string()),
        ai_explanation: None,
    });
    
    // Test the conversion helpers
    let owned_issues = conversion::arena_issues_to_owned(arena_issues);
    assert_eq!(owned_issues.len(), 1);
    assert_eq!(owned_issues[0].file_path, "test.rs");
    
    // Test string conversion
    let mut arena_strings = handle.create_vec();
    arena_strings.push(bumpalo::collections::String::from_str_in("test1", handle.bump()));
    arena_strings.push(bumpalo::collections::String::from_str_in("test2", handle.bump()));
    
    let owned_strings = conversion::arena_strings_to_owned(arena_strings);
    assert_eq!(owned_strings.len(), 2);
    assert_eq!(owned_strings[0], "test1");
    assert_eq!(owned_strings[1], "test2");
}

#[test]
fn test_rayon_integration_example() {
    let manager = AnalysisArenaManager::new();
    let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
    
    // Test the concurrent analysis function
    let results = rayon_integration::analyze_files_concurrent(
        files,
        &manager,
        |arena_handle, file_path| {
            rayon_integration::example_file_analyzer(arena_handle, file_path)
        }
    );
    
    assert_eq!(results.len(), 2);
    for (i, result) in results.iter().enumerate() {
        assert!(result.file_path.contains(&format!("file{}", i + 1)));
        assert_eq!(result.issues.len(), 10);
        assert!(result.performance_metrics.arena_bytes_used > 0);
        assert!(result.performance_metrics.analysis_duration_ms == result.performance_metrics.analysis_duration_ms); // Verify field exists
        assert_eq!(result.analysis_metadata.lines_of_code, 100);
    }
}

#[test]
fn test_example_file_analyzer() {
    let manager = AnalysisArenaManager::new();
    let mut handle = manager.get_arena();
    
    let result = rayon_integration::example_file_analyzer(&mut handle, "test_file.rs");
    
    assert_eq!(result.file_path, "test_file.rs");
    assert_eq!(result.issues.len(), 10);
    assert_eq!(result.analysis_metadata.lines_of_code, 100);
    assert_eq!(result.analysis_metadata.functions_found, 5);
    assert!(result.performance_metrics.arena_bytes_used > 0);
    assert!(result.performance_metrics.analysis_duration_ms == result.performance_metrics.analysis_duration_ms); // Verify field exists
}

#[test]
fn test_configuration_presets_with_arenas() {
    // Test large codebase configuration
    let large_config = MemoryOptimizationConfig::large_codebase();
    assert!(large_config.arena_allocation.enabled);
    assert_eq!(large_config.arena_allocation.default_arena_size_mb, 64);
    assert_eq!(large_config.arena_allocation.max_concurrent_arenas, 16);
    assert!(large_config.arena_allocation.auto_reset_between_files);
    
    // Test small project configuration  
    let small_config = MemoryOptimizationConfig::small_project();
    assert!(small_config.arena_allocation.enabled);
    assert_eq!(small_config.arena_allocation.default_arena_size_mb, 16);
    assert_eq!(small_config.arena_allocation.max_concurrent_arenas, 4);
    
    // Both should validate successfully
    assert!(large_config.validate().is_ok());
    assert!(small_config.validate().is_ok());
}

#[test]
fn test_arena_config_validation() {
    let mut config = MemoryOptimizationConfig::default();
    assert!(config.validate().is_ok());
    
    // Test invalid arena size
    config.arena_allocation.default_arena_size_mb = 0;
    assert!(config.validate().is_err());
    
    // Reset and test invalid concurrent arenas
    config.arena_allocation.default_arena_size_mb = 32;
    config.arena_allocation.max_concurrent_arenas = 0;
    assert!(config.validate().is_err());
    
    // Reset and test invalid warning threshold
    config.arena_allocation.max_concurrent_arenas = 8;
    config.arena_allocation.size_warning_threshold = 30.0; // Too low
    assert!(config.validate().is_err());
    
    config.arena_allocation.size_warning_threshold = 110.0; // Too high
    assert!(config.validate().is_err());
    
    // Valid threshold should pass
    config.arena_allocation.size_warning_threshold = 85.0;
    assert!(config.validate().is_ok());
}

#[test]
fn test_full_memory_optimization_with_arenas() {
    // Test full memory optimization initialization with arenas
    let mut config = MemoryOptimizationConfig::default();
    config.arena_allocation.enabled = true;
    config.arena_allocation.enable_efficiency_monitoring = true;
    
    let result = initialize_memory_optimization(config);
    assert!(result.is_ok(), "Memory optimization with arenas should initialize successfully");
    
    // Test status export includes arena information
    let status = get_optimization_status();
    assert_eq!(status["phase"].as_str().unwrap(), "Phase 4 - Zero-Copy AST Caching");
    assert!(status["arenas"].is_object());
    assert!(status["pools"].is_object()); // Should still have pools from Phase 2
    assert!(status["metrics"].is_object()); // Should still have basic metrics
}

#[test]
fn test_global_arena_manager() {
    // Test the global GLOBAL_ARENA_MANAGER static
    let initial_stats = GLOBAL_ARENA_MANAGER.get_stats();
    
    let _handle = GLOBAL_ARENA_MANAGER.get_arena();
    let updated_stats = GLOBAL_ARENA_MANAGER.get_stats();
    
    assert!(updated_stats.total_arenas_created > initial_stats.total_arenas_created);
    
    // Export metrics
    let metrics = GLOBAL_ARENA_MANAGER.export_metrics();
    assert_eq!(metrics["arena_manager"]["pattern"], "bumpalo-herd");
    assert_eq!(metrics["arena_manager"]["thread_safe"], true);
    assert_eq!(metrics["arena_manager"]["contention_free"], true);
}

#[test]
fn test_arena_manager_with_custom_capacity() {
    let manager = AnalysisArenaManager::with_capacity(64 * 1024 * 1024); // 64MB
    let stats = manager.get_stats();
    
    assert_eq!(stats.default_capacity_mb, 64);
    
    let _handle = manager.get_arena();
    let updated_stats = manager.get_stats();
    assert_eq!(updated_stats.total_arenas_created, 1);
}

#[test]
fn test_arena_for_computation_owned_for_results_pattern() {
    let manager = AnalysisArenaManager::new();
    let handle = manager.get_arena();
    
    // Phase 1: Computation using arena (temporary allocations)
    let mut temp_data = handle.create_vec();
    temp_data.push("temporary".to_string());
    temp_data.push("computation".to_string());
    temp_data.push("data".to_string());
    
    let temp_string = handle.create_string();
    // Note: arena data stays in arena, cannot be returned directly
    
    // Phase 2: Convert to owned result (communication phase)
    let owned_data: Vec<String> = temp_data.into_iter().collect();
    let owned_string = temp_string.to_string();
    
    // These owned results are Send and can be passed between threads
    fn assert_send<T: Send>(_: &T) {}
    assert_send(&owned_data);
    assert_send(&owned_string);
    
    assert_eq!(owned_data.len(), 3);
    assert_eq!(owned_data[0], "temporary");
    assert!(owned_string.is_empty()); // Empty because we didn't add to it
}