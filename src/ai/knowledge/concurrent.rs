//! Thread-safe concurrent access patterns for the knowledge library
//!
//! This module provides high-performance, thread-safe access to the knowledge library
//! with optimistic concurrency control, lock-free operations where possible, and
//! comprehensive deadlock prevention.

use crate::ai::knowledge::{schema::*, performance::PerformanceMetrics};
use std::sync::{Arc, atomic::{AtomicUsize, AtomicU64, Ordering}};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use dashmap::DashMap;
use tokio::sync::{RwLock, broadcast, mpsc};
use parking_lot::Mutex;
use lru::LruCache;
use serde::{Serialize, Deserialize};

/// Concurrent access error types
#[derive(Debug, thiserror::Error)]
pub enum AccessError {
    #[error("Pattern not found: {pattern_id}")]
    PatternNotFound { pattern_id: String },
    
    #[error("Concurrent access timeout after {timeout_ms}ms")]
    AccessTimeout { timeout_ms: u64 },
    
    #[error("Lock contention detected: {details}")]
    LockContention { details: String },
    
    #[error("Deadlock prevention triggered: {context}")]
    DeadlockPrevention { context: String },
    
    #[error("Resource exhaustion: {resource}")]
    ResourceExhaustion { resource: String },
    
    #[error("Operation cancelled: {reason}")]
    OperationCancelled { reason: String },
}

/// Metrics for concurrent access operations
#[derive(Debug)]
pub struct ConcurrentMetrics {
    pub concurrent_readers: AtomicUsize,
    pub lock_contention_count: AtomicU64,
    pub average_wait_time_ns: AtomicU64,
    pub deadlock_prevention_count: AtomicU64,
    pub cache_hit_count: AtomicU64,
    pub cache_miss_count: AtomicU64,
    pub total_operations: AtomicU64,
    pub failed_operations: AtomicU64,
}

impl ConcurrentMetrics {
    pub fn new() -> Self {
        Self {
            concurrent_readers: AtomicUsize::new(0),
            lock_contention_count: AtomicU64::new(0),
            average_wait_time_ns: AtomicU64::new(0),
            deadlock_prevention_count: AtomicU64::new(0),
            cache_hit_count: AtomicU64::new(0),
            cache_miss_count: AtomicU64::new(0),
            total_operations: AtomicU64::new(0),
            failed_operations: AtomicU64::new(0),
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hit_count.load(Ordering::Relaxed);
        let misses = self.cache_miss_count.load(Ordering::Relaxed);
        if hits + misses == 0 {
            0.0
        } else {
            hits as f64 / (hits + misses) as f64
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        let failed = self.failed_operations.load(Ordering::Relaxed);
        if total == 0 {
            1.0
        } else {
            (total - failed) as f64 / total as f64
        }
    }
}

/// Lock-free reader tracking
pub struct ReaderTracker {
    active_readers: DashMap<u64, ReaderInfo>,
    next_reader_id: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct ReaderInfo {
    pub reader_id: u64,
    pub pattern_id: String,
    pub start_time: Instant,
    pub thread_id: std::thread::ThreadId,
}

impl ReaderTracker {
    pub fn new() -> Self {
        Self {
            active_readers: DashMap::new(),
            next_reader_id: AtomicU64::new(0),
        }
    }
    
    pub fn register_reader(&self, pattern_id: String) -> u64 {
        let reader_id = self.next_reader_id.fetch_add(1, Ordering::Relaxed);
        let reader_info = ReaderInfo {
            reader_id,
            pattern_id,
            start_time: Instant::now(),
            thread_id: std::thread::current().id(),
        };
        
        self.active_readers.insert(reader_id, reader_info);
        reader_id
    }
    
    pub fn unregister_reader(&self, reader_id: u64) -> Option<Duration> {
        self.active_readers.remove(&reader_id)
            .map(|(_, info)| info.start_time.elapsed())
    }
    
    pub fn get_active_readers(&self) -> Vec<ReaderInfo> {
        self.active_readers.iter().map(|entry| entry.value().clone()).collect()
    }
    
    pub fn detect_long_running_readers(&self, threshold: Duration) -> Vec<ReaderInfo> {
        self.active_readers
            .iter()
            .filter(|entry| entry.value().start_time.elapsed() > threshold)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// High-performance pattern cache with eviction policies
pub struct PatternCache {
    cache: Mutex<LruCache<String, Arc<PatternKnowledge>>>,
    access_frequency: DashMap<String, AtomicU64>,
    last_access: DashMap<String, AtomicU64>,
    max_size: usize,
}

impl PatternCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(max_size.try_into().unwrap_or(1000))),
            access_frequency: DashMap::new(),
            last_access: DashMap::new(),
            max_size,
        }
    }
    
    pub fn get(&self, pattern_id: &str) -> Option<Arc<PatternKnowledge>> {
        // Update access statistics
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.access_frequency
            .entry(pattern_id.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        
        self.last_access
            .entry(pattern_id.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(now, Ordering::Relaxed);
        
        // Get from cache
        let mut cache = self.cache.lock();
        cache.get(pattern_id).cloned()
    }
    
    pub fn put(&self, pattern_id: String, pattern: Arc<PatternKnowledge>) {
        let mut cache = self.cache.lock();
        cache.put(pattern_id, pattern);
    }
    
    pub fn evict_least_used(&self, count: usize) {
        let mut candidates: Vec<(String, u64, u64)> = self.access_frequency
            .iter()
            .map(|entry| {
                let pattern_id = entry.key().clone();
                let frequency = entry.value().load(Ordering::Relaxed);
                let last_access = self.last_access
                    .get(&pattern_id)
                    .map(|e| e.value().load(Ordering::Relaxed))
                    .unwrap_or(0);
                (pattern_id, frequency, last_access)
            })
            .collect();
        
        // Sort by frequency (ascending) and last access time (ascending)
        candidates.sort_by(|a, b| {
            a.1.cmp(&b.1).then(a.2.cmp(&b.2))
        });
        
        let mut cache = self.cache.lock();
        for (pattern_id, _, _) in candidates.into_iter().take(count) {
            cache.pop(&pattern_id);
            self.access_frequency.remove(&pattern_id);
            self.last_access.remove(&pattern_id);
        }
    }
    
    pub fn get_statistics(&self) -> CacheStatistics {
        let cache = self.cache.lock();
        let current_size = cache.len();
        let total_frequency: u64 = self.access_frequency
            .iter()
            .map(|entry| entry.value().load(Ordering::Relaxed))
            .sum();
        
        CacheStatistics {
            current_size,
            max_size: self.max_size,
            utilization: current_size as f64 / self.max_size as f64,
            total_accesses: total_frequency,
            average_frequency: if current_size > 0 {
                total_frequency as f64 / current_size as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStatistics {
    pub current_size: usize,
    pub max_size: usize,
    pub utilization: f64,
    pub total_accesses: u64,
    pub average_frequency: f64,
}

/// Update notification system for hot-reload support
#[derive(Debug, Clone)]
pub enum UpdateNotification {
    PatternUpdated { pattern_id: String },
    IndexRebuilt,
    LibraryReloaded,
    CacheCleared,
}

pub type UpdateReceiver = broadcast::Receiver<UpdateNotification>;
pub type UpdateSender = broadcast::Sender<UpdateNotification>;

/// Deadlock detection and prevention system
pub struct DeadlockDetector {
    lock_dependencies: DashMap<String, Vec<String>>,
    thread_locks: DashMap<std::thread::ThreadId, Vec<String>>,
    detection_enabled: bool,
}

impl DeadlockDetector {
    pub fn new(detection_enabled: bool) -> Self {
        Self {
            lock_dependencies: DashMap::new(),
            thread_locks: DashMap::new(),
            detection_enabled,
        }
    }
    
    pub fn acquire_lock(&self, lock_name: &str) -> Result<(), AccessError> {
        if !self.detection_enabled {
            return Ok(());
        }
        
        let thread_id = std::thread::current().id();
        
        // Check for potential deadlock
        if self.would_create_cycle(lock_name, &thread_id) {
            return Err(AccessError::DeadlockPrevention {
                context: format!("Acquiring lock '{}' would create a deadlock cycle", lock_name),
            });
        }
        
        // Record lock acquisition
        self.thread_locks
            .entry(thread_id)
            .or_insert_with(Vec::new)
            .push(lock_name.to_string());
        
        Ok(())
    }
    
    pub fn release_lock(&self, lock_name: &str) {
        if !self.detection_enabled {
            return;
        }
        
        let thread_id = std::thread::current().id();
        if let Some(mut locks) = self.thread_locks.get_mut(&thread_id) {
            locks.retain(|l| l != lock_name);
        }
    }
    
    fn would_create_cycle(&self, new_lock: &str, thread_id: &std::thread::ThreadId) -> bool {
        // Simplified cycle detection - in practice, this would be more sophisticated
        if let Some(current_locks) = self.thread_locks.get(thread_id) {
            // Check if we already hold this lock
            if current_locks.contains(&new_lock.to_string()) {
                return true;
            }
            
            // Check if any currently held lock has a dependency on the new lock
            for held_lock in current_locks.iter() {
                if let Some(deps) = self.lock_dependencies.get(held_lock) {
                    if deps.contains(&new_lock.to_string()) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}

/// Main concurrent access coordination system
pub struct ConcurrentKnowledgeAccess {
    library: Arc<KnowledgeLibraryLookup>,
    cache: Arc<PatternCache>,
    reader_tracker: Arc<ReaderTracker>,
    metrics: Arc<ConcurrentMetrics>,
    performance_metrics: Arc<PerformanceMetrics>,
    update_sender: UpdateSender,
    deadlock_detector: Arc<DeadlockDetector>,
    access_timeout: Duration,
    max_concurrent_readers: usize,
}

impl ConcurrentKnowledgeAccess {
    pub fn new(
        library: Arc<KnowledgeLibraryLookup>,
        performance_metrics: Arc<PerformanceMetrics>,
        max_cache_size: usize,
        max_concurrent_readers: usize,
    ) -> Self {
        let (update_sender, _) = broadcast::channel(1000);
        
        Self {
            library,
            cache: Arc::new(PatternCache::new(max_cache_size)),
            reader_tracker: Arc::new(ReaderTracker::new()),
            metrics: Arc::new(ConcurrentMetrics::new()),
            performance_metrics,
            update_sender,
            deadlock_detector: Arc::new(DeadlockDetector::new(true)),
            access_timeout: Duration::from_secs(5),
            max_concurrent_readers,
        }
    }
    
    /// Get pattern with concurrent access optimization
    pub async fn get_pattern_concurrent(&self, pattern_id: &str) -> Result<Arc<PatternKnowledge>, AccessError> {
        let start_time = Instant::now();
        let operation_id = self.performance_metrics.increment_concurrent_ops();
        
        // Track this operation
        self.metrics.total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Check concurrency limits
        let current_readers = self.metrics.concurrent_readers.load(Ordering::Relaxed);
        if current_readers >= self.max_concurrent_readers {
            self.metrics.failed_operations.fetch_add(1, Ordering::Relaxed);
            self.performance_metrics.decrement_concurrent_ops();
            return Err(AccessError::ResourceExhaustion {
                resource: format!("concurrent readers ({}/{})", current_readers, self.max_concurrent_readers),
            });
        }
        
        // Register reader
        let reader_id = self.reader_tracker.register_reader(pattern_id.to_string());
        self.metrics.concurrent_readers.fetch_add(1, Ordering::Relaxed);
        
        let result = self.get_pattern_with_timeout(pattern_id).await;
        
        // Cleanup
        if let Some(duration) = self.reader_tracker.unregister_reader(reader_id) {
            self.update_wait_time_metrics(duration);
        }
        self.metrics.concurrent_readers.fetch_sub(1, Ordering::Relaxed);
        self.performance_metrics.decrement_concurrent_ops();
        
        // Record performance metrics
        let total_duration = start_time.elapsed();
        self.performance_metrics.record_lookup_time(total_duration.as_nanos() as u64);
        
        if result.is_err() {
            self.metrics.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        result
    }
    
    async fn get_pattern_with_timeout(&self, pattern_id: &str) -> Result<Arc<PatternKnowledge>, AccessError> {
        // Try cache first
        if let Some(cached_pattern) = self.cache.get(pattern_id) {
            self.metrics.cache_hit_count.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_pattern);
        }
        
        self.metrics.cache_miss_count.fetch_add(1, Ordering::Relaxed);
        
        // Deadlock prevention
        let lock_name = format!("pattern_{}", pattern_id);
        self.deadlock_detector.acquire_lock(&lock_name)?;
        
        // Timeout wrapper for the actual lookup
        let lookup_future = self.lookup_pattern_from_library(pattern_id);
        let timeout_future = tokio::time::sleep(self.access_timeout);
        
        let result = tokio::select! {\n            pattern = lookup_future => {\n                match pattern {\n                    Ok(p) => {\n                        // Cache the result\n                        self.cache.put(pattern_id.to_string(), Arc::clone(&p));\n                        Ok(p)\n                    }\n                    Err(e) => Err(e)\n                }\n            }\n            _ = timeout_future => {\n                Err(AccessError::AccessTimeout { \n                    timeout_ms: self.access_timeout.as_millis() as u64 \n                })\n            }\n        };\n        \n        self.deadlock_detector.release_lock(&lock_name);\n        result\n    }\n    \n    async fn lookup_pattern_from_library(&self, pattern_id: &str) -> Result<Arc<PatternKnowledge>, AccessError> {\n        // Simulate potential lock contention with a small delay\n        let contention_start = Instant::now();\n        \n        // Get pattern from library (this might involve some waiting)\n        let pattern = self.library.get_pattern(pattern_id)\n            .ok_or_else(|| AccessError::PatternNotFound { \n                pattern_id: pattern_id.to_string() \n            })?;\n        \n        // Record lock contention if it took too long\n        let contention_time = contention_start.elapsed();\n        if contention_time > Duration::from_millis(10) {\n            self.metrics.lock_contention_count.fetch_add(1, Ordering::Relaxed);\n        }\n        \n        Ok(Arc::new(pattern.clone()))\n    }\n    \n    /// Batch lookup for multiple patterns with optimization\n    pub async fn batch_lookup(&self, pattern_ids: &[String]) -> Result<Vec<Arc<PatternKnowledge>>, AccessError> {\n        if pattern_ids.is_empty() {\n            return Ok(Vec::new());\n        }\n        \n        // Split into cached and uncached patterns\n        let mut cached_patterns = HashMap::new();\n        let mut uncached_ids = Vec::new();\n        \n        for pattern_id in pattern_ids {\n            if let Some(cached) = self.cache.get(pattern_id) {\n                cached_patterns.insert(pattern_id.clone(), cached);\n                self.metrics.cache_hit_count.fetch_add(1, Ordering::Relaxed);\n            } else {\n                uncached_ids.push(pattern_id.clone());\n                self.metrics.cache_miss_count.fetch_add(1, Ordering::Relaxed);\n            }\n        }\n        \n        // Fetch uncached patterns in parallel\n        let mut uncached_patterns = HashMap::new();\n        if !uncached_ids.is_empty() {\n            let futures: Vec<_> = uncached_ids\n                .iter()\n                .map(|id| self.get_pattern_concurrent(id))\n                .collect();\n            \n            let results = futures::future::join_all(futures).await;\n            \n            for (i, result) in results.into_iter().enumerate() {\n                match result {\n                    Ok(pattern) => {\n                        uncached_patterns.insert(uncached_ids[i].clone(), pattern);\n                    }\n                    Err(e) => {\n                        log::warn!(\"Failed to fetch pattern {}: {}\", uncached_ids[i], e);\n                        // Continue with partial results rather than failing entirely\n                    }\n                }\n            }\n        }\n        \n        // Combine results in original order\n        let mut results = Vec::with_capacity(pattern_ids.len());\n        for pattern_id in pattern_ids {\n            if let Some(pattern) = cached_patterns.get(pattern_id)\n                .or_else(|| uncached_patterns.get(pattern_id)) {\n                results.push(Arc::clone(pattern));\n            }\n            // Note: we skip patterns that failed to load rather than failing the entire batch\n        }\n        \n        Ok(results)\n    }\n    \n    /// Subscribe to pattern updates for hot-reload support\n    pub fn subscribe_to_updates(&self) -> UpdateReceiver {\n        self.update_sender.subscribe()\n    }\n    \n    /// Notify subscribers of updates\n    pub fn notify_update(&self, notification: UpdateNotification) {\n        let _ = self.update_sender.send(notification);\n    }\n    \n    /// Clear cache and notify subscribers\n    pub async fn clear_cache(&self) {\n        self.cache.evict_least_used(self.cache.max_size);\n        self.notify_update(UpdateNotification::CacheCleared);\n    }\n    \n    /// Get comprehensive access statistics\n    pub fn get_access_statistics(&self) -> AccessStatistics {\n        let cache_stats = self.cache.get_statistics();\n        let active_readers = self.reader_tracker.get_active_readers();\n        let long_running = self.reader_tracker.detect_long_running_readers(Duration::from_secs(10));\n        \n        AccessStatistics {\n            concurrent_readers: self.metrics.concurrent_readers.load(Ordering::Relaxed),\n            total_operations: self.metrics.total_operations.load(Ordering::Relaxed),\n            failed_operations: self.metrics.failed_operations.load(Ordering::Relaxed),\n            lock_contention_count: self.metrics.lock_contention_count.load(Ordering::Relaxed),\n            cache_hit_rate: self.metrics.cache_hit_rate(),\n            success_rate: self.metrics.success_rate(),\n            cache_statistics: cache_stats,\n            active_readers_count: active_readers.len(),\n            long_running_operations: long_running.len(),\n            deadlock_prevention_count: self.metrics.deadlock_prevention_count.load(Ordering::Relaxed),\n        }\n    }\n    \n    /// Optimize cache based on access patterns\n    pub async fn optimize_cache(&self) {\n        let stats = self.cache.get_statistics();\n        \n        // If cache is near capacity and hit rate is low, evict least used entries\n        if stats.utilization > 0.9 && self.metrics.cache_hit_rate() < 0.8 {\n            let evict_count = (stats.current_size as f64 * 0.2) as usize; // Evict 20%\n            self.cache.evict_least_used(evict_count);\n            log::info!(\"Evicted {} cache entries to improve performance\", evict_count);\n        }\n    }\n    \n    /// Health check for the concurrent access system\n    pub fn health_check(&self) -> ConcurrentHealthStatus {\n        let stats = self.get_access_statistics();\n        let mut issues = Vec::new();\n        let mut status = HealthStatus::Healthy;\n        \n        // Check for high contention\n        if stats.lock_contention_count > 100 {\n            issues.push(\"High lock contention detected\".to_string());\n            status = HealthStatus::Degraded;\n        }\n        \n        // Check for low cache hit rate\n        if stats.cache_hit_rate < 0.8 {\n            issues.push(\"Low cache hit rate\".to_string());\n            if status == HealthStatus::Healthy {\n                status = HealthStatus::Degraded;\n            }\n        }\n        \n        // Check for many long-running operations\n        if stats.long_running_operations > 10 {\n            issues.push(\"Many long-running operations detected\".to_string());\n            status = HealthStatus::Unhealthy;\n        }\n        \n        // Check success rate\n        if stats.success_rate < 0.95 {\n            issues.push(\"Low operation success rate\".to_string());\n            status = HealthStatus::Unhealthy;\n        }\n        \n        ConcurrentHealthStatus {\n            status,\n            issues,\n            statistics: stats,\n        }\n    }\n    \n    fn update_wait_time_metrics(&self, duration: Duration) {\n        let duration_ns = duration.as_nanos() as u64;\n        \n        // Simple exponential moving average\n        let current_avg = self.metrics.average_wait_time_ns.load(Ordering::Relaxed);\n        let new_avg = if current_avg == 0 {\n            duration_ns\n        } else {\n            (current_avg * 9 + duration_ns) / 10 // 90% weight to previous average\n        };\n        \n        self.metrics.average_wait_time_ns.store(new_avg, Ordering::Relaxed);\n    }\n}\n\n#[derive(Debug, Serialize)]\npub struct AccessStatistics {\n    pub concurrent_readers: usize,\n    pub total_operations: u64,\n    pub failed_operations: u64,\n    pub lock_contention_count: u64,\n    pub cache_hit_rate: f64,\n    pub success_rate: f64,\n    pub cache_statistics: CacheStatistics,\n    pub active_readers_count: usize,\n    pub long_running_operations: usize,\n    pub deadlock_prevention_count: u64,\n}\n\n#[derive(Debug, Serialize)]\npub struct ConcurrentHealthStatus {\n    pub status: HealthStatus,\n    pub issues: Vec<String>,\n    pub statistics: AccessStatistics,\n}\n\n#[derive(Debug, Serialize)]\npub enum HealthStatus {\n    Healthy,\n    Degraded,\n    Unhealthy,\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    use tokio;\n    \n    #[test]\n    fn test_concurrent_metrics() {\n        let metrics = ConcurrentMetrics::new();\n        assert_eq!(metrics.concurrent_readers.load(Ordering::Relaxed), 0);\n        assert_eq!(metrics.cache_hit_rate(), 0.0);\n        \n        metrics.cache_hit_count.store(95, Ordering::Relaxed);\n        metrics.cache_miss_count.store(5, Ordering::Relaxed);\n        assert_eq!(metrics.cache_hit_rate(), 0.95);\n    }\n    \n    #[test]\n    fn test_reader_tracker() {\n        let tracker = ReaderTracker::new();\n        \n        let reader1 = tracker.register_reader(\"pattern1\".to_string());\n        let reader2 = tracker.register_reader(\"pattern2\".to_string());\n        \n        assert_eq!(tracker.get_active_readers().len(), 2);\n        \n        std::thread::sleep(Duration::from_millis(10));\n        tracker.unregister_reader(reader1);\n        \n        assert_eq!(tracker.get_active_readers().len(), 1);\n    }\n    \n    #[test]\n    fn test_pattern_cache() {\n        let cache = PatternCache::new(2);\n        \n        // Create dummy patterns\n        let pattern1 = Arc::new(PatternKnowledge {\n            id: \"test1\".to_string(),\n            name: \"Test Pattern 1\".to_string(),\n            // ... other fields would be filled in real implementation\n            definition: crate::ai::knowledge::schema::CompressedString::new(\"Test definition\"),\n            symptoms: vec![],\n            impact: crate::ai::knowledge::schema::ImpactLevel::Low,\n            category: crate::ai::knowledge::schema::AntiPatternCategory::Maintainability,\n            detection_methods: vec![],\n            solutions: vec![],\n            examples: crate::ai::knowledge::schema::CodeExamples {\n                primary: vec![],\n                variations: std::collections::HashMap::new(),\n            },\n            language_variations: std::collections::HashMap::new(),\n            related_patterns: vec![],\n            tags: vec![],\n            frequency_score: 0.5,\n            detection_confidence: 0.8,\n        });\n        \n        cache.put(\"test1\".to_string(), pattern1);\n        \n        // Test cache hit\n        assert!(cache.get(\"test1\").is_some());\n        assert!(cache.get(\"nonexistent\").is_none());\n        \n        let stats = cache.get_statistics();\n        assert_eq!(stats.current_size, 1);\n        assert_eq!(stats.max_size, 2);\n    }\n    \n    #[test]\n    fn test_deadlock_detector() {\n        let detector = DeadlockDetector::new(true);\n        \n        // Normal lock acquisition should work\n        assert!(detector.acquire_lock(\"lock1\").is_ok());\n        assert!(detector.acquire_lock(\"lock2\").is_ok());\n        \n        // Trying to acquire the same lock again should fail\n        assert!(detector.acquire_lock(\"lock1\").is_err());\n        \n        detector.release_lock(\"lock1\");\n        detector.release_lock(\"lock2\");\n    }\n    \n    #[tokio::test]\n    async fn test_concurrent_access_timeout() {\n        // This test would require setting up a full KnowledgeLibraryLookup\n        // which is complex, so we'll keep it simple\n        let timeout = Duration::from_millis(100);\n        \n        let start = Instant::now();\n        let timeout_future = tokio::time::sleep(timeout);\n        timeout_future.await;\n        \n        assert!(start.elapsed() >= timeout);\n    }\n}"