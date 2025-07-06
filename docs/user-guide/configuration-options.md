# Configuration Options

## Core Configuration

### Analysis Settings
```toml
[analysis]
max_file_size = "2MB"  # Max file size to analyze
parallel_jobs = 4      # Concurrent analysis jobs
timeout = 300          # Analysis timeout in seconds
```

### Database Settings
```toml
[database]
host = "localhost"
port = 5432
database = "uveddi"
user = "uveddi_user"
```

## AI Providers

### OpenAI
```toml
[ai.providers.openai]
model = "gpt-4"
api_key = "your-api-key"
temperature = 0.7
```

### Anthropic
```toml
[ai.providers.anthropic]
model = "claude-3-opus"
api_key = "your-api-key"
max_tokens = 2000
```

### Ollama (Local)
```toml
[ai.providers.ollama]
url = "http://localhost:11434"
model = "deepseek-coder:6.7b"
```

## Rule Customization

Enable/disable specific rules:
```toml
[rules.cyclic_dependencies]
enabled = true
severity = "critical"

[rules.god_objects]
enabled = true
max_methods = 15
```

## Plugin Configuration

Load custom plugins:
```toml
[plugins]
paths = [
    "~/.uveddi/plugins/custom_detector.wasm",
    "./local_plugins/special_checks.wasm"
]
```

## Environment Overrides

Override any setting with environment variables:
```bash
export UVEDDI_ANALYSIS_PARALLEL_JOBS=8
export UVEDDI_AI_PROVIDER=ollama
