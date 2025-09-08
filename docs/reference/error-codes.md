# Uveddi Error Codes Reference

Comprehensive reference for error codes, troubleshooting, and diagnostic procedures in Uveddi.

## Table of Contents

1. [Error Code Categories](#error-code-categories)
2. [Analysis Errors (1xxx)](#analysis-errors-1xxx)
3. [Configuration Errors (2xxx)](#configuration-errors-2xxx)
4. [AI Integration Errors (3xxx)](#ai-integration-errors-3xxx)
5. [Plugin Errors (4xxx)](#plugin-errors-4xxx)
6. [System Errors (5xxx)](#system-errors-5xxx)
7. [Network Errors (6xxx)](#network-errors-6xxx)
8. [Memory Errors (7xxx)](#memory-errors-7xxx)
9. [Security Errors (8xxx)](#security-errors-8xxx)
10. [Common Solutions](#common-solutions)
11. [Debugging Techniques](#debugging-techniques)
12. [Support and Reporting](#support-and-reporting)

---

## Error Code Categories

Uveddi uses a structured error code system with the following categories:

| Range | Category | Description |
|-------|----------|-------------|
| 1xxx | Analysis | Core analysis engine errors |
| 2xxx | Configuration | Configuration and setup errors |
| 3xxx | AI Integration | AI provider and processing errors |
| 4xxx | Plugin | Plugin loading and execution errors |
| 5xxx | System | System resource and permission errors |
| 6xxx | Network | Network connectivity and API errors |
| 7xxx | Memory | Memory allocation and limit errors |
| 8xxx | Security | Security and permission errors |
| 9xxx | Reserved | Future use |

### Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "ANALYSIS_TIMEOUT",
    "message": "Analysis exceeded the configured timeout limit",
    "details": {
      "timeout_seconds": 300,
      "files_processed": 42,
      "last_file": "src/large_module.rs"
    },
    "suggestion": "Increase timeout with --timeout flag or exclude large files",
    "documentation_url": "https://docs.uveddi.dev/errors/1001",
    "timestamp": "2025-01-28T10:00:00Z",
    "request_id": "req_12345"
  }
}
```

---

## Analysis Errors (1xxx)

Errors related to the core analysis engine and code processing.

### 1001 - ANALYSIS_TIMEOUT

**Description**: Analysis exceeded the configured timeout limit.

**Common Causes**:
- Large codebase with default timeout
- Complex parsing of deeply nested structures
- Infinite loops in detector logic
- Insufficient system resources

**Solutions**:
```bash
# Increase timeout
uveddi analyze ./src --timeout 3600

# Use incremental analysis
uveddi analyze ./src --incremental

# Exclude problematic files
uveddi analyze ./src --exclude "large_generated_file.js"

# Use memory-optimized mode
uveddi analyze ./src --memory-profile large
```

**Related Environment Variables**:
- `UVEDDI_TIMEOUT=3600`

### 1002 - MEMORY_LIMIT_EXCEEDED

**Description**: Analysis used more memory than the configured limit.

**Common Causes**:
- Large files being parsed simultaneously
- Memory leak in detector
- Insufficient memory limit for codebase size
- Memory fragmentation

**Solutions**:
```bash
# Increase memory limit
uveddi analyze ./src --memory-limit-gb 16

# Use small memory profile
uveddi analyze ./src --memory-profile small

# Reduce parallel processing
uveddi analyze ./src --threads 2

# Enable memory optimization (default)
uveddi analyze ./src --memory-profile auto
```

### 1003 - UNSUPPORTED_FILE_TYPE

**Description**: File type is not supported by any available parser.

**Common Causes**:
- Binary files being included in analysis
- New/exotic programming languages
- Corrupted text files
- Incorrect file extension mapping

**Solutions**:
```bash
# Exclude unsupported files
uveddi analyze ./src --exclude "*.bin" --exclude "*.so"

# Force file type
uveddi analyze ./src --force-language rust

# Check supported languages
uveddi --help | grep -A 10 "supported languages"
```

### 1004 - PARSE_ERROR

**Description**: Could not parse source code due to syntax errors.

**Common Causes**:
- Syntax errors in source code
- Incomplete/corrupted files
- Wrong language detection
- Unsupported language features

**Solutions**:
```bash
# Skip files with parse errors
uveddi analyze ./src --skip-parse-errors

# Check specific file
uveddi analyze single_file.rs --verbose

# Use lenient parsing
uveddi analyze ./src --lenient-parsing
```

### 1005 - DETECTOR_EXECUTION_FAILED

**Description**: A detector failed during execution.

**Common Causes**:
- Detector bug or crash
- Invalid AST structure
- Resource exhaustion
- Configuration error

**Solutions**:
```bash
# Skip problematic detector
uveddi analyze ./src --skip-detector god-object

# Run single detector for debugging
uveddi analyze ./src --detector dead-code --verbose

# Enable detector debug mode
RUST_LOG=uveddi::detectors=debug uveddi analyze ./src
```

### 1006 - INCREMENTAL_ANALYSIS_FAILED

**Description**: Incremental analysis could not determine changed files.

**Common Causes**:
- Missing git repository
- Corrupted analysis cache
- File timestamp issues
- Permission errors on cache

**Solutions**:
```bash
# Clear analysis cache
rm -rf ~/.cache/uveddi/

# Disable incremental analysis
uveddi analyze ./src --no-incremental

# Check git status
git status

# Force full analysis
uveddi analyze ./src --force-full-analysis
```

---

## Configuration Errors (2xxx)

Errors related to configuration files, command-line arguments, and setup.

### 2001 - INVALID_CONFIG_FILE

**Description**: Configuration file contains syntax or validation errors.

**Common Causes**:
- Invalid TOML syntax
- Missing required fields
- Type mismatches
- Invalid values

**Solutions**:
```bash
# Validate configuration
uveddi validate-config

# Check syntax with TOML parser
cat config.toml | python -c "import toml, sys; toml.load(sys.stdin)"

# Use default configuration
uveddi analyze ./src --no-config

# Generate example configuration
uveddi config example > config.toml
```

**Example Fix**:
```toml
# ❌ Invalid
[memory]
limit_gb = "invalid"  # Should be number, not string

# ✅ Fixed
[memory]
limit_gb = 8.0
```

### 2002 - MISSING_PROVIDER_CONFIG

**Description**: AI provider is enabled but not properly configured.

**Common Causes**:
- Missing API keys
- Incorrect provider settings
- Network configuration issues

**Solutions**:
```bash
# Set API key
export OPENAI_API_KEY="your-key-here"

# Use local provider
uveddi analyze ./src --ai-provider ollama

# Check provider configuration
uveddi config show | grep -A 10 ai.providers

# Disable AI temporarily
uveddi analyze ./src --disable-ai
```

### 2003 - INVALID_THRESHOLD_CONFIG

**Description**: Detector threshold configuration is invalid.

**Common Causes**:
- Negative threshold values
- Out-of-range confidence scores
- Type mismatches

**Solutions**:
```bash
# Check current thresholds
uveddi config show | grep thresholds

# Reset to defaults
uveddi analyze ./src --reset-thresholds

# Use valid ranges
uveddi analyze ./src --dead-code-confidence 0.8
```

**Valid Ranges**:
- Confidence values: 0.0 - 1.0
- Line counts: > 0
- Complexity scores: > 0

### 2004 - CONFLICTING_OPTIONS

**Description**: Command-line arguments or configuration options conflict.

**Common Causes**:
- Mutually exclusive flags
- Incompatible feature combinations

**Solutions**:
```bash
# ❌ Conflicting
uveddi analyze ./src --enable-ai --disable-ai

# ✅ Fixed
uveddi analyze ./src --enable-ai
```

---

## AI Integration Errors (3xxx)

Errors related to AI providers and AI-powered analysis features.

### 3001 - PROVIDER_UNAVAILABLE

**Description**: AI provider service is not responding or unavailable.

**Common Causes**:
- Service is down
- Network connectivity issues
- Incorrect API endpoint
- Rate limiting

**Solutions**:
```bash
# Check Ollama service
curl http://localhost:11434/api/tags

# Test OpenAI API
curl -H "Authorization: Bearer $OPENAI_API_KEY" https://api.openai.com/v1/models

# Use alternative provider
uveddi analyze ./src --ai-provider anthropic

# Disable AI temporarily
uveddi analyze ./src --disable-ai
```

### 3002 - CONTEXT_WINDOW_EXCEEDED

**Description**: Analysis context is too large for the AI model's context window.

**Common Causes**:
- Very large files
- Complex code structures
- Model limitations

**Solutions**:
```bash
# Use model with larger context window
uveddi analyze ./src --ollama-model deepseek-coder:33b

# Reduce context size
uveddi analyze ./src --ai-max-context-lines 500

# Process smaller chunks
uveddi analyze ./single-file.rs --enable-ai
```

### 3003 - INVALID_API_RESPONSE

**Description**: AI provider returned malformed or invalid response.

**Common Causes**:
- API changes
- Network corruption
- Provider-side issues
- Model output issues

**Solutions**:
```bash
# Retry analysis
uveddi analyze ./src --enable-ai --ai-max-retries 5

# Use different model
uveddi analyze ./src --ollama-model codellama:7b

# Enable AI debugging
RUST_LOG=uveddi::ai=debug uveddi analyze ./src --enable-ai
```

### 3004 - API_KEY_INVALID

**Description**: Provided API key is invalid or expired.

**Common Causes**:
- Incorrect API key
- Expired credentials
- Wrong provider account

**Solutions**:
```bash
# Check API key format
echo $OPENAI_API_KEY | wc -c  # Should be ~51 characters for OpenAI

# Verify key with provider
curl -H "Authorization: Bearer $OPENAI_API_KEY" https://api.openai.com/v1/models

# Update API key
export OPENAI_API_KEY="new-key-here"
```

### 3005 - RATE_LIMIT_EXCEEDED

**Description**: AI provider rate limit has been exceeded.

**Common Causes**:
- Too many requests
- Concurrent analysis runs
- Shared API key usage

**Solutions**:
```bash
# Add delay between requests
uveddi analyze ./src --ai-request-delay 1000

# Reduce parallel AI requests
uveddi analyze ./src --ai-max-concurrent 1

# Use local AI provider
uveddi analyze ./src --ai-provider ollama
```

---

## Plugin Errors (4xxx)

Errors related to WASM plugin loading, execution, and management.

### 4001 - PLUGIN_LOAD_FAILED

**Description**: Could not load or initialize plugin.

**Common Causes**:
- Corrupted WASM file
- Incompatible plugin API version
- Missing plugin dependencies
- Invalid plugin metadata

**Solutions**:
```bash
# Verify plugin file
file ~/.uveddi/plugins/custom-detector.wasm

# Check plugin info
uveddi plugin info custom-detector

# Reinstall plugin
uveddi plugin uninstall custom-detector
uveddi plugin install ./fresh-plugin.wasm

# Skip problematic plugin
uveddi analyze ./src --skip-plugin custom-detector
```

### 4002 - PLUGIN_RUNTIME_ERROR

**Description**: Plugin crashed or encountered runtime error during execution.

**Common Causes**:
- Plugin bug or panic
- Resource exhaustion
- Invalid input data
- Permission violations

**Solutions**:
```bash
# Enable plugin debugging
RUST_LOG=uveddi::plugins=debug uveddi analyze ./src

# Check plugin logs
uveddi plugin logs custom-detector

# Use plugin sandbox mode
uveddi analyze ./src --plugin-sandbox-mode strict

# Disable problematic plugin
uveddi analyze ./src --disable-plugin custom-detector
```

### 4003 - INVALID_PLUGIN_API

**Description**: Plugin uses unsupported or outdated API version.

**Common Causes**:
- Old plugin version
- API version mismatch
- Plugin compiled for different Uveddi version

**Solutions**:
```bash
# Check plugin API version
uveddi plugin info custom-detector | grep api_version

# Update plugin
uveddi plugin update custom-detector

# Check compatible plugins
uveddi plugin list --compatible

# Downgrade Uveddi (if necessary)
git checkout v0.9.0  # Example
cargo build --release
```

### 4004 - PLUGIN_PERMISSION_DENIED

**Description**: Plugin attempted operation not allowed by security policy.

**Common Causes**:
- Insufficient plugin permissions
- Sandbox restrictions
- File access violations

**Solutions**:
```bash
# Check plugin permissions
uveddi plugin permissions custom-detector

# Grant additional permissions (in config)
[plugins.custom_detector.permissions]
file_read = ["src/**", "*.toml"]

# Disable sandbox (not recommended)
uveddi analyze ./src --disable-plugin-sandbox
```

---

## System Errors (5xxx)

Errors related to system resources, file access, and operating system interactions.

### 5001 - FILE_NOT_FOUND

**Description**: Specified file or directory does not exist.

**Common Causes**:
- Incorrect file path
- File moved or deleted
- Permission issues
- Case sensitivity

**Solutions**:
```bash
# Verify path exists
ls -la ./src/

# Use absolute path
uveddi analyze /full/path/to/src

# Check current directory
pwd

# Use relative path correctly
uveddi analyze ./relative/path
```

### 5002 - PERMISSION_DENIED

**Description**: Insufficient permissions to access file or directory.

**Common Causes**:
- File ownership issues
- Insufficient user permissions
- Read-only file systems

**Solutions**:
```bash
# Check permissions
ls -la src/

# Fix ownership
sudo chown -R $USER:$USER src/

# Change permissions
chmod -R 755 src/

# Run with sudo (not recommended)
sudo uveddi analyze ./src
```

### 5003 - DISK_SPACE_FULL

**Description**: Insufficient disk space for analysis or caching.

**Common Causes**:
- Full disk
- Large analysis cache
- Temporary file accumulation

**Solutions**:
```bash
# Check disk space
df -h

# Clear analysis cache
rm -rf ~/.cache/uveddi/

# Use temporary directory with more space
export TMPDIR=/path/to/larger/tmp
uveddi analyze ./src

# Disable caching
uveddi analyze ./src --no-cache
```

### 5004 - TOO_MANY_OPEN_FILES

**Description**: System limit for open files exceeded.

**Common Causes**:
- Large number of files being processed
- Resource leaks
- Low system limits

**Solutions**:
```bash
# Check current limits
ulimit -n

# Increase file limit
ulimit -n 4096

# Process files in smaller batches
uveddi analyze ./src --max-concurrent-files 100

# Check for resource leaks
lsof | grep uveddi
```

---

## Network Errors (6xxx)

Errors related to network connectivity and API communications.

### 6001 - CONNECTION_REFUSED

**Description**: Could not establish network connection.

**Common Causes**:
- Service is not running
- Firewall blocking connection
- Incorrect URL or port

**Solutions**:
```bash
# Check service status
curl http://localhost:11434/health

# Verify port is open
netstat -an | grep 11434

# Check firewall
sudo ufw status

# Use different port
uveddi analyze ./src --ollama-api-url http://localhost:11435
```

### 6002 - DNS_RESOLUTION_FAILED

**Description**: Could not resolve hostname.

**Common Causes**:
- DNS configuration issues
- Network connectivity problems
- Invalid hostname

**Solutions**:
```bash
# Test DNS resolution
nslookup api.openai.com

# Use IP address instead
uveddi analyze ./src --openai-api-url https://23.102.140.112

# Check network connectivity
ping google.com

# Use different DNS server
echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
```

### 6003 - SSL_CERTIFICATE_ERROR

**Description**: SSL/TLS certificate validation failed.

**Common Causes**:
- Self-signed certificates
- Expired certificates
- Certificate chain issues

**Solutions**:
```bash
# Skip certificate validation (not recommended)
uveddi analyze ./src --insecure

# Update CA certificates
sudo apt-get update ca-certificates

# Check certificate
openssl s_client -connect api.openai.com:443
```

---

## Memory Errors (7xxx)

Errors related to memory allocation and management.

### 7001 - OUT_OF_MEMORY

**Description**: System ran out of available memory.

**Common Causes**:
- Large codebase analysis
- Memory leaks
- Insufficient system RAM
- Memory fragmentation

**Solutions**:
```bash
# Use memory-optimized settings
uveddi analyze ./src --memory-profile small

# Increase swap space
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Process smaller chunks
find ./src -name "*.rs" | head -10 | xargs uveddi analyze

# Close other applications
```

### 7002 - MEMORY_ALLOCATION_FAILED

**Description**: Memory allocation request failed.

**Common Causes**:
- Memory fragmentation
- System memory limits
- Process memory limits

**Solutions**:
```bash
# Check memory usage
free -h

# Check process limits
ulimit -v

# Restart analysis
uveddi analyze ./src --memory-limit-gb 4

# Use different memory allocator
export MALLOC_ARENA_MAX=2
uveddi analyze ./src
```

---

## Security Errors (8xxx)

Errors related to security policies and access control.

### 8001 - SANDBOX_VIOLATION

**Description**: Operation violates security sandbox restrictions.

**Common Causes**:
- Plugin attempting unauthorized access
- File access outside allowed paths
- Network access violations

**Solutions**:
```bash
# Check sandbox configuration
uveddi config show | grep -A 10 security

# Adjust plugin permissions
[plugins.custom_detector.permissions]
file_read = ["additional/path/**"]

# Disable sandbox (not recommended)
uveddi analyze ./src --disable-sandbox
```

### 8002 - UNSAFE_OPERATION_BLOCKED

**Description**: Potentially unsafe operation was blocked by security policy.

**Common Causes**:
- Executing external commands
- Writing to restricted paths
- Network access to blocked domains

**Solutions**:
```bash
# Review security policy
uveddi config show | grep -A 20 security

# Whitelist specific operations
[security]
allowed_commands = ["git", "rustc"]

# Use safe mode
uveddi analyze ./src --safe-mode
```

---

## Common Solutions

### 1. Enable Debug Logging

The most effective way to diagnose issues:

```bash
# Enable detailed logging
export RUST_LOG=debug
uveddi analyze ./src 2> debug.log

# Specific module logging
export RUST_LOG=uveddi::parser=debug,uveddi::ai=info
uveddi analyze ./src

# Enable trace logging (very verbose)
export RUST_LOG=trace
uveddi analyze ./src
```

### 2. Check Known Issues

Before diving deep into troubleshooting:

```bash
# List current known issues
uveddi known-issues

# Filter by category
uveddi known-issues --category analysis

# Check if your issue is listed
uveddi known-issues | grep -i "timeout"
```

### 3. Generate Debug Report

Create comprehensive diagnostic information:

```bash
# Generate complete debug report
uveddi debug-report > debug-report.txt

# Include configuration
uveddi debug-report --include-config > debug-report.txt

# Include recent logs
uveddi debug-report --include-logs > debug-report.txt
```

### 4. Update to Latest Version

Many issues are resolved in newer versions:

```bash
# Check current version
uveddi --version

# For alpha release, build from source
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
git pull origin main
cargo build --release --features=production
```

### 5. Clear Caches

Corrupted caches can cause various issues:

```bash
# Clear all caches
rm -rf ~/.cache/uveddi/

# Clear specific caches
rm -rf ~/.cache/uveddi/ast/
rm -rf ~/.cache/uveddi/analysis/

# Disable caching temporarily
uveddi analyze ./src --no-cache
```

### 6. Verify System Resources

Many issues stem from resource limitations:

```bash
# Check memory usage
free -h

# Check disk space
df -h

# Check file descriptor limits
ulimit -n

# Check CPU usage
htop
```

### 7. Test with Minimal Configuration

Isolate configuration issues:

```bash
# Use defaults
uveddi analyze ./src --no-config

# Minimal configuration
uveddi analyze ./src \
  --memory-profile small \
  --timeout 300 \
  --disable-ai \
  --output-format text
```

---

## Debugging Techniques

### AST Parsing Issues

```bash
# Test specific file
uveddi analyze single_file.rs --verbose

# Enable parser debugging
RUST_LOG=uveddi::parser=debug uveddi analyze ./src

# Check tree-sitter output
RUST_LOG=tree_sitter=debug uveddi analyze ./src

# Save AST for inspection
uveddi analyze ./src --dump-ast /tmp/ast-dump/
```

### Memory Issues

```bash
# Monitor memory usage
RUST_LOG=uveddi::memory=debug uveddi analyze ./src

# Use memory profiler
valgrind --tool=massif uveddi analyze ./src
ms_print massif.out.* > memory-profile.txt

# Check for leaks
valgrind --leak-check=full uveddi analyze ./src
```

### Performance Issues

```bash
# Profile performance
perf record uveddi analyze ./src
perf report

# Enable timing information
uveddi analyze ./src --include-timing --verbose

# Check system calls
strace -c uveddi analyze ./src 2>&1 | tail -20
```

### AI Integration Issues

```bash
# Test AI provider directly
curl -X POST http://localhost:11434/api/generate \
  -d '{"model": "deepseek-coder:6.7b", "prompt": "hello"}'

# Enable AI debugging
RUST_LOG=uveddi::ai=debug uveddi analyze ./src --enable-ai

# Test with simple prompt
uveddi analyze simple.rs --enable-ai --verbose
```

### Plugin Issues

```bash
# List plugin status
uveddi plugin list --verbose

# Test plugin in isolation
uveddi plugin test custom-detector ./test-file.rs

# Enable plugin debugging
RUST_LOG=uveddi::plugins=debug uveddi analyze ./src

# Check WASM validity
wasm-validate ~/.uveddi/plugins/custom-detector.wasm
```

---

## Support and Reporting

### Before Reporting Issues

1. **Check Known Issues**: `uveddi known-issues`
2. **Generate Debug Report**: `uveddi debug-report > debug.txt`
3. **Try Latest Version**: Update to most recent version
4. **Minimal Reproduction**: Simplify to smallest failing case

### Issue Report Template

```markdown
## Issue Description
Brief description of the problem

## Environment
- OS: Ubuntu 22.04
- Uveddi Version: 1.0.0-alpha
- Rust Version: 1.75.0
- Available Memory: 16GB

## Reproduction Steps
1. Run: `uveddi analyze ./src --enable-ai`
2. Observe error: "AI provider unavailable"

## Expected Behavior
Analysis should complete with AI insights

## Actual Behavior
Analysis fails with error code 3001

## Debug Information
```bash
uveddi debug-report
```
[Attach debug report output]

## Additional Context
Any other relevant information
```

### Getting Help

#### GitHub Issues
- **Bug Reports**: https://github.com/botzrDev/uveddi/issues
- **Feature Requests**: https://github.com/botzrDev/uveddi/discussions
- **Questions**: https://github.com/botzrDev/uveddi/discussions

#### Community Support
- **Discord**: https://discord.gg/uveddi
- **Reddit**: r/uveddi
- **Stack Overflow**: Tag with `uveddi`

#### Commercial Support
For enterprise users needing priority support:
- **Email**: support@uveddi.dev
- **SLA**: Response within 4 hours
- **Phone**: Available for critical issues

### Diagnostic Commands Quick Reference

```bash
# System diagnostics
uveddi doctor                    # Check system health
uveddi debug-report             # Generate debug report
uveddi known-issues             # List known issues
uveddi config show             # Show current configuration

# Analysis diagnostics
uveddi analyze ./src --dry-run  # Show what would be analyzed
uveddi validate-config          # Validate configuration
RUST_LOG=debug uveddi analyze   # Debug logging

# Resource monitoring
free -h                         # Memory usage
df -h                          # Disk usage
htop                           # CPU usage
lsof | grep uveddi             # Open files
```

---

This error reference provides comprehensive information for diagnosing and resolving issues with Uveddi. For additional help, see the [User Guide](../user-guide/) and [API Reference](api-reference.md).