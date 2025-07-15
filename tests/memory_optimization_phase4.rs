//! Integration tests for Phase 4: Zero-copy AST caching with rkyv and memory mapping
//! Tests the complete zero-copy AST serialization and caching system

use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use serde_json::Value;

use uveddi::analysis::memory::{
    MemoryOptimizationConfig,
    zero_copy::{ZeroCopyAstCache, SerializableAst, ZeroCopyCacheStats},
    initialize_memory_optimization,
    get_optimization_status,
};
use uveddi::analysis::cache::ast::{AstCache, CacheConfig};

#[test]
fn test_zero_copy_ast_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    // Create a test serializable AST
    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Test store and load
    cache.store(&file_path, &test_ast).unwrap();
    let loaded_ast = cache.load(&file_path).unwrap();
    
    assert!(loaded_ast.is_some());
    let loaded_ast = loaded_ast.unwrap();
    assert_eq!(loaded_ast.file_path, "test.rs");
    assert_eq!(loaded_ast.source_hash, 12345);
    assert_eq!(loaded_ast.language, "rust");
}

#[test]
fn test_zero_copy_cache_stats() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    // Initially no stats
    let stats = cache.get_stats();
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.total_stores, 0);

    // Create test AST
    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Store AST
    cache.store(&file_path, &test_ast).unwrap();

    // Check stats after store
    let stats = cache.get_stats();
    assert_eq!(stats.total_stores, 1);
    assert!(stats.total_bytes_stored > 0);

    // Load AST (should be a hit)
    let loaded_ast = cache.load(&file_path).unwrap();
    assert!(loaded_ast.is_some());

    // Check stats after hit
    let stats = cache.get_stats();
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.hit_rate_percentage(), 100.0);

    // Try to load non-existent file (should be a miss)
    let fake_file = temp_dir.path().join("nonexistent.rs");
    let result = cache.load(&fake_file).unwrap();
    assert!(result.is_none());

    // Check stats after miss
    let stats = cache.get_stats();
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.cache_misses, 1);
    assert_eq!(stats.hit_rate_percentage(), 50.0);
}

#[test]
fn test_zero_copy_cache_validation() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Store AST
    cache.store(&file_path, &test_ast).unwrap();

    // Check validity with correct hash
    assert!(cache.is_valid(&file_path, 12345));

    // Check validity with wrong hash
    assert!(!cache.is_valid(&file_path, 54321));
}

#[test]
fn test_zero_copy_cache_removal() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Store AST
    cache.store(&file_path, &test_ast).unwrap();

    // Verify it exists
    assert!(cache.load(&file_path).unwrap().is_some());

    // Remove it
    cache.remove(&file_path).unwrap();

    // Verify it's gone
    assert!(cache.load(&file_path).unwrap().is_none());
}

#[test]
fn test_zero_copy_cache_clear() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Store AST
    cache.store(&file_path, &test_ast).unwrap();

    // Verify it exists
    assert!(cache.load(&file_path).unwrap().is_some());

    // Clear cache
    cache.clear().unwrap();

    // Verify it's gone
    assert!(cache.load(&file_path).unwrap().is_none());

    // Stats should be reset
    let stats = cache.get_stats();
    assert_eq!(stats.total_stores, 0);
    assert_eq!(stats.total_bytes_stored, 0);
}

#[test]
fn test_zero_copy_metrics_export() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();

    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024,
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some("fn main() {}".to_string()),
    };

    let file_path = temp_dir.path().join("test.rs");

    // Store and load to generate metrics
    cache.store(&file_path, &test_ast).unwrap();
    cache.load(&file_path).unwrap();

    let metrics = cache.export_metrics();
    assert!(metrics["zero_copy_ast_cache"].is_object());
    
    let cache_metrics = &metrics["zero_copy_ast_cache"];
    assert!(cache_metrics["cache_hits"].is_number());
    assert!(cache_metrics["cache_misses"].is_number());
    assert!(cache_metrics["hit_rate_percent"].is_number());
    assert!(cache_metrics["total_stores"].is_number());
    assert!(cache_metrics["total_bytes_stored"].is_number());
    assert!(cache_metrics["average_entry_size_kb"].is_number());
    assert!(cache_metrics["active_memory_maps"].is_number());
    assert!(cache_metrics["cache_directory"].is_string());
    
    // Check actual values
    assert_eq!(cache_metrics["cache_hits"], 1);
    assert_eq!(cache_metrics["cache_misses"], 0);
    assert_eq!(cache_metrics["total_stores"], 1);
    assert!(cache_metrics["total_bytes_stored"].as_u64().unwrap() > 0);
    assert_eq!(cache_metrics["hit_rate_percent"], 100.0);
}

#[test]
fn test_ast_cache_zero_copy_integration() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create cache config with zero-copy enabled
    let mut cache_config = CacheConfig::default();
    cache_config.enable_zero_copy = true;
    cache_config.zero_copy_cache_dir = temp_dir.path().join("zero_copy");
    cache_config.zero_copy_threshold_bytes = 100; // Low threshold for testing
    
    let ast_cache = AstCache::new(cache_config).unwrap();
    
    // Test that the cache was created successfully
    assert!(ast_cache.size() == 0);
    
    // Test metrics export includes zero-copy information
    let metrics = ast_cache.export_metrics_for_observability();
    assert!(metrics["ast_cache"]["zero_copy_cache"].is_object());
    
    let zero_copy_metrics = &metrics["ast_cache"]["zero_copy_cache"];
    
    // Zero-copy cache metrics are exported directly as an object
    assert!(zero_copy_metrics["cache_hits"].is_number());
    assert!(zero_copy_metrics["cache_misses"].is_number());
    assert!(zero_copy_metrics["hit_rate_percent"].is_number());
    assert!(zero_copy_metrics["total_stores"].is_number());
    assert!(zero_copy_metrics["total_bytes_stored"].is_number());
    assert!(zero_copy_metrics["average_entry_size_kb"].is_number());
    assert!(zero_copy_metrics["active_memory_maps"].is_number());
    assert!(zero_copy_metrics["cache_directory"].is_string());
}

#[test]
fn test_ast_cache_config_serialization() {
    let config = CacheConfig::default();
    
    // Test that config can be serialized and deserialized
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: CacheConfig = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(config.max_memory_entries, deserialized.max_memory_entries);
    assert_eq!(config.max_memory_size_mb, deserialized.max_memory_size_mb);
    
    #[cfg(feature = "memory-optimization")]
    {
        assert_eq!(config.enable_zero_copy, deserialized.enable_zero_copy);
        assert_eq!(config.zero_copy_cache_dir, deserialized.zero_copy_cache_dir);
        assert_eq!(config.zero_copy_threshold_bytes, deserialized.zero_copy_threshold_bytes);
    }
}

#[test]
fn test_serializable_ast_efficiency_score() {
    let test_ast = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: 12345,
        language: "rust".to_string(),
        file_size_bytes: 1024 * 1024, // 1MB
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024 * 1024,
            start_line: 0,
            start_column: 0,
            end_line: 1000,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 5000,
        max_depth: 25,
        source_text: None,
    };

    let score = test_ast.cache_efficiency_score();
    assert!(score >= 0.0 && score <= 1.0);
    
    // Test with small file
    let small_ast = SerializableAst {
        file_size_bytes: 100,
        total_nodes: 10,
        max_depth: 3,
        ..test_ast
    };
    
    let small_score = small_ast.cache_efficiency_score();
    assert!(small_score >= 0.0 && small_score <= 1.0);
    assert!(small_score < score); // Smaller files should have lower efficiency scores
}

#[test]
fn test_source_hash_consistency() {
    let source1 = "fn main() { println!(\"Hello, world!\"); }";
    let source2 = "fn main() { println!(\"Hello, world!\"); }";
    let source3 = "fn main() { println!(\"Hello, Rust!\"); }";
    
    let hash1 = SerializableAst::calculate_source_hash(source1);
    let hash2 = SerializableAst::calculate_source_hash(source2);
    let hash3 = SerializableAst::calculate_source_hash(source3);
    
    assert_eq!(hash1, hash2); // Same source should have same hash
    assert_ne!(hash1, hash3); // Different source should have different hash
    
    // Test with serializable AST
    let ast1 = SerializableAst {
        file_path: "test.rs".to_string(),
        source_hash: hash1,
        language: "rust".to_string(),
        file_size_bytes: source1.len(),
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: source1.len(),
            start_line: 0,
            start_column: 0,
            end_line: 1,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 1,
        max_depth: 1,
        source_text: Some(source1.to_string()),
    };
    
    assert!(ast1.is_valid_for_source(source1));
    assert!(ast1.is_valid_for_source(source2));
    assert!(!ast1.is_valid_for_source(source3));
}

#[test]
fn test_concurrent_zero_copy_access() {
    let temp_dir = TempDir::new().unwrap();
    let cache = std::sync::Arc::new(ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap());
    
    // Create multiple test ASTs
    let test_files: Vec<_> = (0..10)
        .map(|i| {
            let ast = SerializableAst {
                file_path: format!("test_{}.rs", i),
                source_hash: 12345 + i as u64,
                language: "rust".to_string(),
                file_size_bytes: 1024,
                parse_timestamp: 1000000,
                root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
                    node_type: "source_file".to_string(),
                    kind_id: 1,
                    start_byte: 0,
                    end_byte: 1024,
                    start_line: 0,
                    start_column: 0,
                    end_line: 10,
                    end_column: 0,
                    children: vec![],
                    named_children_count: 0,
                    is_named: true,
                    is_missing: false,
                    is_extra: false,
                    text: None,
                },
                total_nodes: 1,
                max_depth: 1,
                source_text: Some(format!("fn test_{}() {{}}", i)),
            };
            
            let file_path = temp_dir.path().join(format!("test_{}.rs", i));
            (file_path, ast)
        })
        .collect();
    
    // Store all ASTs concurrently
    let handles: Vec<_> = test_files.iter().map(|(path, ast)| {
        let cache_clone = std::sync::Arc::clone(&cache);
        let path_clone = path.clone();
        let ast_clone = ast.clone();
        
        std::thread::spawn(move || {
            cache_clone.store(&path_clone, &ast_clone).unwrap();
        })
    }).collect();
    
    // Wait for all stores to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Load all ASTs concurrently
    let handles: Vec<_> = test_files.iter().map(|(path, _)| {
        let cache_clone = std::sync::Arc::clone(&cache);
        let path_clone = path.clone();
        
        std::thread::spawn(move || {
            let result = cache_clone.load(&path_clone).unwrap();
            assert!(result.is_some());
        })
    }).collect();
    
    // Wait for all loads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify stats
    let stats = cache.get_stats();
    assert_eq!(stats.total_stores, 10);
    assert_eq!(stats.cache_hits, 10);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.hit_rate_percentage(), 100.0);
}

#[test]
fn test_phase4_memory_optimization_initialization() {
    let config = MemoryOptimizationConfig {
        ast_cache_optimization: uveddi::analysis::memory::config::AstCacheOptimizationConfig {
            zero_copy_enabled: true,
            zero_copy_cache_directory: "./cache/test_zero_copy".to_string(),
            zero_copy_threshold_bytes: 1024,
            max_cached_asts: 1000,
            max_ast_cache_memory_mb: 100,
            enable_ast_cache_metrics: true,
            enable_lru_eviction: true,
            enable_concurrent_access: true,
            enable_cache_warming: false,
            cache_warming_concurrency: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let result = initialize_memory_optimization(config);
    assert!(result.is_ok(), "Memory optimization with zero-copy should initialize successfully");
    
    // Test status export includes AST cache optimization information
    let status = get_optimization_status();
    assert_eq!(status["phase"].as_str().unwrap(), "Phase 4 - Zero-Copy AST Caching");
    assert!(status["pools"].is_object());
    assert!(status["arenas"].is_object());
}

#[test]
fn test_memory_optimization_config_validation() {
    let config = MemoryOptimizationConfig::default();
    
    // Should validate successfully
    assert!(config.validate().is_ok());
    
    // Test with invalid config
    let mut invalid_config = config.clone();
    invalid_config.target_max_memory_bytes = 0;
    
    assert!(invalid_config.validate().is_err());
    
    // Test AST cache config
    assert!(config.ast_cache_optimization.zero_copy_enabled);
    assert!(config.ast_cache_optimization.max_cached_asts > 0);
    assert!(config.ast_cache_optimization.max_ast_cache_memory_mb > 0);
    assert!(config.ast_cache_optimization.zero_copy_threshold_bytes > 0);
}

#[test]
fn test_large_ast_zero_copy_performance() {
    let temp_dir = TempDir::new().unwrap();
    let cache = ZeroCopyAstCache::new(temp_dir.path().to_path_buf()).unwrap();
    
    // Create a large AST with many nodes
    let mut large_ast = SerializableAst {
        file_path: "large_test.rs".to_string(),
        source_hash: 99999,
        language: "rust".to_string(),
        file_size_bytes: 1024 * 1024, // 1MB
        parse_timestamp: 1000000,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: "source_file".to_string(),
            kind_id: 1,
            start_byte: 0,
            end_byte: 1024 * 1024,
            start_line: 0,
            start_column: 0,
            end_line: 1000,
            end_column: 0,
            children: vec![],
            named_children_count: 0,
            is_named: true,
            is_missing: false,
            is_extra: false,
            text: None,
        },
        total_nodes: 10000,
        max_depth: 50,
        source_text: None, // Too large for inline storage
    };
    
    // Add many child nodes
    for i in 0..100 {
        large_ast.root_node.children.push(
            uveddi::analysis::memory::zero_copy::SerializableNode {
                node_type: format!("function_{}", i),
                kind_id: 2,
                start_byte: i * 1024,
                end_byte: (i + 1) * 1024,
                start_line: i as u32,
                start_column: 0,
                end_line: i as u32,
                end_column: 50,
                children: vec![],
                named_children_count: 0,
                is_named: true,
                is_missing: false,
                is_extra: false,
                text: Some(format!("fn function_{}() {{}}", i)),
            }
        );
    }
    
    let file_path = temp_dir.path().join("large_test.rs");
    
    // Measure store time
    let start = std::time::Instant::now();
    cache.store(&file_path, &large_ast).unwrap();
    let store_time = start.elapsed();
    
    // Measure load time
    let start = std::time::Instant::now();
    let loaded_ast = cache.load(&file_path).unwrap();
    let load_time = start.elapsed();
    
    assert!(loaded_ast.is_some());
    let loaded_ast = loaded_ast.unwrap();
    
    // Verify the loaded AST matches
    assert_eq!(loaded_ast.file_path, "large_test.rs");
    assert_eq!(loaded_ast.source_hash, 99999);
    assert_eq!(loaded_ast.total_nodes, 10000);
    assert_eq!(loaded_ast.max_depth, 50);
    
    // Performance should be reasonable (less than 1 second each)
    assert!(store_time.as_secs() < 1, "Store time should be less than 1 second");
    assert!(load_time.as_millis() < 100, "Load time should be less than 100ms (zero-copy)");
    
    // Verify cache efficiency score
    let efficiency_score = SerializableAst {
        file_path: loaded_ast.file_path.clone(),
        source_hash: loaded_ast.source_hash,
        language: loaded_ast.language.clone(),
        file_size_bytes: loaded_ast.file_size_bytes,
        parse_timestamp: loaded_ast.parse_timestamp,
        root_node: uveddi::analysis::memory::zero_copy::SerializableNode {
            node_type: loaded_ast.root_node.node_type.clone(),
            kind_id: loaded_ast.root_node.kind_id,
            start_byte: loaded_ast.root_node.start_byte,
            end_byte: loaded_ast.root_node.end_byte,
            start_line: loaded_ast.root_node.start_line,
            start_column: loaded_ast.root_node.start_column,
            end_line: loaded_ast.root_node.end_line,
            end_column: loaded_ast.root_node.end_column,
            children: vec![], // Simplified for test
            named_children_count: loaded_ast.root_node.named_children_count,
            is_named: loaded_ast.root_node.is_named,
            is_missing: loaded_ast.root_node.is_missing,
            is_extra: loaded_ast.root_node.is_extra,
            text: None,
        },
        total_nodes: loaded_ast.total_nodes,
        max_depth: loaded_ast.max_depth,
        source_text: None,
    }.cache_efficiency_score();
    
    assert!(efficiency_score > 0.5, "Large AST should have high efficiency score");
}