//! Rate Limiting Implementation for Uveddi API Security
//!
//! This module provides comprehensive rate limiting functionality using various
//! algorithms and storage backends to prevent abuse and ensure fair usage.

use crate::security::{
    config::RateLimitingConfig,
    errors::{SecurityError, SecurityResult},
    models::{RateLimitIdentifierType, RateLimitInfo},
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_governor::{governor::GovernorConfig, governor::GovernorConfigBuilder};

/// Rate limiter trait for different implementations
#[async_trait::async_trait]
pub trait RateLimitStorage: Send + Sync {
    /// Check if request is within rate limit
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_seconds: u32,
    ) -> SecurityResult<bool>;

    /// Get current rate limit info
    async fn get_rate_limit_info(&self, key: &str) -> SecurityResult<Option<RateLimitInfo>>;

    /// Reset rate limit for a key
    async fn reset_rate_limit(&self, key: &str) -> SecurityResult<()>;

    /// Clean up expired entries
    async fn cleanup_expired(&self) -> SecurityResult<()>;
}

/// In-memory rate limit storage
pub struct InMemoryRateLimitStorage {
    data: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: DateTime<Utc>,
    window_size: u32,
    last_request: DateTime<Utc>,
}

impl InMemoryRateLimitStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn is_expired(&self, entry: &RateLimitEntry) -> bool {
        let now = Utc::now();
        let window_end = entry.window_start + chrono::Duration::seconds(entry.window_size as i64);
        now > window_end
    }
}

impl Default for InMemoryRateLimitStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RateLimitStorage for InMemoryRateLimitStorage {
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_seconds: u32,
    ) -> SecurityResult<bool> {
        let mut data = self.data.write().await;
        let now = Utc::now();

        match data.get_mut(key) {
            Some(entry) => {
                // Check if current window is expired
                if self.is_expired(entry).await {
                    // Start new window
                    entry.count = 1;
                    entry.window_start = now;
                    entry.window_size = window_seconds;
                    entry.last_request = now;
                    Ok(true)
                } else {
                    // Increment counter in current window
                    entry.count += 1;
                    entry.last_request = now;

                    // Check if limit is exceeded
                    Ok(entry.count <= limit)
                }
            }
            None => {
                // First request for this key
                data.insert(
                    key.to_string(),
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                        window_size: window_seconds,
                        last_request: now,
                    },
                );
                Ok(true)
            }
        }
    }

    async fn get_rate_limit_info(&self, key: &str) -> SecurityResult<Option<RateLimitInfo>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) => {
                let identifier_type = if key.starts_with("ip:") {
                    RateLimitIdentifierType::IpAddress
                } else if key.starts_with("user:") {
                    RateLimitIdentifierType::UserId
                } else if key.starts_with("api_key:") {
                    RateLimitIdentifierType::ApiKey
                } else {
                    RateLimitIdentifierType::IpAddress
                };

                Ok(Some(RateLimitInfo {
                    identifier: key.to_string(),
                    identifier_type,
                    endpoint: "unknown".to_string(), // Would need to be passed in
                    request_count: entry.count,
                    window_start: entry.window_start,
                    window_size: entry.window_size,
                    limit: 0, // Would need to be passed in
                }))
            }
            None => Ok(None),
        }
    }

    async fn reset_rate_limit(&self, key: &str) -> SecurityResult<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn cleanup_expired(&self) -> SecurityResult<()> {
        let mut data = self.data.write().await;
        let now = Utc::now();

        data.retain(|_, entry| {
            let window_end =
                entry.window_start + chrono::Duration::seconds(entry.window_size as i64);
            now <= window_end
        });

        Ok(())
    }
}

/// Redis-based rate limit storage (placeholder implementation)
pub struct RedisRateLimitStorage {
    redis_url: String,
    // In a real implementation, this would contain a Redis client
}

impl RedisRateLimitStorage {
    pub fn new(redis_url: String) -> Self {
        Self { redis_url }
    }
}

#[async_trait::async_trait]
impl RateLimitStorage for RedisRateLimitStorage {
    async fn check_rate_limit(
        &self,
        _key: &str,
        _limit: u32,
        _window_seconds: u32,
    ) -> SecurityResult<bool> {
        // This would implement Redis-based rate limiting using sliding window
        // For now, return true (allow all requests)
        Ok(true)
    }

    async fn get_rate_limit_info(&self, _key: &str) -> SecurityResult<Option<RateLimitInfo>> {
        // This would fetch rate limit info from Redis
        Ok(None)
    }

    async fn reset_rate_limit(&self, _key: &str) -> SecurityResult<()> {
        // This would reset the rate limit in Redis
        Ok(())
    }

    async fn cleanup_expired(&self) -> SecurityResult<()> {
        // Redis handles expiration automatically
        Ok(())
    }
}

/// Main rate limiter
pub struct RateLimiter {
    storage: Arc<dyn RateLimitStorage>,
    config: RateLimitingConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with in-memory storage
    pub fn new_in_memory(config: RateLimitingConfig) -> Self {
        Self {
            storage: Arc::new(InMemoryRateLimitStorage::new()),
            config,
        }
    }

    /// Create a new rate limiter with Redis storage
    pub fn new_redis(config: RateLimitingConfig, redis_url: String) -> Self {
        Self {
            storage: Arc::new(RedisRateLimitStorage::new(redis_url)),
            config,
        }
    }

    /// Create a new rate limiter with custom storage
    pub fn new_with_storage(
        storage: Arc<dyn RateLimitStorage>,
        config: RateLimitingConfig,
    ) -> Self {
        Self { storage, config }
    }

    /// Check if a request is within rate limits
    pub async fn check_rate_limit(&self, identifier: &str, endpoint: &str) -> SecurityResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }

        // Determine rate limit based on identifier type and endpoint
        let limit = self.get_rate_limit_for_identifier(identifier, endpoint);
        let window_seconds = self.config.window_size;

        // Create rate limit key
        let key = format!("{}:{}", identifier, endpoint);

        // Check rate limit
        self.storage
            .check_rate_limit(&key, limit, window_seconds)
            .await
    }

    /// Get rate limit information for an identifier
    pub async fn get_rate_limit_info(
        &self,
        identifier: &str,
        endpoint: &str,
    ) -> SecurityResult<Option<RateLimitInfo>> {
        let key = format!("{}:{}", identifier, endpoint);
        self.storage.get_rate_limit_info(&key).await
    }

    /// Reset rate limit for an identifier
    pub async fn reset_rate_limit(&self, identifier: &str, endpoint: &str) -> SecurityResult<()> {
        let key = format!("{}:{}", identifier, endpoint);
        self.storage.reset_rate_limit(&key).await
    }

    /// Clean up expired rate limit entries
    pub async fn cleanup_expired(&self) -> SecurityResult<()> {
        self.storage.cleanup_expired().await
    }

    /// Get rate limit based on identifier type and endpoint
    fn get_rate_limit_for_identifier(&self, identifier: &str, endpoint: &str) -> u32 {
        // Check for endpoint-specific limits first
        if let Some(&limit) = self.config.endpoint_limits.get(endpoint) {
            return limit;
        }

        // Check identifier type
        if identifier.starts_with("user:") {
            self.config.user_rate_limit
        } else if identifier.starts_with("api_key:") {
            self.config.api_key_rate_limit
        } else {
            // Default to IP-based limit
            self.config.default_rate_limit
        }
    }

    /// Get remaining requests for an identifier
    pub async fn get_remaining_requests(
        &self,
        identifier: &str,
        endpoint: &str,
    ) -> SecurityResult<u32> {
        let info = self.get_rate_limit_info(identifier, endpoint).await?;

        match info {
            Some(info) => {
                let limit = self.get_rate_limit_for_identifier(identifier, endpoint);
                Ok(info.remaining_requests().min(limit))
            }
            None => {
                // No requests made yet
                Ok(self.get_rate_limit_for_identifier(identifier, endpoint))
            }
        }
    }

    /// Check if identifier is currently rate limited
    pub async fn is_rate_limited(&self, identifier: &str, endpoint: &str) -> SecurityResult<bool> {
        let allowed = self.check_rate_limit(identifier, endpoint).await?;
        Ok(!allowed)
    }
}

/// Rate limit middleware configuration
pub struct RateLimitMiddlewareConfig {
    /// Rate limiter instance
    pub rate_limiter: Arc<RateLimiter>,
    /// Custom identifier extractor
    pub identifier_extractor: Option<
        Box<dyn Fn(&axum::http::HeaderMap, &axum::extract::Request) -> String + Send + Sync>,
    >,
    /// Custom endpoint extractor
    pub endpoint_extractor: Option<Box<dyn Fn(&axum::extract::Request) -> String + Send + Sync>>,
    /// Custom rate limit exceeded handler
    pub rate_limit_exceeded_handler:
        Option<Box<dyn Fn() -> axum::response::Response + Send + Sync>>,
}

impl RateLimitMiddlewareConfig {
    /// Create new rate limit middleware configuration
    pub fn new(rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            rate_limiter,
            identifier_extractor: None,
            endpoint_extractor: None,
            rate_limit_exceeded_handler: None,
        }
    }

    /// Set custom identifier extractor
    pub fn with_identifier_extractor<F>(mut self, extractor: F) -> Self
    where
        F: Fn(&axum::http::HeaderMap, &axum::extract::Request) -> String + Send + Sync + 'static,
    {
        self.identifier_extractor = Some(Box::new(extractor));
        self
    }

    /// Set custom endpoint extractor
    pub fn with_endpoint_extractor<F>(mut self, extractor: F) -> Self
    where
        F: Fn(&axum::extract::Request) -> String + Send + Sync + 'static,
    {
        self.endpoint_extractor = Some(Box::new(extractor));
        self
    }

    /// Set custom rate limit exceeded handler
    pub fn with_rate_limit_exceeded_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> axum::response::Response + Send + Sync + 'static,
    {
        self.rate_limit_exceeded_handler = Some(Box::new(handler));
        self
    }
}

/// Tower-governor integration for Axum (simplified)  
pub fn create_governor_config(config: &RateLimitingConfig) -> (u32, u32) {
    // Simplified rate limiting configuration
    // In a real implementation, this would return the proper GovernorConfig
    // but for now we'll return the config parameters as a tuple
    (config.default_rate_limit, config.burst_size)
}

/// Rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub rejected_requests: u64,
    pub active_keys: u64,
    pub top_rate_limited_identifiers: Vec<(String, u32)>,
}

impl RateLimitStats {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            allowed_requests: 0,
            rejected_requests: 0,
            active_keys: 0,
            top_rate_limited_identifiers: Vec::new(),
        }
    }

    pub fn rejection_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.rejected_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

impl Default for RateLimitStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit manager for administrative operations
pub struct RateLimitManager {
    limiter: Arc<RateLimiter>,
    stats: Arc<RwLock<RateLimitStats>>,
}

impl RateLimitManager {
    pub fn new(limiter: Arc<RateLimiter>) -> Self {
        Self {
            limiter,
            stats: Arc::new(RwLock::new(RateLimitStats::new())),
        }
    }

    /// Get rate limiting statistics
    pub async fn get_stats(&self) -> RateLimitStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = RateLimitStats::new();
    }

    /// Reset rate limit for a specific identifier
    pub async fn reset_rate_limit(&self, identifier: &str, endpoint: &str) -> SecurityResult<()> {
        self.limiter.reset_rate_limit(identifier, endpoint).await
    }

    /// Get all active rate limits
    pub async fn get_active_limits(&self) -> SecurityResult<Vec<RateLimitInfo>> {
        // This would query the storage backend for all active limits
        // For now, return empty vector
        Ok(Vec::new())
    }

    /// Block an identifier (set rate limit to 0)
    pub async fn block_identifier(&self, identifier: &str, endpoint: &str) -> SecurityResult<()> {
        // This would set a special entry to block all requests
        // For now, just reset the rate limit
        self.limiter.reset_rate_limit(identifier, endpoint).await
    }

    /// Unblock an identifier
    pub async fn unblock_identifier(&self, identifier: &str, endpoint: &str) -> SecurityResult<()> {
        // This would remove the block entry
        // For now, just reset the rate limit
        self.limiter.reset_rate_limit(identifier, endpoint).await
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(&self) {
        let limiter = self.limiter.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                if let Err(e) = limiter.cleanup_expired().await {
                    log::error!("Failed to cleanup expired rate limit entries: {}", e);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_in_memory_rate_limiting() {
        let storage = InMemoryRateLimitStorage::new();

        // First request should be allowed
        let result = storage.check_rate_limit("test_key", 5, 60).await.unwrap();
        assert!(result);

        // More requests within limit should be allowed
        for _ in 0..4 {
            let result = storage.check_rate_limit("test_key", 5, 60).await.unwrap();
            assert!(result);
        }

        // Request exceeding limit should be denied
        let result = storage.check_rate_limit("test_key", 5, 60).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_rate_limit_window_expiration() {
        let storage = InMemoryRateLimitStorage::new();

        // Fill up the rate limit
        for _ in 0..5 {
            storage.check_rate_limit("test_key", 5, 1).await.unwrap(); // 1 second window
        }

        // Should be rate limited
        let result = storage.check_rate_limit("test_key", 5, 1).await.unwrap();
        assert!(!result);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be allowed again
        let result = storage.check_rate_limit("test_key", 5, 1).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_identifiers() {
        let config = RateLimitingConfig::default();
        let limiter = RateLimiter::new_in_memory(config);

        // Different identifier types should have different limits
        let ip_allowed = limiter
            .check_rate_limit("ip:192.168.1.1", "/api/test")
            .await
            .unwrap();
        assert!(ip_allowed);

        let user_allowed = limiter
            .check_rate_limit("user:123", "/api/test")
            .await
            .unwrap();
        assert!(user_allowed);

        let api_key_allowed = limiter
            .check_rate_limit("api_key:abc123", "/api/test")
            .await
            .unwrap();
        assert!(api_key_allowed);
    }

    #[tokio::test]
    async fn test_endpoint_specific_limits() {
        let mut config = RateLimitingConfig::default();
        config
            .endpoint_limits
            .insert("/api/auth/login".to_string(), 2);

        let limiter = RateLimiter::new_in_memory(config);

        // First two requests should be allowed
        assert!(limiter
            .check_rate_limit("ip:192.168.1.1", "/api/auth/login")
            .await
            .unwrap());
        assert!(limiter
            .check_rate_limit("ip:192.168.1.1", "/api/auth/login")
            .await
            .unwrap());

        // Third request should be denied
        assert!(!limiter
            .check_rate_limit("ip:192.168.1.1", "/api/auth/login")
            .await
            .unwrap());

        // Different endpoint should still be allowed
        assert!(limiter
            .check_rate_limit("ip:192.168.1.1", "/api/projects")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let config = RateLimitingConfig::default();
        let limiter = RateLimiter::new_in_memory(config);

        // Fill up rate limit
        for _ in 0..60 {
            limiter
                .check_rate_limit("ip:192.168.1.1", "/api/test")
                .await
                .unwrap();
        }

        // Should be rate limited
        assert!(!limiter
            .check_rate_limit("ip:192.168.1.1", "/api/test")
            .await
            .unwrap());

        // Reset rate limit
        limiter
            .reset_rate_limit("ip:192.168.1.1", "/api/test")
            .await
            .unwrap();

        // Should be allowed again
        assert!(limiter
            .check_rate_limit("ip:192.168.1.1", "/api/test")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let storage = InMemoryRateLimitStorage::new();

        // Add some entries with short window
        storage.check_rate_limit("key1", 5, 1).await.unwrap();
        storage.check_rate_limit("key2", 5, 1).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup expired entries
        storage.cleanup_expired().await.unwrap();

        // Verify entries are cleaned up by checking if new request starts fresh window
        let info1 = storage.get_rate_limit_info("key1").await.unwrap();
        let info2 = storage.get_rate_limit_info("key2").await.unwrap();

        // After cleanup, getting info for cleaned up keys should return None
        // or if they exist, they should be fresh entries
        if let Some(info) = info1 {
            assert_eq!(info.request_count, 0);
        }
        if let Some(info) = info2 {
            assert_eq!(info.request_count, 0);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_manager() {
        let config = RateLimitingConfig::default();
        let limiter = Arc::new(RateLimiter::new_in_memory(config));
        let manager = RateLimitManager::new(limiter);

        // Get initial stats
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.allowed_requests, 0);
        assert_eq!(stats.rejected_requests, 0);

        // Reset rate limit
        manager
            .reset_rate_limit("test_id", "/api/test")
            .await
            .unwrap();

        // Get active limits
        let active_limits = manager.get_active_limits().await.unwrap();
        assert!(active_limits.is_empty()); // Empty in test implementation
    }

    #[test]
    fn test_rate_limit_stats() {
        let mut stats = RateLimitStats::new();
        stats.total_requests = 100;
        stats.rejected_requests = 25;

        assert_eq!(stats.rejection_rate(), 25.0);

        let stats_zero = RateLimitStats::new();
        assert_eq!(stats_zero.rejection_rate(), 0.0);
    }

    #[test]
    fn test_governor_config_creation() {
        let config = RateLimitingConfig::default();
        let governor_config = create_governor_config(&config);

        // This just tests that the config can be created without panicking
        let (rate, burst) = governor_config;
        assert!(rate > 0);
        assert!(burst > 0);
    }
}
