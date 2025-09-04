//! Scalable Database Manager
//!
//! This module provides read/write separation, load balancing, and high-level
//! database management functionality for production scalability.

use super::providers::{DatabaseProvider, DatabaseConfig, DatabaseType, DatabaseMetrics, DatabaseHealthStatus, create_database_provider};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, AnalysisStats, Dependency};
use crate::error::{Result, UveddiError};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::time::{interval, Duration};

/// Scalable database manager with read/write separation and load balancing
pub struct ScalableDatabase {
    write_provider: Arc<dyn DatabaseProvider>,
    read_providers: Vec<Arc<dyn DatabaseProvider>>,
    load_balancer: ReadLoadBalancer,
    metrics: Arc<DatabaseMetrics>,
    config: DatabaseConfig,
}

impl ScalableDatabase {
    /// Create a new scalable database manager
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Create write provider
        let write_provider = Arc::new(create_database_provider(&config)?);
        
        // Create read providers (can be same as write provider for SQLite)
        let mut read_providers = Vec::new();
        
        if config.read_connection_strings.is_empty() {
            // Single instance - use write provider for reads too
            read_providers.push(write_provider.clone());
        } else {
            // Multiple read instances
            for read_conn_str in &config.read_connection_strings {
                let mut read_config = config.clone();
                read_config.connection_string = read_conn_str.clone();
                let read_provider = Arc::new(create_database_provider(&read_config)?);
                read_providers.push(read_provider);
            }
        }
        
        let load_balancer = ReadLoadBalancer::new(read_providers.len());
        let metrics = Arc::new(DatabaseMetrics::default());
        
        let manager = Self {
            write_provider,
            read_providers,
            load_balancer,
            metrics,
            config,
        };
        
        // Initialize all providers
        manager.initialize().await?;
        
        // Start health monitoring
        manager.start_health_monitoring();
        
        Ok(manager)
    }
    
    /// Initialize all database providers
    async fn initialize(&self) -> Result<()> {
        // Initialize write provider
        self.write_provider.initialize().await?;
        
        // Initialize read providers (skip if same as write provider)
        for read_provider in &self.read_providers {
            if !Arc::ptr_eq(read_provider, &self.write_provider) {
                read_provider.initialize().await?;
            }
        }
        
        Ok(())
    }
    
    /// Start background health monitoring
    fn start_health_monitoring(&self) {
        let write_provider = self.write_provider.clone();
        let read_providers = self.read_providers.clone();
        let load_balancer = self.load_balancer.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Health check every 30 seconds
            
            loop {
                interval.tick().await;
                
                // Test write provider health
                match write_provider.test_connection().await {
                    Ok(_) => {
                        tracing::debug!("Write provider health check passed");
                    }
                    Err(e) => {
                        tracing::warn!("Write provider health check failed: {}", e);
                    }
                }
                
                // Test read providers health and update load balancer
                for (idx, read_provider) in read_providers.iter().enumerate() {
                    match read_provider.test_connection().await {
                        Ok(_) => {
                            load_balancer.mark_healthy(idx);
                            tracing::debug!("Read provider {} health check passed", idx);
                        }
                        Err(e) => {
                            load_balancer.mark_unhealthy(idx);
                            tracing::warn!("Read provider {} health check failed: {}", idx, e);
                        }
                    }
                }
            }
        });
    }
    
    /// Execute a read query with load balancing
    async fn read_query<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<dyn DatabaseProvider>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let provider_idx = self.load_balancer.select_read_provider();
        let provider = self.read_providers[provider_idx].clone();
        
        match operation(provider.clone()).await {
            Ok(result) => {
                self.load_balancer.record_success(provider_idx);
                Ok(result)
            }
            Err(e) => {
                self.load_balancer.record_failure(provider_idx);
                
                // Retry with another provider if available
                if self.read_providers.len() > 1 {
                    let retry_idx = self.load_balancer.select_read_provider();
                    if retry_idx != provider_idx {
                        let retry_provider = self.read_providers[retry_idx].clone();
                        match operation(retry_provider).await {
                            Ok(result) => {
                                self.load_balancer.record_success(retry_idx);
                                return Ok(result);
                            }
                            Err(retry_e) => {
                                self.load_balancer.record_failure(retry_idx);
                                tracing::warn!("Retry failed on provider {}: {}", retry_idx, retry_e);
                            }
                        }
                    }
                }
                
                Err(e)
            }
        }
    }
    
    /// Execute a write query
    async fn write_query<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<dyn DatabaseProvider>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        operation(self.write_provider.clone()).await
    }
    
    // Public API methods that delegate to appropriate providers
    
    pub async fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64> {
        self.write_query(|provider| async move {
            provider.get_or_create_project_id(project_path).await
        }).await
    }
    
    pub async fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun> {
        self.write_query(|provider| async move {
            provider.create_analysis_run(project_path).await
        }).await
    }
    
    pub async fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()> {
        self.write_query(|provider| async move {
            provider.update_analysis_run(run).await
        }).await
    }
    
    pub async fn store_anti_pattern_types_batch(&self, anti_pattern_types: &mut [AntiPatternType]) -> Result<()> {
        self.write_query(|provider| async move {
            provider.store_anti_pattern_types_batch(anti_pattern_types).await
        }).await
    }
    
    pub async fn store_issues_batch(&self, issues: &[ArchitecturalIssue]) -> Result<()> {
        self.write_query(|provider| async move {
            provider.store_issues_batch(issues).await
        }).await
    }
    
    pub async fn store_dependencies_batch(&self, run_id: i64, dependencies: &[Dependency]) -> Result<()> {
        self.write_query(|provider| async move {
            provider.store_dependencies_batch(run_id, dependencies).await
        }).await
    }
    
    pub async fn get_analysis_run(&self, run_id: i64) -> Result<Option<AnalysisRun>> {
        self.read_query(|provider| async move {
            provider.get_analysis_run(run_id).await
        }).await
    }
    
    pub async fn get_latest_analysis_run(&self) -> Result<Option<AnalysisRun>> {
        self.read_query(|provider| async move {
            provider.get_latest_analysis_run().await
        }).await
    }
    
    pub async fn get_recent_analysis_runs(&self, limit: u32) -> Result<Vec<AnalysisRun>> {
        self.read_query(|provider| async move {
            provider.get_recent_analysis_runs(limit).await
        }).await
    }
    
    pub async fn get_issues_for_run(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>> {
        self.read_query(|provider| async move {
            provider.get_issues_for_run(run_id).await
        }).await
    }
    
    pub async fn get_dependencies_for_run(&self, run_id: i64) -> Result<Vec<Dependency>> {
        self.read_query(|provider| async move {
            provider.get_dependencies_for_run(run_id).await
        }).await
    }
    
    pub async fn get_issues_with_types_for_run(&self, run_id: i64) -> Result<Vec<(ArchitecturalIssue, AntiPatternType)>> {
        self.read_query(|provider| async move {
            provider.get_issues_with_types_for_run(run_id).await
        }).await
    }
    
    pub async fn get_analysis_stats(&self, run_id: i64) -> Result<AnalysisStats> {
        self.read_query(|provider| async move {
            provider.get_analysis_stats(run_id).await
        }).await
    }
    
    pub async fn get_issues_paginated(
        &self,
        run_id: i64,
        offset: u32,
        limit: u32,
        severity_filter: Option<&str>,
        detector_filter: Option<&str>,
    ) -> Result<Vec<ArchitecturalIssue>> {
        self.read_query(|provider| async move {
            provider.get_issues_paginated(run_id, offset, limit, severity_filter, detector_filter).await
        }).await
    }
    
    pub async fn get_all_anti_pattern_types(&self) -> Result<Vec<AntiPatternType>> {
        self.read_query(|provider| async move {
            provider.get_all_anti_pattern_types().await
        }).await
    }
    
    pub async fn get_project_path(&self, project_id: i64) -> Result<String> {
        self.read_query(|provider| async move {
            provider.get_project_path(project_id).await
        }).await
    }
    
    /// Get comprehensive health status
    pub async fn get_health_status(&self) -> Result<DatabaseHealthStatus> {
        let write_health = self.write_provider.get_health_status().await?;
        
        // Check read provider health
        let mut read_healths = Vec::new();
        for provider in &self.read_providers {
            if let Ok(health) = provider.get_health_status().await {
                read_healths.push(health);
            }
        }
        
        // Aggregate health status
        let is_healthy = write_health.is_healthy && 
            read_healths.iter().any(|h| h.is_healthy);
        
        Ok(DatabaseHealthStatus {
            is_healthy,
            active_connections: write_health.active_connections + 
                read_healths.iter().map(|h| h.active_connections).sum::<u32>(),
            pool_utilization: write_health.pool_utilization,
            last_successful_query: write_health.last_successful_query,
            error_count: write_health.error_count,
            average_query_time: write_health.average_query_time,
        })
    }
    
    /// Cleanup expired connections across all providers
    pub async fn cleanup(&self) -> Result<u64> {
        let mut total_cleaned = 0;
        
        total_cleaned += self.write_provider.cleanup().await?;
        
        for provider in &self.read_providers {
            if !Arc::ptr_eq(provider, &self.write_provider) {
                total_cleaned += provider.cleanup().await?;
            }
        }
        
        Ok(total_cleaned)
    }
    
    /// Get load balancer statistics
    pub fn get_load_balancer_stats(&self) -> LoadBalancerStats {
        self.load_balancer.get_stats()
    }
}

/// Read load balancer for distributing read queries across multiple providers
#[derive(Clone)]
pub struct ReadLoadBalancer {
    current: Arc<AtomicUsize>,
    provider_states: Arc<std::sync::Mutex<Vec<ProviderState>>>,
}

#[derive(Debug, Clone)]
struct ProviderState {
    is_healthy: bool,
    success_count: u64,
    failure_count: u64,
    last_used: std::time::Instant,
}

impl ReadLoadBalancer {
    fn new(provider_count: usize) -> Self {
        let provider_states = (0..provider_count)
            .map(|_| ProviderState {
                is_healthy: true,
                success_count: 0,
                failure_count: 0,
                last_used: std::time::Instant::now(),
            })
            .collect();
        
        Self {
            current: Arc::new(AtomicUsize::new(0)),
            provider_states: Arc::new(std::sync::Mutex::new(provider_states)),
        }
    }
    
    /// Select a read provider using round-robin with health checking
    fn select_read_provider(&self) -> usize {
        let states = self.provider_states.lock().unwrap();
        let provider_count = states.len();
        
        if provider_count == 0 {
            return 0;
        }
        
        // Find healthy providers
        let healthy_indices: Vec<usize> = states
            .iter()
            .enumerate()
            .filter(|(_, state)| state.is_healthy)
            .map(|(idx, _)| idx)
            .collect();
        
        if healthy_indices.is_empty() {
            // All providers unhealthy, use round-robin anyway
            let next = self.current.fetch_add(1, Ordering::Relaxed) % provider_count;
            return next;
        }
        
        // Round-robin among healthy providers
        let next = self.current.fetch_add(1, Ordering::Relaxed) % healthy_indices.len();
        healthy_indices[next]
    }
    
    fn mark_healthy(&self, provider_idx: usize) {
        if let Ok(mut states) = self.provider_states.lock() {
            if let Some(state) = states.get_mut(provider_idx) {
                state.is_healthy = true;
            }
        }
    }
    
    fn mark_unhealthy(&self, provider_idx: usize) {
        if let Ok(mut states) = self.provider_states.lock() {
            if let Some(state) = states.get_mut(provider_idx) {
                state.is_healthy = false;
            }
        }
    }
    
    fn record_success(&self, provider_idx: usize) {
        if let Ok(mut states) = self.provider_states.lock() {
            if let Some(state) = states.get_mut(provider_idx) {
                state.success_count += 1;
                state.last_used = std::time::Instant::now();
            }
        }
    }
    
    fn record_failure(&self, provider_idx: usize) {
        if let Ok(mut states) = self.provider_states.lock() {
            if let Some(state) = states.get_mut(provider_idx) {
                state.failure_count += 1;
                // Mark unhealthy if failure rate is too high
                if state.failure_count > state.success_count * 2 {
                    state.is_healthy = false;
                }
            }
        }
    }
    
    fn get_stats(&self) -> LoadBalancerStats {
        let states = self.provider_states.lock().unwrap();
        let total_requests = states.iter()
            .map(|s| s.success_count + s.failure_count)
            .sum();
        
        let healthy_providers = states.iter()
            .filter(|s| s.is_healthy)
            .count();
        
        LoadBalancerStats {
            total_providers: states.len(),
            healthy_providers,
            total_requests,
            current_provider: self.current.load(Ordering::Relaxed),
        }
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_providers: usize,
    pub healthy_providers: usize,
    pub total_requests: u64,
    pub current_provider: usize,
}

impl Clone for ScalableDatabase {
    fn clone(&self) -> Self {
        Self {
            write_provider: self.write_provider.clone(),
            read_providers: self.read_providers.clone(),
            load_balancer: self.load_balancer.clone(),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
        }
    }
}