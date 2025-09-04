//! Memory tracking and management implementation

use std::sync::{Arc, Mutex, Weak};
use std::collections::HashMap;
use std::time::Instant;
use super::error::{ResourceError, ResourceResult};

/// Tracks memory allocations and enforces memory limits
pub struct MemoryTracker {
    current_usage: Arc<Mutex<u64>>,
    peak_usage: Arc<Mutex<u64>>,
    limit: Arc<Mutex<u64>>,
    allocations: Arc<Mutex<HashMap<String, AllocationInfo>>>,
    allocation_history: Arc<Mutex<Vec<AllocationEvent>>>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: u64,
    allocated_at: Instant,
    component: String,
}

#[derive(Debug, Clone)]
struct AllocationEvent {
    timestamp: Instant,
    component: String,
    size: u64,
    event_type: AllocationEventType,
}

#[derive(Debug, Clone)]
enum AllocationEventType {
    Allocated,
    Freed,
    Failed,
}

impl MemoryTracker {
    /// Creates a new memory tracker with the specified limit
    pub fn new(limit_bytes: u64) -> ResourceResult<Self> {
        if limit_bytes == 0 {
            return Err(ResourceError::InvalidConfiguration(
                "Memory limit must be greater than 0".to_string()
            ));
        }
        
        Ok(Self {
            current_usage: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
            limit: Arc::new(Mutex::new(limit_bytes)),
            allocations: Arc::new(Mutex::new(HashMap::new())),
            allocation_history: Arc::new(Mutex::new(Vec::with_capacity(1000))),
        })
    }
    
    /// Allocates memory for a component
    pub fn allocate(&self, component: &str, size_bytes: u64) -> ResourceResult<MemoryGuard> {
        let mut current = self.current_usage.lock().unwrap();
        let limit = *self.limit.lock().unwrap();
        
        // Check if allocation would exceed limit
        if *current + size_bytes > limit {
            self.record_allocation_event(
                component,
                size_bytes,
                AllocationEventType::Failed,
            );
            
            return Err(ResourceError::MemoryExhausted {
                requested: size_bytes,
                available: limit.saturating_sub(*current),
                limit,
            });
        }
        
        // Update current usage
        *current += size_bytes;
        
        // Update peak usage
        let mut peak = self.peak_usage.lock().unwrap();
        if *current > *peak {
            *peak = *current;
        }
        
        // Record allocation
        let allocation_id = format!("{}_{}", component, Instant::now().elapsed().as_nanos());
        self.allocations.lock().unwrap().insert(
            allocation_id.clone(),
            AllocationInfo {
                size: size_bytes,
                allocated_at: Instant::now(),
                component: component.to_string(),
            },
        );
        
        self.record_allocation_event(
            component,
            size_bytes,
            AllocationEventType::Allocated,
        );
        
        Ok(MemoryGuard::new(
            Arc::downgrade(&self.current_usage),
            Arc::downgrade(&self.allocations),
            allocation_id,
            size_bytes,
            component.to_string(),
        ))
    }
    
    /// Gets current memory usage statistics
    pub fn get_usage_stats(&self) -> MemoryStats {
        let current = *self.current_usage.lock().unwrap();
        let peak = *self.peak_usage.lock().unwrap();
        let limit = *self.limit.lock().unwrap();
        let allocations = self.allocations.lock().unwrap();
        
        let mut component_usage = HashMap::new();
        for (_, info) in allocations.iter() {
            *component_usage.entry(info.component.clone()).or_insert(0) += info.size;
        }
        
        MemoryStats {
            current,
            peak,
            limit,
            available: limit.saturating_sub(current),
            usage_percent: (current as f64 / limit as f64) * 100.0,
            allocation_count: allocations.len(),
            component_usage,
        }
    }
    
    /// Updates the memory limit
    pub fn update_limit(&self, new_limit: u64) -> ResourceResult<()> {
        let current = *self.current_usage.lock().unwrap();
        
        if new_limit < current {
            return Err(ResourceError::InvalidConfiguration(
                format!("Cannot set limit {} below current usage {}", new_limit, current)
            ));
        }
        
        *self.limit.lock().unwrap() = new_limit;
        Ok(())
    }
    
    /// Gets memory pressure level (0.0 to 1.0)
    pub fn get_memory_pressure(&self) -> f64 {
        let current = *self.current_usage.lock().unwrap();
        let limit = *self.limit.lock().unwrap();
        current as f64 / limit as f64
    }
    
    /// Clears allocation history (for maintenance)
    pub fn clear_history(&self) {
        self.allocation_history.lock().unwrap().clear();
    }
    
    fn record_allocation_event(
        &self,
        component: &str,
        size: u64,
        event_type: AllocationEventType,
    ) {
        let mut history = self.allocation_history.lock().unwrap();
        
        history.push(AllocationEvent {
            timestamp: Instant::now(),
            component: component.to_string(),
            size,
            event_type,
        });
        
        // Keep only last 1000 events
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }
}

/// Guard that automatically releases memory when dropped
pub struct MemoryGuard {
    usage_counter: Weak<Mutex<u64>>,
    allocations: Weak<Mutex<HashMap<String, AllocationInfo>>>,
    allocation_id: String,
    size: u64,
    component: String,
    released: bool,
}

impl MemoryGuard {
    fn new(
        usage_counter: Weak<Mutex<u64>>,
        allocations: Weak<Mutex<HashMap<String, AllocationInfo>>>,
        allocation_id: String,
        size: u64,
        component: String,
    ) -> Self {
        Self {
            usage_counter,
            allocations,
            allocation_id,
            size,
            component,
            released: false,
        }
    }
    
    /// Gets the size of this allocation
    pub fn size(&self) -> u64 {
        self.size
    }
    
    /// Gets the component name for this allocation
    pub fn component(&self) -> &str {
        &self.component
    }
    
    /// Manually releases the memory (useful for early cleanup)
    pub fn release(mut self) {
        self.released = true;
        self.cleanup();
    }
    
    fn cleanup(&self) {
        if let Some(counter) = self.usage_counter.upgrade() {
            let mut usage = counter.lock().unwrap();
            *usage = usage.saturating_sub(self.size);
        }
        
        if let Some(allocations) = self.allocations.upgrade() {
            allocations.lock().unwrap().remove(&self.allocation_id);
        }
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        if !self.released {
            self.cleanup();
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current: u64,
    pub peak: u64,
    pub limit: u64,
    pub available: u64,
    pub usage_percent: f64,
    pub allocation_count: usize,
    pub component_usage: HashMap<String, u64>,
}

impl MemoryStats {
    /// Checks if memory usage is critical (>90%)
    pub fn is_critical(&self) -> bool {
        self.usage_percent > 90.0
    }
    
    /// Checks if memory usage is high (>70%)
    pub fn is_high(&self) -> bool {
        self.usage_percent > 70.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_allocation() {
        let tracker = MemoryTracker::new(1000).unwrap();
        
        // Allocate some memory
        let guard1 = tracker.allocate("test1", 100).unwrap();
        assert_eq!(tracker.get_usage_stats().current, 100);
        
        let guard2 = tracker.allocate("test2", 200).unwrap();
        assert_eq!(tracker.get_usage_stats().current, 300);
        
        // Drop guards and check memory is released
        drop(guard1);
        assert_eq!(tracker.get_usage_stats().current, 200);
        
        drop(guard2);
        assert_eq!(tracker.get_usage_stats().current, 0);
    }
    
    #[test]
    fn test_memory_limit_enforcement() {
        let tracker = MemoryTracker::new(500).unwrap();
        
        let _guard1 = tracker.allocate("test1", 400).unwrap();
        
        // This should fail - would exceed limit
        let result = tracker.allocate("test2", 200);
        assert!(matches!(result, Err(ResourceError::MemoryExhausted { .. })));
        
        // This should succeed - within limit
        let _guard2 = tracker.allocate("test3", 100).unwrap();
        assert_eq!(tracker.get_usage_stats().current, 500);
    }
    
    #[test]
    fn test_memory_pressure() {
        let tracker = MemoryTracker::new(1000).unwrap();
        
        let _guard1 = tracker.allocate("test", 700).unwrap();
        assert!(tracker.get_memory_pressure() > 0.69);
        assert!(tracker.get_memory_pressure() < 0.71);
        
        let _guard2 = tracker.allocate("test", 200).unwrap();
        assert_eq!(tracker.get_memory_pressure(), 0.9);
    }
}