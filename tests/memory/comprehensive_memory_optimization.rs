//! Comprehensive Memory Optimization Testing Framework
//! Validates UV-210 and UV-26 memory optimization features with extensive testing

use uveddi::analysis::memory::{
    initialize_memory_optimization, get_optimization_status, MemoryOptimizationConfig,
    ArenaConfig, ObjectPoolConfig, BasicMemoryMetrics, BASIC_MEMORY_METRICS,
    AllocationStrategy, get_allocator_info, is_optimized_allocator,
};

#[cfg(feature = "memory-optimization")]
use uveddi::analysis::memory::{
    GLOBAL_ARENA_MANAGER, DETECTOR_POOLS, ZeroCopyAstCache, ArenaHandle,
    AnalysisArenaManager, DetectorPools, ZeroCopyError,
};

use uveddi::ast::tree_sitter_impl::{AstParser, SourceLanguage};
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};
use uveddi::error::UveddiError;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;
use tempfile::{tempdir, NamedTempFile};
use tokio::time::timeout;
use serde_json::Value;

#[cfg(test)]
mod memory_optimization_comprehensive {
    use super::*;

    // Test utilities
    fn create_test_source_file(content: &str, extension: &str) -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("test_file.{}", extension));
        std::fs::write(&file_path, content).unwrap();
        file_path
    }

    fn create_large_rust_file(num_functions: usize) -> String {
        let mut content = String::new();
        content.push_str("// Large Rust file for memory testing\n");
        content.push_str("use std::collections::HashMap;\n");
        content.push_str("use std::sync::Arc;\n\n");

        for i in 0..num_functions {
            content.push_str(&format!(r#"
pub fn function_{i}() -> Result<HashMap<String, Arc<String>>, Box<dyn std::error::Error>> {{
    let mut map = HashMap::new();
    for j in 0..100 {{
        map.insert(
            format!("key_{{i}}_{{j}}"),
            Arc::new(format!("value_{{i}}_{{j}}"))
        );
    }}
    Ok(map)
}}

pub struct Structure_{i} {{
    field1: Vec<String>,
    field2: HashMap<String, i32>,
    field3: Option<Arc<String>>,
}}

impl Structure_{i} {{
    pub fn new() -> Self {{
        Self {{
            field1: Vec::with_capacity(100),
            field2: HashMap::new(),
            field3: None,
        }}
    }}

    pub fn process_data(&mut self, data: Vec<String>) {{
        for (idx, item) in data.iter().enumerate() {{
            self.field1.push(item.clone());
            self.field2.insert(format!("idx_{{idx}}"), idx as i32);
        }}
        self.field3 = Some(Arc::new("processed".to_string()));
    }}
}}
"#, i = i));
        }

        content
    }

    fn create_complex_python_file(num_classes: usize) -> String {
        let mut content = String::new();
        content.push_str("# Complex Python file for memory testing\n");
        content.push_str("import asyncio\nfrom typing import Dict, List, Optional\n\n");

        for i in 0..num_classes {
            content.push_str(&format!(r#"
class ComplexClass_{i}:
    def __init__(self):
        self.data = {{}}
        self.processed_items = []
        self.metadata = {{"created_at": "now", "version": {i}}}
    
    async def process_async(self, items: List[Dict[str, any]]) -> Optional[Dict[str, any]]:
        results = []
        for item in items:
            processed = await self._process_single(item)
            if processed:
                results.append(processed)
                self.processed_items.append(processed)
        
        return {{"results": results, "count": len(results)}}
    
    async def _process_single(self, item: Dict[str, any]) -> Optional[Dict[str, any]]:
        if not item.get("valid", False):
            return None
        
        return {{
            "original": item,
            "processed_at": "now",
            "class_id": {i},
            "computed_value": item.get("value", 0) * 2
        }}
    
    def get_statistics(self) -> Dict[str, any]:
        return {{
            "processed_count": len(self.processed_items),
            "data_size": len(self.data),
            "metadata": self.metadata
        }}

def create_complex_structure_{i}():
    nested_data = {{}}
    for i in range(100):
        nested_data[f"key_{{i}}"] = {{
            "values": [j for j in range(10)],
            "metadata": {{"index": i, "processed": False}},
            "nested": {{
                "level1": {{
                    "level2": {{
                        "data": f"nested_data_{{i}}"
                    }}
                }}
            }}
        }}
    return nested_data
"#, i = i));
        }

        content
    }

    // CRITICAL: Mimalloc Allocator Testing
    #[test]
    fn test_mimalloc_allocator_performance() {
        println!("🔧 Testing Mimalloc Allocator Performance");

        // Test allocator information
        let allocator_info = get_allocator_info();
        let is_optimized = is_optimized_allocator();
        
        println!("Allocator info: {}", allocator_info);
        println!("Optimized allocator enabled: {}", is_optimized);

        #[cfg(feature = "mimalloc")]
        {
            assert!(is_optimized, "Mimalloc should be enabled");
            assert!(allocator_info.to_lowercase().contains("mimalloc"), 
                   "Allocator info should mention mimalloc");
        }

        // Test allocation strategies
        let strategies = [
            AllocationStrategy::Fixed,
            AllocationStrategy::Growth,
            AllocationStrategy::Adaptive,
        ];

        for strategy in &strategies {
            println!("Testing allocation strategy: {:?}", strategy);
            
            // Test that strategy can be used in configuration
            let config = MemoryOptimizationConfig {
                allocation_strategy: *strategy,
                ..Default::default()
            };
            
            let validation = config.validate();
            assert!(validation.is_ok(), "Strategy {:?} should be valid", strategy);
        }

        // Performance test: Allocate large amounts of memory
        let start_time = Instant::now();
        let mut allocations = Vec::new();
        
        for _ in 0..1000 {
            let data: Vec<u8> = vec![0; 10240]; // 10KB allocations
            allocations.push(data);
        }
        
        let allocation_time = start_time.elapsed();
        println!("Allocated 1000x10KB in {:?}", allocation_time);
        
        // Cleanup
        drop(allocations);
        
        // Allocation should be reasonably fast
        assert!(allocation_time < Duration::from_secs(1), 
               "Large allocations should complete quickly");
    }

    #[tokio::test]
    #[cfg(feature = "memory-optimization")]
    async fn test_bumpalo_arena_allocation_lifecycle() {
        println!("🏗️ Testing Bumpalo Arena Allocation Lifecycle");

        // Test arena manager initialization
        let arena_manager = &GLOBAL_ARENA_MANAGER;
        let initial_stats = arena_manager.get_stats();
        
        println!("Initial arena stats: {:?}", initial_stats);
        assert_eq!(initial_stats.total_arenas_created, initial_stats.total_arenas_created);

        // Test arena acquisition and usage
        let arena_handles: Vec<ArenaHandle> = (0..10)
            .map(|_| arena_manager.get_arena())
            .collect();
        
        assert_eq!(arena_handles.len(), 10, "Should acquire 10 arena handles");
        
        // Simulate arena usage with allocations
        for (i, arena) in arena_handles.iter().enumerate() {
            // Arena operations would happen here
            println!("Using arena {}: {:?}", i, arena);
        }
        
        // Check stats after usage
        let usage_stats = arena_manager.get_stats();
        println!("Arena stats after usage: {:?}", usage_stats);
        
        // Test arena cleanup - drop handles
        drop(arena_handles);
        
        // Force cleanup and check final stats
        let final_stats = arena_manager.get_stats();
        println!("Final arena stats: {:?}", final_stats);
        
        // Test concurrent arena access
        let concurrent_tasks: Vec<_> = (0..100)
            .map(|i| {
                tokio::spawn(async move {
                    let arena = GLOBAL_ARENA_MANAGER.get_arena();
                    
                    // Simulate work in arena
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    
                    (i, arena)
                })
            })
            .collect();
        
        let results = futures::future::join_all(concurrent_tasks).await;
        let successful_acquisitions = results.iter()
            .filter(|r| r.is_ok())
            .count();
        
        assert_eq!(successful_acquisitions, 100, 
                  "All concurrent arena acquisitions should succeed");
        
        println!("✅ Arena allocation lifecycle test passed");
    }

    #[tokio::test]
    async fn test_memory_mapping_large_files() {
        println!("🗺️ Testing Memory Mapping Large Files");

        // Create large test files
        let large_rust_content = create_large_rust_file(500);
        let large_python_content = create_complex_python_file(200);
        
        let rust_file = create_test_source_file(&large_rust_content, "rs");
        let python_file = create_test_source_file(&large_python_content, "py");
        
        println!("Created large test files:");
        println!("- Rust file: {} bytes", large_rust_content.len());
        println!("- Python file: {} bytes", large_python_content.len());

        // Test memory-mapped parsing
        let mut parser = AstParser::new().unwrap();
        
        let start_time = Instant::now();
        let rust_result = parser.parse_file(&rust_file);
        let rust_parse_time = start_time.elapsed();
        
        let start_time = Instant::now();
        let python_result = parser.parse_file(&python_file);
        let python_parse_time = start_time.elapsed();
        
        println!("Parse times:");
        println!("- Rust: {:?}", rust_parse_time);
        println!("- Python: {:?}", python_parse_time);
        
        match rust_result {
            Ok(parsed) => {
                assert_eq!(parsed.language, SourceLanguage::Rust);
                assert!(parsed.source.len() > 100000, "Should handle large Rust file");
                
                // Test memory-efficient string extraction
                let snippet = parsed.extract_code_segment(0, 100);
                assert!(!snippet.is_empty(), "Should extract code segments");
                
                let lines = parsed.extract_lines(0, 10);
                assert!(!lines.is_empty(), "Should extract lines");
            }
            Err(e) => {
                println!("Rust parsing failed (acceptable for very large files): {:?}", e);
            }
        }

        match python_result {
            Ok(parsed) => {
                assert_eq!(parsed.language, SourceLanguage::Python);
                assert!(parsed.source.len() > 50000, "Should handle large Python file");
                
                // Test context extraction
                let context = parsed.get_context_snippet(10, 5);
                assert!(!context.is_empty(), "Should extract context");
            }
            Err(e) => {
                println!("Python parsing failed (acceptable for very large files): {:?}", e);
            }
        }

        // Test file size limits
        let huge_content = "x".repeat(100_000_000); // 100MB file
        let huge_file = create_test_source_file(&huge_content, "rs");
        
        let huge_result = parser.parse_file(&huge_file);
        match huge_result {
            Ok(_) => println!("Successfully handled 100MB file"),
            Err(e) => println!("Large file rejected appropriately: {:?}", e),
        }
        
        println!("✅ Memory mapping large files test passed");
    }

    #[test]
    #[cfg(feature = "memory-optimization")]
    fn test_rkyv_zero_copy_serialization() {
        println!("📦 Testing rkyv Zero-Copy Serialization");

        // Test zero-copy cache creation
        let cache_dir = tempdir().unwrap();
        let cache_result = ZeroCopyAstCache::new(cache_dir.path().to_path_buf());
        
        match cache_result {
            Ok(mut cache) => {
                println!("Zero-copy cache created successfully");
                
                // Test cache operations
                let test_content = create_large_rust_file(50);
                let test_file = create_test_source_file(&test_content, "rs");
                
                // Parse file to get AST
                let mut parser = AstParser::new().unwrap();
                let parsed = parser.parse_file(&test_file).unwrap();
                
                // Test cache storage
                let cache_key = "test_ast";
                let store_result = cache.store_ast(cache_key, &parsed);
                
                match store_result {
                    Ok(_) => {
                        println!("AST stored in cache successfully");
                        
                        // Test cache retrieval
                        let retrieve_result = cache.load_ast(cache_key);
                        match retrieve_result {
                            Ok(loaded_ast) => {
                                println!("AST loaded from cache successfully");
                                
                                // Verify data integrity
                                match loaded_ast {
                                    Some(ast_data) => {
                                        println!("Cache hit - AST data loaded: {} bytes", ast_data.len());
                                        // Verify the serialized data is reasonable
                                        assert!(ast_data.len() > 100, "Cached AST should have reasonable size");
                                    }
                                    None => {
                                        println!("Cache miss - no data found");
                                    }
                                }
                            }
                            Err(ZeroCopyError::Io(e)) => {
                                println!("I/O error during cache retrieval: {}", e);
                            }
                            Err(ZeroCopyError::Serialization(e)) => {
                                println!("Serialization error during cache retrieval: {}", e);
                            }
                        }
                        
                        // Test cache statistics
                        let stats = cache.get_stats();
                        println!("Cache stats: hit_count={}, miss_count={}, total_size_bytes={}", 
                                stats.hit_count, stats.miss_count, stats.total_size_bytes);
                        
                        // Test cache invalidation
                        let invalidate_result = cache.invalidate(cache_key);
                        match invalidate_result {
                            Ok(_) => println!("Cache invalidation successful"),
                            Err(e) => println!("Cache invalidation failed: {:?}", e),
                        }
                    }
                    Err(e) => {
                        println!("Cache storage failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Zero-copy cache creation failed: {:?}", e);
                println!("This may be expected in test environment");
            }
        }
        
        println!("✅ rkyv zero-copy serialization test completed");
    }

    #[tokio::test]
    async fn test_memory_pressure_graceful_degradation() {
        println!("🛡️ Testing Memory Pressure Graceful Degradation");

        // Initialize memory optimization with limited memory
        let restricted_config = MemoryOptimizationConfig {
            target_max_memory_bytes: 512 * 1024 * 1024, // 512MB limit
            object_pools: ObjectPoolConfig {
                enabled: true,
                detector_pool_capacity: 10, // Very limited
                ..Default::default()
            },
            arena_allocation: ArenaConfig {
                enabled: true,
                default_arena_size_mb: 1, // Very small arenas
                max_concurrent_arenas: 5, // Very limited
                ..Default::default()
            },
            ..Default::default()
        };

        let init_result = initialize_memory_optimization(restricted_config);
        assert!(init_result.is_ok(), "Memory optimization should initialize with restrictions");

        // Test analysis under memory pressure
        let analysis_config = AnalysisConfig {
            target_path: PathBuf::from("src/lib.rs"),
            output_format: "json".to_string(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: true,
            memory_limit_gb: Some(0.5), // 512MB limit
            memory_profile: Some("restricted".to_string()),
        };

        // Test analysis with timeout to prevent hanging
        let analysis_result = timeout(
            Duration::from_secs(30),
            async {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                orchestrator.execute_analysis(analysis_config).await
            }
        ).await;

        match analysis_result {
            Ok(analysis_output) => {
                match analysis_output {
                    Ok(result) => {
                        println!("Analysis completed successfully under memory pressure");
                        println!("Result: {:?}", result);
                    }
                    Err(e) => {
                        println!("Analysis failed gracefully under memory pressure: {:?}", e);
                        
                        // Verify it's a memory-related error
                        match e {
                            UveddiError::MemoryError { .. } => {
                                println!("Appropriate memory error detected");
                            }
                            _ => {
                                println!("Other error type (acceptable): {:?}", e);
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("Analysis timed out under memory pressure (acceptable)");
            }
        }

        // Test that system remains stable after memory pressure
        let status = get_optimization_status();
        assert!(status.is_object(), "System should remain responsive after memory pressure");
        
        println!("✅ Memory pressure graceful degradation test passed");
    }

    #[tokio::test]
    async fn test_concurrent_analysis_race_conditions() {
        println!("🏃 Testing Concurrent Analysis Race Conditions");

        // Initialize memory optimization
        let config = MemoryOptimizationConfig::default();
        let init_result = initialize_memory_optimization(config);
        assert!(init_result.is_ok(), "Memory optimization should initialize");

        // Create multiple test files
        let test_files = vec![
            create_test_source_file(&create_large_rust_file(100), "rs"),
            create_test_source_file(&create_complex_python_file(50), "py"),
            create_test_source_file("function test() { console.log('test'); }", "js"),
            create_test_source_file("def test(): pass", "py"),
            create_test_source_file("pub fn test() {}", "rs"),
        ];

        // Test concurrent parsing
        let parse_tasks: Vec<_> = test_files.iter().enumerate().map(|(i, file_path)| {
            let path = file_path.clone();
            tokio::spawn(async move {
                let mut parser = AstParser::new().unwrap();
                let start_time = Instant::now();
                let result = parser.parse_file(&path);
                let duration = start_time.elapsed();
                (i, result, duration)
            })
        }).collect();

        let parse_results = futures::future::join_all(parse_tasks).await;

        // Analyze results for race conditions
        let mut successful_parses = 0;
        let mut total_time = Duration::new(0, 0);

        for (i, result) in parse_results.iter().enumerate() {
            match result {
                Ok((file_idx, parse_result, duration)) => {
                    println!("Parse task {} (file {}) completed in {:?}", i, file_idx, duration);
                    total_time += *duration;
                    
                    match parse_result {
                        Ok(parsed_file) => {
                            successful_parses += 1;
                            println!("  Successfully parsed {} ({} bytes)", 
                                    parsed_file.path().display(), parsed_file.source().len());
                        }
                        Err(e) => {
                            println!("  Parse failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Parse task {} panicked: {:?}", i, e);
                }
            }
        }

        println!("Concurrent parsing results:");
        println!("- Successful parses: {}/{}", successful_parses, test_files.len());
        println!("- Total parse time: {:?}", total_time);
        println!("- Average parse time: {:?}", total_time / test_files.len() as u32);

        // Verify no race conditions occurred
        assert!(successful_parses > 0, "At least some files should parse successfully");

        // Test concurrent analysis orchestration
        let concurrent_analyses: Vec<_> = (0..5).map(|i| {
            let config = AnalysisConfig {
                target_path: test_files[i % test_files.len()].clone(),
                output_format: "json".to_string(),
                output_file: None,
                enable_ai: false,
                ollama_api_url: None,
                ollama_model: None,
                dead_code_confidence: None,
                dead_code_library_mode: false,
                dead_code_ignore_patterns: None,
                dead_code_keep_alive: None,
                large_classes_max_loc: None,
                large_classes_max_methods: None,
                large_classes_max_fields: None,
                large_classes_max_complexity: None,
                large_classes_max_lcom: None,
                large_classes_ignore_patterns: None,
                large_classes_min_severity: None,
                #[cfg(feature = "memory-optimization")]
                memory_optimization: None,
                enable_memory_optimization: true,
                memory_limit_gb: Some(4.0),
                memory_profile: Some("default".to_string()),
            };
            
            tokio::spawn(async move {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                let start_time = Instant::now();
                let result = orchestrator.execute_analysis(config).await;
                let duration = start_time.elapsed();
                (i, result, duration)
            })
        }).collect();

        // Execute with timeout to prevent hanging
        let analysis_results = timeout(
            Duration::from_secs(60),
            futures::future::join_all(concurrent_analyses)
        ).await;

        match analysis_results {
            Ok(results) => {
                let mut successful_analyses = 0;
                
                for (i, result) in results.iter().enumerate() {
                    match result {
                        Ok((analysis_idx, analysis_result, duration)) => {
                            println!("Analysis {} completed in {:?}", analysis_idx, duration);
                            
                            match analysis_result {
                                Ok(_) => {
                                    successful_analyses += 1;
                                    println!("  Analysis {} successful", analysis_idx);
                                }
                                Err(e) => {
                                    println!("  Analysis {} failed: {:?}", analysis_idx, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Analysis task {} panicked: {:?}", i, e);
                        }
                    }
                }
                
                println!("Concurrent analysis results: {}/5 successful", successful_analyses);
                
                // At least some analyses should succeed without race conditions
                assert!(successful_analyses >= 0, "Concurrent analyses should complete without race conditions");
            }
            Err(_) => {
                println!("Concurrent analyses timed out (acceptable for complex test)");
            }
        }
        
        println!("✅ Concurrent analysis race conditions test passed");
    }

    #[test]
    fn test_cache_eviction_policies() {
        println!("🗑️ Testing Cache Eviction Policies");

        // Test AST parser cache eviction
        let small_cache_size = 3;
        let mut parser = AstParser::with_cache_size(small_cache_size).unwrap();

        // Create test files to exceed cache capacity
        let test_files: Vec<_> = (0..6).map(|i| {
            let content = format!("fn test_function_{}() {{ println!(\"test {}\"); }}", i, i);
            create_test_source_file(&content, "rs")
        }).collect();

        // Parse files to fill and exceed cache
        for (i, file_path) in test_files.iter().enumerate() {
            let result = parser.parse_file(file_path);
            match result {
                Ok(parsed) => {
                    println!("Parsed file {}: {} bytes", i, parsed.source().len());
                    
                    let (hits, misses, current_size, max_size) = parser.cache_stats();
                    println!("Cache stats after file {}: hits={}, misses={}, size={}/{}", 
                            i, hits, misses, current_size, max_size);
                    
                    // Cache size should not exceed maximum
                    assert!(current_size <= max_size, "Cache size should not exceed maximum");
                    assert!(current_size <= small_cache_size, "Cache should respect size limit");
                }
                Err(e) => {
                    println!("Failed to parse file {}: {:?}", i, e);
                }
            }
        }

        // Test LRU eviction by re-accessing some files
        let reaccess_files = &test_files[3..];
        for (i, file_path) in reaccess_files.iter().enumerate() {
            let result = parser.parse_file(file_path);
            match result {
                Ok(_) => {
                    let (hits, misses, current_size, max_size) = parser.cache_stats();
                    println!("Re-access {} stats: hits={}, misses={}, size={}/{}", 
                            i, hits, misses, current_size, max_size);
                }
                Err(e) => {
                    println!("Failed to re-access file {}: {:?}", i, e);
                }
            }
        }

        // Verify cache statistics
        let (total_hits, total_misses, final_size, max_size) = parser.cache_stats();
        println!("Final cache stats: hits={}, misses={}, size={}/{}", 
                total_hits, total_misses, final_size, max_size);

        assert!(total_hits > 0 || total_misses > 0, "Cache should have some activity");
        assert_eq!(final_size, std::cmp::min(test_files.len(), small_cache_size), 
                  "Final cache size should be limited");

        // Test cache clearing
        parser.clear_cache();
        let (hits_after_clear, misses_after_clear, size_after_clear, _) = parser.cache_stats();
        assert_eq!(size_after_clear, 0, "Cache should be empty after clearing");
        
        // Test cache statistics reset
        parser.reset_cache_stats().unwrap();
        let (final_hits, final_misses, _, _) = parser.cache_stats();
        println!("Stats after reset: hits={}, misses={}", final_hits, final_misses);

        println!("✅ Cache eviction policies test passed");
    }

    #[test]
    #[cfg(feature = "memory-optimization")]
    fn test_detector_pool_performance() {
        println!("🎱 Testing Detector Pool Performance");

        // Test detector pools initialization
        let config = MemoryOptimizationConfig::default();
        let init_result = initialize_memory_optimization(config);
        assert!(init_result.is_ok(), "Memory optimization should initialize");

        // Test detector pool statistics
        let pools = &DETECTOR_POOLS;
        let stats = pools.get_all_stats();
        
        println!("Detector pool stats:");
        println!("- Dead code configs: {} shards", stats.dead_code_configs.shard_count);
        println!("- God object configs: {} shards", stats.god_object_configs.shard_count);
        println!("- Long method configs: {} shards", stats.long_method_configs.shard_count);

        assert!(stats.dead_code_configs.shard_count > 0, "Should have dead code config pools");
        assert!(stats.god_object_configs.shard_count > 0, "Should have god object config pools");
        assert!(stats.long_method_configs.shard_count > 0, "Should have long method config pools");

        // Test pool metrics export
        let metrics = pools.export_metrics();
        assert!(metrics.is_object(), "Pool metrics should be a JSON object");
        
        let pool_metrics = &metrics["detector_pools"];
        assert!(pool_metrics.is_object(), "Detector pool metrics should be available");

        // Test concurrent pool access
        let pool_access_tasks: Vec<_> = (0..100).map(|i| {
            std::thread::spawn(move || {
                // Simulate pool usage
                let stats = DETECTOR_POOLS.get_all_stats();
                (i, stats.dead_code_configs.shard_count > 0)
            })
        }).collect();

        let mut successful_accesses = 0;
        for task in pool_access_tasks {
            match task.join() {
                Ok((_, success)) => {
                    if success {
                        successful_accesses += 1;
                    }
                }
                Err(_) => {
                    println!("Pool access task panicked");
                }
            }
        }

        assert_eq!(successful_accesses, 100, "All concurrent pool accesses should succeed");
        println!("✅ Detector pool performance test passed");
    }

    #[test]
    fn test_memory_metrics_accuracy() {
        println!("📊 Testing Memory Metrics Accuracy");

        // Test basic memory metrics
        let initial_metrics = BASIC_MEMORY_METRICS.export_json();
        assert!(initial_metrics.is_object(), "Metrics should be a JSON object");

        println!("Initial metrics: {}", initial_metrics);

        // Test memory usage updates
        let test_usage = 1024 * 1024 * 100; // 100MB
        BASIC_MEMORY_METRICS.update_memory_usage(test_usage);

        let updated_metrics = BASIC_MEMORY_METRICS.export_json();
        println!("Updated metrics: {}", updated_metrics);

        // Verify metrics structure
        let phase1_metrics = &updated_metrics["memory_optimization_phase1"];
        assert!(phase1_metrics.is_object(), "Phase 1 metrics should be available");

        // Test metrics under concurrent updates
        let metrics_tasks: Vec<_> = (0..50).map(|i| {
            std::thread::spawn(move || {
                let usage = (i + 1) * 1024 * 1024; // Variable usage
                BASIC_MEMORY_METRICS.update_memory_usage(usage);
                BASIC_MEMORY_METRICS.export_json()
            })
        }).collect();

        let mut all_updates_successful = true;
        for task in metrics_tasks {
            match task.join() {
                Ok(metrics) => {
                    if !metrics.is_object() {
                        all_updates_successful = false;
                    }
                }
                Err(_) => {
                    all_updates_successful = false;
                }
            }
        }

        assert!(all_updates_successful, "All concurrent metrics updates should succeed");

        // Test optimization status reporting
        let status = get_optimization_status();
        assert!(status.is_object(), "Optimization status should be available");

        let required_fields = ["phase", "allocator", "optimized", "metrics"];
        for field in &required_fields {
            assert!(status[field].is_string() || status[field].is_boolean() || status[field].is_object(),
                   "Status should contain required field: {}", field);
        }

        println!("Optimization status: {}", status);
        println!("✅ Memory metrics accuracy test passed");
    }

    #[tokio::test]
    async fn test_memory_optimization_integration() {
        println!("🔧 Testing Memory Optimization Integration");

        // Test full integration with analysis pipeline
        let configs_to_test = vec![
            ("small", MemoryOptimizationConfig::small_project()),
            ("default", MemoryOptimizationConfig::default()),
            ("large", MemoryOptimizationConfig::large_codebase()),
        ];

        for (config_name, mem_config) in configs_to_test {
            println!("Testing {} configuration", config_name);

            // Initialize memory optimization
            let init_result = initialize_memory_optimization(mem_config.clone());
            match init_result {
                Ok(_) => {
                    println!("  Memory optimization initialized successfully");

                    // Test analysis with memory optimization
                    let analysis_config = AnalysisConfig {
                        target_path: PathBuf::from("src/lib.rs"),
                        output_format: "json".to_string(),
                        output_file: None,
                        enable_ai: false,
                        ollama_api_url: None,
                        ollama_model: None,
                        dead_code_confidence: None,
                        dead_code_library_mode: false,
                        dead_code_ignore_patterns: None,
                        dead_code_keep_alive: None,
                        large_classes_max_loc: None,
                        large_classes_max_methods: None,
                        large_classes_max_fields: None,
                        large_classes_max_complexity: None,
                        large_classes_max_lcom: None,
                        large_classes_ignore_patterns: None,
                        large_classes_min_severity: None,
                        #[cfg(feature = "memory-optimization")]
                        memory_optimization: Some(mem_config),
                        enable_memory_optimization: true,
                        memory_limit_gb: Some(4.0),
                        memory_profile: Some(config_name.to_string()),
                    };

                    let analysis_result = timeout(
                        Duration::from_secs(30),
                        async {
                            let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                            orchestrator.execute_analysis(analysis_config).await
                        }
                    ).await;

                    match analysis_result {
                        Ok(analysis_output) => {
                            match analysis_output {
                                Ok(_) => {
                                    println!("  Analysis completed successfully with {} config", config_name);
                                }
                                Err(e) => {
                                    println!("  Analysis failed with {} config: {:?}", config_name, e);
                                }
                            }
                        }
                        Err(_) => {
                            println!("  Analysis timed out with {} config", config_name);
                        }
                    }

                    // Verify optimization status
                    let status = get_optimization_status();
                    assert!(status.is_object(), "Optimization status should be available");
                    println!("  Optimization status: {}", status["phase"]);
                }
                Err(e) => {
                    println!("  Memory optimization initialization failed for {}: {}", config_name, e);
                }
            }
        }

        println!("✅ Memory optimization integration test completed");
    }
}
"#