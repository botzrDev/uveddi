use super::storage::AstCache;
use std::error::Error;
use std::path::{Path, PathBuf};
use tracing::debug;

impl AstCache {
    pub(crate) fn update_lru_order(&self, path: &Path) {
        if let Ok(mut lru_order) = self.safe_lru_order_lock() {
            let path_buf = path.to_path_buf();
            lru_order.retain(|p| p != &path_buf);
            lru_order.insert(0, path_buf);
        }
    }

    pub(crate) fn ensure_cache_capacity(
        &self,
        new_entry_size: usize,
    ) -> Result<(), Box<dyn Error>> {
        if !self.config.lru_eviction_enabled {
            return Ok(());
        }

        let max_memory = self.config.max_memory_size_mb * 1024 * 1024;
        let mut eviction_count = 0;
        const MAX_EVICTIONS: usize = 1000;

        while eviction_count < MAX_EVICTIONS {
            let current_memory = *self.memory_usage.lock().unwrap_or_else(|_| Box::new(0));

            if (current_memory + new_entry_size) <= max_memory
                && self.cache.read().map(|cache| cache.len()).unwrap_or_default()
                    < self.config.max_memory_entries
            {
                break;
            }

            if !self.evict_lru_entry()? {
                break;
            }
            eviction_count += 1;
        }

        Ok(())
    }

    fn evict_lru_entry(&self) -> Result<bool, Box<dyn Error>> {
        let path_to_evict = {
            let Ok(mut lru_order) = self.safe_lru_order_lock() else {
                return Ok(false);
            };
            lru_order.pop()
        };

        if let Some(path) = path_to_evict {
            let evicted_size = self.remove_cached_entry(&path);
            self.update_memory_usage(-(evicted_size as isize));

            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.evictions += 1;
            }

            debug!("Evicted LRU entry: {:?}", path);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn remove_cached_entry(&self, path: &PathBuf) -> usize {
        let mut cache = self.cache.write().unwrap();
        if let Some(cached_ast) = cache.remove(path) {
            cached_ast.memory_size_bytes
        } else {
            0
        }
    }
}
