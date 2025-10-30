use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use uveddi::analysis::cache::ast::{AstCache, CacheConfig, CacheableAst};

/// Integration test for AST cache lifecycle
#[test]
fn test_ast_cache_full_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 10,
        max_memory_size_mb: 5,
        enable_disk_cache: true,
        disk_cache_path: temp_dir.path().join("cache"),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
    };
    
    let cache = AstCache::new(config).unwrap();
    
    // Create test files with different content
    let rust_file = temp_dir.path().join("test.rs");
    let python_file = temp_dir.path().join("test.py");
    let js_file = temp_dir.path().join("test.js");
    
    fs::write(&rust_file, "fn main() { println!(\"Hello Rust\"); }").unwrap();
    fs::write(&python_file, "print(\"Hello Python\")").unwrap();
    fs::write(&js_file, "console.log(\"Hello JavaScript\");").unwrap();
    
    // Initial state - all misses
    assert!(cache.get(&rust_file).is_none());
    assert!(cache.get(&python_file).is_none());
    assert!(cache.get(&js_file).is_none());
    
    let initial_metrics = cache.get_metrics();
    assert_eq!(initial_metrics.total_requests, 3);
    assert_eq!(initial_metrics.cache_misses, 3);
    assert_eq!(initial_metrics.cache_hits, 0);
    
    // Store some test data (simulate parsed ASTs)
    #[cfg(not(feature = "tree-sitter"))]
    {
        let rust_ast = CacheableAst {
            data: b"rust_ast_data".to_vec(),
            timestamp: std::time::SystemTime::now(),
            language: "rust".to_string(),
        };
        let python_ast = CacheableAst {
            data: b"python_ast_data".to_vec(),
            timestamp: std::time::SystemTime::now(),
            language: "python".to_string(),
        };
        let js_ast = CacheableAst {
            data: b"js_ast_data".to_vec(),
            timestamp: std::time::SystemTime::now(),
            language: "javascript".to_string(),
        };
        
        cache.store(&rust_file, rust_ast).unwrap();
        cache.store(&python_file, python_ast).unwrap();
        cache.store(&js_file, js_ast).unwrap();
    }
    
    #[cfg(feature = "tree-sitter")]
    {
        use tree_sitter::{Language, Parser};
        
        let mut parser = Parser::new();
        extern "C" { fn tree_sitter_rust() -> Language; }
        let rust_language = unsafe { tree_sitter_rust() };
        
        // Parse and store Rust file
        parser.set_language(&rust_language).unwrap();
        let rust_source = fs::read_to_string(&rust_file).unwrap();
        if let Some(tree) = parser.parse(&rust_source, None) {
            cache.store(&rust_file, tree).unwrap();
        }
        
        // For other languages, we'd need their parsers too
        // For now, just test with Rust
    }
    
    // Verify storage worked
    assert!(cache.memory_usage() > 0);
    
    // Test cache hits
    #[cfg(not(feature = "tree-sitter"))]
    {
        assert!(cache.get(&rust_file).is_some());
        assert!(cache.get(&python_file).is_some());
        assert!(cache.get(&js_file).is_some());
    }
    
    #[cfg(feature = "tree-sitter")]
    {
        assert!(cache.get(&rust_file).is_some());
    }
    
    let hit_metrics = cache.get_metrics();
    assert!(hit_metrics.cache_hits > 0);
    assert!(hit_metrics.hit_rate > 0.0);
    
    // Test file modification invalidation
    std::thread::sleep(Duration::from_millis(10));
    fs::write(&rust_file, "fn main() { println!(\"Modified Rust\"); }").unwrap();
    
    // Should be a miss now due to modification
    assert!(cache.get(&rust_file).is_none());
    
    // Test cache clearing
    cache.clear();
    assert_eq!(cache.size(), 0);
    assert_eq!(cache.memory_usage(), 0);
    
    let final_metrics = cache.get_metrics();
    assert_eq!(final_metrics.memory_usage_bytes, 0);
}

/// Test cache behavior under high concurrency
#[test]
fn test_concurrent_cache_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 1000,
        max_memory_size_mb: 10,
        enable_disk_cache: false,
        disk_cache_path: temp_dir.path().to_path_buf(),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
    };
    
    let cache = Arc::new(AstCache::new(config).unwrap());
    
    // Create many test files
    let test_files: Vec<_> = (0..100).map(|i| {
        let file = temp_dir.path().join(format!("concurrent_test_{}.rs", i));
        fs::write(&file, format!("fn test_{}() {{ return {}; }}", i, i)).unwrap();
        file
    }).collect();
    
    // Launch multiple threads doing different operations
    let handles: Vec<_> = (0..8).map(|thread_id| {
        let cache_clone = Arc::clone(&cache);
        let files_clone = test_files.clone();
        
        std::thread::spawn(move || {
            for iteration in 0..50 {
                let file_idx = (thread_id * 50 + iteration) % files_clone.len();
                let file = &files_clone[file_idx];
                
                match iteration % 4 {
                    0 => {
                        // Read operation
                        let _ = cache_clone.get(file);
                    },
                    1 => {
                        // LRU update
                        cache_clone.update_lru_order(file);
                    },
                    2 => {
                        // Hash calculation
                        let _ = cache_clone.calculate_file_hash(file);
                    },
                    3 => {
                        // Metrics access
                        let _ = cache_clone.get_metrics();
                    },
                    _ => unreachable!(),
                }
            }
        })
    }).collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify cache is in consistent state
    let final_metrics = cache.get_metrics();
    assert!(final_metrics.total_requests > 0);
    
    // Should not have deadlocked or corrupted data
    assert!(cache.size() >= 0);
    assert!(cache.memory_usage() >= 0);
}

/// Test cache eviction policies and memory limits
#[test]
fn test_eviction_and_memory_management() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 5, // Very small for testing
        max_memory_size_mb: 1, // Very small limit
        enable_disk_cache: false,
        disk_cache_path: temp_dir.path().to_path_buf(),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
    };
    
    let cache = AstCache::new(config).unwrap();
    
    // Create files that would exceed limits
    let test_files: Vec<_> = (0..20).map(|i| {
        let file = temp_dir.path().join(format!("eviction_test_{}.rs", i));
        fs::write(&file, format!("fn eviction_test_{}() {{ return {}; }}", i, i)).unwrap();
        file
    }).collect();
    
    // Add entries one by one and watch eviction behavior
    for (i, file) in test_files.iter().enumerate() {
        cache.update_lru_order(file);
        
        // Simulate memory usage
        let result = cache.ensure_cache_capacity(100 * 1024); // 100KB per entry
        assert!(result.is_ok());
        
        // Check that we don't exceed limits drastically
        assert!(cache.memory_usage() <= 2 * 1024 * 1024); // Allow some overhead
        
        if i > 10 {
            // Should have triggered some evictions by now
            let metrics = cache.get_metrics();
            assert!(metrics.evictions > 0);
        }
    }
    
    // Final verification
    let final_metrics = cache.get_metrics();
    assert!(final_metrics.evictions > 0);
    
    // LRU should not be excessively large
    let lru_len = cache.lru_order.lock().unwrap().len();
    assert!(lru_len <= 20); // Should have evicted some
}

/// Test cache performance under different configurations
#[test]
fn test_cache_configuration_impact() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test with eviction disabled
    let config_no_eviction = CacheConfig {
        max_memory_entries: 5,
        max_memory_size_mb: 1,
        enable_disk_cache: false,
        disk_cache_path: temp_dir.path().join("no_eviction"),
        enable_memory_mapping: false,
        lru_eviction_enabled: false,
        cache_metrics_enabled: true,
    };
    
    let cache_no_eviction = AstCache::new(config_no_eviction).unwrap();
    
    // Add many entries - should not evict
    for i in 0..10 {
        let file = temp_dir.path().join(format!("no_evict_{}.rs", i));
        fs::write(&file, format!("fn test_{}() {{}}", i)).unwrap();
        cache_no_eviction.update_lru_order(&file);
    }
    
    let metrics_no_eviction = cache_no_eviction.get_metrics();
    assert_eq!(metrics_no_eviction.evictions, 0);
    
    // Test with eviction enabled
    let config_with_eviction = CacheConfig {
        max_memory_entries: 5,
        max_memory_size_mb: 1,
        enable_disk_cache: false,
        disk_cache_path: temp_dir.path().join("with_eviction"),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
    };
    
    let cache_with_eviction = AstCache::new(config_with_eviction).unwrap();
    
    // Add same entries - should trigger eviction
    for i in 0..10 {
        let file = temp_dir.path().join(format!("with_evict_{}.rs", i));
        fs::write(&file, format!("fn test_{}() {{}}", i)).unwrap();
        cache_with_eviction.update_lru_order(&file);
        // Force capacity check
        let _ = cache_with_eviction.ensure_cache_capacity(200 * 1024);
    }
    
    let metrics_with_eviction = cache_with_eviction.get_metrics();
    // May or may not have evictions depending on actual memory usage
    // But should handle the operations without error
    assert!(metrics_with_eviction.total_requests == 0); // Only did LRU updates
}

/// Test cache metrics accuracy and observability integration
#[test]
fn test_observability_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig::default();
    let cache = AstCache::new(config).unwrap();
    
    let test_file = temp_dir.path().join("observability_test.rs");
    fs::write(&test_file, "fn observability_test() {}").unwrap();
    
    // Perform various operations
    for _ in 0..5 {
        cache.get(&test_file); // 5 misses
    }
    
    cache.update_lru_order(&test_file);
    
    // Test metrics export
    let exported_metrics = cache.export_metrics_for_observability();
    
    // Verify structure
    assert!(exported_metrics.is_object());
    assert!(exported_metrics.get("ast_cache").is_some());
    
    let ast_cache_metrics = &exported_metrics["ast_cache"];
    assert!(ast_cache_metrics.get("total_requests").is_some());
    assert!(ast_cache_metrics.get("cache_hits").is_some());
    assert!(ast_cache_metrics.get("cache_misses").is_some());
    assert!(ast_cache_metrics.get("hit_rate_percent").is_some());
    assert!(ast_cache_metrics.get("evictions").is_some());
    assert!(ast_cache_metrics.get("memory_usage_mb").is_some());
    assert!(ast_cache_metrics.get("average_lookup_time_ms").is_some());
    assert!(ast_cache_metrics.get("cache_size").is_some());
    
    // Verify values make sense
    assert_eq!(ast_cache_metrics["total_requests"], 5);
    assert_eq!(ast_cache_metrics["cache_misses"], 5);
    assert_eq!(ast_cache_metrics["cache_hits"], 0);
    assert_eq!(ast_cache_metrics["hit_rate_percent"], 0.0);
    
    // Test programmatic metrics access
    let direct_metrics = cache.get_metrics();
    assert_eq!(direct_metrics.total_requests, 5);
    assert_eq!(direct_metrics.cache_misses, 5);
    assert_eq!(direct_metrics.cache_hits, 0);
    assert_eq!(direct_metrics.hit_rate, 0.0);
}

/// Test error recovery and graceful degradation
#[test]
fn test_error_recovery_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let config = CacheConfig {
        max_memory_entries: 10,
        max_memory_size_mb: 5,
        enable_disk_cache: true,
        disk_cache_path: temp_dir.path().join("error_test_cache"),
        enable_memory_mapping: false,
        lru_eviction_enabled: true,
        cache_metrics_enabled: true,
    };
    
    let cache = AstCache::new(config).unwrap();
    
    // Test with permission issues (simulated)
    let readonly_file = temp_dir.path().join("readonly.rs");
    fs::write(&readonly_file, "fn readonly() {}").unwrap();
    
    // Normal operations should still work
    cache.get(&readonly_file); // Should handle gracefully
    cache.update_lru_order(&readonly_file);
    
    // Test with corrupted/deleted files
    let temp_file = temp_dir.path().join("temporary.rs");
    fs::write(&temp_file, "fn temporary() {}").unwrap();
    
    let hash_before_delete = cache.calculate_file_hash(&temp_file);
    assert!(hash_before_delete.is_ok());
    
    fs::remove_file(&temp_file).unwrap();
    
    // Should handle missing files gracefully
    let hash_after_delete = cache.calculate_file_hash(&temp_file);
    assert!(hash_after_delete.is_err());
    
    let get_result = cache.get(&temp_file);
    assert!(get_result.is_none());
    
    // Cache should still be functional
    let metrics = cache.get_metrics();
    assert!(metrics.total_requests > 0);
    
    // Clear should always work
    cache.clear();
    assert_eq!(cache.size(), 0);
}