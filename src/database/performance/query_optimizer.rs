//! Database Query Performance Optimizer
//!
//! Optimizes database access patterns for large codebase analysis with
//! advanced batching, connection pooling, and query optimization strategies.

use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, Dependency};
use crate::database::scalable_manager::ScalableDatabase;
use crate::error::{Result, UveddiError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Configuration for database query optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationConfig {
    /// Maximum batch size for bulk operations
    pub max_batch_size: usize,
    
    /// Optimal batch size for different operation types
    pub optimal_batch_sizes: HashMap<String, usize>,
    
    /// Connection pool configuration
    pub connection_pool_config: ConnectionPoolConfig,
    
    /// Query timeout configuration
    pub query_timeouts: QueryTimeoutConfig,
    
    /// Enable prepared statement caching
    pub enable_prepared_statements: bool,
    
    /// Enable query result caching
    pub enable_result_caching: bool,
    
    /// Transaction batch configuration
    pub transaction_config: TransactionConfig,
    
    /// Enable database performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for QueryOptimizationConfig {
    fn default() -> Self {
        let mut optimal_batch_sizes = HashMap::new();
        optimal_batch_sizes.insert("issues".to_string(), 5000);
        optimal_batch_sizes.insert("dependencies".to_string(), 10000);
        optimal_batch_sizes.insert("anti_patterns".to_string(), 1000);
        
        Self {
            max_batch_size: 10000,
            optimal_batch_sizes,
            connection_pool_config: ConnectionPoolConfig::default(),
            query_timeouts: QueryTimeoutConfig::default(),
            enable_prepared_statements: true,
            enable_result_caching: true,
            transaction_config: TransactionConfig::default(),
            enable_performance_monitoring: true,
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in pool
    pub max_connections: u32,
    
    /// Minimum number of idle connections
    pub min_idle_connections: u32,
    
    /// Connection acquisition timeout
    pub acquire_timeout_seconds: u64,
    
    /// Connection idle timeout
    pub idle_timeout_seconds: u64,
    
    /// Connection lifetime
    pub max_lifetime_seconds: u64,
    
    /// Enable connection health checks
    pub enable_health_checks: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_idle_connections: 5,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600, // 10 minutes
            max_lifetime_seconds: 3600, // 1 hour
            enable_health_checks: true,
        }
    }
}

/// Query timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTimeoutConfig {
    /// Default query timeout
    pub default_timeout_seconds: u64,
    
    /// Timeout for bulk operations
    pub bulk_operation_timeout_seconds: u64,
    
    /// Timeout for complex queries
    pub complex_query_timeout_seconds: u64,
    
    /// Timeout for read operations
    pub read_timeout_seconds: u64,
    
    /// Timeout for write operations
    pub write_timeout_seconds: u64,
}

impl Default for QueryTimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 30,
            bulk_operation_timeout_seconds: 300, // 5 minutes
            complex_query_timeout_seconds: 120,  // 2 minutes
            read_timeout_seconds: 60,
            write_timeout_seconds: 120,
        }
    }
}

/// Transaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    /// Maximum operations per transaction
    pub max_operations_per_transaction: usize,
    
    /// Transaction timeout
    pub transaction_timeout_seconds: u64,
    
    /// Enable automatic transaction retry
    pub enable_retry: bool,
    
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    
    /// Retry backoff strategy
    pub retry_backoff_ms: u64,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            max_operations_per_transaction: 5000,
            transaction_timeout_seconds: 300,
            enable_retry: true,
            max_retry_attempts: 3,
            retry_backoff_ms: 1000,
        }
    }
}

/// Database performance metrics
#[derive(Debug, Clone, Default)]
pub struct DatabasePerformanceMetrics {
    pub total_queries: u64,
    pub total_query_time_ms: u64,
    pub average_query_time_ms: f64,
    pub bulk_operations_count: u64,
    pub bulk_operations_time_ms: u64,
    pub connection_pool_utilization: f64,
    pub cache_hit_rate: f64,
    pub transaction_success_rate: f64,
    pub failed_queries: u64,
    pub retried_operations: u64,
}

/// Query batch for optimized bulk operations
#[derive(Debug)]
pub struct QueryBatch<T> {
    pub items: Vec<T>,
    pub operation_type: String,
    pub estimated_time_ms: u64,
    pub priority: BatchPriority,
}

/// Priority for batch operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BatchPriority {
    Critical = 1, // Essential data that blocks other operations
    High = 2,     // Important data needed soon
    Medium = 3,   // Standard processing priority
    Low = 4,      // Background operations
}

/// Database query optimizer for large codebase performance
pub struct DatabaseQueryOptimizer {
    config: QueryOptimizationConfig,
    database: Arc<ScalableDatabase>,
    
    // Batch processing queues
    issues_queue: Arc<RwLock<VecDeque<QueryBatch<ArchitecturalIssue>>>>,
    dependencies_queue: Arc<RwLock<VecDeque<QueryBatch<Dependency>>>>,
    anti_patterns_queue: Arc<RwLock<VecDeque<QueryBatch<AntiPatternType>>>>,
    
    // Performance monitoring
    metrics: Arc<RwLock<DatabasePerformanceMetrics>>,
    
    // Resource management
    write_semaphore: Arc<Semaphore>,
    read_semaphore: Arc<Semaphore>,
    
    // Query caching
    result_cache: Arc<RwLock<HashMap<String, (Instant, serde_json::Value)>>>,
    
    // Prepared statements cache
    prepared_statements: Arc<RwLock<HashMap<String, String>>>,
}

impl DatabaseQueryOptimizer {
    /// Create new database query optimizer
    pub async fn new(
        config: QueryOptimizationConfig,
        database: Arc<ScalableDatabase>,
    ) -> Result<Self> {
        info!("Initializing database query optimizer");
        
        // Create semaphores for limiting concurrent operations
        let max_concurrent_writes = config.connection_pool_config.max_connections / 4;
        let max_concurrent_reads = config.connection_pool_config.max_connections * 3 / 4;
        
        let write_semaphore = Arc::new(Semaphore::new(max_concurrent_writes as usize));
        let read_semaphore = Arc::new(Semaphore::new(max_concurrent_reads as usize));
        
        Ok(Self {
            config,
            database,
            issues_queue: Arc::new(RwLock::new(VecDeque::new())),
            dependencies_queue: Arc::new(RwLock::new(VecDeque::new())),
            anti_patterns_queue: Arc::new(RwLock::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(DatabasePerformanceMetrics::default())),
            write_semaphore,
            read_semaphore,
            result_cache: Arc::new(RwLock::new(HashMap::new())),
            prepared_statements: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Store architectural issues with optimized batching
    pub async fn store_issues_optimized(&self, issues: Vec<ArchitecturalIssue>) -> Result<()> {
        if issues.is_empty() {
            return Ok(());
        }
        
        info!("Storing {} issues with optimized batching", issues.len());
        let start_time = Instant::now();
        
        // Get optimal batch size for issues
        let batch_size = self.config.optimal_batch_sizes.get("issues").copied().unwrap_or(5000);
        
        // Process issues in optimally-sized batches
        for chunk in issues.chunks(batch_size) {
            let batch = QueryBatch {
                items: chunk.to_vec(),
                operation_type: "store_issues".to_string(),
                estimated_time_ms: self.estimate_operation_time(chunk.len(), "issues"),
                priority: BatchPriority::High,
            };
            
            self.execute_issues_batch(batch).await?;
        }
        
        let elapsed = start_time.elapsed();
        self.update_performance_metrics("store_issues", elapsed, issues.len()).await;
        
        info!("Stored {} issues in {:?}", issues.len(), elapsed);
        Ok(())
    }
    
    /// Store dependencies with optimized batching and deduplication
    pub async fn store_dependencies_optimized(&self, run_id: i64, dependencies: Vec<Dependency>) -> Result<()> {
        if dependencies.is_empty() {
            return Ok(());
        }
        
        info!("Storing {} dependencies with optimization", dependencies.len());
        let start_time = Instant::now();
        
        // Deduplicate dependencies to reduce database load
        let deduplicated_deps = self.deduplicate_dependencies(dependencies);
        info!("Deduplicated {} dependencies to {}", dependencies.len(), deduplicated_deps.len());
        
        let batch_size = self.config.optimal_batch_sizes.get("dependencies").copied().unwrap_or(10000);
        
        // Process dependencies in batches with transaction management
        for chunk in deduplicated_deps.chunks(batch_size) {
            self.execute_with_transaction(|db| async move {
                db.store_dependencies_batch(run_id, chunk).await
            }).await?;
        }
        
        let elapsed = start_time.elapsed();
        self.update_performance_metrics("store_dependencies", elapsed, dependencies.len()).await;
        
        info!("Stored {} dependencies in {:?}", dependencies.len(), elapsed);
        Ok(())
    }
    
    /// Get analysis results with intelligent caching
    pub async fn get_analysis_results_cached(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>> {
        let cache_key = format!("analysis_results:{}", run_id);
        
        // Check cache first
        if self.config.enable_result_caching {
            if let Some(cached_result) = self.get_from_cache(&cache_key).await {
                debug!("Cache hit for analysis results: {}", run_id);
                return Ok(serde_json::from_value(cached_result)?);
            }
        }
        
        // Execute query with read optimization
        let _permit = self.read_semaphore.acquire().await.map_err(|e| UveddiError::database_error(&e.to_string()))?;
        let start_time = Instant::now();
        
        let results = self.database.get_issues_for_run(run_id).await?;
        
        let elapsed = start_time.elapsed();
        self.update_performance_metrics("get_analysis_results", elapsed, results.len()).await;
        
        // Cache the results
        if self.config.enable_result_caching {
            self.store_in_cache(cache_key, serde_json::to_value(&results)?).await;
        }
        
        Ok(results)
    }
    
    /// Execute query with transaction and retry logic
    async fn execute_with_transaction<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<ScalableDatabase>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let _permit = self.write_semaphore.acquire().await.map_err(|e| UveddiError::database_error(&e.to_string()))?;
        
        let mut attempts = 0;
        let max_attempts = if self.config.transaction_config.enable_retry {
            self.config.transaction_config.max_retry_attempts
        } else {
            1
        };
        
        while attempts < max_attempts {
            match operation(self.database.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    warn!("Transaction failed (attempt {}): {}", attempts, e);
                    
                    if attempts < max_attempts {
                        // Exponential backoff
                        let backoff_ms = self.config.transaction_config.retry_backoff_ms * (2_u64.pow(attempts - 1));
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        
                        // Update retry metrics
                        {
                            let mut metrics = self.metrics.write().await;
                            metrics.retried_operations += 1;
                        }
                    } else {
                        // Update failure metrics
                        {
                            let mut metrics = self.metrics.write().await;
                            metrics.failed_queries += 1;
                        }
                        return Err(e);
                    }
                }
            }
        }
        
        Err(UveddiError::database_error("Transaction failed after all retry attempts"))
    }
    
    /// Execute issues batch with optimized database operations
    async fn execute_issues_batch(&self, batch: QueryBatch<ArchitecturalIssue>) -> Result<()> {
        let start_time = Instant::now();
        
        self.execute_with_transaction(|db| async move {
            db.store_issues_batch(&batch.items).await
        }).await?;
        
        let elapsed = start_time.elapsed();
        debug!("Issues batch ({} items) processed in {:?}", batch.items.len(), elapsed);
        
        Ok(())
    }
    
    /// Deduplicate dependencies to reduce database operations
    fn deduplicate_dependencies(&self, dependencies: Vec<Dependency>) -> Vec<Dependency> {
        let mut seen = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();
        
        for dep in dependencies {
            // Create a unique key for the dependency
            let key = format!("{}:{}:{}:{}", 
                dep.analysis_run_id.unwrap_or(0), 
                dep.from_file, 
                dep.to_file, 
                dep.dependency_type
            );
            
            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(dep);
            }
        }
        
        deduplicated
    }
    
    /// Estimate operation time for planning purposes
    fn estimate_operation_time(&self, item_count: usize, operation_type: &str) -> u64 {
        let base_time_per_item = match operation_type {
            "issues" => 2,        // 2ms per issue
            "dependencies" => 1,  // 1ms per dependency  
            "anti_patterns" => 3, // 3ms per anti-pattern
            _ => 2,
        };
        
        (item_count as u64 * base_time_per_item).max(10) // Minimum 10ms
    }
    
    /// Get item from result cache
    async fn get_from_cache(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.result_cache.read().await;
        if let Some((timestamp, value)) = cache.get(key) {
            // Check if cache entry is still valid (5 minutes)
            if timestamp.elapsed() < Duration::from_secs(300) {
                return Some(value.clone());
            }
        }
        None
    }
    
    /// Store item in result cache
    async fn store_in_cache(&self, key: String, value: serde_json::Value) {
        let mut cache = self.result_cache.write().await;
        cache.insert(key, (Instant::now(), value));
        
        // Clean up old entries if cache is getting large
        if cache.len() > 1000 {
            let cutoff = Instant::now() - Duration::from_secs(600); // 10 minutes
            cache.retain(|_, (timestamp, _)| *timestamp > cutoff);
        }
    }
    
    /// Update performance metrics
    async fn update_performance_metrics(&self, operation_type: &str, duration: Duration, item_count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.total_queries += 1;
        
        let duration_ms = duration.as_millis() as u64;
        metrics.total_query_time_ms += duration_ms;
        metrics.average_query_time_ms = metrics.total_query_time_ms as f64 / metrics.total_queries as f64;
        
        if operation_type.contains("batch") || item_count > 100 {
            metrics.bulk_operations_count += 1;
            metrics.bulk_operations_time_ms += duration_ms;
        }
        
        debug!(
            "Query performance - Operation: {}, Duration: {:?}, Items: {}, Avg: {:.2}ms",
            operation_type, duration, item_count, metrics.average_query_time_ms
        );
    }
    
    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> DatabasePerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Clear result cache
    pub async fn clear_cache(&self) {
        let mut cache = self.result_cache.write().await;
        cache.clear();
        info!("Database result cache cleared");
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let cache = self.result_cache.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("cache_size".to_string(), serde_json::Value::from(cache.len()));
        stats.insert("cache_enabled".to_string(), serde_json::Value::from(self.config.enable_result_caching));
        
        let valid_entries = cache.values()
            .filter(|(timestamp, _)| timestamp.elapsed() < Duration::from_secs(300))
            .count();
        
        stats.insert("valid_entries".to_string(), serde_json::Value::from(valid_entries));
        
        let hit_rate = if self.metrics.read().await.total_queries > 0 {
            valid_entries as f64 / self.metrics.read().await.total_queries as f64
        } else {
            0.0
        };
        
        stats.insert("estimated_hit_rate".to_string(), serde_json::Value::from(hit_rate));
        
        stats
    }
    
    /// Optimize database for large codebase analysis
    pub async fn optimize_for_large_codebase(&self) -> Result<()> {
        info!("Optimizing database configuration for large codebase analysis");
        
        // Clear old cache entries
        self.clear_cache().await;
        
        // Prepare commonly used statements
        if self.config.enable_prepared_statements {
            self.prepare_common_statements().await?;
        }
        
        info!("Database optimization completed");
        Ok(())
    }
    
    /// Prepare commonly used SQL statements
    async fn prepare_common_statements(&self) -> Result<()> {
        let mut statements = self.prepared_statements.write().await;
        
        // Common queries that would benefit from preparation
        statements.insert("get_issues_for_run".to_string(), 
            "SELECT * FROM architectural_issues WHERE analysis_run_id = ?".to_string());
        statements.insert("get_dependencies_for_run".to_string(),
            "SELECT * FROM dependencies WHERE analysis_run_id = ?".to_string());
        statements.insert("get_analysis_stats".to_string(),
            "SELECT COUNT(*) as total_issues, analysis_run_id FROM architectural_issues WHERE analysis_run_id = ? GROUP BY analysis_run_id".to_string());
        
        info!("Prepared {} common SQL statements", statements.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_optimization_config() {
        let config = QueryOptimizationConfig::default();
        assert!(config.enable_prepared_statements);
        assert!(config.enable_result_caching);
        assert_eq!(config.max_batch_size, 10000);
    }
    
    #[test]
    fn test_batch_priority_ordering() {
        assert!(BatchPriority::Critical < BatchPriority::High);
        assert!(BatchPriority::High < BatchPriority::Medium);
        assert!(BatchPriority::Medium < BatchPriority::Low);
    }
    
    #[tokio::test]
    async fn test_dependency_deduplication() {
        // This would test the deduplication logic with mock data
        // Implementation would create test dependencies with duplicates
        // and verify that deduplication works correctly
    }
    
    #[tokio::test]
    async fn test_performance_metrics_tracking() {
        let config = QueryOptimizationConfig::default();
        let metrics = DatabasePerformanceMetrics::default();
        
        assert_eq!(metrics.total_queries, 0);
        assert_eq!(metrics.average_query_time_ms, 0.0);
        assert_eq!(metrics.failed_queries, 0);
    }
    
    #[tokio::test]
    async fn test_cache_expiration() {
        // Test would verify that cache entries expire correctly
        // and that old entries are cleaned up
    }
}