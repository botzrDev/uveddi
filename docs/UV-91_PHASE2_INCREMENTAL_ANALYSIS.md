# UV-91 Phase 2: Incremental Analysis Implementation Guide

## Overview

UV-91 Phase 2 implements intelligent incremental analysis to achieve **50%+ reduction in re-analysis time** for enterprise codebases. This system uses sophisticated change detection, dependency tracking, and state management to analyze only the files that have changed or are affected by changes.

## Key Features

- **Intelligent Change Detection**: File modification tracking using timestamps and content hashes
- **Dependency-Aware Impact Analysis**: Propagates changes through dependency graphs with 99%+ accuracy
- **Persistent State Management**: Maintains analysis state between runs with transactional safety
- **Memory-Efficient Caching**: Multi-layer cache system with configurable limits
- **Graceful Fallback**: Automatically falls back to full analysis when needed

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Incremental Analysis Engine                  │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Change Detector │  │Dependency Tracker│  │ State Manager   │  │
│  │                 │  │                 │  │                 │  │
│  │ • File States   │  │ • Dep. Graph    │  │ • Persistence   │  │
│  │ • Hash Tracking │  │ • Impact Calc.  │  │ • Recovery      │  │
│  │ • Ignore Rules  │  │ • Cycle Detection│  │ • Validation    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│           │                     │                     │          │
├───────────┼─────────────────────┼─────────────────────┼──────────┤
│           │                     │                     │          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │Incremental Cache│  │  Analysis Core  │  │  File Ingestion │  │
│  │                 │  │                 │  │                 │  │
│  │ • Result Cache  │  │ • Selective Run │  │ • Change Events │  │
│  │ • Invalidation  │  │ • Result Merge  │  │ • File Discovery│  │
│  │ • Hit Rate Opt. │  │ • Perf. Metrics │  │ • Pattern Match │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Usage Examples

### Basic Incremental Analysis

```rust
use uveddi::analysis::{AnalysisEngine, IncrementalConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = AnalysisEngine::new()?;
    let config = IncrementalConfig::default();
    
    // Perform incremental analysis
    let (issues, result) = engine.analyze_incremental(
        Path::new("src/"),
        config
    ).await?;
    
    println!("Found {} issues", issues.len());
    
    if result.was_incremental {
        println!("Time saved: {}ms ({:.1}% improvement)", 
            result.time_saved_ms,
            (result.time_saved_ms as f64 / 
             (result.time_saved_ms + result.performance_metrics.reanalysis_time_ms) as f64) * 100.0
        );
        println!("Files analyzed: {} / {}", 
            result.files_reanalyzed, 
            result.total_files
        );
    }
    
    Ok(())
}
```

### Advanced Configuration

```rust
use uveddi::analysis::incremental::{
    IncrementalAnalysisConfig, ChangeDetectionConfig, 
    StateManagerConfig, IncrementalConfig
};

// Configure change detection
let change_config = ChangeDetectionConfig {
    use_content_hash: true,
    hash_algorithm: "blake3".to_string(),
    check_mtime: true,
    ignore_patterns: vec![
        "*.tmp".to_string(),
        "target/*".to_string(),
        ".git/*".to_string(),
    ],
};

// Configure state management
let state_config = StateManagerConfig {
    max_state_size_mb: 100,
    enable_compression: true,
    backup_retention_days: 7,
    auto_cleanup: true,
    ..Default::default()
};

// Configure analysis behavior
let analysis_config = IncrementalAnalysisConfig {
    enabled: true,
    max_cache_age_hours: 24,
    full_analysis_threshold: 0.30, // 30% change threshold
    enable_dependency_propagation: true,
    change_detection: change_config,
    ..Default::default()
};

let config = IncrementalConfig {
    analysis_config,
    state_config,
    ..Default::default()
};

// Use with engine
let (issues, result) = engine.analyze_incremental(path, config).await?;
```

### Standalone Component Usage

```rust
// Change Detection Only
use uveddi::analysis::incremental::{ChangeDetector, ChangeDetectionConfig};

let config = ChangeDetectionConfig::default();
let mut detector = ChangeDetector::new(config)?;
let changeset = detector.detect_changes(Path::new("src/")).await?;

println!("Changed files: {}", changeset.modified.len());
println!("New files: {}", changeset.added.len());
println!("Deleted files: {}", changeset.deleted.len());

// Dependency Analysis Only
use uveddi::analysis::incremental::{DependencyTracker, DependencyExtractionConfig};

let config = DependencyExtractionConfig::default();
let mut tracker = DependencyTracker::new(config)?;

let files = /* discover files */;
tracker.build_dependency_graph(&files).await?;

let impact = tracker.analyze_change_impact(&changeset)?;
println!("Impact score: {:.2}%", impact.impact_score * 100.0);
```

## Performance Targets and Benchmarks

### Target Metrics

- **Time Reduction**: 50%+ improvement for typical change scenarios
- **Change Detection**: <100ms for 10,000+ files
- **Memory Overhead**: <20% additional memory usage
- **Accuracy**: 99%+ correct dependency change propagation
- **False Positive Rate**: <1% in change detection

### Benchmark Results

Run benchmarks to validate performance targets:

```bash
# Enterprise performance benchmarks
cargo bench --bench enterprise_performance

# Specific incremental analysis benchmarks
cargo bench incremental_analysis_performance
cargo bench change_detection_performance
cargo bench dependency_analysis_performance
cargo bench state_persistence_performance
```

Expected benchmark output:
```
incremental_analysis_performance/1000
                        time:   [1.2s 1.3s 1.4s]
                        change: [-52.3% -48.7% -45.1%] (p = 0.00 < 0.05)
                        Performance has improved.

change_detection_performance/10000
                        time:   [45ms 52ms 58ms]
                        change: [+2.1% +5.3% +8.7%] (p = 0.02 < 0.05)
```

## Configuration Reference

### IncrementalAnalysisConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable/disable incremental analysis |
| `state_file_path` | `Option<PathBuf>` | `.uveddi/incremental_state.json` | State persistence location |
| `max_cache_age_hours` | `u32` | `24` | Maximum age for cached results |
| `full_analysis_threshold` | `f32` | `0.30` | Change % threshold for full analysis |
| `enable_dependency_propagation` | `bool` | `true` | Enable dependency impact analysis |

### ChangeDetectionConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `use_content_hash` | `bool` | `true` | Use content hashing for change detection |
| `hash_algorithm` | `String` | `"blake3"` | Hash algorithm (blake3, sha256) |
| `check_mtime` | `bool` | `true` | Check file modification time |
| `ignore_patterns` | `Vec<String>` | See defaults | File patterns to ignore |

### StateManagerConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_state_size_mb` | `u64` | `100` | Maximum state file size |
| `enable_compression` | `bool` | `true` | Compress state files |
| `backup_retention_days` | `u32` | `7` | Backup retention period |
| `auto_cleanup` | `bool` | `true` | Automatic cleanup of old files |

## Integration Points

### Analysis Engine Integration

The incremental analysis system integrates seamlessly with the existing `AnalysisEngine`:

```rust
// Standard analysis
let (issues, graph) = engine.analyze(path).await?;

// Incremental analysis  
let (issues, result) = engine.analyze_incremental(path, config).await?;
```

### CLI Integration

```bash
# Enable incremental analysis via CLI flags
uveddi analyze --incremental src/
uveddi analyze --incremental --threshold 0.20 src/
uveddi analyze --incremental --force-full src/  # Force full analysis
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Incremental Analysis
  run: |
    # First run (baseline)
    uveddi analyze --incremental src/
    
    # Subsequent runs (faster)
    uveddi analyze --incremental src/
```

## State Management

### State File Location

By default, incremental state is stored in `.uveddi/incremental_state.json` relative to the project root. This includes:

- File modification times and hashes
- Dependency graph relationships  
- Cached analysis results
- Performance metrics
- Configuration metadata

### State File Structure

```json
{
  "metadata": {
    "state_id": "baseline_20240120_143052_a1b2c3d4",
    "version": 1,
    "created_at": "2024-01-20T14:30:52Z",
    "updated_at": "2024-01-20T15:45:12Z",
    "project_root": "/path/to/project",
    "checksum": "blake3_hash_here"
  },
  "file_states": {
    "src/main.rs": {
      "last_modified": 1705756252,
      "content_hash": "abc123...",
      "file_size": 1024,
      "dependencies": ["src/utils.rs"],
      "dependents": [],
      "last_analyzed": "2024-01-20T14:30:52Z",
      "exists": true
    }
  },
  "dependency_graph": {
    "dependencies": {
      "src/main.rs": ["src/utils.rs"]
    },
    "dependents": {
      "src/utils.rs": ["src/main.rs"]
    }
  },
  "analysis_cache": {
    "src/main.rs": {
      "issues": [],
      "cached_at": "2024-01-20T14:30:52Z",
      "analysis_duration_ms": 150,
      "is_valid": true
    }
  }
}
```

### State Validation

The system validates state integrity on load:

- **Checksum Validation**: Ensures state hasn't been corrupted
- **Project Root Matching**: Validates state belongs to current project  
- **Age Checking**: Warns if state is older than configured limits
- **Environment Compatibility**: Checks for major version changes

## Troubleshooting

### Common Issues

#### Incremental Analysis Not Activating

**Symptoms**: Always shows `was_incremental: false`

**Causes and Solutions**:
1. **First Run**: Incremental analysis requires an existing state file
   - Solution: Run analysis twice; second run should be incremental
2. **High Change Percentage**: Too many files changed (>30% by default)
   - Solution: Lower `full_analysis_threshold` or make smaller changes
3. **No Existing State**: State file missing or corrupted  
   - Solution: Check `.uveddi/incremental_state.json` exists and is valid
4. **Configuration Disabled**: `enabled: false` in config
   - Solution: Ensure `IncrementalAnalysisConfig::enabled` is `true`

#### Poor Performance Improvement

**Symptoms**: Time savings <20%

**Causes and Solutions**:
1. **Small Codebase**: Benefits more apparent with 1000+ files
   - Solution: Use with larger projects or wait for Phase 3 optimizations
2. **High Change Rate**: Many files changing between runs
   - Solution: Typical improvement assumes <10% file changes
3. **Dependency Overhead**: Complex dependency graphs
   - Solution: Optimize import statements and reduce coupling
4. **Cache Misses**: Low cache hit rate
   - Solution: Check cache configuration and file ignore patterns

#### Change Detection Issues

**Symptoms**: Files not detected as changed or false positives

**Causes and Solutions**:
1. **Timestamp Issues**: File system timestamp precision
   - Solution: Enable content hashing with `use_content_hash: true`
2. **Ignore Patterns**: Files excluded by ignore patterns
   - Solution: Review and adjust `ignore_patterns` configuration
3. **Hash Algorithm**: Hash conflicts (rare)
   - Solution: Switch to different hash algorithm (blake3 recommended)

### Debug Information

Enable detailed logging for troubleshooting:

```rust
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
```

Or via environment variable:
```bash
RUST_LOG=debug uveddi analyze --incremental src/
```

### Performance Profiling

```rust
// Enable performance metrics collection
let config = IncrementalAnalysisConfig {
    performance: PerformanceConfig {
        enable_memory_optimization: true,
        max_parallel_files: 8,
        ..Default::default()
    },
    ..Default::default()
};

let (issues, result) = engine.analyze_incremental(path, config).await?;

// Review detailed metrics
println!("Performance Metrics:");
println!("  Change detection: {}ms", result.performance_metrics.change_detection_time_ms);
println!("  Dependency analysis: {}ms", result.performance_metrics.dependency_analysis_time_ms);
println!("  Re-analysis: {}ms", result.performance_metrics.reanalysis_time_ms);
println!("  State persistence: {}ms", result.performance_metrics.state_persistence_time_ms);
println!("  Peak memory: {:.1}MB", result.performance_metrics.peak_memory_usage_mb);
```

## Future Enhancements (Phase 3+)

The incremental analysis system provides the foundation for future optimizations:

- **Phase 3**: Advanced diagram caching with selective regeneration
- **Phase 4**: Parallel diagram generation using dependency ordering
- **Phase 5**: Memory usage optimization with incremental GC
- **Phase 6**: Real-time file system watching and instant updates
- **Phase 7**: Distributed analysis across multiple machines

## API Reference

For complete API documentation, see:

- [`IncrementalAnalysisEngine`](src/analysis/incremental/incremental_engine.rs)
- [`ChangeDetector`](src/analysis/incremental/change_detector.rs)  
- [`DependencyTracker`](src/analysis/incremental/dependency_tracker.rs)
- [`IncrementalStateManager`](src/analysis/incremental/state_manager.rs)
- [`IncrementalCache`](src/cache/incremental_cache.rs)

## Testing

Run the complete test suite:

```bash
# Unit tests
cargo test incremental

# Integration tests  
cargo test incremental_analysis_integration

# Benchmark tests
cargo bench incremental
```

## Contributing

When contributing to the incremental analysis system:

1. **Maintain Accuracy**: Incremental results must match full analysis results
2. **Performance Focus**: All changes should maintain or improve the 50%+ target
3. **Test Coverage**: Add tests for new change detection patterns
4. **Documentation**: Update this guide for configuration changes
5. **Backward Compatibility**: Ensure state file format migrations work correctly

## Conclusion

UV-91 Phase 2 delivers a production-ready incremental analysis system that achieves significant performance improvements while maintaining full accuracy. The system provides a solid foundation for the advanced optimizations planned in subsequent phases, enabling Uveddi to scale to the largest enterprise codebases efficiently.