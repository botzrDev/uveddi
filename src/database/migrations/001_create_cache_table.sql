-- Create cache table for storing analysis cache entries
-- Used by CacheRepository for persisting analysis results

CREATE TABLE cache (
    -- Unique identifier for the cache entry
    id TEXT PRIMARY KEY,

    -- Cache key for lookups (must be unique)
    key TEXT NOT NULL UNIQUE,

    -- Cached data (stored as binary blob)
    data BLOB NOT NULL,

    -- When the cache entry was created
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- When the cache entry expires (NULL for no expiration)
    expires_at DATETIME,

    -- Size in bytes for cache management
    size_bytes INTEGER DEFAULT 0,

    -- Hit count for cache statistics
    hit_count INTEGER DEFAULT 0,

    -- Last access time for LRU eviction
    last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index on key for fast lookups
CREATE INDEX idx_cache_key ON cache(key);

-- Index on expires_at for cleanup jobs
CREATE INDEX idx_cache_expires ON cache(expires_at);

-- Index on last_accessed for LRU eviction
CREATE INDEX idx_cache_last_accessed ON cache(last_accessed);