//! Arena allocation stress testing for memory optimization
//! 
//! This module tests the arena-based memory allocation system under stress
//! conditions to ensure correctness, performance, and memory safety.

use bumpalo::Bump;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uveddi::analysis::memory::{
    ArenaManager,
    DetectorPool,
    MemoryConfig,
    MemoryMetrics,
    MemoryPool,
    ZeroCopyAstCache,
};
use uveddi::analysis::components::ast_provider::AstProvider;

/// Test basic arena allocation and deallocation
#[tokio::test]
async fn test_basic_arena_allocation() {
    let config = MemoryConfig {
        max_arena_size: 64 * 1024 * 1024, // 64MB
        detector_pool_size: 10,
        ast_cache_size: 1000,
        enable_zero_copy: true,
    };
    
    let arena_manager = ArenaManager::new(config)
        .expect("Failed to create ArenaManager");
    
    // Test allocating various sizes
    let sizes = vec![1, 16, 256, 4096, 65536];
    
    for size in sizes {
        let arena = arena_manager.create_arena(size)
            .expect(&format!("Failed to create arena of size {}", size));
        
        // Allocate some data in the arena
        let data: &mut [u8] = arena.alloc_slice_fill_default(size);
        assert_eq!(data.len(), size);
        
        // Fill with test data
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Verify data integrity
        for (i, &byte) in data.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }
}

/// Test arena allocation under memory pressure
#[tokio::test]
async fn test_arena_memory_pressure() {
    let config = MemoryConfig {
        max_arena_size: 1024 * 1024, // 1MB limit
        detector_pool_size: 5,
        ast_cache_size: 100,
        enable_zero_copy: true,
    };
    
    let arena_manager = ArenaManager::new(config)
        .expect("Failed to create ArenaManager");
    
    let mut arenas = Vec::new();
    let allocation_size = 64 * 1024; // 64KB per allocation
    
    // Allocate until we hit memory pressure
    for i in 0..20 {
        match arena_manager.create_arena(allocation_size) {
            Ok(arena) => {
                // Fill the arena to ensure memory is actually used
                let data: &mut [u8] = arena.alloc_slice_fill_default(allocation_size);
                data.fill((i % 256) as u8);
                arenas.push(arena);
            }
            Err(err) => {
                // Should get memory pressure error eventually
                assert!(err.to_string().contains("memory") || err.to_string().contains("limit"));
                break;
            }
        }
    }
    
    // Should have allocated at least some arenas
    assert!(!arenas.is_empty(), "Should have allocated at least one arena");
    
    // Memory metrics should reflect usage
    let metrics = arena_manager.get_memory_metrics();
    assert!(metrics.total_allocated > 0);
    assert!(metrics.arena_count > 0);
}

/// Test concurrent arena allocation
#[tokio::test]
async fn test_concurrent_arena_allocation() {
    let config = MemoryConfig {
        max_arena_size: 32 * 1024 * 1024, // 32MB
        detector_pool_size: 20,
        ast_cache_size: 1000,
        enable_zero_copy: true,
    };
    
    let arena_manager = Arc::new(ArenaManager::new(config)
        .expect("Failed to create ArenaManager"));
    
    let num_threads = 8;
    let allocations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));
    
    let handles: Vec<_> = (0..num_threads).map(|thread_id| {
        let arena_manager = Arc::clone(&arena_manager);
        let barrier = Arc::clone(&barrier);
        
        thread::spawn(move || {
            barrier.wait(); // Synchronize start
            
            let mut thread_arenas = Vec::new();
            
            for i in 0..allocations_per_thread {
                let size = 1024 + (i * 10); // Variable sizes
                
                match arena_manager.create_arena(size) {
                    Ok(arena) => {
                        // Allocate and write test data
                        let data: &mut [u8] = arena.alloc_slice_fill_default(size);
                        let pattern = ((thread_id + i) % 256) as u8;
                        data.fill(pattern);
                        
                        // Verify immediately
                        assert!(data.iter().all(|&b| b == pattern));
                        
                        thread_arenas.push((arena, pattern));
                    }
                    Err(_) => {
                        // Memory pressure is acceptable in concurrent test
                        break;
                    }
                }
            }
            
            // Verify all data is still intact
            for (arena, expected_pattern) in &thread_arenas {
                let data = arena.allocated_bytes();
                assert!(data.iter().all(|&b| b == *expected_pattern));
            }
            
            thread_arenas.len()
        })
    }).collect();
    
    // Wait for all threads and collect results
    let results: Vec<usize> = handles.into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();
    
    // Verify all threads made some allocations
    let total_allocations: usize = results.iter().sum();
    assert!(total_allocations > 0, "No allocations made across all threads");
    
    // Each thread should have made at least some allocations
    for (i, &count) in results.iter().enumerate() {
        assert!(count > 0, "Thread {} made no allocations", i);
    }
}

/// Test detector pool memory management
#[tokio::test]
async fn test_detector_pool_management() {
    let config = MemoryConfig {
        max_arena_size: 16 * 1024 * 1024, // 16MB
        detector_pool_size: 5,
        ast_cache_size: 500,
        enable_zero_copy: true,
    };
    
    let detector_pool = DetectorPool::new(config)
        .expect("Failed to create DetectorPool");
    
    // Test acquiring and releasing detectors
    let mut detectors = Vec::new();
    
    // Acquire all detectors
    for i in 0..config.detector_pool_size {
        match detector_pool.acquire_detector().await {
            Ok(detector) => {
                assert_eq!(detector.id(), i);
                detectors.push(detector);
            }
            Err(err) => panic!("Failed to acquire detector {}: {}", i, err),
        }
    }
    
    // Pool should be exhausted
    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        detector_pool.acquire_detector()
    ).await;
    
    assert!(result.is_err(), "Should timeout when pool is exhausted");
    
    // Release one detector
    let detector = detectors.pop().unwrap();
    detector_pool.release_detector(detector).await
        .expect("Failed to release detector");
    
    // Should be able to acquire again
    let detector = detector_pool.acquire_detector().await
        .expect("Failed to acquire detector after release");
    
    assert!(detector.is_available());
}

/// Test zero-copy AST cache performance
#[tokio::test]
async fn test_zero_copy_ast_cache() {
    let config = MemoryConfig {
        max_arena_size: 8 * 1024 * 1024, // 8MB
        detector_pool_size: 3,
        ast_cache_size: 1000,
        enable_zero_copy: true,
    };
    
    let ast_cache = ZeroCopyAstCache::new(config)
        .expect("Failed to create ZeroCopyAstCache");
    
    // Create test AST data
    let file_path = "test_file.rs";
    let ast_data = create_large_test_ast(10000); // 10K nodes
    
    // Measure cache write performance
    let start = Instant::now();
    ast_cache.store_ast(file_path, &ast_data).await
        .expect("Failed to store AST");
    let write_time = start.elapsed();
    
    println!("Zero-copy cache write time: {:?}", write_time);
    
    // Measure cache read performance
    let start = Instant::now();
    let cached_ast = ast_cache.get_ast(file_path).await
        .expect("Failed to get AST from cache")
        .expect("AST not found in cache");
    let read_time = start.elapsed();
    
    println!("Zero-copy cache read time: {:?}", read_time);
    
    // Verify data integrity
    assert_eq!(cached_ast.node_count(), ast_data.node_count());
    assert_eq!(cached_ast.checksum(), ast_data.checksum());
    
    // Performance assertions
    assert!(write_time < Duration::from_millis(100), 
           "Cache write too slow: {:?}", write_time);
    assert!(read_time < Duration::from_millis(10), 
           "Cache read too slow: {:?}", read_time);
}

/// Test memory leak detection
#[tokio::test]
async fn test_memory_leak_detection() {
    let config = MemoryConfig {
        max_arena_size: 4 * 1024 * 1024, // 4MB
        detector_pool_size: 3,
        ast_cache_size: 100,
        enable_zero_copy: true,
    };
    
    let arena_manager = ArenaManager::new(config)
        .expect("Failed to create ArenaManager");
    
    // Get initial memory metrics
    let initial_metrics = arena_manager.get_memory_metrics();
    
    // Perform allocation cycles
    for cycle in 0..10 {
        let mut arenas = Vec::new();
        
        // Allocate several arenas
        for i in 0..5 {
            let size = 1024 * (i + 1);
            let arena = arena_manager.create_arena(size)
                .expect("Failed to create arena");
            
            // Use the arena
            let data: &mut [u8] = arena.alloc_slice_fill_default(size);
            data.fill((cycle + i) as u8);
            
            arenas.push(arena);
        }
        
        // Arenas go out of scope here and should be deallocated
    }
    
    // Force garbage collection (if available)
    arena_manager.force_cleanup().await
        .expect("Failed to force cleanup");
    
    // Get final metrics
    let final_metrics = arena_manager.get_memory_metrics();
    
    // Memory usage should not have grown significantly
    let memory_growth = final_metrics.total_allocated - initial_metrics.total_allocated;
    let max_acceptable_growth = 1024 * 1024; // 1MB
    
    assert!(memory_growth <= max_acceptable_growth,
           "Potential memory leak detected: {} bytes growth", memory_growth);
    
    // Active arena count should be low
    assert!(final_metrics.arena_count <= 2,
           "Too many arenas still active: {}", final_metrics.arena_count);
}

/// Test memory pool exhaustion and recovery
#[tokio::test]
async fn test_memory_pool_exhaustion_recovery() {
    let config = MemoryConfig {
        max_arena_size: 2 * 1024 * 1024, // 2MB total
        detector_pool_size: 2,
        ast_cache_size: 50,
        enable_zero_copy: true,
    };
    
    let memory_pool = MemoryPool::new(config)
        .expect("Failed to create MemoryPool");
    
    let large_allocation_size = 1024 * 1024; // 1MB
    let mut allocations = Vec::new();
    
    // Exhaust the pool
    loop {
        match memory_pool.allocate(large_allocation_size).await {
            Ok(allocation) => {
                // Fill with test pattern
                let data = allocation.as_mut_slice();
                data.fill(0xAA);
                allocations.push(allocation);
            }
            Err(err) => {
                assert!(err.to_string().contains("exhausted") || 
                       err.to_string().contains("memory"));
                break;
            }
        }
    }
    
    assert!(!allocations.is_empty(), "Should have made at least one allocation");
    
    // Release all allocations
    for allocation in allocations {
        memory_pool.deallocate(allocation).await
            .expect("Failed to deallocate");
    }
    
    // Pool should recover and allow new allocations
    let recovered_allocation = memory_pool.allocate(large_allocation_size).await
        .expect("Pool should recover after deallocation");
    
    // Verify the allocation works
    let data = recovered_allocation.as_mut_slice();
    data.fill(0xBB);
    assert!(data.iter().all(|&b| b == 0xBB));
}

/// Test arena allocation alignment and safety
#[tokio::test]
async fn test_arena_alignment_safety() {
    let config = MemoryConfig {
        max_arena_size: 1024 * 1024, // 1MB
        detector_pool_size: 2,
        ast_cache_size: 100,
        enable_zero_copy: true,
    };
    
    let arena_manager = ArenaManager::new(config)
        .expect("Failed to create ArenaManager");
    
    let arena = arena_manager.create_arena(8192)
        .expect("Failed to create arena");
    
    // Test various alignment requirements
    let alignments = vec![1, 2, 4, 8, 16, 32, 64];
    
    for alignment in alignments {
        let aligned_ptr = arena.alloc_with_align::<u8>(alignment, 64)
            .expect(&format!("Failed to allocate with alignment {}", alignment));
        
        // Verify alignment
        let addr = aligned_ptr.as_ptr() as usize;
        assert_eq!(addr % alignment, 0, 
                  "Allocation not properly aligned to {} bytes", alignment);
        
        // Verify we can write to the memory safely
        for i in 0..64 {
            unsafe {
                *aligned_ptr.as_ptr().add(i) = (i % 256) as u8;
            }
        }
        
        // Verify data integrity
        for i in 0..64 {
            unsafe {
                assert_eq!(*aligned_ptr.as_ptr().add(i), (i % 256) as u8);
            }
        }
    }
}

/// Test memory metrics accuracy
#[tokio::test]
async fn test_memory_metrics_accuracy() {
    let config = MemoryConfig {
        max_arena_size: 4 * 1024 * 1024, // 4MB
        detector_pool_size: 3,
        ast_cache_size: 200,
        enable_zero_copy: true,
    };
    
    let arena_manager = ArenaManager::new(config)
        .expect("Failed to create ArenaManager");
    
    // Get baseline metrics
    let baseline = arena_manager.get_memory_metrics();
    
    // Allocate known amounts
    let allocation_sizes = vec![1024, 2048, 4096, 8192];
    let mut arenas = Vec::new();
    
    for size in &allocation_sizes {
        let arena = arena_manager.create_arena(*size)
            .expect("Failed to create arena");
        
        // Use the arena to ensure memory is committed
        let data: &mut [u8] = arena.alloc_slice_fill_default(*size);
        data.fill(0xFF);
        
        arenas.push(arena);
    }
    
    // Check metrics
    let after_allocation = arena_manager.get_memory_metrics();
    
    let expected_total: usize = allocation_sizes.iter().sum();
    let actual_growth = after_allocation.total_allocated - baseline.total_allocated;
    
    // Allow some overhead but should be reasonably close
    assert!(actual_growth >= expected_total,
           "Reported allocation ({}) less than expected ({})", 
           actual_growth, expected_total);
    
    assert!(actual_growth <= expected_total * 2,
           "Reported allocation ({}) much larger than expected ({})", 
           actual_growth, expected_total);
    
    // Arena count should match
    assert_eq!(after_allocation.arena_count - baseline.arena_count, 
              allocation_sizes.len());
}

#[cfg(test)]
mod helpers {
    use super::*;
    use uveddi::ast::{AstNode, AstNodeType};
    
    /// Create a large test AST for performance testing
    pub fn create_large_test_ast(node_count: usize) -> TestAst {
        let mut ast = TestAst::new();
        
        for i in 0..node_count {
            let node = AstNode {
                id: i,
                node_type: match i % 5 {
                    0 => AstNodeType::Function,
                    1 => AstNodeType::Struct,
                    2 => AstNodeType::Variable,
                    3 => AstNodeType::Expression,
                    _ => AstNodeType::Statement,
                },
                start_byte: i * 10,
                end_byte: (i * 10) + 5,
                text: format!("node_{}", i),
                children: vec![],
            };
            ast.add_node(node);
        }
        
        ast
    }
    
    #[derive(Debug, Clone)]
    pub struct TestAst {
        nodes: Vec<AstNode>,
        checksum: u64,
    }
    
    impl TestAst {
        pub fn new() -> Self {
            Self {
                nodes: Vec::new(),
                checksum: 0,
            }
        }
        
        pub fn add_node(&mut self, node: AstNode) {
            self.nodes.push(node);
            self.update_checksum();
        }
        
        pub fn node_count(&self) -> usize {
            self.nodes.len()
        }
        
        pub fn checksum(&self) -> u64 {
            self.checksum
        }
        
        fn update_checksum(&mut self) {
            // Simple checksum based on node count and content
            self.checksum = self.nodes.len() as u64 * 17 + 
                           self.nodes.iter().map(|n| n.id as u64).sum::<u64>();
        }
    }
    
    /// Memory allocation tracker for testing
    pub struct AllocationTracker {
        allocations: std::sync::Mutex<Vec<(usize, std::time::Instant)>>,
    }
    
    impl AllocationTracker {
        pub fn new() -> Self {
            Self {
                allocations: std::sync::Mutex::new(Vec::new()),
            }
        }
        
        pub fn track_allocation(&self, size: usize) {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.push((size, Instant::now()));
        }
        
        pub fn get_total_allocated(&self) -> usize {
            let allocations = self.allocations.lock().unwrap();
            allocations.iter().map(|(size, _)| *size).sum()
        }
        
        pub fn get_allocation_count(&self) -> usize {
            let allocations = self.allocations.lock().unwrap();
            allocations.len()
        }
    }
}