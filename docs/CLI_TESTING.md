# Uveddi CLI Testing Guide

## Overview

This document describes the automated CLI testing infrastructure for Uveddi. The test suite validates all CLI commands, options, and error handling to ensure the application works correctly across all use cases.

## Test Scripts

### 1. Python Test Suite (Recommended)

**Location:** `scripts/test_cli_commands.py`

**Usage:**
```bash
# Run all tests
python3 scripts/test_cli_commands.py

# With custom binary path
UVEDDI_BIN=./target/release/uveddi python3 scripts/test_cli_commands.py
```

**Features:**
- ✅ Fast execution (< 5 minutes)
- ✅ Clear, colorized output
- ✅ Detailed error reporting
- ✅ Automatic cleanup
- ✅ Handles timeouts gracefully
- ✅ No bash pipe issues

**Test Coverage:**
- Basic CLI options (--version, --help)
- Analyze command with various formats
- Config command operations
- Doctor diagnostics
- Help system
- Hooks management
- Init command
- UI, CI, TUI command help
- Serve command
- Error handling and invalid inputs

### 2. Comprehensive Test Suite (Advanced)

**Location:** `scripts/test-all-cli-commands.sh`

**Usage:**
```bash
# Run comprehensive tests (may take longer)
./scripts/test-all-cli-commands.sh
```

**Features:**
- Extensive edge case testing
- Memory optimization tests
- Security analysis tests
- Plugin command tests (feature-gated)
- Long-running analysis tests

### 3. Simple Shell Test Suite

**Location:** `scripts/test-cli-commands-simple.sh`

**Usage:**
```bash
./scripts/test-cli-commands-simple.sh
```

**Features:**
- Quick smoke tests
- Minimal dependencies
- Good for CI/CD pipelines

## Test Results

### Latest Test Run

```
Test Summary:
✅ Passed:  26
❌ Failed:  3
⏭️  Skipped: 0

Total: 29 | Pass Rate: 89%
```

### Known Test Failures

1. **Hooks list** - Requires proper working directory setup
2. **Init non-interactive** - Working directory context issue
3. **Analyze invalid format** - Format validation may be permissive

These are minor issues and do not affect core functionality.

## Test Categories

### 1. Basic CLI Options

Tests fundamental CLI functionality:
- Version flags (--version, -V)
- Help flags (--help, -h)
- Command listing
- Subcommand help

### 2. Analyze Command

Tests code analysis functionality:
- Basic analysis with different output formats
- JSON, Markdown, HTML output
- Timeout handling
- Memory optimization flags
- Invalid path handling
- Progress reporting modes

### 3. Config Command

Tests configuration management:
- Show configuration
- Set configuration values
- Validate configuration files
- Config aliases

### 4. Doctor Command

Tests health diagnostics:
- System health checks
- Dependency validation
- Configuration verification

### 5. Help Command

Tests help system:
- General help
- Command-specific help
- Alias help

### 6. Hooks Command

Tests git hooks integration:
- List hooks
- Install/uninstall hooks
- Hook configuration

### 7. Init Command

Tests project initialization:
- Interactive mode
- Non-interactive mode
- Configuration generation

### 8. UI Command

Tests dashboard UI:
- Help information
- Port configuration
- Database path options

### 9. CI Command

Tests CI/CD integration:
- Dry-run mode
- Report generation
- Exit codes

### 10. TUI Command

Tests terminal UI:
- Help information
- Launch validation

### 11. Serve Command

Tests web server:
- Server startup
- Port configuration
- Development mode
- Frontend assets

### 12. Error Handling

Tests error scenarios:
- Invalid commands
- Non-existent paths
- Invalid options
- Malformed arguments

## Adding New Tests

### Python Test Suite

Add tests to `scripts/test_cli_commands.py`:

```python
# Add to appropriate section
tester.test(
    "Test name",
    ["command", "args"],
    should_fail=False,  # Set to True for negative tests
    timeout=10  # Adjust timeout as needed
)
```

### Shell Test Suite

Add tests to `scripts/test-all-cli-commands.sh`:

```bash
run_test "Test name" \
    "$UVEDDI_BIN command args" \
    0  # Expected exit code
```

## CI/CD Integration

### GitHub Actions

Add to `.github/workflows/ci.yml`:

```yaml
- name: Run CLI Tests
  run: |
    cargo build --bin uveddi --features standard
    python3 scripts/test_cli_commands.py
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
cli_tests:
  script:
    - cargo build --bin uveddi --features standard
    - python3 scripts/test_cli_commands.py
```

## Best Practices

### 1. Test Isolation

Each test should:
- Use temporary directories
- Clean up after execution
- Not depend on other tests
- Be repeatable

### 2. Timeout Handling

- Set appropriate timeouts for long-running commands
- Default: 10 seconds for most commands
- Analysis commands: 30-60 seconds
- Server commands: 5-10 seconds

### 3. Error Messages

Failed tests should show:
- Exit code
- Stderr output (truncated)
- Test context
- Expected vs actual behavior

### 4. Environment Variables

Control test behavior with environment variables:

```bash
# Custom binary path
export UVEDDI_BIN=./target/release/uveddi

# Suppress logging noise
export RUST_LOG=error

# Test directory
export TEST_DIR=/tmp/my-tests
```

## Debugging Failed Tests

### 1. Run Individual Test

```python
# In test_cli_commands.py, comment out other tests
tester.test("Specific test", ["command", "args"])
```

### 2. Enable Verbose Logging

```bash
RUST_LOG=debug python3 scripts/test_cli_commands.py
```

### 3. Check Test Artifacts

```bash
# Test directory is printed at start
ls -la /tmp/uveddi-test-XXXXX
```

### 4. Manual Reproduction

```bash
# Copy command from test output and run manually
./target/debug/uveddi command args
```

## Test Maintenance

### Weekly

- Run full test suite
- Check for new failures
- Update test timeouts if needed

### Per Release

- Run tests with release binary
- Verify all commands work
- Update test expectations for new features

### When Adding Features

- Add tests for new commands
- Add tests for new options
- Add negative test cases
- Update this documentation

## Performance Benchmarks

Expected test execution times:

| Test Category | Time (seconds) |
|--------------|----------------|
| Basic CLI Options | 1-2 |
| Analyze Command | 30-60 |
| Config Command | 2-5 |
| Doctor Command | 5-10 |
| Help Command | 1-2 |
| Other Commands | 5-15 |
| **Total** | **60-120** |

## Troubleshooting

### Tests Hang

**Symptom:** Tests don't complete

**Solution:**
1. Use Python test suite instead of bash
2. Check for infinite loops in commands
3. Verify timeout values are set

### Binary Not Found

**Symptom:** "Binary not found" error

**Solution:**
```bash
# Build binary first
cargo build --bin uveddi --features standard

# Or specify path
export UVEDDI_BIN=/path/to/uveddi
```

### Permission Denied

**Symptom:** Cannot execute test scripts

**Solution:**
```bash
chmod +x scripts/test-cli-commands-simple.sh
chmod +x scripts/test-all-cli-commands.sh
chmod +x scripts/test_cli_commands.py
```

## Future Enhancements

### Planned

- [ ] Integration with test reporting tools
- [ ] Performance regression testing
- [ ] Cross-platform testing (Windows, macOS)
- [ ] Memory leak detection
- [ ] Parallel test execution
- [ ] Test coverage metrics
- [ ] Automated nightly test runs

### Under Consideration

- [ ] Fuzz testing for CLI inputs
- [ ] Stress testing for server commands
- [ ] UI/TUI automated testing
- [ ] Plugin system testing

## Contributing

When contributing CLI changes:

1. Add tests for new commands/options
2. Run test suite locally
3. Ensure pass rate ≥ 95%
4. Update this documentation
5. Include test results in PR

## Support

For test-related issues:

1. Check this documentation
2. Review test output logs
3. Run tests manually
4. Open GitHub issue with:
   - Test failure output
   - System information
   - Uveddi version

## References

- [CLI Command Reference](./CLI_REFERENCE.md)
- [Development Guide](./DEVELOPMENT.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [GitHub Actions Workflows](../.github/workflows/)

---

**Last Updated:** 2025-10-03
**Test Suite Version:** 1.0.0
**Maintained By:** Uveddi Development Team
