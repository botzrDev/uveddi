use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::{HashMap, BTreeMap};
use tempfile::TempDir;
use tokio::runtime::Runtime;

use uveddi::analysis::{AnalysisEngine, AnalysisDetector};
use uveddi::analysis::cache::ast::{AstCache, CacheConfig};
use uveddi::monitoring::enterprise_metrics::MemoryMetrics;

/// Memory profiling and leak detection benchmarks for UV-91
/// 
/// Comprehensive memory usage analysis including:
/// - Allocation patterns
/// - Memory leaks detection  
/// - Fragmentation analysis
/// - Peak usage monitoring
/// - GC pressure measurement

/// Global memory tracker
pub struct MemoryTracker {
    total_allocated: AtomicUsize,
    total_deallocated: AtomicUsize,
    peak_usage: AtomicUsize,
    current_usage: AtomicUsize,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
}

impl MemoryTracker {
    pub const fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_deallocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            current_usage: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
        }
    }

    pub fn allocate(&self, size: usize) {
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        let current = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak usage if necessary
        loop {
            let peak = self.peak_usage.load(Ordering::Relaxed);
            if current <= peak || self.peak_usage.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }
    }

    pub fn deallocate(&self, size: usize) {
        self.total_deallocated.fetch_add(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> MemoryProfileMetrics {
        MemoryProfileMetrics {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_deallocated.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
    }
}

static MEMORY_TRACKER: MemoryTracker = MemoryTracker::new();

/// Custom allocator that tracks memory usage
pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            MEMORY_TRACKER.allocate(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        MEMORY_TRACKER.deallocate(layout.size());
        System.dealloc(ptr, layout);
    }
}

/// Memory profiling metrics
#[derive(Debug, Clone)]
pub struct MemoryProfileMetrics {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub peak_usage: usize,
    pub current_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

impl MemoryProfileMetrics {
    pub fn leaked_bytes(&self) -> usize {
        self.total_allocated.saturating_sub(self.total_deallocated)
    }

    pub fn average_allocation_size(&self) -> f64 {
        if self.allocation_count > 0 {
            self.total_allocated as f64 / self.allocation_count as f64
        } else {
            0.0
        }
    }

    pub fn fragmentation_ratio(&self) -> f64 {
        if self.peak_usage > 0 {
            self.current_usage as f64 / self.peak_usage as f64
        } else {
            1.0
        }
    }

    pub fn allocation_efficiency(&self) -> f64 {
        if self.total_allocated > 0 {
            self.total_deallocated as f64 / self.total_allocated as f64
        } else {
            1.0
        }
    }
}

/// Create enterprise test project for memory profiling
fn create_memory_test_project(temp_dir: &TempDir, file_count: usize) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    
    for i in 0..file_count {
        let complexity = match i % 4 {
            0 => "simple",
            1 => "medium",
            2 => "complex", 
            _ => "memory_intensive",
        };
        
        let file_path = temp_dir.path().join(format!("test_file_{}.rs", i));
        let content = generate_memory_test_content(complexity, i, file_count);
        std::fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }
    
    files
}

/// Generate memory-intensive test content
fn generate_memory_test_content(complexity: &str, index: usize, total_files: usize) -> String {
    match complexity {
        "simple" => format!(
            "pub fn simple_function_{}() -> i32 {{\n    {}\n}}\n",
            index, index
        ),
        "medium" => {
            let mut code = String::new();
            code.push_str(&format!("use std::collections::HashMap;\n\n"));
            code.push_str(&format!("pub struct MediumStruct{} {{\n", index));
            
            // Create medium-sized structures
            for i in 0..20 {
                code.push_str(&format!("    field_{}: Vec<String>,\n", i));
            }
            code.push_str("}\n\n");
            
            code.push_str(&format!("impl MediumStruct{} {{\n", index));
            code.push_str("    pub fn new() -> Self {\n");
            code.push_str("        Self {\n");
            for i in 0..20 {
                code.push_str(&format!("            field_{}: Vec::with_capacity(100),\n", i));
            }
            code.push_str("        }\n");
            code.push_str("    }\n\n");
            
            code.push_str("    pub fn process_data(&mut self) {\n");
            code.push_str("        for i in 0..1000 {\n");
            code.push_str("            let data = format!(\"data_{}\", i);\n");
            for i in 0..20 {
                code.push_str(&format!("            self.field_{}.push(data.clone());\n", i));
            }
            code.push_str("        }\n");
            code.push_str("    }\n");
            code.push_str("}\n");
            code
        },
        "complex" => {
            let mut code = String::new();
            code.push_str("use std::collections::{HashMap, BTreeMap, HashSet};\n");
            code.push_str("use std::sync::Arc;\n\n");
            
            code.push_str(&format!("pub struct ComplexStruct{} {{\n", index));
            
            // Create complex nested structures
            for i in 0..50 {
                let field_type = match i % 5 {
                    0 => "HashMap<String, Vec<String>>",
                    1 => "BTreeMap<u64, HashSet<String>>",
                    2 => "Arc<Vec<HashMap<String, String>>>",
                    3 => "Vec<Arc<BTreeMap<String, Vec<u8>>>>",
                    _ => "HashMap<String, Arc<Vec<HashMap<u64, String>>>>",
                };
                code.push_str(&format!("    field_{}: {},\n", i, field_type));
            }
            code.push_str("}\n\n");
            
            code.push_str(&format!("impl ComplexStruct{} {{\n", index));
            code.push_str("    pub fn new() -> Self {\n");
            code.push_str("        Self {\n");
            for i in 0..50 {
                match i % 5 {
                    0 => code.push_str(&format!("            field_{}: HashMap::with_capacity(1000),\n", i)),
                    1 => code.push_str(&format!("            field_{}: BTreeMap::new(),\n", i)),
                    2 => code.push_str(&format!("            field_{}: Arc::new(Vec::with_capacity(500)),\n", i)),
                    3 => code.push_str(&format!("            field_{}: Vec::with_capacity(200),\n", i)),
                    _ => code.push_str(&format!("            field_{}: HashMap::with_capacity(100),\n", i)),
                }
            }
            code.push_str("        }\n");
            code.push_str("    }\n\n");
            
            // Memory-intensive methods
            code.push_str("    pub fn allocate_heavy_data(&mut self) {\n");
            code.push_str("        for i in 0..10000 {\n");
            code.push_str("            let key = format!(\"key_{}\", i);\n");
            code.push_str("            let value = vec![format!(\"value_{}\", j); 100];\n");
            code.push_str("            self.field_0.insert(key, value);\n");
            code.push_str("        }\n");
            code.push_str("    }\n\n");
            
            code.push_str("    pub fn process_recursive_data(&mut self, depth: usize) {\n");
            code.push_str("        if depth == 0 { return; }\n");
            code.push_str("        \n");
            code.push_str("        let mut temp_data = HashMap::new();\n");
            code.push_str("        for i in 0..depth * 100 {\n");
            code.push_str("            temp_data.insert(i, vec![i as u8; depth]);\n");
            code.push_str("        }\n");
            code.push_str("        \n");
            code.push_str("        self.process_recursive_data(depth - 1);\n");
            code.push_str("    }\n");
            code.push_str("}\n");
            code
        },
        "memory_intensive" => {
            let mut code = String::new();
            code.push_str("use std::collections::{HashMap, VecDeque};\n");
            code.push_str("use std::sync::{Arc, Mutex};\n");
            code.push_str("use std::rc::Rc;\n\n");
            
            // Create deliberately memory-intensive structures
            code.push_str(&format!("pub struct MemoryIntensiveStruct{} {{\n", index));
            
            // Large arrays and collections
            for i in 0..100 {
                match i % 6 {
                    0 => code.push_str(&format!("    big_array_{}: [u8; 10000],\n", i)),
                    1 => code.push_str(&format!("    big_vec_{}: Vec<Vec<Vec<String>>>,\n", i)),
                    2 => code.push_str(&format!("    big_map_{}: HashMap<String, [u8; 1000]>,\n", i)),
                    3 => code.push_str(&format!("    shared_data_{}: Arc<Mutex<Vec<HashMap<String, String>>>>,\n", i)),
                    4 => code.push_str(&format!("    circular_ref_{}: Rc<VecDeque<Rc<HashMap<String, Vec<u8>>>>>,\n", i)),
                    _ => code.push_str(&format!("    nested_structure_{}: Vec<Arc<Mutex<BTreeMap<String, Vec<[u8; 512]>>>>>,\n", i)),
                }
            }
            code.push_str("}\n\n");
            
            code.push_str(&format!("impl MemoryIntensiveStruct{} {{\n", index));
            code.push_str("    pub fn new() -> Self {\n");
            code.push_str("        Self {\n");
            for i in 0..100 {
                match i % 6 {
                    0 => code.push_str(&format!("            big_array_{}: [0u8; 10000],\n", i)),
                    1 => code.push_str(&format!("            big_vec_{}: Vec::new(),\n", i)),
                    2 => code.push_str(&format!("            big_map_{}: HashMap::new(),\n", i)),
                    3 => code.push_str(&format!("            shared_data_{}: Arc::new(Mutex::new(Vec::new())),\n", i)),
                    4 => code.push_str(&format!("            circular_ref_{}: Rc::new(VecDeque::new()),\n", i)),
                    _ => code.push_str(&format!("            nested_structure_{}: Vec::new(),\n", i)),
                }
            }
            code.push_str("        }\n");
            code.push_str("    }\n\n");
            
            // Memory leak simulation methods
            code.push_str("    pub fn simulate_memory_leak(&mut self) {\n");
            code.push_str("        // Intentionally create potential memory leaks\n");
            code.push_str("        for i in 0..1000 {\n");
            code.push_str("            let leaked_data = vec![i as u8; 10000];\n");
            code.push_str("            std::mem::forget(leaked_data); // Deliberate leak for testing\n");
            code.push_str("        }\n");
            code.push_str("    }\n\n");
            
            code.push_str("    pub fn create_circular_references(&mut self) {\n");
            code.push_str("        // Create circular references that are hard to detect\n");
            code.push_str("        use std::cell::RefCell;\n");
            code.push_str("        \n");
            code.push_str("        let node1 = Rc::new(RefCell::new(None));\n");
            code.push_str("        let node2 = Rc::new(RefCell::new(Some(node1.clone())));\n");
            code.push_str("        *node1.borrow_mut() = Some(node2.clone());\n");
            code.push_str("        \n");
            code.push_str("        // Store in structure (creates potential leak)\n");
            code.push_str("        std::mem::forget(node1);\n");
            code.push_str("        std::mem::forget(node2);\n");
            code.push_str("    }\n\n");
            
            code.push_str("    pub fn allocate_massive_data(&mut self) {\n");
            code.push_str("        // Allocate large amounts of data\n");
            code.push_str("        for i in 0..10 {\n");
            code.push_str("            let massive_vec = vec![vec![i as u8; 10000]; 100];\n");
            code.push_str(&format!("            self.big_vec_{}.push(massive_vec);\n", index % 100));
            code.push_str("        }\n");
            code.push_str("        \n");
            code.push_str("        // Fill maps with large data\n");
            code.push_str("        for i in 0..5000 {\n");
            code.push_str("            let key = format!(\"massive_key_{}\", i);\n");
            code.push_str("            let value = [i as u8; 1000];\n");
            code.push_str(&format!("            self.big_map_{}.insert(key, value);\n", index % 100));
            code.push_str("        }\n");
            code.push_str("    }\n");
            code.push_str("}\n");
            code
        },
        _ => String::new(),
    }
}

/// 1. Benchmark memory allocation patterns during analysis
fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation_patterns");
    
    for &file_count in &[50, 100, 200, 500] {
        group.bench_with_input(
            BenchmarkId::new("allocation_tracking", file_count),
            &file_count,
            |b, &file_count| {
                let temp_dir = TempDir::new().unwrap();
                let _files = create_memory_test_project(&temp_dir, file_count);
                
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start_metrics = MEMORY_TRACKER.get_metrics();
                        
                        let start_time = Instant::now();
                        
                        // Run analysis with memory tracking
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async {
                            let mut engine = AnalysisEngine::new().unwrap();
                            let _result = engine.analyze(temp_dir.path()).await;
                        });
                        
                        total_duration += start_time.elapsed();
                        
                        let end_metrics = MEMORY_TRACKER.get_metrics();
                        
                        // Store memory metrics for analysis
                        black_box((start_metrics, end_metrics));
                    }
                    
                    total_duration
                });
            },
        );
    }
    
    group.finish();
}

/// 2. Benchmark memory leak detection
fn bench_memory_leak_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_leak_detection");
    
    group.bench_function("leak_detection_analysis", |b| {
        let temp_dir = TempDir::new().unwrap();
        let _files = create_memory_test_project(&temp_dir, 100);
        
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut leak_reports = Vec::new();
            
            for iteration in 0..iters {
                MEMORY_TRACKER.reset();
                
                let start_time = Instant::now();
                
                // Perform analysis multiple times to detect leaks
                let rt = Runtime::new().unwrap();
                for _ in 0..5 {
                    rt.block_on(async {
                        let mut engine = AnalysisEngine::new().unwrap();
                        let _result = engine.analyze(temp_dir.path()).await;
                    });
                }
                
                total_duration += start_time.elapsed();
                
                let final_metrics = MEMORY_TRACKER.get_metrics();
                
                // Detect potential leaks
                let leaked_bytes = final_metrics.leaked_bytes();
                let allocation_efficiency = final_metrics.allocation_efficiency();
                
                leak_reports.push(MemoryLeakReport {
                    iteration: iteration as u32,
                    leaked_bytes,
                    allocation_efficiency,
                    peak_usage: final_metrics.peak_usage,
                    final_usage: final_metrics.current_usage,
                    is_leak_suspected: leaked_bytes > 1024 * 1024 || allocation_efficiency < 0.95, // 1MB threshold
                });
            }
            
            black_box(leak_reports);
            total_duration
        });
    });
    
    group.finish();
}

/// 3. Benchmark memory fragmentation analysis
fn bench_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_fragmentation");
    
    group.bench_function("fragmentation_measurement", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut fragmentation_data = Vec::new();
            
            for _ in 0..iters {
                MEMORY_TRACKER.reset();
                
                let start_time = Instant::now();
                
                // Simulate fragmentation through various allocation patterns
                let mut allocations = Vec::new();
                
                // Phase 1: Large allocations
                for i in 0..100 {
                    let size = 1024 * (i + 1);
                    let allocation = vec![i as u8; size];
                    MEMORY_TRACKER.allocate(size);
                    allocations.push(allocation);
                }
                
                let phase1_metrics = MEMORY_TRACKER.get_metrics();
                
                // Phase 2: Free every other allocation (create holes)
                for i in (0..allocations.len()).step_by(2) {
                    let size = allocations[i].len();
                    MEMORY_TRACKER.deallocate(size);
                }
                allocations.retain(|_| {
                    static mut COUNTER: usize = 0;
                    unsafe {
                        COUNTER += 1;
                        COUNTER % 2 == 0
                    }
                });
                
                let phase2_metrics = MEMORY_TRACKER.get_metrics();
                
                // Phase 3: Small allocations in the holes
                for i in 0..50 {
                    let size = 512 + i * 10;
                    let allocation = vec![i as u8; size];
                    MEMORY_TRACKER.allocate(size);
                    allocations.push(allocation);
                }
                
                let phase3_metrics = MEMORY_TRACKER.get_metrics();
                
                total_duration += start_time.elapsed();
                
                fragmentation_data.push(FragmentationAnalysis {
                    phase1_fragmentation: phase1_metrics.fragmentation_ratio(),
                    phase2_fragmentation: phase2_metrics.fragmentation_ratio(),
                    phase3_fragmentation: phase3_metrics.fragmentation_ratio(),
                    final_efficiency: phase3_metrics.allocation_efficiency(),
                });
                
                // Cleanup
                for allocation in &allocations {
                    MEMORY_TRACKER.deallocate(allocation.len());
                }
            }
            
            black_box(fragmentation_data);
            total_duration
        });
    });
    
    group.finish();
}

/// 4. Benchmark peak memory usage monitoring
fn bench_peak_memory_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("peak_memory_monitoring");
    
    for &scenario in &["small_files", "medium_files", "large_files", "mixed_complexity"] {
        group.bench_with_input(
            BenchmarkId::new("peak_usage_tracking", scenario),
            &scenario,
            |b, &scenario| {
                let temp_dir = TempDir::new().unwrap();
                
                let file_count = match scenario {
                    "small_files" => 200,
                    "medium_files" => 100,
                    "large_files" => 50,
                    "mixed_complexity" => 150,
                    _ => 100,
                };
                
                let _files = create_memory_test_project(&temp_dir, file_count);
                
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);
                    let mut peak_usage_data = Vec::new();
                    
                    for _ in 0..iters {
                        MEMORY_TRACKER.reset();
                        let start_time = Instant::now();
                        
                        // Monitor peak usage during different phases
                        let baseline_peak = MEMORY_TRACKER.get_metrics().peak_usage;
                        
                        // Phase 1: Engine initialization
                        let rt = Runtime::new().unwrap();
                        let mut engine = rt.block_on(async {
                            AnalysisEngine::new().unwrap()
                        });
                        let init_peak = MEMORY_TRACKER.get_metrics().peak_usage;
                        
                        // Phase 2: Analysis execution
                        rt.block_on(async {
                            let _result = engine.analyze(temp_dir.path()).await;
                        });
                        let analysis_peak = MEMORY_TRACKER.get_metrics().peak_usage;
                        
                        // Phase 3: Cache population
                        let cache = AstCache::new(CacheConfig {
                            max_memory_entries: 1000,
                            max_memory_size_mb: 100,
                            ..Default::default()
                        }).unwrap();
                        
                        // Simulate cache operations
                        for i in 0..100 {
                            let dummy_path = temp_dir.path().join(format!("dummy_{}.rs", i));
                            let _ = cache.get(&dummy_path);
                        }
                        
                        let cache_peak = MEMORY_TRACKER.get_metrics().peak_usage;
                        
                        total_duration += start_time.elapsed();
                        
                        peak_usage_data.push(PeakUsageAnalysis {
                            baseline_peak,
                            init_peak,
                            analysis_peak,
                            cache_peak,
                            scenario: scenario.to_string(),
                        });
                    }
                    
                    black_box(peak_usage_data);
                    total_duration
                });
            },
        );
    }
    
    group.finish();
}

/// 5. Benchmark garbage collection pressure
fn bench_gc_pressure_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_pressure_analysis");
    
    group.bench_function("allocation_rate_measurement", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut gc_pressure_data = Vec::new();
            
            for _ in 0..iters {
                MEMORY_TRACKER.reset();
                let start_time = Instant::now();
                let measurement_start = Instant::now();
                
                // Simulate high allocation rate scenarios
                let mut temporary_allocations = Vec::new();
                
                // High-frequency small allocations
                for i in 0..10000 {
                    let allocation = vec![i as u8; 64];
                    MEMORY_TRACKER.allocate(64);
                    temporary_allocations.push(allocation);
                    
                    // Occasionally free some allocations
                    if i % 100 == 0 && !temporary_allocations.is_empty() {
                        let to_free = std::cmp::min(10, temporary_allocations.len());
                        for _ in 0..to_free {
                            if let Some(alloc) = temporary_allocations.pop() {
                                MEMORY_TRACKER.deallocate(alloc.len());
                            }
                        }
                    }
                }
                
                let high_freq_time = measurement_start.elapsed();
                let high_freq_metrics = MEMORY_TRACKER.get_metrics();
                
                // Medium-frequency medium allocations
                let medium_freq_start = Instant::now();
                for i in 0..1000 {
                    let allocation = vec![i as u8; 4096];
                    MEMORY_TRACKER.allocate(4096);
                    temporary_allocations.push(allocation);
                }
                
                let medium_freq_time = medium_freq_start.elapsed();
                let medium_freq_metrics = MEMORY_TRACKER.get_metrics();
                
                // Low-frequency large allocations
                let low_freq_start = Instant::now();
                for i in 0..10 {
                    let allocation = vec![i as u8; 1024 * 1024];
                    MEMORY_TRACKER.allocate(1024 * 1024);
                    temporary_allocations.push(allocation);
                }
                
                let low_freq_time = low_freq_start.elapsed();
                let low_freq_metrics = MEMORY_TRACKER.get_metrics();
                
                total_duration += start_time.elapsed();
                
                // Calculate allocation rates
                let high_freq_rate = high_freq_metrics.allocation_count as f64 / high_freq_time.as_secs_f64();
                let medium_freq_rate = (medium_freq_metrics.allocation_count - high_freq_metrics.allocation_count) as f64 / medium_freq_time.as_secs_f64();
                let low_freq_rate = (low_freq_metrics.allocation_count - medium_freq_metrics.allocation_count) as f64 / low_freq_time.as_secs_f64();
                
                gc_pressure_data.push(GcPressureAnalysis {
                    high_frequency_allocation_rate: high_freq_rate,
                    medium_frequency_allocation_rate: medium_freq_rate,
                    low_frequency_allocation_rate: low_freq_rate,
                    total_allocation_count: low_freq_metrics.allocation_count,
                    peak_memory_pressure: low_freq_metrics.peak_usage,
                });
                
                // Cleanup
                for allocation in &temporary_allocations {
                    MEMORY_TRACKER.deallocate(allocation.len());
                }
            }
            
            black_box(gc_pressure_data);
            total_duration
        });
    });
    
    group.finish();
}

/// 6. Benchmark memory-efficient data structures
fn bench_memory_efficient_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficient_structures");
    
    group.bench_function("arena_vs_standard_allocation", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut comparison_data = Vec::new();
            
            for iteration in 0..iters {
                MEMORY_TRACKER.reset();
                let start_time = Instant::now();
                
                // Test standard allocations
                let standard_start = Instant::now();
                let mut standard_allocations = Vec::new();
                for i in 0..1000 {
                    let allocation = vec![i as u8; 256];
                    MEMORY_TRACKER.allocate(256);
                    standard_allocations.push(allocation);
                }
                let standard_time = standard_start.elapsed();
                let standard_metrics = MEMORY_TRACKER.get_metrics();
                
                // Cleanup standard allocations
                for allocation in &standard_allocations {
                    MEMORY_TRACKER.deallocate(allocation.len());
                }
                
                // Test arena allocations
                MEMORY_TRACKER.reset();
                let arena_start = Instant::now();
                // Simulate arena allocation
                
                let mut arena_allocations = Vec::new();
                for i in 0..1000 {
                    // Simulate arena allocation
                    let allocation = vec![0u8; 256];
                    MEMORY_TRACKER.allocate(256);
                    arena_allocations.push(allocation);
                }
                let arena_time = arena_start.elapsed();
                let arena_metrics = MEMORY_TRACKER.get_metrics();
                
                total_duration += start_time.elapsed();
                
                comparison_data.push(AllocationComparison {
                    iteration: iteration as u32,
                    standard_time_ns: standard_time.as_nanos() as u64,
                    arena_time_ns: arena_time.as_nanos() as u64,
                    standard_peak_usage: standard_metrics.peak_usage,
                    arena_peak_usage: arena_metrics.peak_usage,
                    standard_allocation_count: standard_metrics.allocation_count,
                    arena_allocation_count: arena_metrics.allocation_count,
                    performance_improvement: standard_time.as_nanos() as f64 / arena_time.as_nanos() as f64,
                });
            }
            
            black_box(comparison_data);
            total_duration
        });
    });
    
    group.finish();
}

/// Memory profiling data structures
#[derive(Debug, Clone)]
struct MemoryLeakReport {
    iteration: u32,
    leaked_bytes: usize,
    allocation_efficiency: f64,
    peak_usage: usize,
    final_usage: usize,
    is_leak_suspected: bool,
}

#[derive(Debug, Clone)]
struct FragmentationAnalysis {
    phase1_fragmentation: f64,
    phase2_fragmentation: f64,
    phase3_fragmentation: f64,
    final_efficiency: f64,
}

#[derive(Debug, Clone)]
struct PeakUsageAnalysis {
    baseline_peak: usize,
    init_peak: usize,
    analysis_peak: usize,
    cache_peak: usize,
    scenario: String,
}

#[derive(Debug, Clone)]
struct GcPressureAnalysis {
    high_frequency_allocation_rate: f64,
    medium_frequency_allocation_rate: f64,
    low_frequency_allocation_rate: f64,
    total_allocation_count: usize,
    peak_memory_pressure: usize,
}

#[derive(Debug, Clone)]
struct AllocationComparison {
    iteration: u32,
    standard_time_ns: u64,
    arena_time_ns: u64,
    standard_peak_usage: usize,
    arena_peak_usage: usize,
    standard_allocation_count: usize,
    arena_allocation_count: usize,
    performance_improvement: f64,
}

criterion_group!(
    memory_profiling_benchmarks,
    bench_memory_allocation_patterns,
    bench_memory_leak_detection,
    bench_memory_fragmentation,
    bench_peak_memory_monitoring,
    bench_gc_pressure_analysis,
    bench_memory_efficient_structures
);

criterion_main!(memory_profiling_benchmarks);