use std::time::Duration;
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::report::image_renderer::{ImageRenderer, ImageFormat, RenderingServiceConfig};

#[derive(Debug, Clone)]
pub struct RenderingOptimizer {
    cache: Arc<AdvancedCache>,
    semaphore: Arc<Semaphore>,
    timeout_manager: TimeoutManager,
    circuit_breaker: CircuitBreaker,
    memory_pool: RenderBufferPool,
}

#[derive(Debug)]
pub struct AdvancedCache {
    cache: Arc<tokio::sync::RwLock<HashMap<String, CacheEntry>>>,
    access_patterns: Arc<tokio::sync::RwLock<AccessPatternTracker>>,
    stats: Arc<tokio::sync::RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub result: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AccessPatternTracker {
    patterns: HashMap<String, AccessPattern>,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: f64,
    pub last_access: chrono::DateTime<chrono::Utc>,
    pub prediction_weight: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

#[derive(Debug, Clone)]
pub struct TimeoutManager {
    pub render_timeout: Duration,
    pub network_timeout: Duration,
    pub cache_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    failure_count: Arc<tokio::sync::RwLock<u32>>,
    last_failure_time: Arc<tokio::sync::RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_max_calls: u32,
}

#[derive(Debug, Clone)]
pub struct RenderBufferPool {
    objects: Arc<tokio::sync::Mutex<Vec<RenderBuffer>>>,
    max_size: usize,
}

#[derive(Debug)]
pub struct RenderBuffer {
    pub data: Vec<u8>,
    pub capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub mermaid_code: String,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: RenderQuality,
    pub cache_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderQuality {
    Fast,      // Optimized for speed, lower quality
    Balanced,  // Default quality/speed tradeoff
    High,      // High quality, slower rendering
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub render_time_ms: u64,
    pub cache_hit: bool,
    pub quality_used: RenderQuality,
    pub dimensions: (u32, u32),
}

impl RenderingOptimizer {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(AdvancedCache::new()),
            semaphore: Arc::new(Semaphore::new(25)), // Increased concurrent renders for better throughput
            timeout_manager: TimeoutManager::new(),
            circuit_breaker: CircuitBreaker::new(10, Duration::from_secs(30)), // More tolerant circuit breaker
            memory_pool: RenderBufferPool::new(30),
        }
    }

    pub async fn optimize_rendering(&self, request: OptimizationRequest) -> Result<OptimizationResult, OptimizationError> {
        // Check circuit breaker first
        self.circuit_breaker.check_state().await?;

        // Generate cache key if not provided
        let cache_key = request.cache_key.clone().unwrap_or_else(|| 
            self.generate_cache_key(&request.mermaid_code, &request.format, request.width, request.height, &request.quality)
        );

        // Check cache first (content-addressable)
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(OptimizationResult {
                data: cached.result,
                format: request.format,
                render_time_ms: 0, // Cache hit
                cache_hit: true,
                quality_used: request.quality,
                dimensions: request.width.zip(request.height).unwrap_or((1200, 800)),
            });
        }

        // Acquire semaphore to limit concurrency (prevents resource exhaustion)
        let _permit = self.semaphore.acquire().await.map_err(|_| OptimizationError::ResourceExhaustion)?;

        // Get buffer from pool
        let buffer = self.memory_pool.acquire().await;

        // Perform optimized rendering with timeout
        let start_time = std::time::Instant::now();
        
        let result = tokio::time::timeout(
            self.timeout_manager.render_timeout,
            self.render_with_optimizations(request.clone(), buffer)
        ).await;

        let render_result = match result {
            Ok(Ok(result)) => {
                // Success - record in circuit breaker
                self.circuit_breaker.record_success().await;
                result
            }
            Ok(Err(e)) => {
                // Render failed - record failure
                self.circuit_breaker.record_failure().await;
                return Err(e);
            }
            Err(_) => {
                // Timeout - record failure and return specific error
                self.circuit_breaker.record_failure().await;
                return Err(OptimizationError::Timeout);
            }
        };

        let render_time = start_time.elapsed().as_millis() as u64;

        // Store in cache
        self.cache.store(&cache_key, &render_result.data, vec!["diagram".to_string()]).await?;

        Ok(OptimizationResult {
            data: render_result.data,
            format: request.format,
            render_time_ms: render_time,
            cache_hit: false,
            quality_used: request.quality,
            dimensions: render_result.dimensions,
        })
    }

    async fn render_with_optimizations(
        &self, 
        request: OptimizationRequest, 
        _buffer: PooledBuffer
    ) -> Result<RenderResult, OptimizationError> {
        // Create optimized renderer configuration based on quality setting
        let config = self.create_optimized_config(&request.quality);
        let renderer = ImageRenderer::with_config(config);

        // Apply quality-specific optimizations
        let optimized_code = self.apply_quality_optimizations(&request.mermaid_code, &request.quality);

        // Perform the actual rendering
        let rendered_image = renderer.render_diagram(
            &optimized_code,
            request.format,
            request.width.zip(request.height)
        ).await.map_err(|e| OptimizationError::RenderingFailed(e.to_string()))?;

        Ok(RenderResult {
            data: rendered_image.data,
            dimensions: rendered_image.dimensions,
        })
    }

    fn create_optimized_config(&self, quality: &RenderQuality) -> RenderingServiceConfig {
        let mut config = RenderingServiceConfig::default();
        
        match quality {
            RenderQuality::Fast => {
                config.timeout_seconds = 15; // Shorter timeout for fast renders
                config.max_retries = 1;      // Fewer retries
                config.max_complexity_score = 500; // Lower complexity limit
            }
            RenderQuality::Balanced => {
                config.timeout_seconds = 30;
                config.max_retries = 2;
                config.max_complexity_score = 1000;
            }
            RenderQuality::High => {
                config.timeout_seconds = 60; // Longer timeout for high quality
                config.max_retries = 3;
                config.max_complexity_score = 2000; // Higher complexity allowed
            }
        }
        
        config
    }

    fn apply_quality_optimizations(&self, mermaid_code: &str, quality: &RenderQuality) -> String {
        match quality {
            RenderQuality::Fast => {
                // For fast rendering, simplify the diagram
                self.simplify_diagram_for_speed(mermaid_code)
            }
            RenderQuality::Balanced => {
                // Keep as-is for balanced
                mermaid_code.to_string()
            }
            RenderQuality::High => {
                // For high quality, add enhancement directives
                self.enhance_diagram_for_quality(mermaid_code)
            }
        }
    }

    fn simplify_diagram_for_speed(&self, mermaid_code: &str) -> String {
        // Remove complex styling that doesn't significantly impact readability
        let mut simplified = mermaid_code.to_string();
        
        // Remove gradient fills (computationally expensive)
        simplified = simplified.replace("fill:gradient", "fill:solid");
        
        // Simplify complex shapes to basic rectangles for very fast rendering
        if simplified.len() > 1000 { // Only for large diagrams
            simplified = simplified.replace("circle", "rect");
            simplified = simplified.replace("diamond", "rect");
        }
        
        simplified
    }

    fn enhance_diagram_for_quality(&self, mermaid_code: &str) -> String {
        // For high quality, keep the original diagram unchanged
        // Complex enhancement directives can cause rendering service errors
        // The quality improvement is handled by increased timeout and complexity limits
        mermaid_code.to_string()
    }

    fn generate_cache_key(&self, mermaid_code: &str, format: &ImageFormat, width: Option<u32>, height: Option<u32>, quality: &RenderQuality) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(mermaid_code.as_bytes());
        hasher.update(format!("{:?}", format).as_bytes());
        hasher.update(format!("{:?}", width).as_bytes());
        hasher.update(format!("{:?}", height).as_bytes());
        hasher.update(format!("{:?}", quality).as_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let cache_stats = self.cache.get_stats().await;
        let circuit_breaker_stats = self.circuit_breaker.get_stats().await;
        
        PerformanceStats {
            cache_hit_rate: if cache_stats.total_requests > 0 {
                cache_stats.hits as f64 / cache_stats.total_requests as f64
            } else {
                0.0
            },
            total_requests: cache_stats.total_requests,
            circuit_breaker_failures: circuit_breaker_stats.failure_count,
            circuit_breaker_state: circuit_breaker_stats.state,
        }
    }
}

impl AdvancedCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            access_patterns: Arc::new(tokio::sync::RwLock::new(AccessPatternTracker::default())),
            stats: Arc::new(tokio::sync::RwLock::new(CacheStats::default())),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<CacheEntry>, CacheError> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            stats.hits += 1;
            
            // Update access pattern
            self.update_access_pattern(key).await;
            
            Ok(Some(entry.clone()))
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    pub async fn store(&self, key: &str, data: &[u8], tags: Vec<String>) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;
        
        let entry = CacheEntry {
            result: data.to_vec(),
            timestamp: chrono::Utc::now(),
            access_count: 1,
            tags,
        };
        
        cache.insert(key.to_string(), entry);
        stats.stores += 1;
        
        // Implement simple LRU eviction if cache gets too large
        if cache.len() > 1000 {
            let oldest_key = cache
                .iter()
                .min_by_key(|(_, entry)| entry.timestamp)
                .map(|(k, _)| k.clone());
            
            if let Some(oldest) = oldest_key {
                cache.remove(&oldest);
                stats.evictions += 1;
            }
        }
        
        Ok(())
    }

    async fn update_access_pattern(&self, key: &str) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns.patterns.entry(key.to_string()).or_insert_with(|| AccessPattern {
            frequency: 0.0,
            last_access: chrono::Utc::now(),
            prediction_weight: 1.0,
        });
        
        pattern.frequency += 1.0;
        pattern.last_access = chrono::Utc::now();
        pattern.prediction_weight *= 1.1; // Increase prediction weight for frequently accessed items
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

impl TimeoutManager {
    pub fn new() -> Self {
        Self {
            render_timeout: Duration::from_secs(45), // Increased for complex diagrams and high quality
            network_timeout: Duration::from_secs(15), // More generous network timeout
            cache_timeout: Duration::from_millis(200), // Slightly longer cache timeout
        }
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_duration: Duration) -> Self {
        Self {
            failure_count: Arc::new(tokio::sync::RwLock::new(0)),
            last_failure_time: Arc::new(tokio::sync::RwLock::new(None)),
            failure_threshold,
            timeout_duration,
            half_open_max_calls: 3,
        }
    }

    pub async fn check_state(&self) -> Result<(), OptimizationError> {
        let failure_count = *self.failure_count.read().await;
        let last_failure = *self.last_failure_time.read().await;
        
        if failure_count >= self.failure_threshold {
            if let Some(last_failure_time) = last_failure {
                let elapsed = chrono::Utc::now() - last_failure_time;
                if elapsed < chrono::Duration::from_std(self.timeout_duration).unwrap() {
                    return Err(OptimizationError::CircuitBreakerOpen);
                }
            }
        }
        
        Ok(())
    }

    pub async fn record_success(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count = 0;
        
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = None;
    }

    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(chrono::Utc::now());
    }

    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let failure_count = *self.failure_count.read().await;
        let last_failure = *self.last_failure_time.read().await;
        
        let state = if failure_count >= self.failure_threshold {
            if let Some(last_failure_time) = last_failure {
                let elapsed = chrono::Utc::now() - last_failure_time;
                if elapsed < chrono::Duration::from_std(self.timeout_duration).unwrap() {
                    CircuitBreakerState::Open
                } else {
                    CircuitBreakerState::HalfOpen
                }
            } else {
                CircuitBreakerState::Closed
            }
        } else {
            CircuitBreakerState::Closed
        };
        
        CircuitBreakerStats {
            failure_count,
            state,
        }
    }
}

impl RenderBufferPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            objects: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            max_size,
        }
    }

    pub async fn acquire(&self) -> PooledBuffer {
        let mut objects = self.objects.lock().await;
        let buffer = objects.pop().unwrap_or_else(|| RenderBuffer::new());
        PooledBuffer {
            buffer: Some(buffer),
            pool: self.objects.clone(),
        }
    }
}

impl RenderBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(1024 * 1024), // 1MB initial capacity
            capacity: 1024 * 1024,
        }
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }
}

pub struct PooledBuffer {
    buffer: Option<RenderBuffer>,
    pool: Arc<tokio::sync::Mutex<Vec<RenderBuffer>>>,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(mut buffer) = self.buffer.take() {
            buffer.reset();
            
            // Return to pool (non-blocking)
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut objects = pool.lock().await;
                objects.push(buffer);
            });
        }
    }
}

#[derive(Debug)]
pub struct RenderResult {
    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cache_hit_rate: f64,
    pub total_requests: u64,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_state: CircuitBreakerState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub failure_count: u32,
    pub state: CircuitBreakerState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizationError {
    #[error("Rendering failed: {0}")]
    RenderingFailed(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Resource exhaustion - too many concurrent requests")]
    ResourceExhaustion,
    #[error("Request timeout")]
    Timeout,
    #[error("Circuit breaker is open - service temporarily unavailable")]
    CircuitBreakerOpen,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache storage error: {0}")]
    StorageError(String),
    #[error("Cache retrieval error: {0}")]
    RetrievalError(String),
}

impl From<CacheError> for OptimizationError {
    fn from(err: CacheError) -> Self {
        OptimizationError::CacheError(err.to_string())
    }
}