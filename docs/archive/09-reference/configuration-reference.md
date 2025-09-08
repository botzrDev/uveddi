# Configuration Reference

## Overview

Uveddi provides extensive configuration options through TOML files, environment variables, and command-line arguments. This reference documents all 47+ configuration settings available in the system.

## Configuration File Locations

Configuration files are loaded in the following order (later files override earlier ones):

1. **System-wide**: `/etc/uveddi/config.toml`
2. **User-specific**: `~/.config/uveddi/config.toml`
3. **Project-specific**: `./uveddi.toml` or `./.uveddi/config.toml`
4. **Custom path**: Specified via `--config` flag

## Environment Variables

All configuration options can be set via environment variables using the format:
```bash
UVEDDI_<SECTION>_<KEY>=value
```

Example:
```bash
UVEDDI_ANALYSIS_MAX_DEPTH=10
UVEDDI_AI_OLLAMA_MODEL=deepseek-coder:6.7b
```

## Complete Configuration Reference

### Analysis Settings

Core analysis engine configuration options.

```toml
[analysis]
# Languages to analyze (default: all supported)
languages = ["rust", "python", "javascript", "typescript"]

# Maximum directory traversal depth (default: 10)
max_depth = 10

# Analysis timeout in seconds (default: 300)
timeout_seconds = 300

# Enable parallel processing (default: true)
parallel_processing = true

# Maximum number of files to process concurrently (default: 10)
max_concurrent_files = 10

# Per-file timeout in seconds (default: 30)
file_timeout_seconds = 30

# Enable graceful degradation for large codebases (default: true)
enable_graceful_degradation = true

# File threshold before applying degradation (default: 500)
degradation_file_threshold = 500

# Timeout multiplier for large files >1000 lines (default: 2.0)
large_file_timeout_multiplier = 2.0

# Ignore patterns (gitignore format)
ignore_patterns = [
    "target/**",
    "node_modules/**",
    "*.min.js",
    "vendor/**"
]

# Include only these patterns (if specified)
include_patterns = []

# Enable incremental analysis (default: false)
incremental = false

# Enable dead code detection (default: true)
detect_dead_code = true

# Enable anti-pattern detection (default: true)
detect_anti_patterns = true

# Enable security analysis (default: true if feature enabled)
enable_security_analysis = true

# Enable dependency analysis (default: true)
analyze_dependencies = true
```

### Detector Thresholds

Configure detection sensitivity for various anti-patterns and issues.

```toml
[thresholds]
# God Object Detection
god_object_threshold = 100          # Max combined complexity score
god_object_max_methods = 20         # Max number of methods
god_object_max_fields = 15          # Max number of fields
god_object_max_dependencies = 10    # Max external dependencies

# Large Class Detection
max_function_lines = 50             # Max lines per function
max_class_lines = 300               # Max lines per class
max_file_lines = 500                # Max lines per file

# Complexity Thresholds
cyclomatic_complexity_threshold = 10  # Max cyclomatic complexity
cognitive_complexity_threshold = 15   # Max cognitive complexity
nesting_depth_threshold = 5          # Max nesting depth

# Code Duplication
min_duplicate_tokens = 50           # Min tokens for duplication
duplication_similarity_threshold = 0.8  # Similarity threshold (0.0-1.0)

# Tight Coupling
max_coupling_score = 0.7            # Max coupling score (0.0-1.0)
max_efferent_coupling = 20          # Max outgoing dependencies
max_afferent_coupling = 20          # Max incoming dependencies

# Dead Code Detection
dead_code_confidence_threshold = 0.7  # Min confidence to report

# Magic Values
magic_number_exceptions = [0, 1, -1, 100]  # Allowed magic numbers
magic_string_min_length = 3         # Min string length to consider

# Method Length
long_method_threshold = 30          # Lines to consider method long
very_long_method_threshold = 50     # Lines to consider very long
```

### AI Configuration

Settings for AI-powered analysis features.

```toml
[ai]
# AI provider (ollama, openai, anthropic)
provider = "ollama"

# Model to use for analysis
model = "deepseek-coder:6.7b"

# API endpoint URL
api_url = "http://localhost:11434"

# API key (for cloud providers)
api_key = ""

# Enable AI explanations (default: false)
enable_ai = false

# Privacy mode (strict, balanced, enhanced)
privacy_mode = "strict"

# Token budget for prompts (default: 4000)
default_token_budget = 4000

# Context window size (default: 2000)
max_context_size = 2000

# Temperature for generation (0.0-1.0)
temperature = 0.7

# Enable prompt caching (default: true)
enable_prompt_caching = true

# Prompt cache TTL in seconds (default: 3600)
prompt_cache_ttl = 3600

# Retry failed AI requests (default: true)
retry_on_failure = true

# Max retry attempts (default: 3)
max_retry_attempts = 3

# Timeout for AI requests in seconds (default: 30)
ai_request_timeout = 30
```

### Output Configuration

Control report generation and formatting.

```toml
[output]
# Output format (html, json, markdown, sarif)
format = "html"

# Include timing information (default: true)
include_timing = true

# Include AI explanations in output (default: true)
include_ai_explanations = true

# Include source code snippets (default: true)
include_code_snippets = true

# Max snippet lines to include (default: 10)
max_snippet_lines = 10

# Generate dependency diagrams (default: true)
generate_diagrams = true

# Diagram format (mermaid, graphviz, cytoscape)
diagram_format = "mermaid"

# Include metrics in output (default: true)
include_metrics = true

# Verbose output (default: false)
verbose = false

# Silent mode - no console output (default: false)
silent = false

# Output directory for reports
output_directory = "./reports"

# Timestamp format for output files
timestamp_format = "%Y%m%d_%H%M%S"
```

### Cache Configuration

Performance optimization through caching.

```toml
[cache]
# Cache type (memory, disk, hybrid)
cache_type = "hybrid"

# Maximum cache size in MB (default: 100)
max_size = "100MB"

# Cache eviction policy (lru, lfu, fifo)
eviction_policy = "lru"

# Enable AST caching (default: true)
enable_ast_cache = true

# AST cache size in entries (default: 1000)
ast_cache_size = 1000

# Cache directory for disk cache
cache_directory = "~/.cache/uveddi"

# Cache TTL in seconds (default: 3600)
cache_ttl = 3600

# Enable result caching (default: true)
enable_result_cache = true

# Result cache size in MB (default: 50)
result_cache_size = 50

# Enable dependency graph caching (default: true)
enable_graph_cache = true

# Clear cache on startup (default: false)
clear_cache_on_startup = false
```

### Memory Configuration

Memory management and optimization settings.

```toml
[memory]
# Memory allocation strategy (conservative, balanced, aggressive)
allocation_strategy = "balanced"

# Total memory limit in MB (default: 100)
total_limit_mb = 100

# Cache memory limit in MB (default: 50)
cache_limit_mb = 50

# Buffer memory limit in MB (default: 20)
buffer_limit_mb = 20

# Working set limit in MB (default: 30)
working_set_limit_mb = 30

# Enable garbage collection (default: true)
enable_gc = true

# GC trigger threshold in MB (default: 80)
gc_trigger_threshold_mb = 80

# Target reduction in MB (default: 20)
gc_target_reduction_mb = 20

# Max GC pause in milliseconds (default: 10)
max_gc_pause_ms = 10

# Enable memory profiling (default: false)
enable_memory_profiling = false

# Memory sample interval in ms (default: 5000)
memory_sample_interval_ms = 5000
```

### Server Configuration

Web server and API settings.

```toml
[server]
# Server port (default: 8888)
port = 8888

# Bind address (default: 0.0.0.0)
bind_address = "0.0.0.0"

# Enable CORS (default: true)
enable_cors = true

# CORS allowed origins
cors_allowed_origins = ["*"]

# Enable WebSocket (default: true)
enable_websocket = true

# WebSocket port (default: same as server)
websocket_port = 8888

# Rate limiting - requests per minute (default: 100)
rate_limit_per_minute = 100

# Max concurrent connections (default: 100)
max_connections = 100

# Request timeout in seconds (default: 30)
request_timeout = 30

# Enable HTTPS (default: false)
enable_https = false

# SSL certificate path
ssl_cert_path = ""

# SSL key path
ssl_key_path = ""

# Static assets path for SPA
spa_assets_path = "./web/dist"

# Enable GraphQL endpoint (default: true)
enable_graphql = true

# Enable REST API (default: true)
enable_rest_api = true
```

### Plugin Configuration

WebAssembly plugin system settings.

```toml
[plugins]
# Enable plugin system (default: false)
enable_plugins = false

# Plugin directory
plugin_directory = "./plugins"

# Auto-load plugins on startup (default: true)
auto_load = true

# Plugin timeout in seconds (default: 10)
plugin_timeout = 10

# Max plugin memory in MB (default: 50)
max_plugin_memory = 50

# Allowed plugin capabilities
allowed_capabilities = ["analysis", "reporting"]

# Plugin sandbox mode (strict, relaxed)
sandbox_mode = "strict"

# Enable plugin hot-reload (default: false)
enable_hot_reload = false

# Plugin API version
api_version = "1.0.0"
```

### Security Configuration

Security and access control settings.

```toml
[security]
# Enable security features (default: true)
enabled = true

# Enable input validation (default: true)
enable_input_validation = true

# Max input size in bytes (default: 10MB)
max_input_size = 10485760

# Enable rate limiting (default: true)
enable_rate_limiting = true

# Rate limit requests per second (default: 10)
rate_limit_requests_per_second = 10

# Rate limit burst size (default: 20)
rate_limit_burst_size = 20

# Enable audit logging (default: false)
enable_audit_logging = false

# Audit log retention days (default: 30)
audit_retention_days = 30

# Blocked file patterns
blocked_patterns = ["*.exe", "*.dll", "*.so"]

# Enable authentication (default: false)
enable_authentication = false

# Authentication method (jwt, api_key, oauth)
auth_method = "jwt"

# JWT secret (if using JWT auth)
jwt_secret = ""

# Session timeout in seconds (default: 3600)
session_timeout = 3600
```

### Monitoring Configuration

Observability and monitoring settings.

```toml
[monitoring]
# Enable metrics collection (default: true)
enable_metrics = true

# Metrics export interval in seconds (default: 10)
metrics_export_interval = 10

# Enable distributed tracing (default: false)
enable_tracing = false

# Tracing backend (jaeger, zipkin, otlp)
tracing_backend = "otlp"

# Tracing endpoint
tracing_endpoint = "http://localhost:4317"

# Log level (trace, debug, info, warn, error)
log_level = "info"

# Enable structured logging (default: true)
structured_logging = true

# Log sampling rate (0.0-1.0, default: 1.0)
log_sampling_rate = 1.0

# Enable performance logging (default: false)
performance_logging = false

# Health check interval in seconds (default: 30)
health_check_interval = 30

# Health check timeout in seconds (default: 5)
health_check_timeout = 5

# Alert thresholds
[monitoring.alerts]
memory_warning_mb = 80
memory_critical_mb = 95
cpu_warning_percent = 70
cpu_critical_percent = 90
error_rate_warning = 0.05
error_rate_critical = 0.10
```

### Database Configuration

Database and persistence settings.

```toml
[database]
# Database type (sqlite, postgres)
database_type = "sqlite"

# Database path (for SQLite)
database_path = "./uveddi.db"

# Connection string (for PostgreSQL)
connection_string = ""

# Connection pool size (default: 10)
pool_size = 10

# Connection timeout in seconds (default: 5)
connection_timeout = 5

# Enable WAL mode for SQLite (default: true)
enable_wal_mode = true

# Enable foreign keys (default: true)
enable_foreign_keys = true

# Database migrations auto-run (default: true)
auto_migrate = true

# Backup directory
backup_directory = "./backups"

# Auto-backup interval in hours (0 = disabled)
auto_backup_interval = 24
```

### Service Orchestration

Multi-service coordination settings.

```toml
[services]
# Enable service orchestration (default: true)
enable_orchestration = true

# API server port (default: 8888)
api_port = 8888

# Rendering service port (default: 3333)
rendering_port = 3333

# Frontend dev server port (default: 3000)
frontend_port = 3000

# Enable development mode (default: false)
development_mode = false

# Service startup timeout in seconds (default: 30)
startup_timeout = 30

# Health check retries (default: 10)
health_check_retries = 10

# Retry delay in seconds (default: 2)
retry_delay = 2

# Enable auto-restart on failure (default: true)
auto_restart = true

# Max restart attempts (default: 3)
max_restart_attempts = 3
```

## Configuration Examples

### Minimal Configuration

```toml
# Minimal configuration for quick analysis
[analysis]
languages = ["rust"]
timeout_seconds = 60

[output]
format = "markdown"
```

### Production Configuration

```toml
# Full alpha configuration example
[analysis]
languages = ["rust", "python", "javascript", "typescript"]
parallel_processing = true
max_concurrent_files = 20
enable_security_analysis = true

[thresholds]
god_object_threshold = 80
max_function_lines = 40
cyclomatic_complexity_threshold = 8

[ai]
provider = "ollama"
model = "deepseek-coder:6.7b"
enable_ai = true
privacy_mode = "strict"

[cache]
cache_type = "hybrid"
max_size = "500MB"
enable_ast_cache = true

[server]
port = 8080
enable_https = true
ssl_cert_path = "/etc/ssl/certs/uveddi.crt"
ssl_key_path = "/etc/ssl/private/uveddi.key"

[security]
enabled = true
enable_authentication = true
auth_method = "jwt"
enable_audit_logging = true

[monitoring]
enable_metrics = true
enable_tracing = true
log_level = "info"
performance_logging = true

[database]
database_type = "postgres"
connection_string = "postgresql://user:pass@localhost/uveddi"
pool_size = 20
```

### CI/CD Configuration

```toml
# Configuration for CI/CD pipelines
[analysis]
timeout_seconds = 600
parallel_processing = true
incremental = true

[thresholds]
# Strict thresholds for CI
god_object_threshold = 50
max_function_lines = 30
cyclomatic_complexity_threshold = 5

[output]
format = "sarif"  # For GitHub/GitLab integration
include_timing = true
verbose = true

[cache]
cache_type = "disk"
cache_directory = "/tmp/uveddi-cache"

[monitoring]
log_level = "debug"
structured_logging = true
```

### Development Configuration

```toml
# Developer-friendly configuration
[analysis]
languages = ["rust"]
max_depth = 5
timeout_seconds = 120

[ai]
enable_ai = true
provider = "ollama"
model = "llama2"

[output]
format = "html"
include_code_snippets = true
verbose = true

[server]
port = 8888
enable_cors = true
development_mode = true

[monitoring]
log_level = "debug"
performance_logging = true
```

## Configuration Precedence

Configuration values are resolved in the following order (highest to lowest precedence):

1. **Command-line arguments**: `--max-depth 5`
2. **Environment variables**: `UVEDDI_ANALYSIS_MAX_DEPTH=5`
3. **Project config file**: `./uveddi.toml`
4. **User config file**: `~/.config/uveddi/config.toml`
5. **System config file**: `/etc/uveddi/config.toml`
6. **Default values**: Built-in defaults

## Configuration Validation

Uveddi validates configuration on startup and reports any issues:

```bash
# Validate configuration without running analysis
uveddi config validate

# Show effective configuration
uveddi config show

# Generate default configuration file
uveddi config generate > uveddi.toml
```

## Performance Tuning

### For Large Codebases (>10,000 files)

```toml
[analysis]
parallel_processing = true
max_concurrent_files = 20
enable_graceful_degradation = true
degradation_file_threshold = 1000

[cache]
cache_type = "hybrid"
max_size = "1GB"
ast_cache_size = 5000

[memory]
allocation_strategy = "aggressive"
total_limit_mb = 500
```

### For Limited Resources

```toml
[analysis]
parallel_processing = false
max_concurrent_files = 2

[cache]
cache_type = "memory"
max_size = "50MB"

[memory]
allocation_strategy = "conservative"
total_limit_mb = 50
```

### For Maximum Accuracy

```toml
[thresholds]
# Lower thresholds for more sensitive detection
god_object_threshold = 50
max_function_lines = 25
cyclomatic_complexity_threshold = 5
dead_code_confidence_threshold = 0.5

[ai]
enable_ai = true
temperature = 0.3  # Lower temperature for consistency
max_retry_attempts = 5
```

## Troubleshooting Configuration Issues

### Common Problems

1. **Configuration not loading**: Check file permissions and TOML syntax
2. **Environment variables not working**: Ensure correct prefix `UVEDDI_`
3. **Performance issues**: Adjust memory and cache settings
4. **AI not working**: Verify API URL and model availability

### Debug Configuration Loading

```bash
# Enable configuration debug logging
UVEDDI_MONITORING_LOG_LEVEL=trace uveddi analyze ./src

# Show configuration source for each setting
uveddi config show --sources
```

## Migration Guide

### From v0.8.x to v0.9.x

Key changes:
- `detectors` section renamed to `thresholds`
- New `standard_detectors` format with severity levels
- `enable_ai` moved from `analysis` to `ai` section
- New `services` section for orchestration

Migration example:
```toml
# Old format (v0.8.x)
[detectors.god_object]
threshold_methods = 5

# New format (v0.9.x)
[thresholds]
god_object_max_methods = 5
```

## Related Documentation

- [User Guide](../03-user-guide/README.md) - Getting started with Uveddi
- [CLI Reference](../03-user-guide/cli-reference.md) - Command-line options
- [API Reference](../08-api/rest-api-reference.md) - REST API configuration
- [Plugin Development](../04-development/plugin-development.md) - Plugin configuration