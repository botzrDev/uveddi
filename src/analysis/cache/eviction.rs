//! Cache Eviction Policies
//!
//! This module provides various eviction strategies for cache management,
//! including LRU, LFU, and memory-based eviction policies.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::Instant;

/// Available eviction policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumeration of evictionpolicy variants.
pub enum EvictionPolicy {
    /// Least Recently Used - removes least recently accessed items
    LRU,
    /// Least Frequently Used - removes items with lowest access frequency
    LFU,
    /// First In, First Out - removes items in insertion order
    FIFO,
    /// Time-based - removes items older than a threshold
    TTL,
    /// Memory-based - removes items when memory pressure is high
    MemoryBased,
}

/// Metadata for cache entries used in eviction decisions
#[derive(Debug, Clone)]
/// Metadata describing entry properties and attributes.
pub struct EntryMetadata {
    pub last_accessed: Instant,
    pub access_count: u64,
    pub inserted_at: Instant,
    pub estimated_size_bytes: usize,
}

impl EntryMetadata {
    /// Creates a new instance.
    pub fn new(estimated_size: usize) -> Self {
        let now = Instant::now();
        Self {
            last_accessed: now,
            access_count: 1,
            inserted_at: now,
            estimated_size_bytes: estimated_size,
        }
    }

    /// Performs touch operation.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count = self.access_count.saturating_add(1);
    }

    /// Performs age operation.
    pub fn age(&self) -> std::time::Duration {
        self.inserted_at.elapsed()
    }

    /// Performs idle time operation.
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_accessed.elapsed()
    }
}

/// Eviction manager handles different eviction strategies
pub struct EvictionManager<K: Clone + Eq + Hash> {
    policy: EvictionPolicy,
    metadata: HashMap<K, EntryMetadata>,
    access_order: VecDeque<K>, // For LRU tracking
    total_memory_bytes: usize,
    max_memory_bytes: Option<usize>,
    max_entries: Option<usize>,
    ttl_seconds: Option<u64>,
}

impl<K: Clone + Eq + Hash> EvictionManager<K> {
    /// Create a new eviction manager
    pub fn new(policy: EvictionPolicy) -> Self {
        Self {
            policy,
            metadata: HashMap::new(),
            access_order: VecDeque::new(),
            total_memory_bytes: 0,
            max_memory_bytes: None,
            max_entries: None,
            ttl_seconds: None,
        }
    }

    /// Set maximum memory limit in bytes
    pub fn with_max_memory(mut self, max_bytes: usize) -> Self {
        self.max_memory_bytes = Some(max_bytes);
        self
    }

    /// Set maximum number of entries
    pub fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = Some(max_entries);
        self
    }

    /// Set TTL in seconds
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }

    /// Record access to an entry
    pub fn record_access(&mut self, key: &K, estimated_size: usize) {
        if let Some(metadata) = self.metadata.get_mut(key) {
            metadata.touch();

            // Update LRU order
            if self.policy == EvictionPolicy::LRU {
                // Remove from current position and add to end
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                }
                self.access_order.push_back(key.clone());
            }
        } else {
            // New entry
            let metadata = EntryMetadata::new(estimated_size);
            self.metadata.insert(key.clone(), metadata);
            self.total_memory_bytes = self.total_memory_bytes.saturating_add(estimated_size);

            if self.policy == EvictionPolicy::LRU || self.policy == EvictionPolicy::FIFO {
                self.access_order.push_back(key.clone());
            }
        }
    }

    /// Remove an entry from tracking
    pub fn remove_entry(&mut self, key: &K) -> Option<usize> {
        if let Some(metadata) = self.metadata.remove(key) {
            self.total_memory_bytes = self
                .total_memory_bytes
                .saturating_sub(metadata.estimated_size_bytes);

            // Remove from access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }

            Some(metadata.estimated_size_bytes)
        } else {
            None
        }
    }

    /// Check if eviction is needed based on configured limits
    pub fn needs_eviction(&self) -> bool {
        // Check memory limit
        if let Some(max_memory) = self.max_memory_bytes {
            if self.total_memory_bytes > max_memory {
                return true;
            }
        }

        // Check entry count limit
        if let Some(max_entries) = self.max_entries {
            if self.metadata.len() > max_entries {
                return true;
            }
        }

        // Check for expired entries if TTL is set
        if let Some(ttl) = self.ttl_seconds {
            let ttl_duration = std::time::Duration::from_secs(ttl);
            if self.metadata.values().any(|meta| meta.age() > ttl_duration) {
                return true;
            }
        }

        false
    }

    /// Select entries for eviction based on the configured policy
    pub fn select_for_eviction(&self, count: usize) -> Vec<K> {
        match self.policy {
            EvictionPolicy::LRU => self.select_lru(count),
            EvictionPolicy::LFU => self.select_lfu(count),
            EvictionPolicy::FIFO => self.select_fifo(count),
            EvictionPolicy::TTL => self.select_expired(),
            EvictionPolicy::MemoryBased => self.select_memory_based(count),
        }
    }

    /// Select entries using LRU policy
    fn select_lru(&self, count: usize) -> Vec<K> {
        self.access_order.iter().take(count).cloned().collect()
    }

    /// Select entries using LFU policy
    fn select_lfu(&self, count: usize) -> Vec<K> {
        let mut entries: Vec<_> = self.metadata.iter().collect();
        entries.sort_by_key(|(_, meta)| meta.access_count);

        entries
            .into_iter()
            .take(count)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Select entries using FIFO policy
    fn select_fifo(&self, count: usize) -> Vec<K> {
        self.access_order.iter().take(count).cloned().collect()
    }

    /// Select expired entries
    fn select_expired(&self) -> Vec<K> {
        if let Some(ttl) = self.ttl_seconds {
            let ttl_duration = std::time::Duration::from_secs(ttl);
            self.metadata
                .iter()
                .filter(|(_, meta)| meta.age() > ttl_duration)
                .map(|(key, _)| key.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Select entries based on memory usage (largest first)
    fn select_memory_based(&self, count: usize) -> Vec<K> {
        let mut entries: Vec<_> = self.metadata.iter().collect();
        entries.sort_by_key(|(_, meta)| std::cmp::Reverse(meta.estimated_size_bytes));

        entries
            .into_iter()
            .take(count)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Get current memory usage
    pub fn current_memory_usage(&self) -> usize {
        self.total_memory_bytes
    }

    /// Get current entry count
    pub fn current_entry_count(&self) -> usize {
        self.metadata.len()
    }

    /// Get statistics for a specific entry
    pub fn get_entry_stats(&self, key: &K) -> Option<&EntryMetadata> {
        self.metadata.get(key)
    }

    /// Get overall statistics
    pub fn get_stats(&self) -> EvictionStats {
        let total_access_count: u64 = self.metadata.values().map(|meta| meta.access_count).sum();
        let average_age = if !self.metadata.is_empty() {
            self.metadata
                .values()
                .map(|meta| meta.age().as_secs())
                .sum::<u64>()
                / self.metadata.len() as u64
        } else {
            0
        };

        EvictionStats {
            total_entries: self.metadata.len(),
            total_memory_bytes: self.total_memory_bytes,
            total_access_count,
            average_age_seconds: average_age,
            policy: self.policy,
        }
    }
}

/// Statistics about the eviction manager
#[derive(Debug, Clone)]
/// Represents eviction stats in the system.
pub struct EvictionStats {
    pub total_entries: usize,
    pub total_memory_bytes: usize,
    pub total_access_count: u64,
    pub average_age_seconds: u64,
    pub policy: EvictionPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_eviction() {
        let mut manager = EvictionManager::new(EvictionPolicy::LRU).with_max_entries(3);

        // Add entries
        manager.record_access(&"key1", 100);
        manager.record_access(&"key2", 100);
        manager.record_access(&"key3", 100);

        // Access key1 to make it most recent
        manager.record_access(&"key1", 100);

        // Add one more entry (should trigger eviction)
        assert!(manager.needs_eviction());

        let to_evict = manager.select_for_eviction(1);
        assert_eq!(to_evict.len(), 1);
        // key2 should be evicted as it's least recently used
        assert_eq!(to_evict[0], "key2");
    }

    #[test]
    fn test_lfu_eviction() {
        let mut manager = EvictionManager::new(EvictionPolicy::LFU).with_max_entries(3);

        // Add entries with different access patterns
        manager.record_access(&"key1", 100);
        manager.record_access(&"key1", 100); // 2 accesses

        manager.record_access(&"key2", 100); // 1 access

        manager.record_access(&"key3", 100);
        manager.record_access(&"key3", 100);
        manager.record_access(&"key3", 100); // 3 accesses

        let to_evict = manager.select_for_eviction(1);
        assert_eq!(to_evict.len(), 1);
        // key2 should be evicted as it has least accesses
        assert_eq!(to_evict[0], "key2");
    }

    #[test]
    fn test_memory_based_eviction() {
        let mut manager = EvictionManager::new(EvictionPolicy::MemoryBased).with_max_memory(500);

        // Add entries with different sizes
        manager.record_access(&"small", 100);
        manager.record_access(&"large", 300);
        manager.record_access(&"medium", 200);

        assert!(manager.needs_eviction());

        let to_evict = manager.select_for_eviction(1);
        assert_eq!(to_evict.len(), 1);
        // "large" should be evicted first as it uses most memory
        assert_eq!(to_evict[0], "large");
    }

    #[test]
    fn test_ttl_eviction() {
        let mut manager = EvictionManager::new(EvictionPolicy::TTL).with_ttl(1); // 1 second TTL

        manager.record_access(&"key1", 100);

        // Initially should not need eviction
        assert!(!manager.needs_eviction());

        // Sleep to simulate time passing (in real tests, you'd mock time)
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Now should need eviction
        assert!(manager.needs_eviction());

        let expired = manager.select_expired();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], "key1");
    }

    #[test]
    fn test_memory_tracking() {
        let mut manager = EvictionManager::new(EvictionPolicy::LRU);

        assert_eq!(manager.current_memory_usage(), 0);

        manager.record_access(&"key1", 100);
        assert_eq!(manager.current_memory_usage(), 100);

        manager.record_access(&"key2", 200);
        assert_eq!(manager.current_memory_usage(), 300);

        manager.remove_entry(&"key1");
        assert_eq!(manager.current_memory_usage(), 200);
    }
}
