//! Cache-related database models
//!
//! This module defines models for caching system used by the analysis engine
//! to store and retrieve parsed ASTs and analysis results.

use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Database record for cache entries
///
/// Represents cached data stored in the database for faster retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryRecord {
    /// Primary key identifier
    pub cache_id: Option<i64>,
    /// Key used to identify the cached item
    pub cache_key: String,
    /// Serialized cache data
    pub data: String,
    /// When this cache entry was created
    pub created_at: DateTime<Utc>,
    /// When this cache entry expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Size of the cached data in bytes
    pub size_bytes: i64,
}

/// Domain model for cache entries
///
/// Business logic representation of cached data with type safety.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cache identifier
    pub id: Option<i64>,
    /// Key for cache lookup
    pub key: String,
    /// The cached data (could be AST, analysis results, etc.)
    pub data: Vec<u8>,
    /// Cache metadata
    pub metadata: CacheMetadata,
}

/// Metadata associated with cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Type of cached data (ast, analysis_result, etc.)
    pub data_type: String,
    /// File path that this cache entry relates to
    pub file_path: Option<PathBuf>,
    /// Hash of the original file content
    pub content_hash: Option<String>,
    /// When the cache entry was created
    pub created_at: DateTime<Utc>,
    /// When the cache entry expires (None = no expiration)
    pub expires_at: Option<DateTime<Utc>>,
    /// Size in bytes
    pub size_bytes: u64,
    /// Number of times this cache entry has been accessed
    pub access_count: u64,
    /// Last time this cache entry was accessed
    pub last_accessed_at: Option<DateTime<Utc>>,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self {
            data_type: "unknown".to_string(),
            file_path: None,
            content_hash: None,
            created_at: Utc::now(),
            expires_at: None,
            size_bytes: 0,
            access_count: 0,
            last_accessed_at: None,
        }
    }
}

/// Conversion from domain model to persistence model
impl From<CacheEntry> for CacheEntryRecord {
    fn from(entry: CacheEntry) -> Self {
        let data_str = base64::engine::general_purpose::STANDARD.encode(&entry.data);
        Self {
            cache_id: entry.id,
            cache_key: entry.key,
            data: data_str,
            created_at: entry.metadata.created_at,
            expires_at: entry.metadata.expires_at,
            size_bytes: entry.metadata.size_bytes as i64,
        }
    }
}

/// Conversion from persistence model to domain model
impl TryFrom<CacheEntryRecord> for CacheEntry {
    type Error = base64::DecodeError;

    fn try_from(record: CacheEntryRecord) -> Result<Self, Self::Error> {
        let data = base64::engine::general_purpose::STANDARD.decode(&record.data)?;
        let metadata = CacheMetadata {
            data_type: "unknown".to_string(), // Would need to be stored separately
            file_path: None,
            content_hash: None,
            created_at: record.created_at,
            expires_at: record.expires_at,
            size_bytes: record.size_bytes as u64,
            access_count: 0,
            last_accessed_at: None,
        };

        Ok(Self {
            id: record.cache_id,
            key: record.cache_key,
            data,
            metadata,
        })
    }
}
