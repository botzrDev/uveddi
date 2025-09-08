# Configuration Options Reference

This section documents all configuration options available for Uveddi.

## Configuration File

Uveddi can be configured via a TOML file (default: `uveddi.toml`).

### Example

```toml
[general]
output_format = "markdown"
jobs = 4
ai_provider = "ollama"

[detectors]
god_object = true
tight_coupling = true
large_class = false

[performance]
timeout = 600
max_file_size = "2MB"
```

## CLI Overrides

All configuration options can be overridden via CLI flags. See [CLI Commands](./cli-commands.md).

## Key Options

- `output_format`: Output format for reports (markdown, json, mermaid, etc.)
- `jobs`: Number of parallel jobs
- `ai_provider`: AI backend to use (ollama, openai, anthropic, gemini)
- `timeout`: Analysis timeout in seconds
- `max_file_size`: Maximum file size to analyze
- `detectors`: Enable/disable specific detectors

---

> **Note:** For a full list of options, run `uveddi --help` or see the sample config in `examples/analysis_config.toml`.
