# Uveddi CLI Reference

**Version:** 1.0.0
**Last Updated:** 2025-10-09

This document provides comprehensive command-by-command reference for the Uveddi CLI, including usage examples, common patterns, and best practices.

## Table of Contents

- [Overview](#overview)
- [Global Options](#global-options)
- [Commands](#commands)
  - [analyze](#analyze)
  - [config](#config)
  - [doctor](#doctor)
  - [hooks](#hooks)
  - [init](#init)
  - [ci](#ci)
- [Common Usage Patterns](#common-usage-patterns)
- [Environment Variables](#environment-variables)

---

## Overview

Uveddi is a comprehensive code analysis tool that combines static analysis with optional AI-powered insights. It supports Rust, Python, JavaScript, and TypeScript codebases.

```bash
uveddi <COMMAND> [OPTIONS]
```

### Quick Start

```bash
# Basic analysis
uveddi analyze ./src

# With AI explanations
uveddi analyze ./src --enable-ai

# JSON output for CI/CD
uveddi analyze ./src --output-format json --output report.json

# CI quality gate
uveddi ci check ./src --max-debt 50 --max-critical 0
```

---

## Global Options

- `-h, --help` - Print help information
- `-V, --version` - Print version information

---

## Commands

### analyze

Perform comprehensive code analysis on a project or directory.

#### Usage

```bash
uveddi analyze [OPTIONS] <PATH>
```

#### Arguments

- `<PATH>` - Path to the project or directory to analyze (required)

#### Core Options

**Output Configuration:**

- `--output-format <FORMAT>` - Output format: `text`, `json`, `markdown`, `html` (default: `markdown`)
- `--output <FILE>` - Write output to file instead of stdout

**AI Integration:**

- `--enable-ai` - Enable AI-powered explanations
- `--ollama-api-url <URL>` - Ollama API URL (default: env `OLLAMA_API_URL`)
- `--ollama-model <MODEL>` - Ollama model name (default: env `OLLAMA_MODEL`)

**Progress & Verbosity:**

- `--verbose` - Show detailed error information and stack traces
- `--progress-format <FORMAT>` - Progress format: `terminal`, `json`, `silent` (default: `terminal`)
- `--progress-details` - Show detailed progress information
- `--timeout <SECONDS>` - Maximum analysis timeout (default: 300)

#### Detector Configuration

**Dead Code Detection:**

- `--dead-code-confidence <THRESHOLD>` - Confidence threshold (0.0 to 1.0)
- `--dead-code-library-mode` - Enable library mode (conservative for public APIs)
- `--dead-code-ignore-patterns <PATTERNS>` - Comma-separated patterns to ignore
- `--dead-code-keep-alive <SYMBOLS>` - Comma-separated symbols to keep alive

**Large Classes Detection:**

- `--large-classes-max-loc <LINES>` - Maximum lines of code
- `--large-classes-max-methods <COUNT>` - Maximum number of methods
- `--large-classes-max-fields <COUNT>` - Maximum number of fields
- `--large-classes-max-complexity <COMPLEXITY>` - Maximum cyclomatic complexity
- `--large-classes-max-lcom <SCORE>` - Maximum LCOM score (0.0 to 1.0)
- `--large-classes-ignore-patterns <PATTERNS>` - Comma-separated patterns to ignore
- `--large-classes-min-severity <SCORE>` - Minimum severity (0-100, default: 25)

**Security Analysis:**

- `--security` - Enable security vulnerability analysis
- `--security-only` - Run only security analysis (skip other detectors)
- `--min-security-confidence <THRESHOLD>` - Minimum confidence (0.0 to 1.0, default: 0.5)
- `--export-sarif` - Export findings in SARIF 2.1.0 format
- `--sarif-output <FILE>` - SARIF output file path
- `--enable-taint-analysis` - Enable taint flow analysis
- `--taint-analysis-depth <DEPTH>` - Max taint analysis depth (1-20, default: 10)
- `--owasp-categories <CATEGORIES>` - Focus on specific OWASP categories (comma-separated)

#### Memory Management

- `--disable-memory-optimization` - Disable memory optimization features
- `--memory-limit-gb <GB>` - Soft memory limit in gigabytes
- `--memory-profile <PROFILE>` - Profile: `small`, `default`, `large`

#### Diagram Generation

- `--no-diagrams` - Disable diagram generation completely
- `--max-diagrams <COUNT>` - Maximum diagrams per report (default: 20)
- `--mermaid-only` - Force Mermaid-only mode (no image rendering)
- `--enable-image-rendering` - Enable image rendering (requires service)
- `--rendering-service-url <URL>` - Rendering service URL (default: http://localhost:3001)
- `--no-fallback` - Fail if image rendering unavailable
- `--check-rendering-service` - Check service availability and exit
- `--diagram-output-dir <DIR>` - Output directory for diagram files

#### Dashboard Integration

- `--open-dashboard` - Automatically open dashboard after analysis

#### Examples

**Basic analysis with markdown output:**
```bash
uveddi analyze ./src
```

**Analysis with JSON output to file:**
```bash
uveddi analyze ./src --output-format json --output analysis.json
```

**Security-focused analysis with SARIF export:**
```bash
uveddi analyze ./src --security --export-sarif --sarif-output security.sarif
```

**AI-powered analysis with Ollama:**
```bash
uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b-instruct-q4_0
```

**Large codebase with memory constraints:**
```bash
uveddi analyze ./monorepo --memory-profile large --memory-limit-gb 8
```

**HTML report with interactive diagrams:**
```bash
uveddi analyze ./src --output-format html --output report.html
```

**Dead code analysis for a library:**
```bash
uveddi analyze ./lib --dead-code-library-mode --dead-code-keep-alive "new,init,builder"
```

**Taint analysis for security vulnerabilities:**
```bash
uveddi analyze ./api --enable-taint-analysis --taint-analysis-depth 15 --min-security-confidence 0.8
```

---

### config

Manage Uveddi configuration settings.

#### Usage

```bash
uveddi config <COMMAND>
```

#### Subcommands

**show** - Display current configuration from file or environment

```bash
uveddi config show [--file <PATH>]
```

**set** - Set a configuration value in the config file

```bash
uveddi config set <KEY> <VALUE> [--file <PATH>]
```

**validate** - Validate configuration file with suggestions

```bash
uveddi config validate [--file <PATH>] [--suggestions] [--format <FORMAT>]
```

#### Examples

**Show current configuration:**
```bash
uveddi config show
```

**Show configuration from specific file:**
```bash
uveddi config show --file ./custom-config.toml
```

**Set Ollama model:**
```bash
uveddi config set ollama_model "deepseek-coder:6.7b"
```

**Validate configuration with suggestions:**
```bash
uveddi config validate --suggestions
```

**Validate with JSON output:**
```bash
uveddi config validate --format json
```

---

### doctor

Run health diagnostics and fix common issues.

#### Usage

```bash
uveddi doctor [OPTIONS]
```

#### Options

- `--fix` - Automatically fix issues where possible
- `--parsers` - Check only language parser availability
- `--ai` - Check only AI service connectivity
- `--system` - Check only system resources and permissions
- `--output-format <FORMAT>` - Output format: `human`, `json`, `markdown` (default: `human`)
- `--extended` - Run extended functionality tests (slower)
- `--quiet` - Only report critical issues

#### Examples

**Full health check:**
```bash
uveddi doctor
```

**Check and fix issues:**
```bash
uveddi doctor --fix
```

**Check only AI connectivity:**
```bash
uveddi doctor --ai
```

**Extended diagnostics with JSON output:**
```bash
uveddi doctor --extended --output-format json
```

**Quick check (critical issues only):**
```bash
uveddi doctor --quiet
```

---

### hooks

Manage Git hooks for automated analysis.

#### Usage

```bash
uveddi hooks <COMMAND>
```

#### Subcommands

**install** - Install Git hooks

```bash
uveddi hooks install
```

**uninstall** - Remove Uveddi Git hooks

```bash
uveddi hooks uninstall
```

**list** - List installed hooks

```bash
uveddi hooks list
```

**test** - Test hooks without committing

```bash
uveddi hooks test
```

**config** - Show hook configuration

```bash
uveddi hooks config
```

#### Examples

**Install pre-commit hook:**
```bash
uveddi hooks install
```

**Test hook configuration:**
```bash
uveddi hooks test
```

**List active hooks:**
```bash
uveddi hooks list
```

**Remove hooks:**
```bash
uveddi hooks uninstall
```

---

### init

Initialize Uveddi configuration for a project.

#### Usage

```bash
uveddi init [OPTIONS] [PATH]
```

#### Arguments

- `[PATH]` - Path to initialize (default: `.`)

#### Options

- `--non-interactive` - Skip prompts, use defaults
- `--template <TEMPLATE>` - Project template: `rust`, `python`, `javascript`, `typescript`, `web`, `library`, `monorepo`, `custom`
- `--force` - Overwrite existing configuration
- `--git-hooks` - Generate Git hooks

#### Examples

**Interactive initialization:**
```bash
uveddi init
```

**Initialize Rust project with hooks:**
```bash
uveddi init --template rust --git-hooks
```

**Initialize in specific directory:**
```bash
uveddi init ./my-project --template python
```

**Non-interactive with defaults:**
```bash
uveddi init --non-interactive
```

**Force overwrite existing config:**
```bash
uveddi init --force --template custom
```

---

### ci

CI/CD integration for quality gates.

#### Usage

```bash
uveddi ci <COMMAND>
```

#### Subcommands

**check** - Run analysis and fail if thresholds exceeded

```bash
uveddi ci check <PATH> [OPTIONS]
```

#### Options

- `--max-debt <SCORE>` - Maximum allowed technical debt score (0-100, default: 50)
- `--max-critical <COUNT>` - Maximum allowed critical issues (default: 0)
- `--output-format <FORMAT>` - Output format (default: `json`)

#### Exit Codes

- `0` - Quality gate passed
- Non-zero - Quality gate failed or error occurred

#### Examples

**Basic CI quality gate:**
```bash
uveddi ci check ./src
```

**Strict quality gate (no critical issues):**
```bash
uveddi ci check ./src --max-critical 0 --max-debt 30
```

**Lenient quality gate:**
```bash
uveddi ci check ./src --max-critical 5 --max-debt 75
```

**CI with JSON output:**
```bash
uveddi ci check ./src --output-format json > ci-results.json
```

---

## Common Usage Patterns

### Local Development Workflow

```bash
# 1. Initialize project
uveddi init --template rust --git-hooks

# 2. Run analysis during development
uveddi analyze ./src --progress-details

# 3. Check configuration
uveddi doctor

# 4. Generate HTML report
uveddi analyze ./src --output-format html --output report.html --open-dashboard
```

### CI/CD Pipeline Integration

```bash
# GitHub Actions / GitLab CI
uveddi ci check ./src --max-debt 50 --max-critical 0
if [ $? -ne 0 ]; then
  echo "Quality gate failed"
  exit 1
fi
```

### Security Audit

```bash
# Comprehensive security scan
uveddi analyze ./src \
  --security \
  --enable-taint-analysis \
  --taint-analysis-depth 15 \
  --min-security-confidence 0.8 \
  --export-sarif \
  --sarif-output security-audit.sarif \
  --output-format html \
  --output security-report.html
```

### Large Codebase Analysis

```bash
# Optimized for large repositories
uveddi analyze ./monorepo \
  --memory-profile large \
  --memory-limit-gb 16 \
  --max-diagrams 50 \
  --timeout 1800 \
  --progress-format terminal \
  --progress-details
```

### AI-Enhanced Analysis

```bash
# Set up Ollama model
uveddi config set ollama_model "deepseek-coder:6.7b-instruct-q4_0"

# Run AI-powered analysis
uveddi analyze ./src --enable-ai --verbose
```

---

## Environment Variables

Uveddi respects the following environment variables:

### AI Configuration

- `OLLAMA_API_URL` - Ollama API endpoint (default: not set)
- `OLLAMA_MODEL` - Default Ollama model name (default: not set)

### Logging

- `RUST_LOG` - Log level: `error`, `warn`, `info`, `debug`, `trace` (default: `info`)
- `LOG_FORMAT` - Log format: `compact`, `json` (default: `compact`)

### Example

```bash
export OLLAMA_API_URL=http://localhost:11434
export OLLAMA_MODEL=deepseek-coder:6.7b
export RUST_LOG=debug

uveddi analyze ./src --enable-ai
```

---

## Configuration File

Uveddi looks for `uveddi.toml` in the project root:

```toml
# Example uveddi.toml
ollama_model = "deepseek-coder:6.7b-instruct-q4_0"

[dead_code]
confidence_threshold = 0.7
library_mode = true
ignore_patterns = ["test", "mock", "generated"]
keep_alive = ["main", "init", "new"]

[large_classes]
max_loc = 400
max_methods = 20
max_fields = 15
max_complexity = 50
max_lcom = 0.8
ignore_patterns = ["test", "fixture"]
min_severity = 25
```

---

## Getting Help

For additional help on any command:

```bash
uveddi <command> --help
uveddi analyze --help
uveddi config --help
```

For troubleshooting common issues, see [TROUBLESHOOTING.md](./TROUBLESHOOTING.md).

---

**Generated from Uveddi v1.0.0 CLI help output**
**Last Updated:** 2025-10-09
