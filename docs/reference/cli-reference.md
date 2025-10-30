# Uveddi CLI Reference

Complete command-line interface reference for Uveddi architectural analysis tool.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Global Options](#global-options)
3. [Commands](#commands)
4. [Analysis Options](#analysis-options)
5. [Memory Management](#memory-management)
6. [Output Formats](#output-formats)
7. [AI Integration](#ai-integration)
8. [Examples](#examples)
9. [Environment Variables](#environment-variables)
10. [Exit Codes](#exit-codes)

---

## Basic Usage

```bash
uveddi [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

### Quick Examples
```bash
# Basic analysis with HTML output
uveddi analyze ./src --output-format html

# AI-powered analysis
uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Memory-optimized for large codebases
uveddi analyze ./src --memory-profile large --memory-limit-gb 8

# Generate debug report
uveddi debug-report > debug.log

# Check known issues
uveddi known-issues
```

---

## Global Options

Options that apply to all commands:

| Flag | Description | Default | Example |
|------|-------------|---------|---------|
| `--version` | Show version information | - | `uveddi --version` |
| `--help` | Show help message | - | `uveddi --help` |
| `--config <FILE>` | Use specific config file | `~/.config/uveddi/config.toml` | `--config ./custom.toml` |
| `--profile <NAME>` | Use configuration profile | `default` | `--profile production` |
| `--verbose` | Enable verbose output | `false` | `--verbose` |
| `--quiet` | Suppress output except errors | `false` | `--quiet` |
| `--log-level <LEVEL>` | Set logging level | `info` | `--log-level debug` |
| `--no-color` | Disable colored output | `false` | `--no-color` |

---

## Commands

### `analyze` - Analyze Codebase

Primary command for analyzing codebases and detecting architectural issues.

```bash
uveddi analyze [OPTIONS] <PATH>
```

**Arguments:**
- `<PATH>` - Path to analyze (file or directory)

### `serve` - Start Web Services

Start the web dashboard and API server.

```bash
uveddi serve [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--port <PORT>` | API server port | `8080` |
| `--rendering-port <PORT>` | Rendering service port | `3001` |
| `--host <HOST>` | Bind address | `localhost` |
| `--workers <NUM>` | Number of worker processes | Auto-detected |

### `doctor` - System Diagnostic

Check system configuration and dependencies.

```bash
uveddi doctor [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--fix` | Attempt to fix detected issues |
| `--check-deps` | Verify all dependencies |
| `--check-config` | Validate configuration files |

### `known-issues` - List Known Issues

Display current known issues and limitations.

```bash
uveddi known-issues [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--format <FORMAT>` | Output format (`text`, `json`, `markdown`) |
| `--severity <LEVEL>` | Filter by severity (`low`, `medium`, `high`, `critical`) |

### `debug-report` - Generate Debug Report

Create comprehensive diagnostic report for troubleshooting.

```bash
uveddi debug-report [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--include-logs` | Include recent log files |
| `--include-config` | Include configuration (sanitized) |
| `--output <FILE>` | Save to file instead of stdout |

### `validate-config` - Validate Configuration

Validate configuration files for syntax and consistency.

```bash
uveddi validate-config [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--config <FILE>` | Specific config file to validate |
| `--strict` | Enable strict validation |

### `plugin` - Plugin Management

Manage WASM plugins (planned feature).

```bash
uveddi plugin <SUBCOMMAND>
```

**Subcommands:**
- `install <PLUGIN>` - Install plugin
- `uninstall <PLUGIN>` - Uninstall plugin
- `list` - List installed plugins
- `info <PLUGIN>` - Show plugin information
- `update <PLUGIN>` - Update plugin

---

## Analysis Options

Options specific to the `analyze` command:

### Basic Analysis Options

| Flag | Description | Default | Example |
|------|-------------|---------|---------|
| `--output <FILE>` | Output file path | stdout | `--output report.html` |
| `--output-format <FORMAT>` | Output format | `text` | `--output-format html` |
| `--include-timing` | Include timing information | `false` | `--include-timing` |
| `--dry-run` | Show what would be analyzed | `false` | `--dry-run` |

### Detection Options

| Flag | Description | Default |
|------|-------------|---------|
| `--detector <DETECTOR>` | Run specific detector(s) | `all` |
| `--skip-detector <DETECTOR>` | Skip specific detector(s) | None |
| `--fail-on-critical` | Exit with error on critical issues | `false` |

**Available Detectors:**
- `god-object` - Classes with too many responsibilities
- `dead-code` - Unused functions, variables, and modules
- `circular-dependencies` - Cyclic dependencies
- `tight-coupling` - Excessive inter-module dependencies
- `magic-values` - Hardcoded constants
- `code-duplication` - Copy-paste patterns
- `long-methods` - Complex functions
- `large-classes` - Oversized classes
- `leaky-abstractions` - Implementation detail exposure
- `security` - Security vulnerabilities
- `all` - Run all detectors

### Threshold Configuration

| Flag | Description | Default |
|------|-------------|---------|
| `--god-object-threshold <NUM>` | Max methods for god object detection | `100` |
| `--large-class-loc-threshold <NUM>` | Max lines for large class detection | `300` |
| `--large-class-method-threshold <NUM>` | Max methods for large class detection | `20` |
| `--long-method-threshold <NUM>` | Max lines for long method detection | `50` |
| `--dead-code-confidence <FLOAT>` | Dead code detection confidence (0.0-1.0) | `0.8` |
| `--complexity-threshold <NUM>` | Cyclomatic complexity threshold | `15` |

### File Processing Options

| Flag | Description | Default |
|------|-------------|---------|
| `--max-file-size <SIZE>` | Skip files larger than SIZE | `2MB` |
| `--exclude <PATTERN>` | Exclude files matching pattern | None |
| `--include <PATTERN>` | Include only files matching pattern | None |
| `--follow-symlinks` | Follow symbolic links | `false` |
| `--include-tests` | Analyze test files | `false` |

---

## Memory Management

Uveddi includes advanced memory management for analyzing large codebases:

### Automatic Memory Optimization (Default)

Memory optimization is **enabled by default** with automatic configuration:

```bash
# These happen automatically:
# - Object pooling for reduced allocations
# - Arena allocation for temporary objects
# - Zero-copy AST caching
# - Automatic memory limits based on system RAM
```

### Memory Control Options

| Flag | Description | Default |
|------|-------------|---------|
| `--disable-memory-optimization` | Disable memory optimization (not recommended) | `false` |
| `--memory-limit-gb <GB>` | Manual memory limit override | Auto-detected |
| `--memory-profile <PROFILE>` | Memory profile (`small`, `default`, `large`) | Auto-detected |
| `--threads <NUM>` | Number of analysis threads | CPU cores |

### Memory Profiles

**small** - For systems with limited RAM (< 4GB)
- Reduced cache sizes
- Sequential processing
- Minimal object pooling

**default** - For typical development systems (4-16GB)
- Balanced memory usage
- Moderate parallelism
- Standard caching

**large** - For high-memory systems (16GB+)
- Aggressive caching
- Maximum parallelism
- Large object pools

### Memory Usage Examples

```bash
# Automatic memory management (recommended)
uveddi analyze ./large-codebase

# Force small memory profile
uveddi analyze ./codebase --memory-profile small

# Set specific memory limit
uveddi analyze ./codebase --memory-limit-gb 4

# Disable optimization for debugging
uveddi analyze ./codebase --disable-memory-optimization
```

---

## Output Formats

Uveddi supports multiple output formats for different use cases:

### Available Formats

| Format | Description | Extension | Use Case |
|--------|-------------|-----------|----------|
| `text` | Plain text output | `.txt` | Terminal display, CI logs |
| `json` | Structured JSON data | `.json` | Tool integration, APIs |
| `html` | Interactive HTML report | `.html` | Web viewing, sharing |
| `markdown` | Markdown with diagrams | `.md` | Documentation, GitHub |
| `csv` | Comma-separated values | `.csv` | Spreadsheet analysis |
| `xml` | XML structured output | `.xml` | Enterprise integration |

### Format-Specific Options

#### HTML Output
```bash
uveddi analyze ./src --output-format html \
    --output report.html \
    --html-theme dark \
    --include-diagrams
```

**HTML Options:**
- `--html-theme <THEME>` - Theme (`light`, `dark`, `auto`)
- `--include-diagrams` - Include Mermaid diagrams
- `--html-standalone` - Generate self-contained HTML

#### JSON Output
```bash
uveddi analyze ./src --output-format json \
    --output analysis.json \
    --json-pretty
```

**JSON Options:**
- `--json-pretty` - Pretty-print JSON output
- `--json-compact` - Minimize JSON size

#### Markdown Output
```bash
uveddi analyze ./src --output-format markdown \
    --output report.md \
    --include-toc \
    --include-diagrams
```

**Markdown Options:**
- `--include-toc` - Include table of contents
- `--include-diagrams` - Include Mermaid diagrams
- `--markdown-style <STYLE>` - Markdown style (`github`, `commonmark`)

---

## AI Integration

Uveddi supports multiple AI providers for enhanced analysis insights:

### AI Provider Options

| Flag | Description | Default |
|------|-------------|---------|
| `--enable-ai` | Enable AI-powered analysis | `false` |
| `--ai-provider <PROVIDER>` | AI provider to use | `ollama` |
| `--ai-timeout <SECONDS>` | AI request timeout | `30` |
| `--ai-max-tokens <NUM>` | Maximum tokens per request | `2000` |

### Ollama (Local AI)

```bash
# Basic Ollama usage
uveddi analyze ./src --enable-ai \
    --ollama-model deepseek-coder:6.7b

# Custom Ollama configuration
uveddi analyze ./src --enable-ai \
    --ai-provider ollama \
    --ollama-api-url http://localhost:11434 \
    --ollama-model deepseek-coder:6.7b-instruct
```

**Ollama Options:**
- `--ollama-api-url <URL>` - Ollama API endpoint
- `--ollama-model <MODEL>` - Model to use
- `--ollama-timeout <SECONDS>` - Request timeout

**Recommended Models:**
- `deepseek-coder:6.7b` - General code analysis
- `deepseek-coder:6.7b-instruct` - Enhanced explanations  
- `codellama:7b-code` - Alternative code model
- `llama3:8b` - General purpose model

### OpenAI Integration

```bash
# OpenAI GPT models
uveddi analyze ./src --enable-ai \
    --ai-provider openai \
    --openai-model gpt-4 \
    --openai-api-key $OPENAI_API_KEY
```

**OpenAI Options:**
- `--openai-api-key <KEY>` - API key (or use environment variable)
- `--openai-model <MODEL>` - Model (`gpt-4`, `gpt-3.5-turbo`)
- `--openai-organization <ORG>` - Organization ID

### Anthropic Claude Integration

```bash
# Anthropic Claude models
uveddi analyze ./src --enable-ai \
    --ai-provider anthropic \
    --anthropic-model claude-3-opus \
    --anthropic-api-key $ANTHROPIC_API_KEY
```

**Anthropic Options:**
- `--anthropic-api-key <KEY>` - API key
- `--anthropic-model <MODEL>` - Model (`claude-3-opus`, `claude-3-sonnet`)

### Google Gemini Integration

```bash
# Google Gemini models
uveddi analyze ./src --enable-ai \
    --ai-provider gemini \
    --gemini-model gemini-pro \
    --gemini-api-key $GEMINI_API_KEY
```

---

## Examples

### Basic Usage Examples

```bash
# Simple analysis with text output
uveddi analyze ./src

# Analyze specific directory with HTML report
uveddi analyze ./backend --output-format html --output backend-analysis.html

# Quick security check
uveddi analyze ./src --detector security --output-format json

# Performance analysis with timing
uveddi analyze ./src --include-timing --verbose
```

### Advanced Analysis Examples

```bash
# Comprehensive analysis with AI insights
uveddi analyze ./src \
    --enable-ai \
    --ollama-model deepseek-coder:6.7b \
    --output-format html \
    --output comprehensive-report.html \
    --include-diagrams \
    --include-timing

# Large codebase optimization
uveddi analyze ./monorepo \
    --memory-profile large \
    --memory-limit-gb 16 \
    --threads 12 \
    --exclude "node_modules/*" \
    --exclude "target/*" \
    --max-file-size 5MB

# Custom threshold analysis
uveddi analyze ./legacy-code \
    --god-object-threshold 200 \
    --large-class-loc-threshold 500 \
    --complexity-threshold 25 \
    --dead-code-confidence 0.6

# Multi-format output
uveddi analyze ./src --output-format html --output report.html
uveddi analyze ./src --output-format json --output data.json
uveddi analyze ./src --output-format markdown --output README-analysis.md
```

### CI/CD Integration Examples

```bash
# GitHub Actions
- name: Run Uveddi Analysis
  run: |
    uveddi analyze ./src \
      --output-format json \
      --output analysis.json \
      --fail-on-critical \
      --quiet

# GitLab CI
script:
  - uveddi analyze ./src --detector security --fail-on-critical

# Jenkins Pipeline
sh '''
  uveddi analyze ./src \
    --output-format html \
    --output analysis-${BUILD_NUMBER}.html \
    --include-timing
'''
```

### Development Workflow Examples

```bash
# Pre-commit hook
uveddi analyze ./src \
    --detector god-object,dead-code \
    --output-format text \
    --quiet

# Code review preparation
uveddi analyze ./feature-branch \
    --enable-ai \
    --output-format markdown \
    --output code-review.md

# Refactoring planning
uveddi analyze ./legacy-module \
    --detector god-object,tight-coupling,large-classes \
    --output-format html \
    --output refactoring-plan.html \
    --ai-suggest-refactoring
```

---

## Environment Variables

Environment variables that affect CLI behavior:

### Core Configuration
| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `UVEDDI_CONFIG_PATH` | Default config file path | `~/.config/uveddi/config.toml` | `./custom-config.toml` |
| `UVEDDI_LOG_LEVEL` | Default log level | `info` | `debug` |
| `UVEDDI_NO_COLOR` | Disable colored output | `false` | `true` |
| `UVEDDI_CACHE_DIR` | Cache directory | `~/.cache/uveddi` | `/tmp/uveddi-cache` |

### Memory Management
| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_MEMORY_LIMIT_GB` | Memory limit in GB | `8.0` |
| `UVEDDI_MEMORY_PROFILE` | Memory profile | `large` |
| `UVEDDI_DISABLE_MEMORY_OPT` | Disable memory optimization | `true` |

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

### Performance Tuning
| Variable | Description | Example |
|----------|-------------|---------|
| `UVEDDI_THREADS` | Number of analysis threads | `8` |
| `UVEDDI_MAX_FILE_SIZE` | Maximum file size | `5MB` |
| `UVEDDI_TIMEOUT` | Analysis timeout | `600` |

### Debugging
| Variable | Description | Example |
|----------|-------------|---------|
| `RUST_LOG` | Rust logging configuration | `debug` |
| `RUST_BACKTRACE` | Enable backtraces | `1` |
| `UVEDDI_DEBUG_AST` | Debug AST parsing | `true` |
| `UVEDDI_DEBUG_MEMORY` | Debug memory usage | `true` |

---

## Exit Codes

Uveddi uses the following exit codes:

| Code | Meaning | Description |
|------|---------|-------------|
| 0 | Success | Analysis completed successfully |
| 1 | General error | Unspecified error occurred |
| 2 | Configuration error | Invalid configuration or arguments |
| 3 | Analysis error | Analysis failed (parsing, etc.) |
| 4 | Critical issues found | Used with `--fail-on-critical` |
| 5 | Memory error | Out of memory or memory limit exceeded |
| 6 | Timeout error | Analysis timed out |
| 7 | Permission error | File access or permission denied |
| 8 | Plugin error | Plugin loading or execution failed |
| 9 | AI provider error | AI service unavailable or error |
| 10 | Network error | Network connectivity issues |

### Exit Code Examples

```bash
# Check exit code in scripts
uveddi analyze ./src --fail-on-critical
if [ $? -eq 4 ]; then
    echo "Critical issues found!"
    exit 1
fi

# Handle specific error types
uveddi analyze ./large-codebase --memory-limit-gb 2
case $? in
    0) echo "Analysis successful" ;;
    5) echo "Memory limit exceeded, try increasing --memory-limit-gb" ;;
    6) echo "Analysis timed out, try --timeout or smaller scope" ;;
    *) echo "Analysis failed with code $?" ;;
esac
```

---

## Performance Tips

### For Large Codebases

```bash
# Use memory optimization (enabled by default)
uveddi analyze ./monorepo --memory-profile large

# Exclude unnecessary files
uveddi analyze ./src \
    --exclude "node_modules/*" \
    --exclude "target/*" \
    --exclude "*.generated.*"

# Parallel processing
uveddi analyze ./src --threads $(nproc)

# Skip expensive detectors if not needed
uveddi analyze ./src --skip-detector security,code-duplication
```

### For CI/CD Environments

```bash
# Optimize for speed and resource usage
uveddi analyze ./src \
    --memory-profile small \
    --threads 2 \
    --quiet \
    --output-format json

# Focus on critical issues only
uveddi analyze ./src \
    --detector security,god-object,circular-dependencies \
    --fail-on-critical
```

### For Development

```bash
# Quick checks during development
uveddi analyze ./modified-files \
    --detector dead-code,magic-values \
    --output-format text

# Incremental analysis (when available)
uveddi analyze ./src --incremental
```

---

This CLI reference covers all available commands, options, and usage patterns for Uveddi. For additional examples and advanced usage patterns, see the [User Guide](../user-guide/) and [Examples](../examples/) sections.