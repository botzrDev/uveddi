# User Manual - Uveddi High-Performance Analysis Engine

## Table of Contents
1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Installation](#installation)
4. [Basic Usage](#basic-usage)
5. [Advanced Features](#advanced-features)
6. [API Usage](#api-usage)
7. [TUI Interface](#tui-interface)
8. [Performance Optimization](#performance-optimization)
9. [Troubleshooting](#troubleshooting)
10. [Best Practices](#best-practices)

## Introduction

Uveddi is a high-performance code analysis engine that delivers 2-30x speedup improvements through intelligent caching and modern architecture. It provides comprehensive code analysis capabilities across multiple programming languages with real-time performance monitoring.

### Key Features
- **Lightning Fast Performance**: 2-30x speedup through multi-layer intelligent caching
- **Multi-Language Support**: Rust, Python, JavaScript, TypeScript, and more
- **Real-Time Analysis**: WebSocket streaming for live results
- **Enterprise Ready**: Production deployment with Kubernetes support
- **Interactive Interface**: Command-line and Terminal UI options
- **Comprehensive APIs**: REST and WebSocket APIs with detailed documentation

### Performance Benefits
- **Small Projects (1K-10K files)**: 2-5x speedup
- **Medium Projects (10K-50K files)**: 5-15x speedup  
- **Large Projects (50K+ files)**: 15-30x speedup
- **Cache Hit Rates**: 85-98% across all analysis types

## Getting Started

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores
- **Memory**: 4GB RAM
- **Storage**: 10GB available space
- **OS**: Linux, macOS, or Windows

#### Recommended Requirements  
- **CPU**: 4+ cores
- **Memory**: 8GB+ RAM
- **Storage**: 50GB+ SSD storage
- **OS**: Linux (Ubuntu 20.04+) or macOS

#### For Large Codebases
- **CPU**: 8+ cores
- **Memory**: 16GB+ RAM
- **Storage**: 100GB+ NVMe SSD
- **Network**: High-speed connection for distributed deployments

### Supported Languages

| Language   | AST Analysis | Semantic Analysis | Dependencies | Magic Values |
|------------|-------------|-------------------|--------------|-------------|
| Rust       | ✅ Full      | ✅ Full            | ✅ Cargo      | ✅ Advanced  |
| Python     | ✅ Full      | ✅ Full            | ✅ pip/conda  | ✅ Advanced  |
| JavaScript | ✅ Full      | ✅ Full            | ✅ npm        | ✅ Advanced  |
| TypeScript | ✅ Full      | ✅ Full            | ✅ npm        | ✅ Advanced  |
| Go         | ✅ Full      | ⚡ Partial         | ✅ modules    | ✅ Basic     |
| Java       | ✅ Full      | ⚡ Partial         | ⚡ Maven      | ✅ Basic     |
| C++        | ✅ Full      | ⚡ Basic           | ⚡ Manual     | ✅ Basic     |

## Installation

### Binary Installation

#### Linux/macOS (Recommended)
```bash
# Download latest release
curl -LO https://github.com/example/uveddi/releases/latest/download/uveddi-linux-x64.tar.gz

# Extract and install
tar -xzf uveddi-linux-x64.tar.gz
sudo mv uveddi /usr/local/bin/
chmod +x /usr/local/bin/uveddi

# Verify installation
uveddi --version
```

#### Windows
```powershell
# Download from releases page or use chocolatey
choco install uveddi

# Verify installation
uveddi --version
```

### Docker Installation

```bash
# Pull the latest image
docker pull uveddi/analysis-engine:latest

# Create a working directory
mkdir ~/uveddi-workspace
cd ~/uveddi-workspace

# Run with Docker
docker run -it --rm \
  -v $(pwd):/workspace \
  -v ~/.uveddi:/home/uveddi/.uveddi \
  -p 3000:3000 \
  uveddi/analysis-engine:latest
```

### Building from Source

```bash
# Prerequisites: Rust 1.70+, Git
git clone https://github.com/example/uveddi.git
cd uveddi

# Build with optimizations
cargo build --release --features="analysis-cache,ast-cache,tui"

# Install locally
cargo install --path . --features="analysis-cache,ast-cache,tui"

# Verify installation
uveddi --version
```

### Configuration Setup

Create a configuration file at `~/.uveddi/config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000
websocket_port = 3001

[logging]
level = "info"
file = "~/.uveddi/logs/uveddi.log"

[cache]
enabled = true
data_dir = "~/.uveddi/cache"

[cache.ast]
enabled = true
max_entries = 10000
max_memory_mb = 2048
eviction_policy = "lru_with_ttl"
ttl_seconds = 3600

[cache.analysis]
enabled = true
max_entries = 5000
max_memory_mb = 4096
eviction_policy = "adaptive"

[cache.memory]
max_total_memory_mb = 8192
enable_pressure_monitoring = true
```

## Basic Usage

### Command Line Interface

#### Basic Analysis
```bash
# Analyze current directory
uveddi analyze

# Analyze specific directory
uveddi analyze /path/to/project

# Analyze with specific language
uveddi analyze --language rust /path/to/rust/project

# Analyze with output format
uveddi analyze --format json --output results.json /path/to/project

# Get analysis summary
uveddi analyze --summary /path/to/project
```

#### Cache Management
```bash
# Check cache status
uveddi cache status

# Clear specific cache
uveddi cache clear --type ast

# Clear all caches
uveddi cache clear --all

# Cache statistics
uveddi cache stats

# Warm cache for project
uveddi cache warm /path/to/project
```

#### Server Mode
```bash
# Start server
uveddi serve

# Start server with custom config
uveddi serve --config /path/to/config.toml

# Start server on specific port
uveddi serve --port 8080

# Start with debug logging
RUST_LOG=debug uveddi serve
```

### Configuration Options

#### Analysis Configuration
```bash
# Enable specific analyzers
uveddi analyze --analyzers dependencies,magic-values,complexity

# Set analysis depth
uveddi analyze --depth full  # full, partial, syntax-only

# Include/exclude patterns
uveddi analyze \
  --include "**/*.rs,**/*.py" \
  --exclude "**/target/**,**/node_modules/**"

# Set timeout
uveddi analyze --timeout 300  # 5 minutes
```

#### Performance Tuning
```bash
# Adjust thread count
uveddi analyze --threads 8

# Memory limit
uveddi analyze --memory-limit 16G

# Enable aggressive caching
uveddi analyze --aggressive-cache

# Parallel analysis
uveddi analyze --parallel --max-parallel 4
```

### Output Formats

#### JSON Output
```bash
uveddi analyze --format json /path/to/project
```

```json
{
  "analysis_id": "uuid-here",
  "timestamp": "2024-01-15T10:30:00Z",
  "project_path": "/path/to/project",
  "performance": {
    "total_duration_ms": 1250,
    "files_analyzed": 1500,
    "speedup_factor": 12.5,
    "cache_hit_rate": 0.92
  },
  "results": {
    "file_count": 1500,
    "language_distribution": {
      "rust": 850,
      "python": 400,
      "javascript": 250
    },
    "complexity": {
      "total_functions": 3400,
      "average_complexity": 4.2,
      "high_complexity_functions": 45
    },
    "dependencies": {
      "total_dependencies": 120,
      "direct_dependencies": 25,
      "outdated_dependencies": 3
    },
    "issues": {
      "total": 15,
      "critical": 2,
      "warnings": 13
    }
  }
}
```

#### Human-Readable Output
```bash
uveddi analyze --format table /path/to/project
```

```
┌─────────────────────────────────────────────────────────────┐
│                    Uveddi Analysis Results                   │
├─────────────────────────────────────────────────────────────┤
│ Project: /home/user/my-project                              │
│ Duration: 1.25s (12.5x speedup)                            │
│ Cache Hit Rate: 92%                                         │
└─────────────────────────────────────────────────────────────┘

📊 Files Analyzed: 1,500 files
   • Rust: 850 files (56.7%)
   • Python: 400 files (26.7%)  
   • JavaScript: 250 files (16.6%)

🔧 Complexity Analysis:
   • Total Functions: 3,400
   • Average Complexity: 4.2
   • High Complexity: 45 functions

📦 Dependencies:
   • Total: 120 dependencies
   • Direct: 25 dependencies
   • Outdated: 3 dependencies

⚠️  Issues Found: 15 total
   • Critical: 2
   • Warnings: 13
```

## Advanced Features

### Real-Time Analysis

#### File Watcher Mode
```bash
# Start watching for file changes
uveddi watch /path/to/project

# Watch with custom patterns
uveddi watch \
  --include "**/*.rs" \
  --exclude "**/target/**" \
  /path/to/project

# Watch with WebSocket streaming
uveddi watch --stream ws://localhost:3001/analysis /path/to/project
```

#### Live Analysis Dashboard
```bash
# Start server with dashboard
uveddi serve --dashboard

# Access dashboard at http://localhost:3000/dashboard
```

### Batch Processing

#### Multiple Projects
```bash
# Analyze multiple projects
uveddi batch \
  --projects /path/to/project1,/path/to/project2,/path/to/project3 \
  --output batch_results.json

# Analyze all projects in directory
uveddi batch \
  --scan-directory /path/to/projects \
  --output-directory /path/to/results
```

#### Scheduled Analysis
```bash
# Set up cron job for daily analysis
crontab -e

# Add line for daily analysis at 2 AM
0 2 * * * /usr/local/bin/uveddi analyze --quiet --output /var/log/uveddi/daily.json /path/to/project
```

### Integration Options

#### CI/CD Integration

##### GitHub Actions
```yaml
name: Code Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Uveddi
        run: |
          curl -LO https://github.com/example/uveddi/releases/latest/download/uveddi-linux-x64.tar.gz
          tar -xzf uveddi-linux-x64.tar.gz
          sudo mv uveddi /usr/local/bin/
          
      - name: Run Analysis
        run: |
          uveddi analyze --format json --output analysis.json
          
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: analysis-results
          path: analysis.json
```

##### Jenkins Pipeline
```groovy
pipeline {
    agent any
    
    stages {
        stage('Analysis') {
            steps {
                sh '''
                    uveddi analyze \
                        --format json \
                        --output analysis.json \
                        --fail-on-critical
                '''
            }
            
            post {
                always {
                    archiveArtifacts artifacts: 'analysis.json'
                }
            }
        }
    }
}
```

#### IDE Integration

##### VS Code Extension
1. Install the Uveddi VS Code extension
2. Configure workspace settings:
```json
{
  "uveddi.enable": true,
  "uveddi.server.port": 3000,
  "uveddi.analysis.realTime": true,
  "uveddi.cache.enabled": true
}
```

##### Vim Plugin
```vim
" Add to .vimrc
Plugin 'uveddi/vim-uveddi'

" Configure Uveddi
let g:uveddi_enable = 1
let g:uveddi_server_port = 3000
let g:uveddi_realtime = 1
```

## API Usage

### REST API

#### Authentication
```bash
# Get API token
curl -X POST http://localhost:3000/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass"}'

# Use token in subsequent requests
export UVEDDI_TOKEN="your-token-here"
curl -H "Authorization: Bearer $UVEDDI_TOKEN" \
  http://localhost:3000/api/v1/analysis/status
```

#### Analysis Endpoints

##### Start Analysis
```bash
curl -X POST http://localhost:3000/api/v1/analysis \
  -H "Authorization: Bearer $UVEDDI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "target": {
      "type": "path",
      "path": "/path/to/project"
    },
    "options": {
      "language": "auto",
      "analyzers": ["dependencies", "complexity", "magic-values"],
      "cache_strategy": "adaptive"
    }
  }'
```

##### Get Analysis Results
```bash
curl -H "Authorization: Bearer $UVEDDI_TOKEN" \
  http://localhost:3000/api/v1/analysis/12345/results
```

##### Analysis Status
```bash
curl -H "Authorization: Bearer $UVEDDI_TOKEN" \
  http://localhost:3000/api/v1/analysis/12345/status
```

#### Cache Management Endpoints

##### Cache Statistics
```bash
curl -H "Authorization: Bearer $UVEDDI_TOKEN" \
  http://localhost:3000/api/v1/cache/stats
```

##### Clear Cache
```bash
curl -X POST http://localhost:3000/api/v1/cache/control \
  -H "Authorization: Bearer $UVEDDI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "clear",
    "target": "ast",
    "confirm": true
  }'
```

##### Warm Cache
```bash
curl -X POST http://localhost:3000/api/v1/cache/warmup \
  -H "Authorization: Bearer $UVEDDI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "target": {
      "type": "path", 
      "path": "/path/to/project"
    },
    "strategy": "common_patterns"
  }'
```

### WebSocket API

#### Real-Time Analysis Updates
```javascript
const ws = new WebSocket('ws://localhost:3001/analysis');

ws.onopen = function() {
    // Subscribe to analysis updates
    ws.send(JSON.stringify({
        type: 'subscribe',
        channel: 'analysis_progress',
        analysis_id: '12345'
    }));
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    
    switch(data.type) {
        case 'analysis_started':
            console.log('Analysis started:', data.analysis_id);
            break;
            
        case 'progress_update':
            console.log(`Progress: ${data.percentage}%`);
            break;
            
        case 'analysis_complete':
            console.log('Analysis complete:', data.results);
            break;
            
        case 'cache_update':
            console.log('Cache hit rate:', data.hit_rate);
            break;
    }
};
```

#### Live Performance Monitoring
```javascript
const performanceWs = new WebSocket('ws://localhost:3001/metrics');

performanceWs.onopen = function() {
    performanceWs.send(JSON.stringify({
        type: 'subscribe',
        channels: ['cache_metrics', 'performance_metrics']
    }));
};

performanceWs.onmessage = function(event) {
    const data = JSON.parse(event.data);
    updateDashboard(data);
};
```

### SDK Usage

#### JavaScript/Node.js SDK
```bash
npm install @uveddi/sdk
```

```javascript
const { UveddiClient } = require('@uveddi/sdk');

const client = new UveddiClient({
    baseUrl: 'http://localhost:3000',
    token: 'your-token-here'
});

// Start analysis
const analysis = await client.analyze('/path/to/project', {
    analyzers: ['dependencies', 'complexity'],
    cache_strategy: 'adaptive'
});

console.log('Analysis ID:', analysis.id);

// Wait for completion
const results = await analysis.wait();
console.log('Results:', results);

// Stream progress updates
analysis.onProgress((progress) => {
    console.log(`${progress.percentage}% complete`);
});
```

#### Python SDK
```bash
pip install uveddi-sdk
```

```python
from uveddi import UveddiClient

client = UveddiClient(
    base_url='http://localhost:3000',
    token='your-token-here'
)

# Start analysis
analysis = client.analyze('/path/to/project', 
    analyzers=['dependencies', 'complexity'],
    cache_strategy='adaptive'
)

print(f'Analysis ID: {analysis.id}')

# Stream results
for progress in analysis.stream_progress():
    print(f'{progress.percentage}% complete')

# Get final results
results = analysis.get_results()
print(f'Speedup: {results.performance.speedup_factor}x')
```

## TUI Interface

### Starting the TUI

```bash
# Start Terminal UI
uveddi tui

# Start with specific project
uveddi tui --project /path/to/project

# Start with custom configuration
uveddi tui --config ~/.uveddi/tui.toml
```

### TUI Navigation

#### Main Interface
```
┌─ Uveddi Analysis Engine ─────────────────────────────────────┐
│  [F1] Help  [F2] Config  [F3] Cache  [F4] Analysis  [F10] Quit │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  📊 Current Analysis Status                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Project: /home/user/my-rust-project                    │  │
│  │ Status: Analyzing... (65% complete)                    │  │
│  │ Files: 1,250 / 1,920                                   │  │
│  │ Cache Hit Rate: 94%                                    │  │
│  │ Current Speedup: 15.2x                                 │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  🔍 Recent Findings                                          │
│  • 3 high-complexity functions detected                      │
│  • 12 dependencies need updates                             │
│  • 2 potential magic value issues                           │
│                                                              │
│  ⚡ Performance Metrics                                       │
│  • Memory Usage: 2.1GB / 8.0GB (26%)                        │
│  • CPU Usage: 65%                                           │
│  • Disk I/O: 45 MB/s read, 12 MB/s write                    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### Cache Management View
```
┌─ Cache Management ───────────────────────────────────────────┐
│                                                              │
│  📦 AST Cache                                                │
│  ├─ Status: Active                                           │
│  ├─ Entries: 8,450 / 10,000 (85%)                          │
│  ├─ Hit Rate: 96%                                           │
│  ├─ Memory: 1.8GB / 2.0GB                                   │
│  └─ Last Eviction: 2 minutes ago                            │
│                                                              │
│  🔍 Analysis Cache                                           │
│  ├─ Status: Active                                           │
│  ├─ Entries: 3,200 / 5,000 (64%)                           │
│  ├─ Hit Rate: 91%                                           │
│  ├─ Memory: 2.8GB / 4.0GB                                   │
│  └─ Last Eviction: 15 minutes ago                           │
│                                                              │
│  🌐 Graph Cache                                              │
│  ├─ Status: Active                                           │
│  ├─ Entries: 450 / 1,000 (45%)                             │
│  ├─ Hit Rate: 88%                                           │
│  ├─ Memory: 512MB / 1.0GB                                   │
│  └─ Last Eviction: Never                                    │
│                                                              │
│  [C] Clear All  [A] Clear AST  [N] Clear Analysis  [G] Clear Graph │
└──────────────────────────────────────────────────────────────┘
```

### TUI Keyboard Shortcuts

#### Global Shortcuts
- **F1**: Help screen
- **F2**: Configuration editor
- **F3**: Cache management
- **F4**: Analysis controls
- **F5**: Refresh current view
- **F10** / **Ctrl+Q**: Quit application
- **Tab**: Next panel
- **Shift+Tab**: Previous panel

#### Analysis Control
- **Space**: Start/pause analysis
- **S**: Stop current analysis
- **R**: Restart analysis
- **Enter**: Analyze selected directory

#### Cache Management
- **C**: Clear all caches
- **A**: Clear AST cache only
- **N**: Clear analysis cache only
- **G**: Clear graph cache only
- **W**: Warm cache for current project

#### Navigation
- **↑/↓**: Navigate lists
- **PgUp/PgDn**: Page up/down
- **Home/End**: Go to start/end
- **Ctrl+F**: Search/filter

### TUI Configuration

Create `~/.uveddi/tui.toml`:

```toml
[tui]
theme = "dark"  # dark, light, auto
refresh_rate_ms = 1000
show_progress_bars = true
show_file_details = true

[tui.colors]
primary = "#007acc"
secondary = "#f0f0f0" 
success = "#00aa00"
warning = "#ffaa00"
error = "#cc0000"

[tui.panels]
show_metrics = true
show_cache_stats = true
show_recent_files = true
max_recent_files = 20

[tui.performance]
update_frequency_ms = 500
enable_real_time_metrics = true
history_buffer_size = 1000
```

## Performance Optimization

### Cache Configuration

#### Optimal Cache Settings for Different Scenarios

##### Small Projects (< 1K files)
```toml
[cache.ast]
max_entries = 5000
max_memory_mb = 1024
eviction_policy = "lru"

[cache.analysis]  
max_entries = 2000
max_memory_mb = 2048
eviction_policy = "lru"

[cache.memory]
max_total_memory_mb = 4096
```

##### Medium Projects (1K-10K files)
```toml
[cache.ast]
max_entries = 25000
max_memory_mb = 4096
eviction_policy = "lru_with_ttl"
ttl_seconds = 1800

[cache.analysis]
max_entries = 10000
max_memory_mb = 8192
eviction_policy = "adaptive"

[cache.memory]
max_total_memory_mb = 16384
```

##### Large Projects (10K+ files)
```toml
[cache.ast]
max_entries = 100000
max_memory_mb = 16384
eviction_policy = "adaptive"

[cache.analysis]
max_entries = 50000
max_memory_mb = 32768
eviction_policy = "adaptive"
persist_on_shutdown = true

[cache.memory]
max_total_memory_mb = 65536
enable_pressure_monitoring = true
```

### Performance Monitoring

#### Built-in Metrics
```bash
# Get performance metrics
curl http://localhost:3000/api/v1/metrics

# Get cache-specific metrics
curl http://localhost:3000/api/v1/cache/metrics

# Get real-time performance data
curl http://localhost:3000/api/v1/performance/current
```

#### Custom Performance Tuning
```bash
# Set custom thread pool size
uveddi serve --threads 16

# Adjust memory limits
uveddi serve --max-memory 32G

# Enable performance profiling
uveddi serve --enable-profiling

# Custom cache settings
uveddi serve \
  --cache-memory 16G \
  --cache-entries 100000 \
  --enable-adaptive-caching
```

### Optimization Strategies

#### For Maximum Speed
1. **Increase memory allocation** for caches
2. **Enable aggressive caching** with longer TTL
3. **Use fast storage** (NVMe SSD)
4. **Increase thread count** to match CPU cores
5. **Enable file watcher** for incremental updates

#### For Memory Efficiency
1. **Use conservative cache limits**
2. **Enable aggressive eviction**
3. **Shorter TTL values**
4. **Disable persistence** for analysis cache
5. **Enable memory pressure monitoring**

#### For Network Deployments
1. **Use distributed caching** with Redis
2. **Enable cache persistence**
3. **Configure cache replication**
4. **Use WebSocket streaming** for large results
5. **Enable compression** for API responses

## Troubleshooting

### Common Issues

#### High Memory Usage
**Problem**: Uveddi consuming excessive memory

**Solutions**:
```bash
# Check memory usage
uveddi cache stats

# Reduce cache limits
uveddi cache config --ast-memory 2G --analysis-memory 4G

# Enable aggressive eviction
uveddi cache config --enable-aggressive-eviction

# Clear caches
uveddi cache clear --all
```

#### Slow Analysis Performance
**Problem**: Analysis taking too long despite caching

**Diagnostics**:
```bash
# Check cache hit rates
curl http://localhost:3000/api/v1/cache/stats

# Monitor file watcher status
curl http://localhost:3000/api/v1/cache/watcher/status

# Check system resources
uveddi system status
```

**Solutions**:
```bash
# Warm cache for project
uveddi cache warm /path/to/project

# Increase cache sizes
uveddi cache config --max-memory 16G

# Enable parallel analysis
uveddi analyze --parallel --threads 8
```

#### Cache Miss Issues
**Problem**: Low cache hit rates

**Diagnostics**:
```bash
# Detailed cache analysis
curl http://localhost:3000/api/v1/cache/analysis

# Check file modification patterns
uveddi cache debug --show-misses
```

**Solutions**:
```bash
# Adjust TTL settings
uveddi cache config --ttl 3600

# Enable smarter eviction
uveddi cache config --eviction-policy adaptive

# Increase cache entry limits
uveddi cache config --max-entries 50000
```

#### Server Connection Issues
**Problem**: Unable to connect to Uveddi server

**Diagnostics**:
```bash
# Check server status
curl http://localhost:3000/health

# Check port usage
netstat -tlnp | grep 3000

# Check logs
tail -f ~/.uveddi/logs/uveddi.log
```

**Solutions**:
```bash
# Restart server
uveddi serve --force-restart

# Change port
uveddi serve --port 8080

# Check firewall settings
sudo ufw status
```

### Debug Commands

#### Enable Debug Logging
```bash
# Debug mode
RUST_LOG=debug uveddi analyze /path/to/project

# Trace mode (very verbose)
RUST_LOG=trace uveddi analyze /path/to/project

# Component-specific logging
RUST_LOG=uveddi::cache=debug,uveddi::analysis=info uveddi serve
```

#### Cache Debugging
```bash
# Cache debug information
uveddi cache debug

# Cache consistency check
uveddi cache check --fix-issues

# Export cache data
uveddi cache export --format json --output cache_dump.json

# Import cache data
uveddi cache import cache_dump.json
```

#### Performance Debugging
```bash
# Enable performance profiling
uveddi serve --profile --profile-output profile.json

# System resource monitoring
uveddi system monitor --interval 5

# Analysis timing breakdown
uveddi analyze --show-timing /path/to/project
```

### Getting Help

#### Built-in Help
```bash
# General help
uveddi --help

# Command-specific help
uveddi analyze --help
uveddi cache --help
uveddi serve --help

# Configuration help
uveddi config --help
```

#### Documentation
- **Online Documentation**: https://docs.uveddi.com
- **API Reference**: https://api.uveddi.com
- **GitHub Issues**: https://github.com/example/uveddi/issues
- **Community Forum**: https://forum.uveddi.com

#### Support Channels
- **Email**: support@uveddi.com  
- **Discord**: https://discord.gg/uveddi
- **Stack Overflow**: Tag questions with `uveddi`

## Best Practices

### Project Setup

#### Initial Configuration
1. **Configure cache appropriately** for your project size
2. **Set up file watchers** for real-time updates
3. **Configure language-specific settings**
4. **Set up monitoring** and alerting
5. **Test with small subset** before full analysis

#### Continuous Integration
1. **Use appropriate timeouts** for CI environments
2. **Cache results between runs**
3. **Fail builds on critical issues**
4. **Archive analysis results**
5. **Set up performance monitoring**

### Performance Best Practices

#### Cache Management
1. **Monitor cache hit rates** regularly
2. **Adjust cache sizes** based on usage patterns
3. **Use appropriate eviction policies**
4. **Enable persistence** for long-running analyses
5. **Clear caches** when necessary

#### Resource Optimization
1. **Allocate sufficient memory** for caches
2. **Use fast storage** (SSD preferred)
3. **Monitor system resources**
4. **Tune thread pools** for your hardware
5. **Enable parallel processing** when beneficial

#### Analysis Strategy
1. **Use incremental analysis** for large projects
2. **Configure appropriate timeouts**
3. **Filter unnecessary files/directories**
4. **Use appropriate analysis depth**
5. **Monitor analysis patterns**

This comprehensive user manual provides everything needed to effectively use Uveddi's high-performance analysis engine, from basic usage to advanced optimization strategies.