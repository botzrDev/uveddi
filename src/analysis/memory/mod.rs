//! Memory optimization module for UV-210/UV-26
//! Phase 4: Zero-copy AST caching with rkyv and memory mapping

pub mod allocator;
pub mod config;
pub mod detector_pools;
pub mod metrics;
pub mod pool;

#[cfg(feature = "memory-optimization")]
pub mod arena;
// pub mod file_arena; // Not needed with bumpalo-herd pattern

#[cfg(feature = "memory-optimization")]
pub mod zero_copy;

// Re-export key types for easy access
pub use allocator::{get_allocator_info, is_optimized_allocator, AllocationStrategy};

#[cfg(feature = "memory-optimization")]
pub use arena::{
    conversion, rayon_integration, AnalysisArenaManager, AnalysisMetadata,
    AnalysisPerformanceMetrics, ArenaAnalysisResult, ArenaHandle, ArenaManagerStats,
    GLOBAL_ARENA_MANAGER,
};
pub use config::{ArenaConfig, MemoryOptimizationConfig, ObjectPoolConfig};
pub use detector_pools::{
    initialize_detector_pools, DetectorPoolStats, DetectorPools, DETECTOR_POOLS,
};
pub use metrics::{BasicMemoryMetrics, BasicMemoryMetricsCollector, BASIC_MEMORY_METRICS};
pub use pool::{MemoryPool, PoolStats, PooledObject};
// file_arena module not needed with bumpalo-herd pattern
// pub use file_arena::{FileAnalysisArena, FileAnalysisStats, BatchFileArenaManager};

#[cfg(feature = "memory-optimization")]
pub use zero_copy::{
    LoadedAst, SerializableAst, ZeroCopyAstCache, ZeroCopyCacheStats, ZeroCopyError,
};

/// Initialize memory optimization system with object pools and arenas
/// Call this early in application startup
pub fn initialize_memory_optimization(config: MemoryOptimizationConfig) -> Result<(), String> {
    // Validate configuration
    config.validate()?;

    // Log allocator information
    log::info!(
        "Memory optimization Phase 3 initialized with allocator: {}",
        get_allocator_info()
    );
    log::info!(
        "High-performance allocator enabled: {}",
        is_optimized_allocator()
    );
    log::info!(
        "Target memory limit: {:.2} GB",
        config.target_max_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    // Initialize object pools if enabled
    if config.object_pools.enabled {
        let pools = initialize_detector_pools(&config);
        log::info!(
            "Object pools initialized with {} total pools",
            pools.get_all_stats().dead_code_configs.shard_count
        );
    }

    // Initialize arena system if enabled
    if config.arena_allocation.enabled {
        log::info!(
            "Arena allocation initialized with default size: {:.1} MB",
            config.arena_allocation.default_arena_size_mb as f64
        );
        log::info!(
            "Maximum concurrent arenas: {}",
            config.arena_allocation.max_concurrent_arenas
        );
    }

    // Initialize metrics with target from config
    BASIC_MEMORY_METRICS.update_memory_usage(0);

    Ok(())
}

/// Get current memory optimization status
pub fn get_optimization_status() -> serde_json::Value {
    let metrics = BASIC_MEMORY_METRICS.export_json();
    let pool_metrics = DETECTOR_POOLS.export_metrics();

    #[cfg(feature = "memory-optimization")]
    let arena_metrics = GLOBAL_ARENA_MANAGER.export_metrics();
    #[cfg(not(feature = "memory-optimization"))]
    let arena_metrics = serde_json::json!({"disabled": "memory-optimization feature not enabled"});

    serde_json::json!({
        "phase": "Phase 4 - Zero-Copy AST Caching",
        "allocator": get_allocator_info(),
        "optimized": is_optimized_allocator(),
        "metrics": metrics["memory_optimization_phase1"],
        "pools": pool_metrics["detector_pools"],
        "arenas": arena_metrics["arena_manager"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_optimization_initialization() {
        let config = MemoryOptimizationConfig::default();
        let result = initialize_memory_optimization(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_config_initialization() {
        let mut config = MemoryOptimizationConfig::default();
        config.target_max_memory_bytes = 0; // Invalid

        let result = initialize_memory_optimization(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimization_status() {
        let status = get_optimization_status();
        assert!(status["phase"].is_string());
        assert!(status["allocator"].is_string());
        assert!(status["optimized"].is_boolean());
        assert!(status["metrics"].is_object());
    }
}
