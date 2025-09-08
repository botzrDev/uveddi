# Uveddi Configuration Guide

## Configuration Methods

Uveddi is pre-configured with sensible defaults for optimal performance:

- **Memory optimization**: Enabled by default with automatic system detection
- **Tree-sitter parsing**: Enabled by default for all supported languages
- **Smart memory profiles**: Automatically selected based on your system (small/default/large)
- **Memory limits**: Automatically set based on available RAM

### Memory Optimization (New Default Behavior)
Starting with the latest version, memory optimization is enabled by default:
- **Object pooling** for reduced allocations
- **Arena allocation** for temporary objects  
- **Zero-copy AST caching** for better performance
- **Automatic memory limits** based on system RAM

To disable memory optimization (not recommended):
```bash
uveddi analyze --disable-memory-optimization
```

### Advanced Configuration Options
If you need to disable tree-sitter for compatibility or debugging, use the following environment variable or cargo feature flag:

- Environment: `UVEDDI_DISABLE_TREE_SITTER=1`
- Cargo: `--no-default-features --features="..."`

Uveddi can be configured through multiple methods, with the following precedence:
1. Command-line arguments
2. Environment variables
3. Configuration file
4. Default values

## Configuration File

Default location: `~/.config/uveddi/config.toml`

Example configuration:
```toml
[database]
host = "localhost"
port = 5432
database = "uveddi"
user = "uveddi_user"
password = "your_password"

[ai]
default_provider = "openai"
timeout = 30

[ai.providers.openai]
model = "gpt-4"
api_key = "your_api_key"

[analysis]
max_file_size = 1048576  # 1MB
parallel_jobs = 4

[memory]
# Memory optimization is enabled by default
# These settings override automatic detection
disable_optimization = false  # Set to true to disable (not recommended)
limit_gb = 4.0               # Manual memory limit (auto-detected by default)
profile = "default"          # "small", "default", or "large" (auto-detected by default)
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_DB_HOST` | Database host | `localhost` |
| `UVEDDI_DB_PORT` | Database port | `5432` |
| `UVEDDI_AI_PROVIDER` | Default AI provider | `openai` |
| `OPENAI_API_KEY` | OpenAI API key | `sk-...` |
| `ANTHROPIC_API_KEY` | Anthropic API key | `sk-ant-...` |

## Command-line Options

Common options:
```bash
uveddi analyze --config /path/to/config.toml \
    --ai-provider openai \
    --max-file-size 2MB \
    ./project-path
```

## Verifying Configuration

Check effective configuration:
```bash
uveddi config show
```

## Configuration Profiles

Create different profiles for different environments:
```toml
[profile.dev]
database.host = "localhost"

[profile.prod]
database.host = "db.prod.internal"
ai.timeout = 60
```

Activate a profile:
```bash
uveddi --profile prod analyze ./project
```

## Advanced Configuration Options

### Analysis Settings
```toml
[analysis]
max_file_size = "2MB"  # Max file size to analyze
parallel_jobs = 4      # Concurrent analysis jobs
timeout = 300          # Analysis timeout in seconds
```

### AI Provider Details

#### Anthropic
```toml
[ai.providers.anthropic]
model = "claude-3-opus"
api_key = "your-api-key"
max_tokens = 2000
```

#### Ollama (Local)
```toml
[ai.providers.ollama]
url = "http://localhost:11434"
model = "deepseek-coder:6.7b"
```

### Rule Customization

Enable/disable specific rules:
```toml
[rules.cyclic_dependencies]
enabled = true
severity = "critical"

[rules.god_objects]
enabled = true
max_methods = 15
```

### Plugin Configuration

Load custom plugins:
```toml
[plugins]
paths = [
    "~/.uveddi/plugins/custom_detector.wasm",
    "./local_plugins/special_checks.wasm"
]
```

### Environment Variable Overrides

Override any setting with environment variables:
```bash
export UVEDDI_ANALYSIS_PARALLEL_JOBS=8
export UVEDDI_AI_PROVIDER=ollama
```
