# Complete CLI Commands Reference

## Overview

Uveddi provides a comprehensive command-line interface with 9 main commands and 47+ configuration options for code analysis, service orchestration, and plugin management.

## Command Structure

```bash
uveddi [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

## Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--help`, `-h` | Display help information | - |
| `--version`, `-V` | Display version information | - |
| `--verbose`, `-v` | Increase logging verbosity (can be used multiple times) | - |
| `--quiet`, `-q` | Suppress non-error output | - |
| `--config <FILE>` | Path to configuration file | `uveddi.toml` |

## Commands

### 1. `analyze` - Perform Code Analysis

Analyzes a codebase for architectural issues, anti-patterns, and quality metrics.

#### Syntax
```bash
uveddi analyze <PATH> [OPTIONS]
```

#### Arguments
- `<PATH>` - Path to the directory or file to analyze (required)

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output <FILE>` | Save report to file | - |
| `--output-format <FORMAT>` | Output format: `markdown`, `json`, `html`, `csv` | `markdown` |
| `--enable-ai` | Enable AI-powered insights | `false` |
| `--ollama-api-url <URL>` | Ollama API endpoint | `http://localhost:11434` |
| `--ollama-model <MODEL>` | Ollama model name | `deepseek-coder:6.7b` |
| `--timeout-seconds <SECONDS>` | Analysis timeout (0 = no timeout) | `300` |
| `--parallel-jobs <COUNT>` | Number of parallel analysis jobs | CPU count |
| `--ignore-patterns <PATTERNS>` | Comma-separated patterns to ignore | - |
| `--include-tests` | Include test files in analysis | `false` |
| `--max-file-size <BYTES>` | Maximum file size to analyze | `10485760` |

#### Detector Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `--dead-code-confidence <FLOAT>` | Dead code detection confidence (0.0-1.0) | `0.8` |
| `--dead-code-library-mode` | Enable library mode for dead code detection | `false` |
| `--dead-code-ignore-patterns <PATTERNS>` | Patterns to ignore in dead code detection | - |
| `--dead-code-keep-alive <SYMBOLS>` | Symbols to always keep alive | - |
| `--large-classes-max-loc <LINES>` | Maximum lines for large class detection | `500` |
| `--large-classes-max-methods <COUNT>` | Maximum methods threshold | `20` |
| `--large-classes-max-fields <COUNT>` | Maximum fields threshold | `15` |
| `--large-classes-max-complexity <SCORE>` | Maximum cyclomatic complexity | `10` |
| `--large-classes-max-lcom <SCORE>` | Maximum LCOM score | `0.8` |
| `--large-classes-min-severity <SCORE>` | Minimum severity for reporting | `5` |

#### Memory Optimization

| Option | Description | Default |
|--------|-------------|---------|
| `--enable-memory-optimization` | Enable automatic memory optimization | `true` |
| `--disable-memory-optimization` | Disable memory optimization | - |
| `--memory-limit-gb <GB>` | Manual memory limit in gigabytes | Auto-detected |
| `--memory-profile <PROFILE>` | Memory profile: `small`, `default`, `large` | Auto-detected |
| `--cache-strategy <STRATEGY>` | Cache strategy: `aggressive`, `balanced`, `minimal` | `balanced` |

#### Examples

```bash
# Basic analysis
uveddi analyze ./src

# With AI insights and HTML output
uveddi analyze ./src --enable-ai --output-format html --output report.html

# Memory-optimized for large codebase
uveddi analyze ./project --memory-limit-gb 8 --memory-profile large

# With custom thresholds
uveddi analyze ./src \
  --dead-code-confidence 0.9 \
  --large-classes-max-loc 300 \
  --large-classes-max-methods 15

# Parallel analysis with timeout
uveddi analyze ./src --parallel-jobs 8 --timeout-seconds 600
```

### 2. `serve` - Start Web Dashboard Services

Launches the web dashboard with integrated services for interactive analysis.

#### Syntax
```bash
uveddi serve [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--port`, `-p` | API server and dashboard port | `8080` |
| `--rendering-port` | Rendering service port | `3333` |
| `--frontend-port` | Frontend dev server port (dev mode) | `3000` |
| `--database-path` | Path to SQLite database | `./uveddi.db` |
| `--development` | Enable development mode | `false` |
| `--frontend-assets` | Path to frontend assets | Built-in |
| `--bind <ADDRESS>` | Bind address (use 0.0.0.0 for external) | `127.0.0.1` |
| `--enable-cors` | Enable CORS headers | `false` |
| `--cors-origins <ORIGINS>` | Allowed CORS origins | `*` |
| `--max-connections <COUNT>` | Maximum concurrent connections | `100` |
| `--request-timeout <SECONDS>` | Request timeout | `30` |

#### Examples

```bash
# Start with default settings
uveddi serve

# Custom ports configuration
uveddi serve --port 8888 --rendering-port 3333

# Development mode with hot-reload
uveddi serve --development --frontend-port 3000

# External access (careful!)
uveddi serve --bind 0.0.0.0 --enable-cors
```

### 3. `tui` - Terminal User Interface

Launches an interactive terminal-based interface for analysis exploration.

#### Syntax
```bash
uveddi tui [PATH] [OPTIONS]
```

#### Arguments
- `[PATH]` - Path to analyze (default: current directory)

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--skip-analysis` | Skip initial analysis, load existing results | `false` |
| `--load-results <FILE>` | Load analysis results from file | - |
| `--debug` | Enable debug mode | `false` |
| `--force` | Force re-analysis even if cache exists | `false` |
| `--theme <THEME>` | Color theme: `dark`, `light`, `auto` | `auto` |
| `--refresh-rate <MS>` | UI refresh rate in milliseconds | `100` |

#### Examples

```bash
# Launch TUI for current directory
uveddi tui

# Load existing results
uveddi tui --load-results analysis.json

# Force re-analysis
uveddi tui ./src --force
```

### 4. `config` - Configuration Management

Manages Uveddi configuration files and settings.

#### Syntax
```bash
uveddi config <SUBCOMMAND> [OPTIONS]
```

#### Subcommands

##### `config show`
Display current configuration.

```bash
uveddi config show [--file <PATH>]
```

##### `config set`
Set a configuration value.

```bash
uveddi config set <KEY> <VALUE> [--file <PATH>]
```

##### `config validate`
Validate configuration file.

```bash
uveddi config validate [--file <PATH>]
```

#### Examples

```bash
# Show current configuration
uveddi config show

# Set Ollama model
uveddi config set ollama.model "codellama:7b"

# Validate custom config file
uveddi config validate --file custom.toml
```

### 5. `plugin` - Plugin Management

Manages WebAssembly plugins for extended functionality with production-ready WASM runtime.

#### Syntax
```bash
uveddi plugin <SUBCOMMAND> [OPTIONS]
```

#### Subcommands

##### `plugin list`
List installed plugins with status and metadata.

```bash
uveddi plugin list [OPTIONS]
```

**Options:**
- `--verbose`, `-v` - Show detailed plugin information including permissions and resource usage
- `--status <STATUS>` - Filter by status: `enabled`, `disabled`, `error`
- `--format <FORMAT>` - Output format: `table`, `json`, `yaml` (default: `table`)

##### `plugin install`
Install a plugin from WebAssembly binary with automatic validation.

```bash
uveddi plugin install <PLUGIN_FILE> [OPTIONS]
```

**Arguments:**
- `<PLUGIN_FILE>` - Path to `.wasm` plugin binary file

**Options:**
- `--manifest <FILE>` - Path to plugin manifest file (default: `plugin.toml` in same directory)
- `--force` - Force installation even if plugin exists
- `--enable` - Enable plugin immediately after installation
- `--validate-security` - Run security validation on plugin binary
- `--dry-run` - Validate plugin without installing

##### `plugin remove`
Remove an installed plugin and cleanup resources.

```bash
uveddi plugin remove <PLUGIN_ID> [OPTIONS]
```

**Arguments:**
- `<PLUGIN_ID>` - Plugin identifier from `plugin list`

**Options:**
- `--force` - Force removal even if plugin is in use
- `--keep-data` - Preserve plugin data and configuration

##### `plugin info`
Display comprehensive plugin information and statistics.

```bash
uveddi plugin info <PLUGIN_ID> [OPTIONS]
```

**Arguments:**
- `<PLUGIN_ID>` - Plugin identifier from `plugin list`

**Options:**
- `--show-permissions` - Display plugin security permissions
- `--show-stats` - Show runtime execution statistics
- `--show-manifest` - Display plugin manifest content

##### `plugin test`
Test plugin functionality and validate operation.

```bash
uveddi plugin test <PLUGIN_ID> [OPTIONS]
```

**Arguments:**
- `<PLUGIN_ID>` - Plugin identifier to test

**Options:**
- `--test-file <PATH>` - Use specific file for testing
- `--timeout <SECONDS>` - Test execution timeout (default: 30)
- `--verbose` - Show detailed test output

##### `plugin update`
Update an installed plugin to newer version.

```bash
uveddi plugin update <PLUGIN_ID> [OPTIONS]
```

**Arguments:**
- `<PLUGIN_ID>` - Plugin identifier to update

**Options:**
- `--binary <FILE>` - Path to new plugin binary
- `--check-only` - Check for updates without installing
- `--backup` - Create backup before updating

#### Security and Permissions

Plugins run in a secure WebAssembly sandbox with capability-based permissions:

**Available Permissions:**
- `ConfigRead` - Read project configuration
- `TempFileCreate` - Create temporary files
- `Logging` - Write to log output
- `FileRead` - Read specific files (path-restricted)
- `NetworkConnect` - Network access (URL-restricted)

**Resource Limits:**
- Memory limit: 32MB (configurable)
- Execution timeout: 30 seconds (configurable)
- Fuel limit: 2,000,000 operations (prevents infinite loops)

#### Plugin Integration

Plugins integrate seamlessly with Uveddi's analysis pipeline:

```bash
# Analyze with all enabled plugins
uveddi analyze ./src

# Analyze with specific plugins only
uveddi analyze ./src --plugins security-scanner,performance-analyzer

# Disable all plugins for analysis
uveddi analyze ./src --no-plugins
```

#### Examples

```bash
# List all plugins with details
uveddi plugin list --verbose

# Install plugin with security validation
uveddi plugin install security-analyzer.wasm --validate-security --enable

# View plugin information and statistics
uveddi plugin info security-analyzer --show-stats --show-permissions

# Test plugin functionality
uveddi plugin test security-analyzer --test-file src/main.rs --verbose

# Update plugin to newer version
uveddi plugin update security-analyzer --binary security-analyzer-v2.wasm

# Remove plugin and cleanup
uveddi plugin remove security-analyzer --force

# Install from URL (if supported)
uveddi plugin install https://plugins.uveddi.dev/security-analyzer.wasm

# Dry run installation
uveddi plugin install new-plugin.wasm --dry-run --validate-security
```

#### Plugin Development

Create new plugins using the built-in scaffolding:

```bash
# Generate plugin template
scripts/generate-plugin.py my-analyzer

# Build plugin
cd plugins/my-analyzer && make build

# Test during development
uveddi plugin test my-analyzer --verbose
```

### 6. `ci` - CI/CD Integration

Commands optimized for continuous integration pipelines.

#### Syntax
```bash
uveddi ci <SUBCOMMAND> [OPTIONS]
```

#### Subcommands

##### `ci check`
Run analysis with quality gates.

```bash
uveddi ci check <PATH> [OPTIONS]
```

Options:
- `--fail-on-issues` - Exit with error if issues found
- `--max-issues <COUNT>` - Maximum allowed issues
- `--max-complexity <SCORE>` - Maximum allowed complexity
- `--min-coverage <PERCENT>` - Minimum required coverage
- `--output-format <FORMAT>` - CI-friendly output format
- `--junit-xml <FILE>` - Generate JUnit XML report
- `--github-annotations` - Generate GitHub annotations

#### Examples

```bash
# Basic CI check
uveddi ci check ./src --fail-on-issues

# With quality gates
uveddi ci check ./src \
  --max-issues 10 \
  --max-complexity 15 \
  --junit-xml report.xml

# GitHub Actions integration
uveddi ci check ./src --github-annotations
```

### 7. `ui` - UI Operations

Manages interactive reporting and UI-related operations.

#### Syntax
```bash
uveddi ui <SUBCOMMAND> [OPTIONS]
```

#### Subcommands

##### `ui serve`
Start the interactive reporting server.

```bash
uveddi ui serve [OPTIONS]
```

##### `ui open`
Generate and open a report in browser.

```bash
uveddi ui open <ANALYSIS_ID> [--format <FORMAT>]
```

##### `ui export`
Export report as portable bundle.

```bash
uveddi ui export <ANALYSIS_ID> <OUTPUT_FILE>
```

#### Examples

```bash
# Start UI server
uveddi ui serve --port 8080

# Open specific analysis
uveddi ui open abc123 --format html

# Export report bundle
uveddi ui export abc123 report-bundle.zip
```

### 8. `migrate` - Database Migration

Manages database schema migrations.

#### Syntax
```bash
uveddi migrate [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--from <VERSION>` | Source version | Auto-detected |
| `--to <VERSION>` | Target version | Latest |
| `--database <PATH>` | Database path | `./uveddi.db` |
| `--backup` | Create backup before migration | `true` |
| `--dry-run` | Preview migration without applying | `false` |

#### Examples

```bash
# Migrate to latest version
uveddi migrate

# Specific version migration
uveddi migrate --from v0.8 --to v0.9

# Dry run
uveddi migrate --dry-run
```

### 9. `benchmark` - Performance Testing

Runs performance benchmarks on analysis operations.

#### Syntax
```bash
uveddi benchmark [PATH] [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--iterations <COUNT>` | Number of benchmark iterations | `10` |
| `--warmup <COUNT>` | Warmup iterations | `3` |
| `--profile` | Generate performance profile | `false` |
| `--compare <BASELINE>` | Compare with baseline results | - |
| `--save-baseline <FILE>` | Save results as baseline | - |

#### Examples

```bash
# Basic benchmark
uveddi benchmark ./src

# With profiling
uveddi benchmark ./src --iterations 100 --profile

# Compare with baseline
uveddi benchmark ./src --compare baseline.json
```

## Environment Variables

Uveddi respects the following environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `UVEDDI_CONFIG` | Configuration file path | `uveddi.toml` |
| `UVEDDI_HOME` | Uveddi home directory | `~/.uveddi` |
| `UVEDDI_CACHE_DIR` | Cache directory | `~/.uveddi/cache` |
| `UVEDDI_PLUGIN_DIR` | Plugin directory | `~/.uveddi/plugins` |
| `OLLAMA_API_URL` | Ollama API endpoint | `http://localhost:11434` |
| `OLLAMA_MODEL` | Default Ollama model | `deepseek-coder:6.7b` |
| `RUST_LOG` | Logging level | `info` |
| `RUST_BACKTRACE` | Show backtraces on error | `0` |
| `NO_COLOR` | Disable colored output | - |
| `FORCE_COLOR` | Force colored output | - |

## Configuration File

Uveddi can be configured via `uveddi.toml`:

```toml
[analysis]
languages = ["rust", "python", "javascript", "typescript"]
max_depth = 10
timeout_seconds = 300
parallel_jobs = 8

[thresholds]
god_object_threshold = 100
max_function_lines = 50
max_complexity = 10
min_test_coverage = 80

[ai]
provider = "ollama"
model = "deepseek-coder:6.7b"
api_url = "http://localhost:11434"
timeout = 30

[output]
format = "html"
include_timing = true
include_metrics = true
syntax_highlighting = true

[memory]
optimization_enabled = true
limit_gb = 8
profile = "default"
cache_strategy = "balanced"

[server]
port = 8080
rendering_port = 3333
bind_address = "127.0.0.1"
max_connections = 100

[plugins]
enabled = true
directory = "~/.uveddi/plugins"
auto_reload = true
sandbox_memory_mb = 100
```

## Exit Codes

Uveddi uses standard exit codes:

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Path not found |
| 4 | Analysis timeout |
| 5 | Memory limit exceeded |
| 10 | CI quality gate failed |
| 11 | Plugin error |
| 12 | Database error |
| 13 | Network error |
| 127 | Command not found |

## Compatibility

### Supported Platforms
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x64, WSL2)

### Language Support
- Rust (all versions)
- Python (2.7+, 3.x)
- JavaScript (ES5+)
- TypeScript (all versions)

## Getting Help

```bash
# General help
uveddi --help

# Command-specific help
uveddi analyze --help
uveddi serve --help

# Version information
uveddi --version

# Debug information
RUST_LOG=debug uveddi analyze ./src
```

## See Also

- [Web Dashboard Guide](../03-user-guide/web-dashboard.md)
- [Configuration Guide](../03-user-guide/configuration-options.md)
- [API Reference](../08-api/rest-api-reference.md)
- [Plugin Development](../04-development/plugin-development.md)