//! Comprehensive performance tests for the multi-layered cache system
//!
//! This test suite validates the performance characteristics of the new
//! cache implementation, including hit rates, latency, and memory usage.

use prometheus::Registry;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use uveddi::analysis::cache::{
    engine_cache::{EngineCache, EngineCacheConfig},
    metrics::CacheMetrics,
};
use uveddi::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use uveddi::database::models::ArchitecturalIssue;

fn create_test_parsed_file(name: &str) -> ParsedFile {
    ParsedFile {
        file_path: Arc::new(std::path::PathBuf::from(name)),
        language: SourceLanguage::Rust,
        tree: None,
        source: Arc::new(format!("fn {}() {{}}", name)),
        custom_ast: Arc::new(None),
        modified_at: uveddi::analysis::cache::compat::ArchivableSystemTime(
            std::time::SystemTime::now(),
        ),
    }
}

fn create_test_issues(count: usize) -> Vec<ArchitecturalIssue> {
    (0..count)
        .map(|i| ArchitecturalIssue {
            issue_id: Some(i as i64),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: format!("test_{}.rs", i),
            start_line: Some(1),
            end_line: Some(10),
            description: format!("Test issue {}", i),
            severity: "medium".to_string(),
            code_snippet: Some("test code".to_string()),
            ai_explanation: Some("Test AI explanation".to_string()),
        })
        .collect()
}

#[tokio::test]
async fn test_cache_performance_characteristics() {
    let registry = Registry::new();
    let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());

    let config = EngineCacheConfig {
        ast_capacity: 100,
        result_capacity: 100,
        result_ttl: std::time::Duration::from_secs(60),
    };

    let cache = EngineCache::new_with_config(config, metrics.clone())
        .await
        .unwrap();

    // Test AST cache performance
    let start_time = Instant::now();

    for i in 0..50 {
        let file_path_str = format!("test_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let file_name = format!("test_{}", i);

        let parser = move || Ok(create_test_parsed_file(&file_name));

        let _result = cache.get_or_parse_ast(file_path, parser).await.unwrap();
    }

    let initial_load_time = start_time.elapsed();
    println!("Initial load time (50 files): {:?}", initial_load_time);

    // Test cache hit performance
    let start_time = Instant::now();

    for i in 0..50 {
        let file_path_str = format!("test_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);

        let parser = || panic!("Should not be called - cache hit expected");

        let _result = cache.get_or_parse_ast(file_path, parser).await.unwrap();
    }

    let cache_hit_time = start_time.elapsed();
    println!("Cache hit time (50 files): {:?}", cache_hit_time);

    // Cache hits should be significantly faster
    assert!(cache_hit_time < initial_load_time / 2);

    // Test result cache performance
    for i in 0..50 {
        let file_path_str = format!("result_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let issues = create_test_issues(5);

        cache.cache_results(file_path, issues).await;
    }

    // Verify results can be retrieved
    for i in 0..50 {
        let file_path_str = format!("result_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let cached_results = cache.get_cached_results(file_path).await;

        assert!(cached_results.is_some());
        assert_eq!(cached_results.unwrap().len(), 5);
    }

    // Check cache statistics
    let stats = cache.stats().await;
    assert_eq!(stats.ast_entries, 50);
    assert_eq!(stats.result_entries, 50);
    assert!(stats.hit_rate > 0.5); // Should have good hit rate

    println!("Cache stats: {:?}", stats);
}

#[tokio::test]
async fn test_cache_eviction_behavior() {
    let registry = Registry::new();
    let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());

    // Small cache to trigger eviction
    let config = EngineCacheConfig {
        ast_capacity: 5,
        result_capacity: 5,
        result_ttl: std::time::Duration::from_secs(60),
    };

    let cache = EngineCache::new_with_config(config, metrics.clone())
        .await
        .unwrap();

    // Fill cache beyond capacity
    for i in 0..10 {
        let file_path_str = format!("test_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let file_name = format!("test_{}", i);

        let parser = move || Ok(create_test_parsed_file(&file_name));

        let _result = cache.get_or_parse_ast(file_path, parser).await.unwrap();
    }

    let stats = cache.stats().await;
    assert_eq!(stats.ast_entries, 5); // Should be limited by capacity

    // Test result cache eviction
    for i in 0..10 {
        let file_path_str = format!("result_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let issues = create_test_issues(3);

        cache.cache_results(file_path, issues).await;
    }

    let stats = cache.stats().await;
    assert_eq!(stats.result_entries, 5); // Should be limited by capacity
}

#[tokio::test]
async fn test_cache_expiration() {
    let registry = Registry::new();
    let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());

    // Very short TTL for testing
    let config = EngineCacheConfig {
        ast_capacity: 100,
        result_capacity: 100,
        result_ttl: std::time::Duration::from_millis(50),
    };

    let cache = EngineCache::new_with_config(config, metrics.clone())
        .await
        .unwrap();

    // Cache some results
    let file_path = std::path::Path::new("expiry_test.rs");
    let issues = create_test_issues(3);

    cache.cache_results(file_path, issues).await;

    // Should be cached
    let cached = cache.get_cached_results(file_path).await;
    assert!(cached.is_some());

    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Should be expired
    let expired = cache.get_cached_results(file_path).await;
    assert!(expired.is_none());
}

#[tokio::test]
async fn test_cache_maintenance() {
    let registry = Registry::new();
    let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());

    let config = EngineCacheConfig {
        ast_capacity: 100,
        result_capacity: 100,
        result_ttl: std::time::Duration::from_millis(50),
    };

    let cache = EngineCache::new_with_config(config, metrics.clone())
        .await
        .unwrap();

    // Add many results
    for i in 0..20 {
        let file_path_str = format!("maintenance_{}.rs", i);
        let file_path = std::path::Path::new(&file_path_str);
        let issues = create_test_issues(2);

        cache.cache_results(file_path, issues).await;
    }

    let stats_before = cache.stats().await;
    assert_eq!(stats_before.result_entries, 20);

    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Run maintenance
    cache.maintain().await;

    // Check stats after maintenance - should be fewer entries
    let stats_after = cache.stats().await;
    assert!(stats_after.result_entries < stats_before.result_entries);
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let registry = Registry::new();
    let metrics = Arc::new(CacheMetrics::new(&registry).unwrap());
    let cache = Arc::new(EngineCache::new(metrics.clone()).await.unwrap());

    let mut handles = Vec::new();

    // Spawn multiple concurrent tasks
    for task_id in 0..10 {
        let cache_clone = cache.clone();

        let handle = tokio::spawn(async move {
            for i in 0..10 {
                let file_path_str = format!("concurrent_{}_{}.rs", task_id, i);
                let file_path = std::path::Path::new(&file_path_str);
                let file_name = format!("concurrent_{}_{}", task_id, i);

                let parser = move || Ok(create_test_parsed_file(&file_name));

                let _result = cache_clone
                    .get_or_parse_ast(file_path, parser)
                    .await
                    .unwrap();

                // Also test result caching
                let issues = create_test_issues(2);
                cache_clone.cache_results(file_path, issues).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let stats = cache.stats().await;
    assert_eq!(stats.ast_entries, 100); // All unique files should be cached
    assert_eq!(stats.result_entries, 100);

    println!("Concurrent access completed successfully: {:?}", stats);
}
