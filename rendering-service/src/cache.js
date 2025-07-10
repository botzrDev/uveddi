const crypto = require('crypto');
const fs = require('fs').promises;
const path = require('path');

/**
 * Content-addressable cache implementation using SHA-256 hashing
 * Implements the caching strategy from UV-78 research requirements
 */
class ContentAddressableCache {
  constructor(options = {}) {
    this.cacheDir = options.cacheDir || path.join(__dirname, '../cache');
    this.maxCacheSize = options.maxCacheSize || 1024 * 1024 * 1024; // 1GB default
    this.maxAge = options.maxAge || 7 * 24 * 60 * 60 * 1000; // 7 days default
    this.stats = {
      hits: 0,
      misses: 0,
      stores: 0,
      evictions: 0
    };
    
    this.initializeCache();
  }

  async initializeCache() {
    try {
      await fs.mkdir(this.cacheDir, { recursive: true });
      console.log(`Cache initialized at: ${this.cacheDir}`);
    } catch (error) {
      console.error('Failed to initialize cache directory:', error);
    }
  }

  /**
   * Generate SHA-256 hash for content-addressable storage
   * @param {string} mermaidCode - The Mermaid diagram code
   * @param {string} format - Output format (svg/png)
   * @param {number} width - Viewport width
   * @param {number} height - Viewport height
   * @returns {string} SHA-256 hash
   */
  generateCacheKey(mermaidCode, format, width, height) {
    const content = JSON.stringify({
      code: mermaidCode.trim(),
      format: format.toLowerCase(),
      width: parseInt(width),
      height: parseInt(height),
      version: '1.0' // Cache version for invalidation
    });
    
    return crypto.createHash('sha256').update(content, 'utf8').digest('hex');
  }

  /**
   * Get cached item by content hash
   * @param {string} cacheKey - SHA-256 hash key
   * @returns {Object|null} Cached data or null if not found
   */
  async get(cacheKey) {
    try {
      const metadataPath = path.join(this.cacheDir, `${cacheKey}.meta.json`);
      const dataPath = path.join(this.cacheDir, `${cacheKey}.data`);
      
      // Check if both metadata and data files exist
      const [metadataExists, dataExists] = await Promise.all([
        this.fileExists(metadataPath),
        this.fileExists(dataPath)
      ]);
      
      if (!metadataExists || !dataExists) {
        this.stats.misses++;
        return null;
      }
      
      // Read and validate metadata
      const metadataContent = await fs.readFile(metadataPath, 'utf8');
      const metadata = JSON.parse(metadataContent);
      
      // Check if cache entry has expired
      if (Date.now() - metadata.timestamp > this.maxAge) {
        await this.delete(cacheKey);
        this.stats.misses++;
        this.stats.evictions++;
        return null;
      }
      
      // Read cached data
      const data = await fs.readFile(dataPath, 'utf8');
      
      // Update access time for LRU tracking
      metadata.lastAccessed = Date.now();
      await fs.writeFile(metadataPath, JSON.stringify(metadata, null, 2));
      
      this.stats.hits++;
      
      return {
        format: metadata.format,
        data: data,
        dimensions: metadata.dimensions,
        cached: true,
        cacheKey: cacheKey,
        timestamp: metadata.timestamp
      };
      
    } catch (error) {
      console.error(`Cache read error for key ${cacheKey}:`, error);
      this.stats.misses++;
      return null;
    }
  }

  /**
   * Store item in cache with content-addressable key
   * @param {string} cacheKey - SHA-256 hash key
   * @param {Object} result - Rendering result to cache
   * @returns {boolean} Success status
   */
  async set(cacheKey, result) {
    try {
      const timestamp = Date.now();
      const metadataPath = path.join(this.cacheDir, `${cacheKey}.meta.json`);
      const dataPath = path.join(this.cacheDir, `${cacheKey}.data`);
      
      // Prepare metadata
      const metadata = {
        cacheKey: cacheKey,
        format: result.format,
        dimensions: result.dimensions,
        timestamp: timestamp,
        lastAccessed: timestamp,
        size: result.data.length
      };
      
      // Write data and metadata atomically
      await Promise.all([
        fs.writeFile(dataPath, result.data),
        fs.writeFile(metadataPath, JSON.stringify(metadata, null, 2))
      ]);
      
      this.stats.stores++;
      
      // Trigger cleanup if cache is getting large
      setImmediate(() => this.cleanupIfNeeded());
      
      return true;
      
    } catch (error) {
      console.error(`Cache write error for key ${cacheKey}:`, error);
      return false;
    }
  }

  /**
   * Delete cache entry
   * @param {string} cacheKey - SHA-256 hash key
   */
  async delete(cacheKey) {
    try {
      const metadataPath = path.join(this.cacheDir, `${cacheKey}.meta.json`);
      const dataPath = path.join(this.cacheDir, `${cacheKey}.data`);
      
      await Promise.all([
        fs.unlink(metadataPath).catch(() => {}),
        fs.unlink(dataPath).catch(() => {})
      ]);
      
    } catch (error) {
      console.error(`Cache delete error for key ${cacheKey}:`, error);
    }
  }

  /**
   * Get cache statistics
   * @returns {Object} Cache performance metrics
   */
  getStats() {
    const hitRate = this.stats.hits + this.stats.misses > 0 
      ? (this.stats.hits / (this.stats.hits + this.stats.misses) * 100).toFixed(2)
      : 0;
      
    return {
      ...this.stats,
      hitRate: `${hitRate}%`,
      totalRequests: this.stats.hits + this.stats.misses
    };
  }

  /**
   * Clean up expired and oversized cache entries
   */
  async cleanupIfNeeded() {
    try {
      const files = await fs.readdir(this.cacheDir);
      const metaFiles = files.filter(f => f.endsWith('.meta.json'));
      
      if (metaFiles.length === 0) return;
      
      // Read all metadata files
      const entries = [];
      for (const metaFile of metaFiles) {
        try {
          const metadataPath = path.join(this.cacheDir, metaFile);
          const content = await fs.readFile(metadataPath, 'utf8');
          const metadata = JSON.parse(content);
          entries.push(metadata);
        } catch (error) {
          // Skip corrupted metadata files
          continue;
        }
      }
      
      // Remove expired entries
      const now = Date.now();
      const expiredEntries = entries.filter(entry => 
        now - entry.timestamp > this.maxAge
      );
      
      for (const entry of expiredEntries) {
        await this.delete(entry.cacheKey);
        this.stats.evictions++;
      }
      
      // Check total cache size and remove oldest if needed
      const remainingEntries = entries.filter(entry => 
        now - entry.timestamp <= this.maxAge
      );
      
      const totalSize = remainingEntries.reduce((sum, entry) => sum + (entry.size || 0), 0);
      
      if (totalSize > this.maxCacheSize) {
        // Sort by last accessed time (LRU)
        remainingEntries.sort((a, b) => a.lastAccessed - b.lastAccessed);
        
        let currentSize = totalSize;
        const targetSize = this.maxCacheSize * 0.8; // Remove 20% when cleaning
        
        for (const entry of remainingEntries) {
          if (currentSize <= targetSize) break;
          
          await this.delete(entry.cacheKey);
          currentSize -= (entry.size || 0);
          this.stats.evictions++;
        }
      }
      
    } catch (error) {
      console.error('Cache cleanup error:', error);
    }
  }

  /**
   * Clear all cache entries
   */
  async clear() {
    try {
      const files = await fs.readdir(this.cacheDir);
      await Promise.all(
        files.map(file => 
          fs.unlink(path.join(this.cacheDir, file)).catch(() => {})
        )
      );
      
      // Reset stats
      this.stats = {
        hits: 0,
        misses: 0,
        stores: 0,
        evictions: 0
      };
      
      console.log('Cache cleared successfully');
    } catch (error) {
      console.error('Cache clear error:', error);
    }
  }

  /**
   * Check if file exists
   * @param {string} filePath - Path to check
   * @returns {boolean} File existence status
   */
  async fileExists(filePath) {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }
}

class AdvancedCache extends ContentAddressableCache {
  // UV-8: Predictive cache warming using access patterns
  async predictiveCacheWarm(diagramKeys) {
    for (const key of diagramKeys) {
      // Preload cache entries if not present
      const cached = await this.get(key);
      if (!cached) {
        // Optionally trigger background render or prefetch
        // This is a stub for future ML-based prediction
      }
    }
  }
  // UV-8: Context-aware invalidation using dependency graphs (stub)
  async invalidateByDependency(depKey) {
    // Invalidate all cache entries related to a dependency
    // This is a stub for future dependency graph integration
  }
}

module.exports = AdvancedCache;