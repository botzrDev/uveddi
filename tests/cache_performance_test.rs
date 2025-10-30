//! Cache Performance Validation Test
//!
//! This test module validates cache performance and measures hit rates
//! to ensure our cache system provides measurable performance improvements.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tempfile::{tempdir, NamedTempFile};

use uveddi::analysis::components::cache_manager::{CacheManager, CacheManagerImpl};
use uveddi::database::models::ArchitecturalIssue;

/// Create a temporary Rust file with test content
fn create_test_file(content: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

/// Test code samples for performance testing
struct TestSamples;

impl TestSamples {
    fn small_rust_code() -> &'static str {
        r#"
fn simple_function() -> i32 {
    42
}

struct SimpleStruct {
    value: i32,
}

impl SimpleStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }
}
"#
    }

    fn large_rust_code() -> &'static str {
        r#"
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

pub struct ComplexDataProcessor {
    primary_cache: HashMap<String, ComplexData>,
    secondary_cache: HashMap<i32, Vec<String>>,
    metadata: HashSet<String>,
    state: Arc<RwLock<ProcessorState>>,
}

#[derive(Clone, Debug)]
pub struct ComplexData {
    id: i32,
    name: String,
    values: Vec<f64>,
    nested: Option<Box<ComplexData>>,
    references: HashMap<String, i32>,
}

#[derive(Debug)]
enum ProcessorState {
    Initializing,
    Ready,
    Processing,
    Error(String),
    Shutdown,
}

impl ComplexDataProcessor {
    pub fn new() -> Self {
        Self {
            primary_cache: HashMap::new(),
            secondary_cache: HashMap::new(),
            metadata: HashSet::new(),
            state: Arc::new(RwLock::new(ProcessorState::Initializing)),
        }
    }

    pub fn process_data(&mut self, key: &str, data: &[u8]) -> Result<ComplexData, String> {
        if let Some(cached) = self.primary_cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = self.intensive_processing(key, data)?;
        self.primary_cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn intensive_processing(&self, key: &str, data: &[u8]) -> Result<ComplexData, String> {
        let mut values = Vec::new();
        for chunk in data.chunks(4) {
            let mut sum = 0.0;
            for &byte in chunk {
                sum += byte as f64;
            }
            values.push(sum / chunk.len() as f64);
        }

        let mut references = HashMap::new();
        for (i, &byte) in data.iter().enumerate().take(10) {
            references.insert(format!("ref_{}", i), byte as i32);
        }

        Ok(ComplexData {
            id: key.len() as i32,
            name: key.to_string(),
            values,
            nested: None,
            references,
        })
    }

    pub fn clear_cache(&mut self) {
        self.primary_cache.clear();
        self.secondary_cache.clear();
        self.metadata.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.primary_cache.len()
    }
}

// God Object anti-pattern for detection
pub struct GodObject {
    field1: String,
    field2: i32,
    field3: Vec<u8>,
    field4: HashMap<String, String>,
    field5: Option<bool>,
    field6: Result<i32, String>,
}

impl GodObject {
    pub fn method1(&self) -> i32 { 1 }
    pub fn method2(&mut self, x: i32) { self.field2 = x; }
    pub fn method3(&self) -> String { self.field1.clone() }
    pub fn method4(&self, s: &str) -> bool { self.field1.contains(s) }
    pub fn method5(&mut self) { self.field1.clear(); }
    pub fn method6(&self) -> Vec<u8> { self.field3.clone() }
    pub fn method7(&self, v: Vec<u8>) { }
    pub fn method8(&mut self, x: i32, y: String) { }
    pub fn method9(&self) -> Option<i32> { Some(self.field2) }
    pub fn method10(&self) -> Result<String, String> { Ok(self.field1.clone()) }
    pub fn method11(&self) -> bool { true }
    pub fn method12(&mut self) { }
}

// Dead code examples
fn unused_function_1() -> i32 { 42 }
fn unused_function_2(x: String) -> String { x }
fn unused_function_3(a: i32, b: i32) -> i32 { a + b }

pub fn main() {
    let mut processor = ComplexDataProcessor::new();
    let test_data = vec![1u8, 2, 3, 4, 5];
    let _ = processor.process_data("test", &test_data);
}
"#
    }
}

#[tokio::test]
async fn test_ast_cache_performance() {
    let cache_manager = CacheManagerImpl::new()
        .await
        .expect("Failed to create cache manager");

    let (_temp_dir, file_path) = create_test_file(TestSamples::large_rust_code());

    // Measure first parse (cache miss)
    let start = Instant::now();
    let first_parse = cache_manager
        .get_or_parse_ast(&file_path)
        .await
        .expect("Failed to parse AST first time");
    let first_duration = start.elapsed();

    // Measure second parse (cache hit)
    let start = Instant::now();
    let second_parse = cache_manager
        .get_or_parse_ast(&file_path)
        .await
        .expect("Failed to parse AST second time");
    let second_duration = start.elapsed();

    // Validate cache hit occurred
    assert!(Arc::ptr_eq(&first_parse, &second_parse));

    // Cache hit should be significantly faster
    println!("First parse (cache miss): {:?}", first_duration);
    println!("Second parse (cache hit): {:?}", second_duration);

    // The second parse should be at least 10x faster than the first
    // This is a reasonable expectation for cache performance
    assert!(
        second_duration.as_nanos() * 10 < first_duration.as_nanos(),
        "Cache hit should be significantly faster than cache miss"
    );

    // Validate cache statistics
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.ast_cache_size > 0, "AST cache should contain entries");
    assert!(
        stats.ast_hit_rate > 0.0,
        "AST cache should have recorded hits"
    );
}

#[tokio::test]
async fn test_result_cache_performance() {
    let cache_manager = CacheManagerImpl::new()
        .await
        .expect("Failed to create cache manager");

    let (_temp_dir, file_path) = create_test_file(TestSamples::small_rust_code());

    // Create sample analysis results
    let sample_results = vec![ArchitecturalIssue {
        id: None,
        detector_name: "test_detector".to_string(),
        issue_type: "test_issue".to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        start_line: 1,
        end_line: 10,
        start_column: 0,
        end_column: 20,
        message: "Test issue detected".to_string(),
        description: Some("This is a test issue for benchmarking".to_string()),
        severity: "medium".to_string(),
        confidence: 0.8,
        suggestion: Some("Fix the test issue".to_string()),
        metadata: None,
        created_at: None,
    }];

    // Test cache miss scenario
    let start = Instant::now();
    let cached_results = cache_manager.get_cached_results(&file_path).await;
    let miss_duration = start.elapsed();
    assert!(cached_results.is_none(), "Should be cache miss initially");

    // Cache the results
    let start = Instant::now();
    cache_manager
        .cache_results(&file_path, sample_results.clone())
        .await;
    let cache_duration = start.elapsed();

    // Test cache hit scenario
    let start = Instant::now();
    let cached_results = cache_manager.get_cached_results(&file_path).await;
    let hit_duration = start.elapsed();

    // Validate results
    assert!(
        cached_results.is_some(),
        "Should be cache hit after caching"
    );
    let cached = cached_results.unwrap();
    assert_eq!(
        cached.len(),
        sample_results.len(),
        "Cached results should match original"
    );

    // Cache hit should be faster than caching operation
    println!("Cache miss: {:?}", miss_duration);
    println!("Cache store: {:?}", cache_duration);
    println!("Cache hit: {:?}", hit_duration);

    assert!(
        hit_duration < cache_duration,
        "Cache hit should be faster than cache store"
    );

    // Validate cache statistics
    let stats = cache_manager.get_cache_stats().await;
    assert!(
        stats.result_cache_size > 0,
        "Result cache should contain entries"
    );
}

#[tokio::test]
async fn test_cache_memory_scaling() {
    let cache_manager = CacheManagerImpl::new()
        .await
        .expect("Failed to create cache manager");

    let mut temp_files = Vec::new();
    let test_cases = [
        ("small_1.rs", TestSamples::small_rust_code()),
        ("small_2.rs", TestSamples::small_rust_code()),
        ("large_1.rs", TestSamples::large_rust_code()),
        ("large_2.rs", TestSamples::large_rust_code()),
        ("large_3.rs", TestSamples::large_rust_code()),
    ];

    // Create multiple files and populate cache
    for (name, content) in test_cases.iter() {
        let (_temp_dir, file_path) = create_test_file(content);

        let start = Instant::now();
        let _parsed = cache_manager
            .get_or_parse_ast(&file_path)
            .await
            .expect("Failed to parse file");
        let duration = start.elapsed();

        println!("Parsed {} in {:?}", name, duration);
        temp_files.push((_temp_dir, file_path));
    }

    // Validate cache scaling
    let stats = cache_manager.get_cache_stats().await;
    assert_eq!(
        stats.ast_cache_size,
        test_cases.len(),
        "Cache should contain all parsed files"
    );
    assert!(
        stats.total_memory_usage > 0,
        "Cache should report memory usage"
    );

    // Test cache hit performance on all files
    let start = Instant::now();
    for (_temp_dir, file_path) in &temp_files {
        let _parsed = cache_manager
            .get_or_parse_ast(file_path)
            .await
            .expect("Failed to get cached AST");
    }
    let total_hit_duration = start.elapsed();

    println!(
        "Total cache hit duration for {} files: {:?}",
        test_cases.len(),
        total_hit_duration
    );

    // Average hit time should be very low
    let avg_hit_time = total_hit_duration / test_cases.len() as u32;
    println!("Average cache hit time: {:?}", avg_hit_time);

    // Cache hits should average less than 1ms per file
    assert!(
        avg_hit_time.as_millis() < 1,
        "Cache hits should be very fast"
    );
}

#[tokio::test]
async fn test_cache_invalidation_performance() {
    let cache_manager = CacheManagerImpl::new()
        .await
        .expect("Failed to create cache manager");

    let (_temp_dir, file_path) = create_test_file(TestSamples::large_rust_code());

    // Populate cache
    let _parsed = cache_manager
        .get_or_parse_ast(&file_path)
        .await
        .expect("Failed to parse file");

    let sample_results = vec![ArchitecturalIssue {
        id: None,
        detector_name: "test_detector".to_string(),
        issue_type: "test_issue".to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        start_line: 1,
        end_line: 10,
        start_column: 0,
        end_column: 20,
        message: "Test issue".to_string(),
        description: None,
        severity: "low".to_string(),
        confidence: 0.5,
        suggestion: None,
        metadata: None,
        created_at: None,
    }];
    cache_manager
        .cache_results(&file_path, sample_results)
        .await;

    // Verify cache is populated
    let stats_before = cache_manager.get_cache_stats().await;
    assert!(stats_before.ast_cache_size > 0, "Cache should be populated");
    assert!(
        stats_before.result_cache_size > 0,
        "Result cache should be populated"
    );

    // Measure cache invalidation performance
    let start = Instant::now();
    cache_manager.clear_all_caches().await;
    let clear_duration = start.elapsed();

    println!("Cache clear duration: {:?}", clear_duration);

    // Verify cache is cleared
    let stats_after = cache_manager.get_cache_stats().await;
    assert_eq!(
        stats_after.ast_cache_size, 0,
        "AST cache should be empty after clear"
    );
    assert_eq!(
        stats_after.result_cache_size, 0,
        "Result cache should be empty after clear"
    );

    // Cache clear should be fast (less than 100ms even for large caches)
    assert!(
        clear_duration.as_millis() < 100,
        "Cache clear should be fast"
    );
}

#[tokio::test]
async fn test_cache_hit_rate_measurement() {
    let cache_manager = CacheManagerImpl::new()
        .await
        .expect("Failed to create cache manager");

    let (_temp_dir, file_path) = create_test_file(TestSamples::small_rust_code());

    // Multiple accesses to the same file should increase hit rate
    let mut parse_times = Vec::new();

    for i in 0..5 {
        let start = Instant::now();
        let _parsed = cache_manager
            .get_or_parse_ast(&file_path)
            .await
            .expect("Failed to parse file");
        let duration = start.elapsed();

        parse_times.push(duration);
        println!("Parse {} duration: {:?}", i + 1, duration);
    }

    // First parse should be slowest (cache miss)
    // Subsequent parses should be faster (cache hits)
    assert!(
        parse_times[0] > parse_times[1],
        "First parse should be slower than second"
    );
    assert!(
        parse_times[1] <= parse_times[2],
        "Cache hits should be consistently fast"
    );
    assert!(
        parse_times[2] <= parse_times[3],
        "Cache hits should be consistently fast"
    );

    // Get final statistics
    let stats = cache_manager.get_cache_stats().await;
    println!("Final AST hit rate: {:.2}%", stats.ast_hit_rate * 100.0);

    // Hit rate should be > 80% (4 hits out of 5 accesses)
    assert!(
        stats.ast_hit_rate >= 0.8,
        "Hit rate should be high with repeated accesses"
    );
}
