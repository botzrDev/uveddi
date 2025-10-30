# Uveddi Configuration Reference

Comprehensive reference for all configuration options, settings, and customization possibilities in Uveddi.

## Table of Contents

1. [Configuration Overview](#configuration-overview)
2. [Configuration Methods](#configuration-methods)
3. [Configuration File Format](#configuration-file-format)
4. [Environment Variables](#environment-variables)
5. [Command Line Overrides](#command-line-overrides)
6. [Memory Management](#memory-management)
7. [Analysis Settings](#analysis-settings)
8. [AI Provider Configuration](#ai-provider-configuration)
9. [Plugin Configuration](#plugin-configuration)
10. [Output Configuration](#output-configuration)
11. [Performance Tuning](#performance-tuning)
12. [Security Settings](#security-settings)
13. [Examples](#examples)
14. [Validation](#validation)

---

## Configuration Overview

Uveddi uses a hierarchical configuration system with the following precedence (highest to lowest):

1. **Command-line arguments** - Direct CLI flags
2. **Environment variables** - `UVEDDI_*` prefixed variables  
3. **Project configuration** - `uveddi.toml` in project root
4. **Global configuration** - `~/.config/uveddi/config.toml`
5. **Default values** - Built-in sensible defaults

### Default Behavior

Uveddi is pre-configured with optimized defaults:

- ✅ **Memory optimization enabled** with automatic system detection
- ✅ **Tree-sitter parsing enabled** for all supported languages
- ✅ **Smart memory profiles** automatically selected (small/default/large)
- ✅ **Automatic memory limits** based on available system RAM
- ✅ **Parallel processing** using all available CPU cores

---

## Configuration Methods

### 1. Configuration Files

#### Global Configuration
```bash
# Default location
~/.config/uveddi/config.toml

# Custom location via environment
export UVEDDI_CONFIG_PATH=/path/to/custom/config.toml

# Custom location via CLI
uveddi --config /path/to/config.toml analyze ./src
```

#### Project Configuration
```bash
# Project root
./uveddi.toml

# Custom name
./custom-uveddi-config.toml
```

### 2. Environment Variables
```bash
export UVEDDI_MEMORY_LIMIT_GB=8
export UVEDDI_AI_ENABLED=true
export OLLAMA_MODEL=deepseek-coder:6.7b
```

### 3. Command Line Arguments
```bash
uveddi analyze ./src --memory-limit-gb 8 --enable-ai --ollama-model deepseek-coder:6.7b
```

### 4. Configuration Profiles
```bash
# Use specific profile
uveddi --profile production analyze ./src

# List available profiles
uveddi config profiles
```

---

## Configuration File Format

### Complete Configuration Example

```toml
# ~/.config/uveddi/config.toml or ./uveddi.toml

[general]
version = "1.0"
profile = "default"

[project]
name = "my-project"
version = "1.0.0"
description = "Project description"
exclude_patterns = [
    "target/",
    "node_modules/",
    ".git/",
    "*.generated.*",
    "test_data/"
]
include_patterns = [
    "src/**/*.rs",
    "src/**/*.py",
    "src/**/*.js",
    "src/**/*.ts"
]

[analysis]
# Language support
languages = ["rust", "python", "javascript", "typescript"]
max_file_size = "5MB"
max_depth = 10
timeout_seconds = 600
parallel_jobs = 0  # 0 = auto-detect CPU cores
follow_symlinks = false
include_tests = false
incremental_analysis = true

# Parser configuration
parser_timeout = 30
max_parse_errors = 10
strict_parsing = false

[memory]
# Memory optimization (enabled by default)
enable_optimization = true
profile = "auto"  # "small", "default", "large", "auto"
limit_gb = 0.0    # 0.0 = auto-detect
arena_size_mb = 64
pool_initial_capacity = 1000
gc_threshold_mb = 512

# Advanced memory settings
zero_copy_caching = true
object_pooling = true
lazy_loading = true
memory_mapping = true

[thresholds]
# Anti-pattern detection thresholds
god_object_threshold = 100
max_function_lines = 50
max_class_lines = 300
max_complexity = 15
max_parameters = 8
max_nesting_depth = 5
dead_code_confidence = 0.8
duplication_threshold = 0.85
coupling_threshold = 0.7

# Size thresholds
max_file_lines = 1000
max_method_count = 20
max_field_count = 30

[detectors]
# Enable/disable specific detectors
god_object = true
dead_code = true
circular_dependencies = true
tight_coupling = true
magic_values = true
code_duplication = true
long_methods = true
large_classes = true
leaky_abstractions = true
security_vulnerabilities = true

# Detector-specific configuration
[detectors.god_object]
enabled = true
method_threshold = 100
field_threshold = 50
complexity_threshold = 20

[detectors.dead_code]
enabled = true
confidence_threshold = 0.8
check_private_methods = true
check_unused_imports = true
check_unused_variables = true

[detectors.security]
enabled = true
check_sql_injection = true
check_xss = true
check_hardcoded_secrets = true
check_insecure_crypto = true

[ai]
# AI integration settings
enabled = false
default_provider = "ollama"
timeout_seconds = 30
max_tokens = 4000
temperature = 0.1
enable_explanations = true
enable_suggestions = true
enable_refactoring_advice = true

# Provider configurations
[ai.providers.ollama]
api_url = "http://localhost:11434"
model = "deepseek-coder:6.7b"
timeout = 30
max_retries = 3
retry_delay = 1

[ai.providers.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
organization = "${OPENAI_ORG_ID}"
max_tokens = 4000
temperature = 0.1

[ai.providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-opus"
max_tokens = 4000

[ai.providers.gemini]
api_key = "${GEMINI_API_KEY}"
model = "gemini-pro"
max_tokens = 4000

[output]
# Default output settings
default_format = "html"
include_timing = true
include_diagrams = true
include_metrics = true
verbose = false
colored_output = true

# Format-specific settings
[output.html]
theme = "auto"  # "light", "dark", "auto"
standalone = true
include_toc = true
syntax_highlighting = true
responsive_design = true

[output.json]
pretty_print = true
compact = false
include_metadata = true

[output.markdown]
style = "github"  # "github", "commonmark"
include_toc = true
include_diagrams = true
code_blocks = true

[plugins]
# Plugin system configuration
enabled = true
plugin_directory = "~/.uveddi/plugins/"
auto_load = true
sandbox_enabled = true
max_memory_mb = 128
max_execution_seconds = 60
max_file_operations = 100
max_network_requests = 10

# Plugin permissions
[plugins.permissions]
file_read = ["src/**", "*.toml", "*.json"]
file_write = ["target/cache/**"]
network_access = ["api.example.com"]
system_commands = []
environment_vars = ["CI", "BUILD_*"]

# Specific plugin configurations
[plugins.custom_detector]
enabled = true
config_file = "./custom-detector-config.toml"
debug = false

[logging]
# Logging configuration
level = "info"  # "trace", "debug", "info", "warn", "error"
format = "pretty"  # "pretty", "json", "compact"
output = "stderr"  # "stderr", "stdout", "file"
file_path = "/var/log/uveddi.log"
max_file_size = "10MB"
max_files = 5
include_timestamps = true
include_modules = true
colored = true

# Log filters
[logging.filters]
"uveddi::parser" = "debug"
"uveddi::ai" = "warn"
"uveddi::plugins" = "info"

[performance]
# Performance optimization
cpu_affinity = []  # Empty = no affinity
priority = "normal"  # "low", "normal", "high"
preload_grammars = true
cache_ast = true
cache_analysis_results = true
cache_ttl_seconds = 3600

# Resource limits
max_open_files = 1000
max_memory_per_thread_mb = 512
gc_interval_seconds = 300

[database]
# Analysis result storage (if enabled)
enabled = false
type = "sqlite"  # "sqlite", "postgres", "memory"
path = "~/.cache/uveddi/analysis.db"
connection_pool_size = 10
timeout_seconds = 30

[database.postgres]
host = "localhost"
port = 5432
database = "uveddi"
username = "uveddi_user"
password = "${DATABASE_PASSWORD}"
ssl_mode = "require"

[network]
# Network configuration
timeout_seconds = 30
max_redirects = 3
user_agent = "Uveddi/1.0"
proxy_url = "${HTTP_PROXY}"

[security]
# Security settings
sandbox_plugins = true
validate_file_paths = true
max_file_size = "100MB"
allowed_protocols = ["file", "http", "https"]
blocked_domains = ["malicious.example.com"]

# Profile-specific configurations
[profiles.development]
analysis.timeout_seconds = 60
memory.profile = "small"
ai.enabled = false
logging.level = "debug"
performance.cache_ast = false

[profiles.production]
analysis.timeout_seconds = 3600
memory.profile = "large"
ai.enabled = true
ai.default_provider = "ollama"
logging.level = "warn"
output.include_timing = false

[profiles.ci]
analysis.timeout_seconds = 300
memory.profile = "default"
ai.enabled = false
output.default_format = "json"
logging.level = "error"
performance.cpu_affinity = [0, 1]

[profiles.security_audit]
detectors.security_vulnerabilities = true
detectors.god_object = false
detectors.dead_code = false
ai.enabled = true
ai.enable_security_analysis = true
thresholds.security_confidence = 0.9
```

---

## Environment Variables

### Core Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `UVEDDI_CONFIG_PATH` | Configuration file path | `~/.config/uveddi/config.toml` | `/custom/config.toml` |
| `UVEDDI_PROFILE` | Configuration profile to use | `default` | `production` |
| `UVEDDI_LOG_LEVEL` | Logging level | `info` | `debug` |
| `UVEDDI_NO_COLOR` | Disable colored output | `false` | `true` |
| `UVEDDI_CACHE_DIR` | Cache directory | `~/.cache/uveddi` | `/tmp/uveddi-cache` |

### Memory Management

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_MEMORY_LIMIT_GB` | Memory limit in gigabytes | `8.0` |
| `UVEDDI_MEMORY_PROFILE` | Memory profile | `large` |
| `UVEDDI_DISABLE_MEMORY_OPT` | Disable memory optimization | `true` |
| `UVEDDI_ARENA_SIZE_MB` | Arena allocator size | `128` |
| `UVEDDI_GC_THRESHOLD_MB` | Garbage collection threshold | `1024` |

### Analysis Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_MAX_FILE_SIZE` | Maximum file size to analyze | `5MB` |
| `UVEDDI_TIMEOUT` | Analysis timeout in seconds | `600` |
| `UVEDDI_PARALLEL_JOBS` | Number of parallel analysis jobs | `8` |
| `UVEDDI_LANGUAGES` | Supported languages (comma-separated) | `rust,python,javascript` |

### AI Integration

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_AI_ENABLED` | Enable AI features | `true` |
| `UVEDDI_AI_PROVIDER` | Default AI provider | `ollama` |
| `OLLAMA_API_URL` | Ollama API endpoint | `http://localhost:11434` |
| `OLLAMA_MODEL` | Default Ollama model | `deepseek-coder:6.7b` |
| `OPENAI_API_KEY` | OpenAI API key | `sk-...` |
| `ANTHROPIC_API_KEY` | Anthropic API key | `sk-ant-...` |
| `GEMINI_API_KEY` | Google Gemini API key | `...` |

### Detector Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_GOD_OBJECT_THRESHOLD` | God object method threshold | `100` |
| `UVEDDI_MAX_FUNCTION_LINES` | Maximum function lines | `50` |
| `UVEDDI_DEAD_CODE_CONFIDENCE` | Dead code detection confidence | `0.8` |
| `UVEDDI_COMPLEXITY_THRESHOLD` | Cyclomatic complexity threshold | `15` |

### Plugin System

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_PLUGINS_ENABLED` | Enable plugin system | `true` |
| `UVEDDI_PLUGIN_DIR` | Plugin directory | `~/.uveddi/plugins` |
| `UVEDDI_PLUGIN_SANDBOX` | Enable plugin sandboxing | `true` |
| `UVEDDI_PLUGIN_MEMORY_MB` | Max plugin memory | `128` |

### Output Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_OUTPUT_FORMAT` | Default output format | `html` |
| `UVEDDI_INCLUDE_TIMING` | Include timing information | `true` |
| `UVEDDI_VERBOSE` | Enable verbose output | `true` |
| `UVEDDI_HTML_THEME` | HTML report theme | `dark` |

### Debugging

| Variable | Description | Example |
|----------|-------------|---------|
| `RUST_LOG` | Rust logging configuration | `uveddi=debug` |
| `RUST_BACKTRACE` | Enable stack traces | `1` |
| `UVEDDI_DEBUG_AST` | Debug AST parsing | `true` |
| `UVEDDI_DEBUG_MEMORY` | Debug memory usage | `true` |
| `UVEDDI_TRACE_ANALYSIS` | Trace analysis steps | `true` |

---

## Command Line Overrides

All configuration options can be overridden via command-line flags:

### Analysis Configuration
```bash
uveddi analyze ./src \
  --timeout 600 \
  --max-file-size 5MB \
  --parallel-jobs 8 \
  --languages rust,python
```

### Memory Configuration
```bash
uveddi analyze ./src \
  --memory-profile large \
  --memory-limit-gb 16 \
  --disable-memory-optimization
```

### Detector Configuration
```bash
uveddi analyze ./src \
  --god-object-threshold 150 \
  --max-function-lines 75 \
  --dead-code-confidence 0.9 \
  --complexity-threshold 20
```

### AI Configuration
```bash
uveddi analyze ./src \
  --enable-ai \
  --ai-provider ollama \
  --ollama-model deepseek-coder:6.7b \
  --ai-timeout 60
```

### Output Configuration
```bash
uveddi analyze ./src \
  --output-format html \
  --html-theme dark \
  --include-timing \
  --include-diagrams
```

---

## Memory Management

### Automatic Memory Management (Default)

Uveddi automatically configures memory settings based on system resources:

```toml
[memory]
enable_optimization = true  # Always recommended
profile = "auto"           # Automatically detected
limit_gb = 0.0             # Auto-detected based on available RAM
```

#### Memory Profiles

**Small Profile** (< 4GB system RAM)
- Minimal caching
- Sequential processing
- Basic object pooling

**Default Profile** (4-16GB system RAM)
- Moderate caching
- Parallel processing
- Standard object pooling

**Large Profile** (16GB+ system RAM)
- Aggressive caching
- Maximum parallelism
- Large object pools

### Manual Memory Configuration

```toml
[memory]
enable_optimization = true
profile = "large"
limit_gb = 16.0
arena_size_mb = 256
pool_initial_capacity = 5000
gc_threshold_mb = 1024

# Advanced settings
zero_copy_caching = true
object_pooling = true
lazy_loading = true
memory_mapping = true
```

### Memory Monitoring

```bash
# Enable memory debugging
export UVEDDI_DEBUG_MEMORY=true
export RUST_LOG=uveddi::memory=debug

# Monitor memory usage during analysis
uveddi analyze ./large-codebase --include-timing --verbose
```

---

## Analysis Settings

### Language Configuration

```toml
[analysis]
languages = ["rust", "python", "javascript", "typescript"]

# Language-specific settings
[analysis.rust]
edition = "2021"
features = ["serde", "tokio"]
check_unsafe = true

[analysis.python]
version = "3.9"
check_type_hints = true
virtual_env = ".venv"

[analysis.javascript]
ecma_version = "2022"
jsx = true
check_flow = false

[analysis.typescript]
strict = true
target = "es2022"
check_js = false
```

### File Processing

```toml
[analysis]
max_file_size = "5MB"
max_depth = 10
follow_symlinks = false
include_tests = false

exclude_patterns = [
    "target/",
    "node_modules/",
    ".git/",
    "*.min.js",
    "*.generated.*",
    "test_data/",
    "docs/",
    "*.lock"
]

include_patterns = [
    "src/**/*.rs",
    "lib/**/*.py",
    "app/**/*.js",
    "components/**/*.tsx"
]

# File type detection
[analysis.file_types]
".rs" = "rust"
".py" = "python" 
".js" = "javascript"
".ts" = "typescript"
".jsx" = "javascript"
".tsx" = "typescript"
```

### Parser Configuration

```toml
[analysis.parser]
timeout_seconds = 30
max_errors = 10
strict_mode = false
tree_sitter_debug = false

# Grammar-specific settings
[analysis.parser.rust]
highlight_queries = true
locals_queries = true

[analysis.parser.python]
python_version = 3
```

---

## AI Provider Configuration

### Ollama (Local AI)

```toml
[ai.providers.ollama]
api_url = "http://localhost:11434"
model = "deepseek-coder:6.7b"
timeout = 30
max_retries = 3
retry_delay = 1
context_window = 8192
temperature = 0.1
top_p = 0.9

# Model-specific settings
[ai.providers.ollama.models."deepseek-coder:6.7b"]
context_window = 8192
max_tokens = 2048
temperature = 0.1

[ai.providers.ollama.models."codellama:7b-code"]
context_window = 4096
max_tokens = 1024
temperature = 0.2
```

### OpenAI Configuration

```toml
[ai.providers.openai]
api_key = "${OPENAI_API_KEY}"
organization = "${OPENAI_ORG_ID}"
model = "gpt-4"
max_tokens = 4000
temperature = 0.1
top_p = 1.0
frequency_penalty = 0.0
presence_penalty = 0.0
timeout = 30
max_retries = 3

# Model-specific overrides
[ai.providers.openai.models.gpt-4]
max_tokens = 8000
temperature = 0.05

[ai.providers.openai.models."gpt-3.5-turbo"]
max_tokens = 4000
temperature = 0.1
```

### Anthropic Configuration

```toml
[ai.providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-opus"
max_tokens = 4000
temperature = 0.1
timeout = 30
max_retries = 3

# Model configurations
[ai.providers.anthropic.models."claude-3-opus"]
max_tokens = 8000
temperature = 0.05

[ai.providers.anthropic.models."claude-3-sonnet"]
max_tokens = 4000
temperature = 0.1
```

### AI Feature Configuration

```toml
[ai.features]
enable_explanations = true
enable_suggestions = true
enable_refactoring_advice = true
enable_security_analysis = true
enable_performance_tips = true

# Context configuration
max_context_files = 5
max_context_lines = 1000
include_dependencies = false
include_test_files = false
```

---

## Plugin Configuration

### Plugin System Settings

```toml
[plugins]
enabled = true
plugin_directory = "~/.uveddi/plugins/"
auto_load = true
sandbox_enabled = true
max_memory_mb = 128
max_execution_seconds = 60
max_file_operations = 100
max_network_requests = 10
debug_mode = false
```

### Plugin Permissions

```toml
[plugins.permissions]
# File system access
file_read = ["src/**", "*.toml", "*.json", "README*"]
file_write = ["target/cache/**", "/tmp/uveddi-*"]
temp_file_create = true

# Network access
network_domains = ["api.example.com", "registry.npmjs.org"]
http_methods = ["GET", "POST"]

# System access
environment_vars = ["CI", "BUILD_*", "NODE_ENV"]
system_commands = ["git", "rustc", "python"]

# Resource limits
max_memory_mb = 64
max_execution_seconds = 30
max_file_size = "1MB"
```

### Individual Plugin Configuration

```toml
[plugins.custom_detector]
enabled = true
path = "./plugins/custom-detector.wasm"
config_file = "./custom-detector-config.toml"
priority = 100
debug = false

[plugins.custom_detector.settings]
threshold = 0.8
check_patterns = ["TODO", "FIXME", "XXX"]
severity = "warning"

[plugins.security_scanner]
enabled = true
path = "~/.uveddi/plugins/security-scanner.wasm"
auto_update = true
```

---

## Output Configuration

### Format-Specific Settings

#### HTML Output
```toml
[output.html]
theme = "auto"              # "light", "dark", "auto"
standalone = true           # Include CSS/JS inline
include_toc = true         # Table of contents
syntax_highlighting = true  # Code syntax highlighting
responsive_design = true    # Mobile-friendly
include_search = true      # JavaScript search
include_navigation = true   # Navigation sidebar
custom_css = "custom.css"  # Additional CSS file
```

#### JSON Output
```toml
[output.json]
pretty_print = true        # Formatted JSON
compact = false           # Minified output
include_metadata = true   # Include analysis metadata
include_timing = true     # Include performance data
schema_version = "1.0"    # JSON schema version
```

#### Markdown Output
```toml
[output.markdown]
style = "github"          # "github", "commonmark"
include_toc = true       # Table of contents
include_diagrams = true  # Mermaid diagrams
code_blocks = true       # Syntax highlighting
front_matter = true      # YAML front matter
line_breaks = "soft"     # "soft", "hard"
```

#### CSV Output
```toml
[output.csv]
separator = ","          # Field separator
quote_char = "\""        # Quote character
header_row = true        # Include headers
escape_quotes = true     # Escape quotes in data
```

### Report Customization

```toml
[output.report]
title = "Architectural Analysis Report"
author = "Uveddi Analysis Engine"
company = "Your Company"
logo_path = "./logo.png"
custom_footer = "Generated by Uveddi v1.0"
```

---

## Performance Tuning

### CPU and Threading

```toml
[performance]
parallel_jobs = 0          # 0 = auto-detect CPU cores
cpu_affinity = []          # Pin to specific cores
priority = "normal"        # "low", "normal", "high"
thread_stack_size = "2MB"  # Stack size per thread
```

### Caching Configuration

```toml
[performance.caching]
cache_ast = true
cache_analysis_results = true
cache_ai_responses = true
cache_ttl_seconds = 3600
cache_max_size_mb = 512
cache_compression = true

# Cache locations
ast_cache_dir = "~/.cache/uveddi/ast/"
results_cache_dir = "~/.cache/uveddi/results/"
ai_cache_dir = "~/.cache/uveddi/ai/"
```

### Resource Limits

```toml
[performance.limits]
max_open_files = 1000
max_memory_per_thread_mb = 512
max_heap_size_mb = 2048
gc_interval_seconds = 300
timeout_per_file_seconds = 30
```

### Optimization Flags

```toml
[performance.optimizations]
preload_grammars = true    # Load parsers at startup
lazy_loading = true        # Load files on-demand
zero_copy = true          # Avoid unnecessary copying
vectorized_operations = true # Use SIMD when available
```

---

## Security Settings

### Sandbox Configuration

```toml
[security.sandbox]
enable_for_plugins = true
restrict_file_access = true
block_network_access = false
limit_system_calls = true
enforce_memory_limits = true
```

### File Access Control

```toml
[security.file_access]
allowed_paths = [
    "src/",
    "lib/",
    "tests/",
    "*.toml",
    "*.json",
    "README*"
]

blocked_paths = [
    "/etc/",
    "/usr/",
    "~/.ssh/",
    "*.key",
    "*.pem"
]

max_file_size = "100MB"
validate_file_paths = true
follow_symlinks = false
```

### Network Security

```toml
[security.network]
allowed_domains = [
    "api.ollama.ai",
    "api.openai.com",
    "api.anthropic.com"
]

blocked_domains = [
    "malicious.example.com"
]

allowed_protocols = ["https", "http"]
validate_certificates = true
timeout_seconds = 30
```

---

## Examples

### Minimal Configuration

```toml
# ~/.config/uveddi/config.toml
[analysis]
languages = ["rust", "python"]

[ai]
enabled = true
default_provider = "ollama"

[ai.providers.ollama]
model = "deepseek-coder:6.7b"
```

### Development Configuration

```toml
[analysis]
timeout_seconds = 60
include_tests = true

[memory]
profile = "small"
limit_gb = 4.0

[ai]
enabled = false

[logging]
level = "debug"
format = "pretty"

[output]
default_format = "text"
include_timing = true
```

### Production Configuration

```toml
[analysis]
timeout_seconds = 3600
exclude_patterns = [
    "target/",
    "node_modules/", 
    "test*/",
    "*.min.*"
]

[memory]
profile = "large"
limit_gb = 32.0

[ai]
enabled = true
default_provider = "anthropic"

[ai.providers.anthropic]
model = "claude-3-opus"
max_tokens = 8000

[logging]
level = "warn"
output = "file"
file_path = "/var/log/uveddi.log"

[output]
default_format = "json"
include_timing = false

[plugins]
auto_load = true
sandbox_enabled = true
```

### CI/CD Configuration

```toml
[analysis]
timeout_seconds = 300
fail_on_critical = true

[memory]
profile = "default"
limit_gb = 8.0

[ai]
enabled = false

[logging]
level = "error"
format = "json"

[output]
default_format = "json"
include_timing = false
verbose = false

[performance]
parallel_jobs = 4
priority = "high"
```

### Large Codebase Configuration

```toml
[analysis]
timeout_seconds = 7200
max_file_size = "10MB"
exclude_patterns = [
    "vendor/",
    "third_party/",
    "generated/",
    "*.min.*",
    "*.bundle.*"
]

[memory]
profile = "large"
limit_gb = 64.0
arena_size_mb = 512
gc_threshold_mb = 4096

[performance]
parallel_jobs = 16
preload_grammars = true
cache_ast = true
vectorized_operations = true

[performance.caching]
cache_ttl_seconds = 7200
cache_max_size_mb = 2048
```

---

## Validation

### Configuration Validation

```bash
# Validate current configuration
uveddi validate-config

# Validate specific config file
uveddi validate-config --config ./custom-config.toml

# Strict validation (fail on warnings)
uveddi validate-config --strict
```

### Common Validation Errors

#### Invalid Memory Configuration
```toml
[memory]
limit_gb = -1  # ❌ Cannot be negative
profile = "invalid"  # ❌ Must be "small", "default", "large", or "auto"
```

#### Invalid Threshold Values
```toml
[thresholds]
dead_code_confidence = 1.5  # ❌ Must be between 0.0 and 1.0
god_object_threshold = 0    # ❌ Must be positive
```

#### Invalid File Patterns
```toml
[analysis]
exclude_patterns = [
    "**/[invalid"  # ❌ Invalid glob pattern
]
```

### Configuration Schema Validation

Uveddi validates configuration against a JSON schema:

```bash
# Generate schema
uveddi config schema > config-schema.json

# Validate against schema
uveddi validate-config --schema config-schema.json
```

### Environment Variable Validation

```bash
# Check environment variables
env | grep UVEDDI

# Validate environment configuration
uveddi config validate-env
```

---

## Configuration Best Practices

### 1. Use Profiles for Different Environments
```toml
[profiles.dev]
ai.enabled = false
logging.level = "debug"

[profiles.prod]
ai.enabled = true
logging.level = "warn"
```

### 2. Leverage Environment Variables for Secrets
```toml
[ai.providers.openai]
api_key = "${OPENAI_API_KEY}"  # ✅ Use environment variable
# api_key = "sk-..."            # ❌ Never hardcode keys
```

### 3. Optimize for Your Use Case
```toml
# For CI/CD: Fast and focused
[analysis]
timeout_seconds = 300
detectors = { security = true, god_object = true }

# For development: Comprehensive and educational
[analysis]
ai.enabled = true
ai.enable_explanations = true
```

### 4. Use Incremental Configuration
```toml
# Base configuration in global config
[analysis]
languages = ["rust", "python"]

# Project-specific overrides in project config
[thresholds]
god_object_threshold = 200  # Larger threshold for legacy project
```

### 5. Monitor Resource Usage
```bash
# Enable resource monitoring
export UVEDDI_DEBUG_MEMORY=true
export RUST_LOG=uveddi::performance=info

# Review logs for optimization opportunities
```

---

This configuration reference covers all available options for customizing Uveddi's behavior. For additional examples and advanced configuration patterns, see the [User Guide](../user-guide/) and [Examples](../examples/) sections.