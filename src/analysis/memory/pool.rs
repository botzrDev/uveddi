//! High-performance concurrent object pools for frequently reused analysis objects
//! Based on UV-210 research: sharded pools for multi-threaded scenarios

use crate::analysis::memory::allocator::AllocationStrategy;
use crate::analysis::memory::metrics::BASIC_MEMORY_METRICS;
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// Thread-safe object pool with configurable allocation strategy and sharding
pub struct MemoryPool<T> {
    pools: Vec<Arc<Mutex<VecDeque<Box<T>>>>>,
    capacity_per_shard: usize,
    allocation_strategy: AllocationStrategy,
    shard_count: usize,
    total_capacity: usize,
    pool_name: String,
}

impl<T> MemoryPool<T>
where
    T: Default + Send + 'static,
{
    /// Create a new memory pool with specified capacity and strategy
    pub fn new(total_capacity: usize, allocation_strategy: AllocationStrategy, name: &str) -> Self {
        let shard_count = if num_cpus::get() < 4 {
            4
        } else {
            num_cpus::get()
        };
        let capacity_per_shard = (total_capacity + shard_count - 1) / shard_count; // Ceiling division

        let pools = (0..shard_count)
            .map(|_| Arc::new(Mutex::new(VecDeque::with_capacity(capacity_per_shard))))
            .collect();

        log::debug!(
            "Created memory pool '{}' with {} shards, {} capacity per shard",
            name,
            shard_count,
            capacity_per_shard
        );

        Self {
            pools,
            capacity_per_shard,
            allocation_strategy,
            shard_count,
            total_capacity,
            pool_name: name.to_string(),
        }
    }

    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let shard_id = self.get_shard_id();
        let object = {
            let mut pool = self.pools[shard_id].lock().unwrap();
            pool.pop_front()
        };

        let object = match object {
            Some(obj) => {
                // Record pool hit
                self.record_pool_hit();
                obj
            }
            None => {
                // Record pool miss and create new object
                self.record_pool_miss();
                Box::new(T::default())
            }
        };

        PooledObject {
            object: Some(object),
            pool: Arc::clone(&self.pools[shard_id]),
            pool_name: self.pool_name.clone(),
            _phantom: PhantomData,
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let mut total_objects = 0;
        let mut shard_stats = Vec::new();

        for (i, pool) in self.pools.iter().enumerate() {
            let pool_guard = pool.lock().unwrap();
            let shard_size = pool_guard.len();
            total_objects += shard_size;
            shard_stats.push(ShardStats {
                shard_id: i,
                objects_available: shard_size,
                capacity: self.capacity_per_shard,
            });
        }

        PoolStats {
            pool_name: self.pool_name.clone(),
            total_capacity: self.total_capacity,
            total_objects_available: total_objects,
            shard_count: self.shard_count,
            shard_stats,
            utilization_percentage: (total_objects as f64 / self.total_capacity as f64) * 100.0,
        }
    }

    /// Pre-populate the pool with objects
    pub fn pre_populate(&self, count: usize) {
        let objects_per_shard = (count + self.shard_count - 1) / self.shard_count;

        for pool in &self.pools {
            let mut pool_guard = pool.lock().unwrap();
            for _ in 0..objects_per_shard.min(self.capacity_per_shard - pool_guard.len()) {
                pool_guard.push_back(Box::new(T::default()));
            }
        }

        log::debug!(
            "Pre-populated pool '{}' with {} objects",
            self.pool_name,
            count
        );
    }

    fn get_shard_id(&self) -> usize {
        // Use thread ID for consistent shard selection to improve cache locality
        let thread_id = std::thread::current().id();
        let mut hasher = DefaultHasher::new();
        thread_id.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }

    fn record_pool_hit(&self) {
        BASIC_MEMORY_METRICS.record_pool_hit();
    }

    fn record_pool_miss(&self) {
        BASIC_MEMORY_METRICS.record_pool_miss();
    }
}

/// RAII wrapper that returns objects to pool on drop
pub struct PooledObject<T> {
    object: Option<Box<T>>,
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    pool_name: String,
    _phantom: PhantomData<T>,
}

impl<T> PooledObject<T> {
    /// Reset the object to its default state (useful for reuse)
    pub fn reset(&mut self)
    where
        T: Default,
    {
        if let Some(ref mut obj) = self.object {
            **obj = T::default();
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            if let Ok(mut pool) = self.pool.lock() {
                // Only return to pool if there's space
                if pool.len() < pool.capacity() {
                    pool.push_back(object);
                }
                // If pool is full, object will be dropped (deallocated)
            }
        }
    }
}

/// Statistics for a memory pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub pool_name: String,
    pub total_capacity: usize,
    pub total_objects_available: usize,
    pub shard_count: usize,
    pub shard_stats: Vec<ShardStats>,
    pub utilization_percentage: f64,
}

/// Statistics for a single shard within a pool
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: usize,
    pub objects_available: usize,
    pub capacity: usize,
}

impl PoolStats {
    /// Check if pool is efficiently utilized (not too empty or too full)
    pub fn is_efficiently_utilized(&self) -> bool {
        self.utilization_percentage >= 20.0 && self.utilization_percentage <= 80.0
    }

    /// Get efficiency recommendation
    pub fn get_efficiency_recommendation(&self) -> String {
        if self.utilization_percentage < 20.0 {
            format!(
                "Pool '{}' is under-utilized ({}%). Consider reducing capacity.",
                self.pool_name, self.utilization_percentage
            )
        } else if self.utilization_percentage > 80.0 {
            format!(
                "Pool '{}' is over-utilized ({}%). Consider increasing capacity.",
                self.pool_name, self.utilization_percentage
            )
        } else {
            format!(
                "Pool '{}' is efficiently utilized ({}%).",
                self.pool_name, self.utilization_percentage
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct TestObject {
        value: i32,
        data: String,
    }

    #[test]
    fn test_pool_creation() {
        let pool =
            MemoryPool::<TestObject>::new(100, AllocationStrategy::FixedSize(100), "test_pool");

        let stats = pool.get_stats();
        assert_eq!(stats.total_capacity, 100);
        assert_eq!(stats.pool_name, "test_pool");
        assert!(stats.shard_count >= 4);
    }

    #[test]
    fn test_pool_get_and_return() {
        let pool =
            MemoryPool::<TestObject>::new(10, AllocationStrategy::FixedSize(10), "test_pool");

        // Pre-populate with one object
        pool.pre_populate(1);

        let initial_stats = pool.get_stats();
        assert!(initial_stats.total_objects_available > 0);

        // Get object from pool
        {
            let mut obj = pool.get();
            obj.value = 42;
            obj.data = "test".to_string();

            // Object should be available
            assert_eq!(obj.value, 42);
            assert_eq!(obj.data, "test");
        } // Object returned to pool here

        // Pool should have object available again
        let final_stats = pool.get_stats();
        assert!(final_stats.total_objects_available > 0);
    }

    #[test]
    fn test_pool_reset_functionality() {
        let pool =
            MemoryPool::<TestObject>::new(10, AllocationStrategy::FixedSize(10), "test_pool");

        let mut obj = pool.get();
        obj.value = 42;
        obj.data = "test".to_string();

        obj.reset();
        assert_eq!(obj.value, 0);
        assert_eq!(obj.data, "");
    }

    #[test]
    fn test_pool_stats() {
        let pool =
            MemoryPool::<TestObject>::new(20, AllocationStrategy::FixedSize(20), "stats_test");

        pool.pre_populate(10);
        let stats = pool.get_stats();

        assert_eq!(stats.pool_name, "stats_test");
        assert_eq!(stats.total_capacity, 20);
        assert!(stats.total_objects_available <= 20); // With sharding, may be distributed differently
        assert!(stats.utilization_percentage <= 100.0);
        assert!(stats.is_efficiently_utilized());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(MemoryPool::<TestObject>::new(
            100,
            AllocationStrategy::FixedSize(100),
            "concurrent_test",
        ));

        pool.pre_populate(50);

        let mut handles = vec![];

        // Spawn multiple threads to access pool concurrently
        for i in 0..10 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for j in 0..5 {
                    let mut obj = pool_clone.get();
                    obj.value = i * 10 + j;
                    // Object automatically returned when dropped
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Pool should still be functional
        let stats = pool.get_stats();
        assert_eq!(stats.pool_name, "concurrent_test");
    }

    #[test]
    fn test_efficiency_recommendations() {
        let pool = MemoryPool::<TestObject>::new(
            100,
            AllocationStrategy::FixedSize(100),
            "efficiency_test",
        );

        // Test under-utilized
        pool.pre_populate(10); // 10% utilization
        let stats = pool.get_stats();
        let recommendation = stats.get_efficiency_recommendation();
        assert!(recommendation.contains("under-utilized"));

        // Test over-utilized
        pool.pre_populate(90); // 90%+ utilization
        let stats = pool.get_stats();
        let recommendation = stats.get_efficiency_recommendation();
        assert!(recommendation.contains("over-utilized"));
    }
}
