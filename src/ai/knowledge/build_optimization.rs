//! Build-Time Optimization for Knowledge Library
//!
//! This module optimizes the build-time compression and generation pipeline
//! to achieve the <60 second build time target while maintaining quality.
//!
//! Research implementation following docs/compression_research.md patterns:
//! - build.rs preprocessing and compression
//! - Parallel processing for speed
//! - Incremental builds for efficiency

use std::time::Instant;
use rayon::prelude::*;

/// Build-time optimization system (research-backed)
pub struct BuildOptimizationSystem {
    /// Build configuration
    config: BuildOptimizationConfig,
    /// Parallel processing manager
    parallel_manager: ParallelProcessingManager,
    /// Build cache system
    build_cache: BuildCacheSystem,
    /// Build metrics
    metrics: BuildMetrics,
}

impl BuildOptimizationSystem {
    /// Optimize complete build pipeline (epic target: <60s)
    pub fn optimize_build_pipeline(
        &mut self,
        library: &KnowledgeLibrary,
    ) -> Result<BuildOptimizationResult, BuildOptimizationError> {
        let start_time = Instant::now();

        // 1. Parallel knowledge processing (research: significant speedup)
        let processed_library = if self.config.enable_parallel_processing {
            self.parallel_process_knowledge(library)?
        } else {
            library.clone()
        };

        // 2. Incremental compression (research: build.rs pattern)
        let compressed = if self.config.enable_incremental_builds {
            self.incremental_compression(&processed_library)?
        } else {
            self.full_compression(&processed_library)?
        };

        // 3. Parallel index generation (research: PHF generation)
        let indices = self.parallel_index_generation(&compressed)?;

        // 4. Validation and optimization
        let validated = self.validate_and_optimize(&compressed, &indices)?;

        let build_time = start_time.elapsed();

        // 5. Validate build time target (epic: <60s)
        if build_time.as_secs() > self.config.target_build_time_seconds {
            return Err(BuildOptimizationError::BuildTimeTargetNotMet {
                target: self.config.target_build_time_seconds,
                actual: build_time.as_secs(),
            });
        }

        // 6. Update metrics
        self.metrics.record_build_time(build_time);

        Ok(BuildOptimizationResult {
            optimized_library: validated,
            build_time,
            optimization_metrics: self.generate_optimization_metrics(),
        })
    }
}
