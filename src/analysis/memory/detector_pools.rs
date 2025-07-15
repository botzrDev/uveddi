//! Specialized object pools for analysis detector configurations
//! Targets expensive-to-construct detector configurations and temporary objects

use super::pool::{MemoryPool, PoolStats};
use crate::analysis::memory::allocator::AllocationStrategy;
use crate::analysis::memory::config::MemoryOptimizationConfig;
use std::sync::Arc;

// Import detector configuration types
use crate::analysis::detectors::anti_patterns::dead_code::DeadCodeConfig;

/// Centralized detector configuration pools for memory optimization
pub struct DetectorPools {
    pub dead_code_configs: Arc<MemoryPool<DeadCodeConfig>>,

    // Pools for frequently allocated temporary objects
    pub string_vectors: Arc<MemoryPool<Vec<String>>>,
    pub issue_vectors: Arc<MemoryPool<Vec<crate::database::models::ArchitecturalIssue>>>,
}

impl DetectorPools {
    /// Create detector pools with default configuration
    pub fn new() -> Self {
        let config = MemoryOptimizationConfig::default();
        Self::with_config(&config)
    }

    /// Create detector pools with custom configuration
    pub fn with_config(config: &MemoryOptimizationConfig) -> Self {
        let pool_config = &config.object_pools;
        let strategy = pool_config.allocation_strategy.clone();
        let capacity = pool_config.detector_pool_capacity;

        log::info!("Initializing detector pools with capacity: {}", capacity);

        Self {
            // Detector configuration pools
            dead_code_configs: Arc::new(MemoryPool::new(
                capacity, strategy.clone(), "dead_code_configs"
            )),

            // Temporary object pools
            string_vectors: Arc::new(MemoryPool::new(
                capacity * 2, strategy.clone(), "string_vectors"
            )),
            issue_vectors: Arc::new(MemoryPool::new(
                capacity, strategy.clone(), "issue_vectors"
            )),
        }
    }

    /// Pre-populate all pools with initial objects
    pub fn pre_populate(&self, percentage: f64) {
        let populate_count = |capacity: usize| -> usize {
            ((capacity as f64) * (percentage / 100.0)) as usize
        };

        // Pre-populate detector config pools
        self.dead_code_configs.pre_populate(populate_count(100));

        // Pre-populate temporary object pools
        self.string_vectors.pre_populate(populate_count(200));
        self.issue_vectors.pre_populate(populate_count(100));

        log::info!("Pre-populated detector pools with {}% capacity", percentage);
    }

    /// Get comprehensive statistics for all pools
    pub fn get_all_stats(&self) -> DetectorPoolStats {
        DetectorPoolStats {
            dead_code_configs: self.dead_code_configs.get_stats(),
            string_vectors: self.string_vectors.get_stats(),
            issue_vectors: self.issue_vectors.get_stats(),
        }
    }

    /// Get efficiency report for all pools
    pub fn get_efficiency_report(&self) -> Vec<String> {
        let stats = self.get_all_stats();
        vec![
            stats.dead_code_configs.get_efficiency_recommendation(),
            stats.string_vectors.get_efficiency_recommendation(),
            stats.issue_vectors.get_efficiency_recommendation(),
        ]
    }

    /// Export metrics for observability
    pub fn export_metrics(&self) -> serde_json::Value {
        let stats = self.get_all_stats();

        serde_json::json!({
            "detector_pools": {
                "dead_code_configs": {
                    "capacity": stats.dead_code_configs.total_capacity,
                    "available": stats.dead_code_configs.total_objects_available,
                    "utilization_percent": stats.dead_code_configs.utilization_percentage,
                    "efficient": stats.dead_code_configs.is_efficiently_utilized()
                },
                "temporary_objects": {
                    "string_vectors": {
                        "capacity": stats.string_vectors.total_capacity,
                        "available": stats.string_vectors.total_objects_available,
                        "utilization_percent": stats.string_vectors.utilization_percentage
                    },
                    "issue_vectors": {
                        "capacity": stats.issue_vectors.total_capacity,
                        "available": stats.issue_vectors.total_objects_available,
                        "utilization_percent": stats.issue_vectors.utilization_percentage
                    }
                }
            }
        })
    }
}

impl Default for DetectorPools {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive statistics for all detector pools
#[derive(Debug, Clone)]
pub struct DetectorPoolStats {
    pub dead_code_configs: PoolStats,
    pub string_vectors: PoolStats,
    pub issue_vectors: PoolStats,
}

impl DetectorPoolStats {
    /// Calculate overall pool efficiency
    pub fn overall_efficiency(&self) -> f64 {
        let pools = vec![
            &self.dead_code_configs,
            &self.string_vectors,
            &self.issue_vectors,
        ];

        let total_utilization: f64 = pools.iter()
            .map(|pool| pool.utilization_percentage)
            .sum();

        total_utilization / pools.len() as f64
    }

    /// Count efficiently utilized pools
    pub fn efficiently_utilized_count(&self) -> usize {
        let pools = vec![
            &self.dead_code_configs,
            &self.string_vectors,
            &self.issue_vectors,
        ];

        pools.iter()
            .filter(|pool| pool.is_efficiently_utilized())
            .count()
    }
}

// Global detector pools instance
lazy_static::lazy_static! {
    pub static ref DETECTOR_POOLS: DetectorPools = DetectorPools::new();
}

/// Initialize detector pools with custom configuration
pub fn initialize_detector_pools(config: &MemoryOptimizationConfig) -> DetectorPools {
    let pools = DetectorPools::with_config(config);

    // Pre-populate pools if enabled
    if config.object_pools.enabled {
        pools.pre_populate(25.0); // Start with 25% pre-population
    }

    log::info!("Detector pools initialized successfully");
    pools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_pools_creation() {
        let pools = DetectorPools::new();
        let stats = pools.get_all_stats();

        // Verify all pools are created
        assert_eq!(stats.dead_code_configs.pool_name, "dead_code_configs");
        assert_eq!(stats.string_vectors.pool_name, "string_vectors");
        assert_eq!(stats.issue_vectors.pool_name, "issue_vectors");
    }

    #[test]
    fn test_detector_pools_with_config() {
        let mut config = MemoryOptimizationConfig::default();
        config.object_pools.detector_pool_capacity = 50;

        let pools = DetectorPools::with_config(&config);
        let stats = pools.get_all_stats();

        // Verify capacity is set correctly
        assert_eq!(stats.dead_code_configs.total_capacity, 50);
        assert_eq!(stats.issue_vectors.total_capacity, 50);
        assert_eq!(stats.string_vectors.total_capacity, 100); // 2x capacity for string vectors
    }

    #[test]
    fn test_pre_population() {
        let pools = DetectorPools::new();
        pools.pre_populate(50.0); // 50% pre-population

        let stats = pools.get_all_stats();

        // At least some pools should have objects
        let total_objects: usize = stats.dead_code_configs.total_objects_available +
                                  stats.string_vectors.total_objects_available +
                                  stats.issue_vectors.total_objects_available;

        assert!(total_objects > 0, "Pre-population should create objects");
    }

    #[test]
    fn test_efficiency_reporting() {
        let pools = DetectorPools::new();
        pools.pre_populate(30.0);

        let efficiency_report = pools.get_efficiency_report();
        assert_eq!(efficiency_report.len(), 3); // 3 pools total

        // Each report should contain pool name
        for report in efficiency_report {
            assert!(report.contains("Pool"));
        }
    }

    #[test]
    fn test_metrics_export() {
        let pools = DetectorPools::new();
        pools.pre_populate(25.0);

        let metrics = pools.export_metrics();

        // Verify structure
        assert!(metrics["detector_pools"].is_object());
        assert!(metrics["detector_pools"]["dead_code_configs"].is_object());
        assert!(metrics["detector_pools"]["temporary_objects"].is_object());

        // Verify data types
        assert!(metrics["detector_pools"]["dead_code_configs"]["capacity"].is_number());
        assert!(metrics["detector_pools"]["dead_code_configs"]["efficient"].is_boolean());
    }

    #[test]
    fn test_overall_efficiency_calculation() {
        let pools = DetectorPools::new();
        pools.pre_populate(40.0);

        let stats = pools.get_all_stats();
        let efficiency = stats.overall_efficiency();

        assert!(efficiency >= 0.0 && efficiency <= 100.0);

        let efficient_count = stats.efficiently_utilized_count();
        assert!(efficient_count <= 3); // Maximum 3 pools
    }

    #[test]
    fn test_pool_usage() {
        let pools = DetectorPools::new();

        // Test getting objects from pools
        {
            let _dead_code_config = pools.dead_code_configs.get();
            let _string_vector = pools.string_vectors.get();
            let _issue_vector = pools.issue_vectors.get();

            // Objects should be available
        } // Objects returned to pool here

        // Pools should still be functional
        let stats = pools.get_all_stats();
        assert_eq!(stats.dead_code_configs.pool_name, "dead_code_configs");
    }

    #[test]
    fn test_initialization_function() {
        let config = MemoryOptimizationConfig::default();
        let pools = initialize_detector_pools(&config);

        let stats = pools.get_all_stats();

        // Should have some pre-populated objects
        let total_objects: usize = stats.dead_code_configs.total_objects_available +
                                  stats.string_vectors.total_objects_available +
                                  stats.issue_vectors.total_objects_available;

        assert!(total_objects > 0, "Initialization should pre-populate pools");
    }
}