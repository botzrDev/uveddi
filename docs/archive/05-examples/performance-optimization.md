# Performance Optimization Guide

> **New in Latest Version**: Memory optimization is **enabled by default** with automatic system detection. Most users don't need manual configuration!

## Automatic Optimizations (Default Behavior)

Uveddi now automatically:
- **Detects system memory** and sets appropriate limits
- **Selects optimal memory profile** (small/default/large) based on RAM
- **Enables object pooling** for reduced allocations
- **Uses arena allocation** for temporary objects 
- **Applies zero-copy AST caching** for better performance

### Basic Usage (Optimized by Default)
```bash
# This automatically uses the best settings for your system
uveddi analyze ./large-project

# Memory optimization details will be logged
RUST_LOG=debug uveddi analyze ./project
```

## Manual Optimization (Advanced Users)

### Memory Profile Override
```bash
# Force specific memory profile
uveddi analyze ./project --memory-profile large    # For systems with 16GB+ RAM
uveddi analyze ./project --memory-profile default  # Balanced (auto-selected for 8GB+ RAM)
uveddi analyze ./project --memory-profile small    # For systems with <8GB RAM
```

### Memory Limit Override
```bash
# Set manual memory limit (auto-detected by default)
uveddi analyze ./project --memory-limit-gb 8

# Disable memory optimization (not recommended)
uveddi analyze ./project --disable-memory-optimization
```

## Configuration Optimizations

### Exclude Paths
```toml
[analysis]
exclude = [
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**"
]
```

### File Size Limits
```toml
[analysis]
max_file_size = "1MB"  # Skip large files
skip_binaries = true   # Skip binary files
```

## Caching Strategies

### Enable Analysis Cache
```bash
uveddi analyze ./project --cache-dir .uveddi/cache
```

### Cache Configuration
```toml
[cache]
enabled = true
directory = ".uveddi/cache"
ttl = "24h"  # Cache expiration
```

## Benchmarking

### Generate Performance Report
```bash
uveddi benchmark ./project --output perf.json
```

### Example Benchmark Results
```json
{
  "total_files": 1245,
  "processed_files": 1021,
  "analysis_time": "45.23s",
  "memory_usage": "1.2GB",
  "throughput": "22.6 files/second"
}
```

## Optimization Tips

1. **Profile First**: Run `uveddi benchmark` to identify bottlenecks
2. **Exclude Non-Essential Files**: Skip tests, dependencies, and generated files
3. **Use Cache**: Especially helpful for incremental analysis
4. **Adjust Parallelism**: Find optimal job count for your hardware
5. **Schedule Heavy Jobs**: Run full analysis during off-hours
