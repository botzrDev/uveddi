//! Large code sample for memory pressure testing

use std::collections::{HashMap, BTreeMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeDataStructure {
    pub id: u64,
    pub name: String,
    pub metadata: HashMap<String, String>,
    pub values: Vec<f64>,
    pub nested_data: BTreeMap<String, NestedStructure>,
    pub flags: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedStructure {
    pub inner_id: u32,
    pub data: Vec<u8>,
    pub mapping: HashMap<u32, String>,
    pub optional_field: Option<Box<NestedStructure>>,
}

pub trait DataProcessor {
    fn process_data(&self, input: &[u8]) -> Result<Vec<u8>, ProcessingError>;
    fn validate_structure(&self, structure: &LargeDataStructure) -> bool;
    fn optimize_performance(&mut self) -> Result<(), OptimizationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Memory allocation error")]
    MemoryError,
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("Optimization not available")]
    NotAvailable,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub struct ComplexProcessor {
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    config: ProcessorConfig,
    workers: Vec<thread::JoinHandle<()>>,
    metrics: Arc<Mutex<ProcessingMetrics>>,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub worker_count: usize,
    pub cache_size: usize,
    pub timeout: Duration,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Extreme,
}

#[derive(Debug, Default)]
pub struct ProcessingMetrics {
    pub total_processed: u64,
    pub errors_count: u64,
    pub average_processing_time: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ComplexProcessor {
    pub fn new(config: ProcessorConfig) -> Self {
        let cache = Arc::new(RwLock::new(HashMap::with_capacity(config.cache_size)));
        let metrics = Arc::new(Mutex::new(ProcessingMetrics::default()));
        
        Self {
            cache,
            config,
            workers: Vec::new(),
            metrics,
        }
    }

    pub fn start_workers(&mut self) {
        for i in 0..self.config.worker_count {
            let cache = Arc::clone(&self.cache);
            let metrics = Arc::clone(&self.metrics);
            let timeout = self.config.timeout;
            
            let handle = thread::spawn(move || {
                worker_loop(i, cache, metrics, timeout);
            });
            
            self.workers.push(handle);
        }
    }

    pub fn process_large_dataset(&self, dataset: &[LargeDataStructure]) -> Result<Vec<ProcessingResult>, ProcessingError> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(dataset.len());
        
        for (index, structure) in dataset.iter().enumerate() {
            let result = self.process_single_structure(structure, index)?;
            results.push(result);
            
            // Simulate complex processing
            if index % 1000 == 0 {
                thread::sleep(Duration::from_millis(1));
            }
        }
        
        let processing_time = start_time.elapsed();
        self.update_metrics(dataset.len(), processing_time);
        
        Ok(results)
    }

    fn process_single_structure(&self, structure: &LargeDataStructure, index: usize) -> Result<ProcessingResult, ProcessingError> {
        // Complex processing logic
        let cache_key = format!("structure_{}_{}", structure.id, index);
        
        // Check cache first
        if let Ok(cache) = self.cache.read() {
            if let Some(cached_result) = cache.get(&cache_key) {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.cache_hits += 1;
                
                return Ok(ProcessingResult {
                    id: structure.id,
                    processed_data: cached_result.clone(),
                    metadata: create_result_metadata(structure),
                    processing_time: Duration::from_nanos(100), // Cached result
                });
            }
        }
        
        // Cache miss - perform actual processing
        let start = Instant::now();
        let processed_data = self.perform_complex_processing(structure)?;
        let processing_time = start.elapsed();
        
        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            if cache.len() < self.config.cache_size {
                cache.insert(cache_key, processed_data.clone());
            }
        }
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cache_misses += 1;
        
        Ok(ProcessingResult {
            id: structure.id,
            processed_data,
            metadata: create_result_metadata(structure),
            processing_time,
        })
    }

    fn perform_complex_processing(&self, structure: &LargeDataStructure) -> Result<Vec<u8>, ProcessingError> {
        let mut result = Vec::new();
        
        // Serialize the structure
        let serialized = bincode::serialize(structure)
            .map_err(|_| ProcessingError::ProcessingFailed("Serialization failed".to_string()))?;
        
        // Apply transformations based on optimization level
        match self.config.optimization_level {
            OptimizationLevel::None => {
                result = serialized;
            }
            OptimizationLevel::Basic => {
                result = apply_basic_compression(&serialized)?;
            }
            OptimizationLevel::Aggressive => {
                result = apply_aggressive_optimization(&serialized)?;
            }
            OptimizationLevel::Extreme => {
                result = apply_extreme_optimization(&serialized)?;
            }
        }
        
        // Simulate CPU-intensive work
        let _ = perform_cpu_intensive_calculation(&result);
        
        Ok(result)
    }

    fn update_metrics(&self, processed_count: usize, total_time: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_processed += processed_count as u64;
            
            let avg_time = total_time / processed_count as u32;
            metrics.average_processing_time = 
                if metrics.total_processed == processed_count as u64 {
                    avg_time
                } else {
                    Duration::from_nanos(
                        (metrics.average_processing_time.as_nanos() + avg_time.as_nanos()) / 2
                    )
                };
        }
    }
}

impl DataProcessor for ComplexProcessor {
    fn process_data(&self, input: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        if input.is_empty() {
            return Err(ProcessingError::InvalidInput);
        }
        
        // Simulate complex data processing
        let mut processed = Vec::with_capacity(input.len() * 2);
        
        for chunk in input.chunks(1024) {
            let transformed = transform_chunk(chunk)?;
            processed.extend(transformed);
        }
        
        Ok(processed)
    }

    fn validate_structure(&self, structure: &LargeDataStructure) -> bool {
        // Complex validation logic
        if structure.name.is_empty() {
            return false;
        }
        
        if structure.values.len() > 10000 {
            return false;
        }
        
        for (key, nested) in &structure.nested_data {
            if key.len() > 255 {
                return false;
            }
            
            if !validate_nested_structure(nested) {
                return false;
            }
        }
        
        true
    }

    fn optimize_performance(&mut self) -> Result<(), OptimizationError> {
        // Simulate performance optimization
        if self.workers.is_empty() {
            return Err(OptimizationError::NotAvailable);
        }
        
        // Adjust cache size based on usage patterns
        if let Ok(metrics) = self.metrics.lock() {
            let hit_rate = metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64;
            
            if hit_rate < 0.5 {
                // Increase cache size
                self.config.cache_size *= 2;
            } else if hit_rate > 0.9 {
                // Decrease cache size to save memory
                self.config.cache_size = (self.config.cache_size * 0.8) as usize;
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub id: u64,
    pub processed_data: Vec<u8>,
    pub metadata: ResultMetadata,
    pub processing_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ResultMetadata {
    pub original_size: usize,
    pub processed_size: usize,
    pub compression_ratio: f64,
    pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Valid,
    Warning(String),
    Invalid(String),
}

// Helper functions

fn worker_loop(
    worker_id: usize,
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    metrics: Arc<Mutex<ProcessingMetrics>>,
    timeout: Duration,
) {
    loop {
        // Simulate worker processing
        thread::sleep(Duration::from_millis(100));
        
        // Periodically clean up cache
        if worker_id == 0 && rand::random::<u8>() % 100 == 0 {
            if let Ok(mut cache) = cache.write() {
                if cache.len() > 10000 {
                    cache.clear();
                }
            }
        }
        
        // Check for shutdown condition
        if timeout.as_secs() == 0 {
            break;
        }
    }
}

fn apply_basic_compression(data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|_| ProcessingError::ProcessingFailed("Compression failed".to_string()))?;
    
    encoder.finish()
        .map_err(|_| ProcessingError::ProcessingFailed("Compression finalization failed".to_string()))
}

fn apply_aggressive_optimization(data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
    // First apply compression
    let compressed = apply_basic_compression(data)?;
    
    // Then apply additional optimizations
    let mut optimized = Vec::new();
    
    for chunk in compressed.chunks(64) {
        // Simulate complex optimization algorithm
        let optimized_chunk = chunk.iter()
            .enumerate()
            .map(|(i, &byte)| byte.wrapping_add(i as u8))
            .collect::<Vec<_>>();
        
        optimized.extend(optimized_chunk);
    }
    
    Ok(optimized)
}

fn apply_extreme_optimization(data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
    // Start with aggressive optimization
    let aggressive = apply_aggressive_optimization(data)?;
    
    // Apply extreme transformations
    let mut extreme = Vec::new();
    
    // Complex mathematical transformations
    for window in aggressive.windows(4) {
        let sum: u32 = window.iter().map(|&b| b as u32).sum();
        let transformed = (sum % 256) as u8;
        extreme.push(transformed);
    }
    
    // Apply additional complexity
    for i in 0..extreme.len() {
        extreme[i] = extreme[i].wrapping_mul(3).wrapping_add(7);
    }
    
    Ok(extreme)
}

fn transform_chunk(chunk: &[u8]) -> Result<Vec<u8>, ProcessingError> {
    let mut transformed = Vec::with_capacity(chunk.len() * 2);
    
    for &byte in chunk {
        // Complex transformation
        let transform1 = byte.wrapping_mul(3);
        let transform2 = transform1.wrapping_add(7);
        let transform3 = transform2 ^ 0xAA;
        
        transformed.push(transform2);
        transformed.push(transform3);
    }
    
    Ok(transformed)
}

fn validate_nested_structure(nested: &NestedStructure) -> bool {
    if nested.data.len() > 1024 * 1024 {
        return false;
    }
    
    if nested.mapping.len() > 1000 {
        return false;
    }
    
    // Recursive validation
    if let Some(ref inner) = nested.optional_field {
        return validate_nested_structure(inner);
    }
    
    true
}

fn create_result_metadata(structure: &LargeDataStructure) -> ResultMetadata {
    let original_size = bincode::serialized_size(structure).unwrap_or(0) as usize;
    
    ResultMetadata {
        original_size,
        processed_size: original_size * 2, // Simulated
        compression_ratio: 0.75,
        validation_status: if structure.values.len() < 1000 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Warning("Large dataset".to_string())
        },
    }
}

fn perform_cpu_intensive_calculation(data: &[u8]) -> u64 {
    let mut result = 0u64;
    
    for (i, &byte) in data.iter().enumerate() {
        result = result.wrapping_add((byte as u64).wrapping_mul(i as u64));
        
        // Add some complexity
        if i % 100 == 0 {
            result = result.wrapping_mul(1664525).wrapping_add(1013904223);
        }
    }
    
    result
}

// Factory functions for testing

pub fn create_large_test_structure(id: u64, size_factor: usize) -> LargeDataStructure {
    let mut metadata = HashMap::new();
    for i in 0..size_factor * 10 {
        metadata.insert(
            format!("key_{}", i),
            format!("value_{}_with_longer_content_for_memory_pressure", i),
        );
    }
    
    let values: Vec<f64> = (0..size_factor * 1000)
        .map(|i| i as f64 * 1.5 + 0.7)
        .collect();
    
    let mut nested_data = BTreeMap::new();
    for i in 0..size_factor * 5 {
        let nested = NestedStructure {
            inner_id: i as u32,
            data: vec![0u8; size_factor * 100],
            mapping: (0..size_factor * 10)
                .map(|j| (j as u32, format!("nested_value_{}", j)))
                .collect(),
            optional_field: if i % 3 == 0 {
                Some(Box::new(NestedStructure {
                    inner_id: (i + 1000) as u32,
                    data: vec![255u8; size_factor * 50],
                    mapping: HashMap::new(),
                    optional_field: None,
                }))
            } else {
                None
            },
        };
        
        nested_data.insert(format!("nested_{}", i), nested);
    }
    
    let flags: HashSet<String> = (0..size_factor * 20)
        .map(|i| format!("flag_{}", i))
        .collect();
    
    LargeDataStructure {
        id,
        name: format!("LargeStructure_{}_with_extended_name_for_memory_testing", id),
        metadata,
        values,
        nested_data,
        flags,
    }
}

pub fn create_test_dataset(count: usize, size_factor: usize) -> Vec<LargeDataStructure> {
    (0..count)
        .map(|i| create_large_test_structure(i as u64, size_factor))
        .collect()
}