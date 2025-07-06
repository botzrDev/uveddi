# Performance Optimization Example

## Analyzing Large Codebases

### Parallel Processing
```bash
# Use all available cores
uveddi analyze ./large-project --jobs 0

# Limit to 4 cores
uveddi analyze ./large-project --jobs 4
```

### Memory Management
```bash
# Set memory limit (in MB)
uveddi analyze ./project --memory-limit 4096
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
