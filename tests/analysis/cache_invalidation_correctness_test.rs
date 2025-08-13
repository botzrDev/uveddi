//! Cache invalidation correctness testing
//! 
//! This module tests the cache invalidation system to ensure data consistency
//! and correct behavior under various invalidation scenarios.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::{TempDir, NamedTempFile};
use tokio::fs;
use tokio::time::sleep;
use uveddi::analysis::cache::{
    AstCache,
    EngineCache,
    MultilayerCache,
    CacheInvalidationManager,
    CacheMetrics,
    InvalidationEvent,
    InvalidationStrategy,
};
use uveddi::analysis::components::ast_provider::AstProvider;

/// Test basic cache invalidation on file changes
#[tokio::test]
async fn test_file_change_invalidation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = AstCache::new(temp_dir.path(), 1000)
        .expect("Failed to create cache");
    
    // Create test file
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn hello() { println!(\"Hello\"); }")
        .await
        .expect("Failed to write test file");
    
    // Parse and cache the file
    let ast_provider = AstProvider::new();
    let initial_ast = ast_provider.parse_file(&test_file).await
        .expect("Failed to parse file");
    
    cache.store_ast(&test_file, &initial_ast).await
        .expect("Failed to store AST");
    
    // Verify cache hit
    let cached_ast = cache.get_ast(&test_file).await
        .expect("Failed to get cached AST")
        .expect("AST not found in cache");
    
    assert_eq!(cached_ast.checksum(), initial_ast.checksum());
    
    // Modify the file
    sleep(Duration::from_millis(10)).await; // Ensure different timestamp
    fs::write(&test_file, "fn hello() { println!(\"Hello, World!\"); }")
        .await
        .expect("Failed to modify test file");
    
    // Cache should detect file change and return None
    let cache_result = cache.get_ast(&test_file).await
        .expect("Failed to check cache");
    
    assert!(cache_result.is_none(), "Cache should be invalidated after file change");
    
    // Re-parse and verify new content is cached
    let new_ast = ast_provider.parse_file(&test_file).await
        .expect("Failed to parse modified file");
    
    cache.store_ast(&test_file, &new_ast).await
        .expect("Failed to store new AST");
    
    let cached_new_ast = cache.get_ast(&test_file).await
        .expect("Failed to get new cached AST")
        .expect("New AST not found in cache");
    
    assert_eq!(cached_new_ast.checksum(), new_ast.checksum());
    assert_ne!(cached_new_ast.checksum(), initial_ast.checksum());
}

/// Test dependency-based cache invalidation
#[tokio::test]
async fn test_dependency_based_invalidation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = MultilayerCache::new(temp_dir.path(), 1000)
        .expect("Failed to create multilayer cache");
    
    // Create dependency chain: main.rs -> lib.rs -> utils.rs
    let main_file = temp_dir.path().join("main.rs");
    let lib_file = temp_dir.path().join("lib.rs");
    let utils_file = temp_dir.path().join("utils.rs");
    
    fs::write(&utils_file, "pub fn utility() -> i32 { 42 }")
        .await
        .expect("Failed to write utils.rs");
    
    fs::write(&lib_file, "mod utils; pub use utils::utility;")
        .await
        .expect("Failed to write lib.rs");
    
    fs::write(&main_file, "mod lib; fn main() { println!(\"{}\", lib::utility()); }")
        .await
        .expect("Failed to write main.rs");
    
    // Build dependency graph and cache all files
    let ast_provider = AstProvider::new();
    let utils_ast = ast_provider.parse_file(&utils_file).await
        .expect("Failed to parse utils.rs");
    let lib_ast = ast_provider.parse_file(&lib_file).await
        .expect("Failed to parse lib.rs");
    let main_ast = ast_provider.parse_file(&main_file).await
        .expect("Failed to parse main.rs");
    
    cache.store_ast(&utils_file, &utils_ast).await
        .expect("Failed to cache utils.rs");
    cache.store_ast(&lib_file, &lib_ast).await
        .expect("Failed to cache lib.rs");
    cache.store_ast(&main_file, &main_ast).await
        .expect("Failed to cache main.rs");
    
    // Register dependencies
    cache.add_dependency(&lib_file, &utils_file).await
        .expect("Failed to add lib -> utils dependency");
    cache.add_dependency(&main_file, &lib_file).await
        .expect("Failed to add main -> lib dependency");
    
    // Verify all files are cached
    assert!(cache.get_ast(&utils_file).await.expect("Cache check failed").is_some());
    assert!(cache.get_ast(&lib_file).await.expect("Cache check failed").is_some());
    assert!(cache.get_ast(&main_file).await.expect("Cache check failed").is_some());
    
    // Modify utils.rs
    sleep(Duration::from_millis(10)).await;
    fs::write(&utils_file, "pub fn utility() -> i32 { 100 }")
        .await
        .expect("Failed to modify utils.rs");
    
    // Trigger dependency invalidation
    cache.invalidate_dependencies(&utils_file).await
        .expect("Failed to invalidate dependencies");
    
    // utils.rs should be invalidated (direct change)
    assert!(cache.get_ast(&utils_file).await.expect("Cache check failed").is_none());
    
    // lib.rs should be invalidated (depends on utils.rs)
    assert!(cache.get_ast(&lib_file).await.expect("Cache check failed").is_none());
    
    // main.rs should be invalidated (depends on lib.rs)
    assert!(cache.get_ast(&main_file).await.expect("Cache check failed").is_none());
}

/// Test multilayer cache consistency
#[tokio::test]
async fn test_multilayer_cache_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = MultilayerCache::new(temp_dir.path(), 1000)
        .expect("Failed to create multilayer cache");
    
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn test() {}")
        .await
        .expect("Failed to write test file");
    
    let ast_provider = AstProvider::new();
    let ast = ast_provider.parse_file(&test_file).await
        .expect("Failed to parse file");
    
    // Store in cache
    cache.store_ast(&test_file, &ast).await
        .expect("Failed to store AST");
    
    // Should be in memory layer
    assert!(cache.get_from_memory(&test_file).await.is_some());
    
    // Flush to disk layer
    cache.flush_to_disk().await
        .expect("Failed to flush to disk");
    
    // Should be in both layers
    assert!(cache.get_from_memory(&test_file).await.is_some());
    assert!(cache.get_from_disk(&test_file).await.expect("Disk read failed").is_some());
    
    // Clear memory layer
    cache.clear_memory().await
        .expect("Failed to clear memory");
    
    // Should only be in disk layer
    assert!(cache.get_from_memory(&test_file).await.is_none());
    assert!(cache.get_from_disk(&test_file).await.expect("Disk read failed").is_some());
    
    // Get should populate memory layer from disk
    let retrieved_ast = cache.get_ast(&test_file).await
        .expect("Failed to get AST")
        .expect("AST not found");
    
    assert_eq!(retrieved_ast.checksum(), ast.checksum());
    assert!(cache.get_from_memory(&test_file).await.is_some());
    
    // Invalidate should clear both layers
    cache.invalidate(&test_file).await
        .expect("Failed to invalidate");
    
    assert!(cache.get_from_memory(&test_file).await.is_none());
    assert!(cache.get_from_disk(&test_file).await.expect("Disk read failed").is_none());
}

/// Test cache invalidation strategies
#[tokio::test]
async fn test_invalidation_strategies() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalidation_manager = CacheInvalidationManager::new()
        .expect("Failed to create invalidation manager");
    
    // Test time-based invalidation
    let time_based = InvalidationStrategy::TimeBased {
        max_age: Duration::from_millis(100),
    };
    
    let test_file = temp_dir.path().join("time_test.rs");
    fs::write(&test_file, "fn time_test() {}")
        .await
        .expect("Failed to write test file");
    
    invalidation_manager.set_strategy(&test_file, time_based).await
        .expect("Failed to set time-based strategy");
    
    // File should be valid initially
    assert!(invalidation_manager.is_valid(&test_file).await
        .expect("Validity check failed"));
    
    // Wait for expiration
    sleep(Duration::from_millis(150)).await;
    
    // File should be invalid after max_age
    assert!(!invalidation_manager.is_valid(&test_file).await
        .expect("Validity check failed"));
    
    // Test size-based invalidation
    let size_based = InvalidationStrategy::SizeBased {
        max_file_size: 50, // 50 bytes
    };
    
    let size_test_file = temp_dir.path().join("size_test.rs");
    fs::write(&size_test_file, "fn small() {}")
        .await
        .expect("Failed to write small file");
    
    invalidation_manager.set_strategy(&size_test_file, size_based).await
        .expect("Failed to set size-based strategy");
    
    // Small file should be valid
    assert!(invalidation_manager.is_valid(&size_test_file).await
        .expect("Validity check failed"));
    
    // Make file larger
    fs::write(&size_test_file, "fn large_function_with_very_long_name_that_exceeds_size_limit() { println!(\"This is a very long function\"); }")
        .await
        .expect("Failed to write large file");
    
    // Large file should be invalid
    assert!(!invalidation_manager.is_valid(&size_test_file).await
        .expect("Validity check failed"));
    
    // Test checksum-based invalidation
    let checksum_based = InvalidationStrategy::ChecksumBased;
    
    let checksum_test_file = temp_dir.path().join("checksum_test.rs");
    fs::write(&checksum_test_file, "fn checksum_test() {}")
        .await
        .expect("Failed to write checksum test file");
    
    invalidation_manager.set_strategy(&checksum_test_file, checksum_based).await
        .expect("Failed to set checksum-based strategy");
    
    // Store initial checksum
    invalidation_manager.store_checksum(&checksum_test_file).await
        .expect("Failed to store checksum");
    
    // File should be valid with same content
    assert!(invalidation_manager.is_valid(&checksum_test_file).await
        .expect("Validity check failed"));
    
    // Modify content
    sleep(Duration::from_millis(10)).await;
    fs::write(&checksum_test_file, "fn checksum_test_modified() {}")
        .await
        .expect("Failed to modify checksum test file");
    
    // File should be invalid with different content
    assert!(!invalidation_manager.is_valid(&checksum_test_file).await
        .expect("Validity check failed"));
}

/// Test concurrent cache invalidation
#[tokio::test]
async fn test_concurrent_invalidation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = Arc::new(MultilayerCache::new(temp_dir.path(), 1000)
        .expect("Failed to create cache"));
    
    let num_files = 10;
    let mut file_paths = Vec::new();
    let ast_provider = AstProvider::new();
    
    // Create test files and cache them
    for i in 0..num_files {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        fs::write(&file_path, format!("fn test_{}() {{}}", i))
            .await
            .expect("Failed to write test file");
        
        let ast = ast_provider.parse_file(&file_path).await
            .expect("Failed to parse file");
        
        cache.store_ast(&file_path, &ast).await
            .expect("Failed to store AST");
        
        file_paths.push(file_path);
    }
    
    // Verify all files are cached
    for file_path in &file_paths {
        assert!(cache.get_ast(file_path).await
            .expect("Cache check failed").is_some());
    }
    
    // Spawn concurrent invalidation tasks
    let handles: Vec<_> = file_paths.into_iter().enumerate().map(|(i, file_path)| {
        let cache = Arc::clone(&cache);
        tokio::spawn(async move {
            // Random delay to create race conditions
            sleep(Duration::from_millis((i * 10) as u64)).await;
            
            // Invalidate the file
            cache.invalidate(&file_path).await
                .expect("Failed to invalidate file");
            
            // Verify invalidation
            let result = cache.get_ast(&file_path).await
                .expect("Cache check failed");
            assert!(result.is_none(), "File should be invalidated");
            
            file_path
        })
    }).collect();
    
    // Wait for all invalidations to complete
    let results: Vec<PathBuf> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.expect("Task failed"))
        .collect();
    
    // Verify all files were processed
    assert_eq!(results.len(), num_files);
    
    // Verify cache is empty
    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.cached_files, 0);
}

/// Test invalidation event propagation
#[tokio::test]
async fn test_invalidation_event_propagation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = MultilayerCache::new(temp_dir.path(), 1000)
        .expect("Failed to create cache");
    
    let mut event_receiver = cache.subscribe_to_invalidation_events()
        .expect("Failed to subscribe to events");
    
    let test_file = temp_dir.path().join("event_test.rs");
    fs::write(&test_file, "fn event_test() {}")
        .await
        .expect("Failed to write test file");
    
    let ast_provider = AstProvider::new();
    let ast = ast_provider.parse_file(&test_file).await
        .expect("Failed to parse file");
    
    cache.store_ast(&test_file, &ast).await
        .expect("Failed to store AST");
    
    // Invalidate and check for event
    cache.invalidate(&test_file).await
        .expect("Failed to invalidate");
    
    // Should receive invalidation event
    let event = tokio::time::timeout(
        Duration::from_millis(100),
        event_receiver.recv()
    ).await
        .expect("Timeout waiting for event")
        .expect("Failed to receive event");
    
    match event {
        InvalidationEvent::FileInvalidated { path, reason } => {
            assert_eq!(path, test_file);
            assert!(matches!(reason, InvalidationReason::Manual));
        }
        other => panic!("Unexpected event type: {:?}", other),
    }
}

/// Test cache invalidation performance
#[tokio::test]
async fn test_invalidation_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = MultilayerCache::new(temp_dir.path(), 10000)
        .expect("Failed to create cache");
    
    let num_files = 1000;
    let mut file_paths = Vec::new();
    let ast_provider = AstProvider::new();
    
    // Create and cache many files
    for i in 0..num_files {
        let file_path = temp_dir.path().join(format!("perf_test_{}.rs", i));
        fs::write(&file_path, format!("fn test_{}() {{}}", i))
            .await
            .expect("Failed to write test file");
        
        let ast = ast_provider.parse_file(&file_path).await
            .expect("Failed to parse file");
        
        cache.store_ast(&file_path, &ast).await
            .expect("Failed to store AST");
        
        file_paths.push(file_path);
    }
    
    // Measure invalidation performance
    let start = std::time::Instant::now();
    
    // Invalidate all files
    for file_path in &file_paths {
        cache.invalidate(file_path).await
            .expect("Failed to invalidate file");
    }
    
    let invalidation_time = start.elapsed();
    
    println!("Invalidated {} files in {:?}", num_files, invalidation_time);
    
    // Performance assertion
    let max_time_per_file = Duration::from_micros(100); // 0.1ms per file
    let max_total_time = max_time_per_file * num_files as u32;
    
    assert!(invalidation_time < max_total_time,
           "Invalidation too slow: {:?} for {} files", 
           invalidation_time, num_files);
    
    // Verify all files are invalidated
    for file_path in &file_paths {
        assert!(cache.get_ast(file_path).await
            .expect("Cache check failed").is_none());
    }
}

/// Test partial cache invalidation
#[tokio::test]
async fn test_partial_invalidation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache = MultilayerCache::new(temp_dir.path(), 1000)
        .expect("Failed to create cache");
    
    // Create files in different directories
    let src_dir = temp_dir.path().join("src");
    let test_dir = temp_dir.path().join("tests");
    
    fs::create_dir_all(&src_dir).await
        .expect("Failed to create src directory");
    fs::create_dir_all(&test_dir).await
        .expect("Failed to create test directory");
    
    let src_files = vec![
        src_dir.join("main.rs"),
        src_dir.join("lib.rs"),
        src_dir.join("utils.rs"),
    ];
    
    let test_files = vec![
        test_dir.join("integration.rs"),
        test_dir.join("unit.rs"),
    ];
    
    let ast_provider = AstProvider::new();
    
    // Cache all files
    for file_path in src_files.iter().chain(test_files.iter()) {
        fs::write(file_path, "fn placeholder() {}")
            .await
            .expect("Failed to write file");
        
        let ast = ast_provider.parse_file(file_path).await
            .expect("Failed to parse file");
        
        cache.store_ast(file_path, &ast).await
            .expect("Failed to store AST");
    }
    
    // Verify all files are cached
    for file_path in src_files.iter().chain(test_files.iter()) {
        assert!(cache.get_ast(file_path).await
            .expect("Cache check failed").is_some());
    }
    
    // Invalidate only src directory
    cache.invalidate_directory(&src_dir).await
        .expect("Failed to invalidate src directory");
    
    // src files should be invalidated
    for file_path in &src_files {
        assert!(cache.get_ast(file_path).await
            .expect("Cache check failed").is_none());
    }
    
    // test files should still be cached
    for file_path in &test_files {
        assert!(cache.get_ast(file_path).await
            .expect("Cache check failed").is_some());
    }
}

#[cfg(test)]
mod helpers {
    use super::*;
    use uveddi::analysis::cache::InvalidationReason;
    
    /// Helper to create test files with known content
    pub async fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).await
            .expect("Failed to write test file");
        file_path
    }
    
    /// Helper to verify cache metrics
    pub async fn verify_cache_metrics(cache: &MultilayerCache, expected_files: usize) {
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.cached_files, expected_files,
                  "Cache file count mismatch");
    }
    
    /// Helper to create dependency chain
    pub async fn create_dependency_chain(
        cache: &MultilayerCache,
        files: &[(PathBuf, Vec<PathBuf>)]
    ) {
        for (file, dependencies) in files {
            for dependency in dependencies {
                cache.add_dependency(file, dependency).await
                    .expect("Failed to add dependency");
            }
        }
    }
    
    #[derive(Debug, Clone)]
    pub enum InvalidationReason {
        Manual,
        FileChanged,
        DependencyChanged,
        TimeExpired,
        SizeExceeded,
        ChecksumMismatch,
    }
    
    /// Mock invalidation event for testing
    #[derive(Debug, Clone)]
    pub struct MockInvalidationEvent {
        pub file_path: PathBuf,
        pub reason: InvalidationReason,
        pub timestamp: SystemTime,
    }
}