# Uveddi Troubleshooting Guide

**Version:** 1.0.0
**Last Updated:** 2025-10-09

This guide covers common issues, error messages, and solutions for Uveddi CLI users.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Analysis Errors](#analysis-errors)
- [AI Integration Problems](#ai-integration-problems)
- [Configuration Issues](#configuration-issues)
- [Performance Problems](#performance-problems)
- [Cache & Database Issues](#cache--database-issues)
- [CI/CD Integration](#cicd-integration)
- [Using the Doctor Command](#using-the-doctor-command)

---

## Installation Issues

### Binary Not Found After Installation

**Problem:** After installing Uveddi, running `uveddi` returns "command not found".

**Solution:**

1. Verify the binary is in your PATH:
   ```bash
   which uveddi
   ```

2. If installed via cargo:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   # Add to ~/.bashrc or ~/.zshrc for persistence
   ```

3. Verify installation:
   ```bash
   uveddi --version
   ```

### Permission Denied on Linux/macOS

**Problem:** `permission denied` when trying to run uveddi binary.

**Solution:**

```bash
chmod +x /path/to/uveddi
```

For cargo installations, binaries should already have execute permissions. If not:
```bash
chmod +x ~/.cargo/bin/uveddi
```

### Missing System Dependencies

**Problem:** Error about missing system libraries on Linux.

**Solution:**

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc openssl-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel openssl
```

---

## Analysis Errors

### Parser Failure

**Error:** `Failed to parse file: <filename>`

**Common Causes:**
1. Unsupported language or syntax
2. Corrupted or invalid source file
3. Memory limits exceeded

**Solution:**

1. Check file encoding (must be UTF-8):
   ```bash
   file <filename>
   ```

2. Verify file syntax manually:
   ```bash
   # For Rust
   rustc --parse-only <filename>

   # For Python
   python -m py_compile <filename>

   # For JavaScript/TypeScript
   node --check <filename>
   ```

3. Run doctor to check parser health:
   ```bash
   uveddi doctor --parsers
   ```

4. Try with verbose output:
   ```bash
   uveddi analyze ./src --verbose
   ```

### Out of Memory During Analysis

**Error:** `Out of memory` or system becomes unresponsive

**Solution:**

1. Use memory limits:
   ```bash
   uveddi analyze ./src --memory-limit-gb 4
   ```

2. Use appropriate memory profile:
   ```bash
   # For large codebases
   uveddi analyze ./src --memory-profile large

   # For small projects
   uveddi analyze ./src --memory-profile small
   ```

3. Disable memory optimizations if causing issues:
   ```bash
   uveddi analyze ./src --disable-memory-optimization
   ```

4. Reduce diagram generation:
   ```bash
   uveddi analyze ./src --max-diagrams 5
   # or
   uveddi analyze ./src --no-diagrams
   ```

### Analysis Timeout

**Error:** `Analysis timed out after X seconds`

**Solution:**

1. Increase timeout:
   ```bash
   uveddi analyze ./src --timeout 600  # 10 minutes
   ```

2. For very large codebases:
   ```bash
   uveddi analyze ./src --timeout 0  # No timeout (use with caution)
   ```

3. Optimize analysis scope:
   ```bash
   # Analyze specific directory
   uveddi analyze ./src/core

   # Skip non-essential detectors
   uveddi analyze ./src --no-diagrams
   ```

### Permission Denied Reading Files

**Error:** `Permission denied: <path>`

**Solution:**

1. Check file permissions:
   ```bash
   ls -la <path>
   ```

2. Run with appropriate permissions:
   ```bash
   # On Unix systems, avoid sudo unless necessary
   chmod +r <path>  # Make file readable
   ```

3. Verify directory access:
   ```bash
   uveddi doctor --system
   ```

---

## AI Integration Problems

### Cannot Connect to Ollama

**Error:** `Failed to connect to Ollama API at http://localhost:11434`

**Solution:**

1. Verify Ollama is running:
   ```bash
   curl http://localhost:11434/api/tags
   ```

2. Check Ollama service status:
   ```bash
   # macOS/Linux
   ps aux | grep ollama

   # Start Ollama if not running
   ollama serve
   ```

3. Verify API URL:
   ```bash
   uveddi doctor --ai
   ```

4. Set custom URL:
   ```bash
   export OLLAMA_API_URL=http://localhost:11434
   uveddi analyze ./src --enable-ai
   ```

### Model Not Found

**Error:** `Model 'xyz' not found`

**Solution:**

1. List available models:
   ```bash
   ollama list
   ```

2. Pull required model:
   ```bash
   ollama pull deepseek-coder:6.7b-instruct-q4_0
   ```

3. Set model in config:
   ```bash
   uveddi config set ollama_model "deepseek-coder:6.7b-instruct-q4_0"
   ```

4. Verify model works:
   ```bash
   ollama run deepseek-coder:6.7b-instruct-q4_0 "test prompt"
   ```

### AI Analysis Very Slow

**Problem:** AI-powered analysis takes extremely long.

**Solution:**

1. Use quantized models (faster inference):
   ```bash
   ollama pull deepseek-coder:6.7b-instruct-q4_0  # 4-bit quantized
   ```

2. Limit analysis scope:
   ```bash
   # Analyze critical files only
   uveddi analyze ./src/core --enable-ai
   ```

3. Check Ollama resource allocation:
   ```bash
   # Monitor resource usage
   top -p $(pgrep ollama)
   ```

4. Disable AI for routine analyses:
   ```bash
   uveddi analyze ./src  # No --enable-ai flag
   ```

---

## Configuration Issues

### Configuration File Not Found

**Error:** `Config file not found: uveddi.toml`

**Solution:**

1. Create configuration file:
   ```bash
   uveddi init
   ```

2. Specify custom config path:
   ```bash
   uveddi config show --file ./custom-config.toml
   ```

3. Use environment variables instead:
   ```bash
   export OLLAMA_MODEL=deepseek-coder:6.7b
   uveddi analyze ./src --enable-ai
   ```

### Invalid Configuration Format

**Error:** `Failed to parse config file: invalid TOML`

**Solution:**

1. Validate configuration:
   ```bash
   uveddi config validate --suggestions
   ```

2. Check TOML syntax:
   - Use a TOML linter online or via editor plugin
   - Common issues: missing quotes, incorrect brackets, wrong key names

3. Example valid config:
   ```toml
   ollama_model = "deepseek-coder:6.7b"

   [dead_code]
   confidence_threshold = 0.7
   library_mode = false

   [large_classes]
   max_loc = 400
   max_methods = 20
   ```

4. Regenerate configuration:
   ```bash
   mv uveddi.toml uveddi.toml.backup
   uveddi init --template rust
   ```

### Config Changes Not Taking Effect

**Problem:** Configuration changes don't affect analysis.

**Solution:**

1. Verify correct config file is used:
   ```bash
   uveddi config show
   ```

2. Check precedence:
   - Command-line flags override config file
   - Environment variables override config file
   - Config file values are defaults

3. Clear cache if needed:
   ```bash
   rm -rf ./.uveddi/
   ```

---

## Performance Problems

### Slow Analysis on Large Codebase

**Problem:** Analysis takes too long on large projects.

**Solution:**

1. Use large memory profile:
   ```bash
   uveddi analyze ./monorepo --memory-profile large
   ```

2. Limit diagram generation:
   ```bash
   uveddi analyze ./src --max-diagrams 10
   ```

3. Skip non-essential detectors:
   ```bash
   # Security only
   uveddi analyze ./src --security-only
   ```

4. Monitor progress:
   ```bash
   uveddi analyze ./src --progress-format terminal --progress-details
   ```

5. Analyze in chunks:
   ```bash
   uveddi analyze ./src/module1
   uveddi analyze ./src/module2
   ```

### High Memory Usage

**Problem:** Uveddi consumes excessive memory.

**Solution:**

1. Set memory limit:
   ```bash
   uveddi analyze ./src --memory-limit-gb 4
   ```

2. Use small memory profile:
   ```bash
   uveddi analyze ./src --memory-profile small
   ```

3. Disable caching:
   ```bash
   # Remove cache directory
   rm -rf ./.uveddi/
   ```

4. Monitor memory:
   ```bash
   # During analysis
   watch -n 1 "ps aux | grep uveddi"
   ```

---

## Cache & Database Issues

### Corrupted Cache

**Error:** `Failed to read cache` or inconsistent analysis results

**Solution:**

1. Clear cache directory:
   ```bash
   rm -rf ./.uveddi/
   ```

2. Re-run analysis:
   ```bash
   uveddi analyze ./src
   ```

3. Verify cache health:
   ```bash
   uveddi doctor
   ```

### Database Lock Error

**Error:** `Database is locked` or `Cannot acquire lock`

**Solution:**

1. Ensure no other Uveddi instances are running:
   ```bash
   ps aux | grep uveddi
   # Kill any lingering processes
   pkill -9 uveddi
   ```

2. Remove lock file:
   ```bash
   rm ./.uveddi/database.db-wal
   rm ./.uveddi/database.db-shm
   ```

3. If persistent, remove database:
   ```bash
   rm ./.uveddi/database.db
   uveddi analyze ./src  # Rebuilds database
   ```

### Stale Cache Results

**Problem:** Analysis doesn't reflect recent code changes.

**Solution:**

1. Cache is automatically invalidated on file changes, but if issues persist:
   ```bash
   rm -rf ./.uveddi/
   ```

2. Force fresh analysis:
   ```bash
   uveddi analyze ./src
   ```

---

## CI/CD Integration

### CI Check Always Fails

**Problem:** `uveddi ci check` fails even with clean code.

**Solution:**

1. Review thresholds:
   ```bash
   uveddi ci check ./src --max-debt 100 --max-critical 10
   ```

2. Check exit code and output:
   ```bash
   uveddi ci check ./src
   echo "Exit code: $?"
   ```

3. Run regular analysis first:
   ```bash
   uveddi analyze ./src --output-format json
   # Review metrics
   ```

4. Adjust thresholds based on baseline:
   ```bash
   # Get baseline metrics
   uveddi analyze ./src --output-format json | jq '.summary.debtScore'

   # Set appropriate threshold
   uveddi ci check ./src --max-debt 60
   ```

### CI Times Out

**Problem:** CI pipeline times out during analysis.

**Solution:**

1. Set appropriate timeout:
   ```bash
   uveddi ci check ./src --timeout 300
   ```

2. Optimize analysis:
   ```bash
   uveddi ci check ./src --no-diagrams --memory-profile small
   ```

3. Cache between CI runs:
   ```yaml
   # Example GitHub Actions
   - name: Cache Uveddi
     uses: actions/cache@v3
     with:
       path: .uveddi
       key: uveddi-${{ hashFiles('**/*.rs') }}
   ```

### Inconsistent CI Results

**Problem:** CI results differ from local runs.

**Solution:**

1. Ensure same Uveddi version:
   ```bash
   uveddi --version
   ```

2. Pin version in CI:
   ```yaml
   # Example
   run: curl -L https://github.com/uveddi/uveddi/releases/download/v1.0.0/uveddi-linux-amd64 -o uveddi
   ```

3. Use deterministic configuration:
   ```bash
   uveddi ci check ./src --max-debt 50 --max-critical 0
   ```

4. Clear cache in CI:
   ```bash
   rm -rf ./.uveddi/
   uveddi ci check ./src
   ```

---

## Using the Doctor Command

The `uveddi doctor` command is your first-line diagnostic tool.

### Full Health Check

```bash
uveddi doctor
```

Checks:
- Parser availability for all languages
- AI service connectivity (if configured)
- System resources and permissions
- Cache integrity
- Configuration validity

### Targeted Diagnostics

**Check Parser Health:**
```bash
uveddi doctor --parsers
```

**Check AI Integration:**
```bash
uveddi doctor --ai
```

**Check System Resources:**
```bash
uveddi doctor --system
```

### Auto-Fix Issues

```bash
uveddi doctor --fix
```

This will attempt to automatically fix common issues like:
- Missing cache directories
- Incorrect permissions
- Corrupted cache entries

### Extended Diagnostics

```bash
uveddi doctor --extended
```

Runs comprehensive tests (slower but more thorough):
- Full parser validation with test files
- AI model inference tests
- Memory allocation tests
- Cache read/write performance

### Output Formats

**Human-readable (default):**
```bash
uveddi doctor
```

**JSON (for automation):**
```bash
uveddi doctor --output-format json > health-check.json
```

**Markdown (for documentation):**
```bash
uveddi doctor --output-format markdown > HEALTH.md
```

---

## Getting More Help

### Verbose Mode

For detailed debugging information:
```bash
uveddi analyze ./src --verbose
```

### Debug Logging

Enable debug-level logs:
```bash
RUST_LOG=debug uveddi analyze ./src
```

Enable trace-level logs (very verbose):
```bash
RUST_LOG=trace uveddi analyze ./src 2> debug.log
```

### Issue Reporting

If problems persist, gather diagnostic information:

```bash
# System information
uveddi --version
uname -a

# Health check
uveddi doctor --extended --output-format json > doctor-report.json

# Debug logs
RUST_LOG=debug uveddi analyze ./src > analysis.log 2>&1

# Include these files when reporting issues
```

### Community Support

- GitHub Issues: https://github.com/botzrDev/uveddi/issues
- Documentation: Check README.md and docs/ directory
- CLI Reference: See [CLI_REFERENCE.md](./CLI_REFERENCE.md)

---

## Common Error Messages

### "Analysis failed: No supported files found"

**Cause:** No Rust, Python, JavaScript, or TypeScript files in target directory.

**Fix:** Verify target path and file extensions.

### "SARIF export failed: Output path not writable"

**Cause:** Insufficient permissions or invalid path.

**Fix:**
```bash
# Check directory permissions
ls -la $(dirname sarif-output-path)

# Use writable location
uveddi analyze ./src --export-sarif --sarif-output ./reports/security.sarif
```

### "Taint analysis exceeded maximum depth"

**Cause:** Taint analysis depth limit reached.

**Fix:**
```bash
# Increase depth
uveddi analyze ./src --enable-taint-analysis --taint-analysis-depth 20
```

### "Rendering service unavailable"

**Cause:** Image rendering service not running (optional feature).

**Fix:**
```bash
# Use Mermaid-only mode (recommended)
uveddi analyze ./src --mermaid-only

# Or start rendering service
# (See rendering service documentation)
```

---

**Last Updated:** 2025-10-09
**Applies to:** Uveddi v1.0.0
