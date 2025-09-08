# CLI Commands Reference

This section documents all available CLI commands for Uveddi.

## Basic Usage

```bash
uveddi analyze <path> [options]
uveddi known-issues
uveddi debug-report
```

## Commands

- `analyze` — Analyze a codebase for architectural issues
  - `--output <file>`: Save report to file
  - `--output-format <format>`: Output as markdown, json, html, etc.
  - `--enable-ai`: Enable AI-powered insights and explanations
  - `--ollama-api-url <url>`: Ollama API URL for local AI analysis
  - `--ollama-model <model>`: Ollama model name for analysis
  - `--dead-code-confidence <threshold>`: Dead code detection confidence (0.0-1.0)
  - `--large-classes-max-loc <lines>`: Maximum lines of code for large class detection
  - `--disable-memory-optimization`: Disable automatic memory optimization (not recommended)
  - `--memory-limit-gb <gb>`: Manual memory limit override
  - `--memory-profile <profile>`: Memory profile (small/default/large) - auto-detected by default

- `known-issues` — List current known issues
- `debug-report` — Generate a debug report for support
- `validate-config` — Validate your configuration file

## Examples

```bash
# Basic analysis (memory optimization enabled automatically)
uveddi analyze ./src --output report.md

# With AI-powered insights
uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b-instruct

# Advanced configuration
uveddi analyze ./src --memory-limit-gb 8 --memory-profile large --output-format json

# Disable memory optimization (not recommended)
uveddi analyze ./src --disable-memory-optimization

# Utility commands
uveddi known-issues
uveddi debug-report > debug.log
```

> **Performance Note**: Memory optimization is enabled by default and automatically configured based on your system. Most users don't need to specify memory-related flags.

---

> **Tip:** Run `uveddi --help` for the most up-to-date list of commands and options.
