//! High-performance global allocator configuration for Uveddi
//! Based on UV-210 research: mimalloc for multi-threaded performance

use std::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Memory allocation strategy configuration for different workload patterns
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStrategy {
    /// Fixed-size pools for predictable allocation patterns
    FixedSize(usize),
    /// Growth-based allocation with configurable factor
    GrowthBased { initial: usize, growth_factor: f64 },
    /// Adaptive allocation based on target memory usage
    AdaptiveBased { target_memory: usize },
}

impl Default for AllocationStrategy {
    fn default() -> Self {
        Self::AdaptiveBased {
            target_memory: 8 * 1024 * 1024 * 1024, // 8GB target from UV-210/UV-26
        }
    }
}

impl AllocationStrategy {
    /// Calculate initial capacity based on strategy
    pub fn initial_capacity(&self) -> usize {
        match self {
            Self::FixedSize(size) => *size,
            Self::GrowthBased { initial, .. } => *initial,
            Self::AdaptiveBased { target_memory } => {
                // Start with 1% of target memory for initial capacity
                target_memory / 100
            }
        }
    }

    /// Calculate next capacity when growth is needed
    pub fn next_capacity(&self, current: usize) -> usize {
        match self {
            Self::FixedSize(size) => *size, // No growth for fixed size
            Self::GrowthBased { growth_factor, .. } => ((current as f64) * growth_factor) as usize,
            Self::AdaptiveBased { target_memory } => {
                // Grow by 50% but don't exceed target
                let next = current + (current / 2);
                next.min(*target_memory / 1024) // Conservative growth
            }
        }
    }
}

/// Get information about the current global allocator
pub fn get_allocator_info() -> &'static str {
    #[cfg(feature = "mimalloc")]
    return "mimalloc";

    #[cfg(not(feature = "mimalloc"))]
    return "system";
}

/// Check if high-performance allocator is enabled
pub fn is_optimized_allocator() -> bool {
    cfg!(feature = "mimalloc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_strategy_default() {
        let strategy = AllocationStrategy::default();
        match strategy {
            AllocationStrategy::AdaptiveBased { target_memory } => {
                assert_eq!(target_memory, 8 * 1024 * 1024 * 1024);
            }
            _ => panic!("Default should be AdaptiveBased"),
        }
    }

    #[test]
    fn test_allocation_strategy_capacities() {
        let fixed = AllocationStrategy::FixedSize(1000);
        assert_eq!(fixed.initial_capacity(), 1000);
        assert_eq!(fixed.next_capacity(500), 1000);

        let growth = AllocationStrategy::GrowthBased {
            initial: 100,
            growth_factor: 2.0,
        };
        assert_eq!(growth.initial_capacity(), 100);
        assert_eq!(growth.next_capacity(100), 200);

        let adaptive = AllocationStrategy::AdaptiveBased {
            target_memory: 1024 * 1024,
        };
        assert_eq!(adaptive.initial_capacity(), 10485); // 1% of target (1048576/100)
    }

    #[test]
    fn test_allocator_info() {
        let info = get_allocator_info();
        assert!(info == "mimalloc" || info == "system");

        // Test consistency
        let is_optimized = is_optimized_allocator();
        if cfg!(feature = "mimalloc") {
            assert_eq!(info, "mimalloc");
            assert!(is_optimized);
        } else {
            assert_eq!(info, "system");
            assert!(!is_optimized);
        }
    }
}
