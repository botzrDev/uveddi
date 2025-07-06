# Uveddi Configuration Guide

## Configuration Methods

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
