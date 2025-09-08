# Known Issues and Limitations

> **Version**: 0.9.0-alpha  
> **Last Updated**: January 2025  
> **Status**: Active Development

## 🚨 Critical Issues

These issues significantly impact functionality and should be considered before using Uveddi:

### 1. Anti-Pattern Detection Requires Specific Build Flags
**Impact**: HIGH - Core functionality affected  
**Symptoms**: 
- Analysis reports "0 issues found" despite obvious anti-patterns
- `UnsupportedLanguage` errors during analysis

**Root Cause**: Anti-pattern detection depends on AST parsing via tree-sitter

**Solution**:
```bash
# Build with tree-sitter support
cargo build --release --features=production
# OR
cargo build --release --features=tree-sitter
```

**Workaround**: Always use production builds for real analysis

---

### 2. Test Suite Failures (18 of 679 tests failing)
**Impact**: MEDIUM - Development confidence affected  
**Pass Rate**: 97.3%

**Failing Test Categories**:
| Category | Count | Impact | Details |
|----------|-------|--------|---------|
| Detector Registry | 5 | Low | Count mismatches in assertions |
| Template Loading | 4 | Low | Test environment path issues |
| Observability Init | 3 | Low | Test-only initialization |
| Cache Serialization | 2 | Medium | Some cache features limited |
| Plugin Loading | 2 | Low | WASM plugin test failures |
| Memory Allocator | 2 | Low | Test allocator conflicts |

**Note**: Core functionality works despite test failures

---

### 3. Web Dashboard Non-Functional
**Impact**: HIGH - Major feature unavailable  
**Symptoms**:
- Dashboard fails to load
- JavaScript errors in console
- Blank page or 404 errors

**Root Cause**: React frontend is incomplete and unstable

**Workaround**: Use JSON/HTML report output instead:
```bash
uveddi analyze . --output-format html --output report.html
```

---

### 4. TypeScript Support Incomplete
**Impact**: MEDIUM - Limited language support  
**Symptoms**:
- Complex type definitions not parsed
- Generic types cause errors
- Decorators not recognized

**Affected Constructs**:
- Complex generics
- Conditional types
- Mapped types
- Decorators
- Type guards

**Workaround**: Simplify TypeScript code or use JavaScript analysis

---

## ⚠️ Known Limitations

### Build System Issues

#### WSL Build Timeouts
**Platform**: Windows Subsystem for Linux  
**Symptoms**: Build hangs after ~500 seconds

**Solutions**:
1. Use incremental build script:
```bash
./scripts/wsl-build-incremental.sh
```

2. Install faster linker:
```bash
sudo apt-get install lld clang
cargo install sccache
```

3. Configure WSL memory (`~/.wslconfig`):
```ini
[wsl2]
memory=8GB
processors=4
```

#### Feature Compilation Conflicts
**Symptoms**: Compilation errors with certain feature combinations

**Working Combinations**:
- `--features=dev-core` (fast, no anti-patterns)
- `--features=dev-minimal` (minimal features)
- `--features=production` (all features)

**Broken Combinations**:
- Mixing dev and production features
- Individual language features without tree-sitter

---

### Performance Issues

#### Large File Analysis (>1MB)
**Symptoms**: 
- High memory usage
- Slow analysis
- Potential crashes

**Workaround**:
```bash
# Enable memory optimization
cargo build --features="production memory-optimization"
```

#### Large Codebase Timeouts (>10k files)
**Symptoms**: Analysis times out after 300 seconds

**Solutions**:
1. Increase timeout:
```bash
uveddi analyze . --timeout 600
```

2. Analyze subdirectories:
```bash
uveddi analyze ./src
uveddi analyze ./lib
```

---

### Plugin System Issues

#### Plugin Loading Failures
**Symptoms**:
- "Plugin not found" errors
- Plugin crashes on load
- Missing plugin commands

**Common Causes**:
1. Missing feature flag:
```bash
cargo build --features="production wasm-plugins"
```

2. Invalid plugin manifest
3. Incompatible plugin version

#### Plugin Runtime Errors
**Symptoms**:
- Plugins timeout during execution
- Memory limit exceeded
- Permission denied errors

**Solutions**:
- Check plugin resource limits
- Verify security policy
- Update plugin to latest version

---

### Web Services Issues

#### Service Startup Failures
**Symptoms**:
- Port binding errors
- Service health check timeouts
- Rendering service crashes

**Common Causes**:
1. Port already in use:
```bash
# Check port usage
lsof -i :8080
# Use different port
uveddi serve --port 8888
```

2. Missing Playwright dependencies:
```bash
npx playwright install
npx playwright install-deps
```

#### API Endpoint Issues
**Symptoms**:
- 404 errors on documented endpoints
- Unexpected response formats
- Missing CORS headers

**Affected Endpoints**:
- `/api/v1/dashboard` - Not implemented
- `/api/v1/plugins/*` - Partially working
- `/api/v1/auth/*` - Not implemented

---

### AI Integration Issues

#### Ollama Connection Failures
**Symptoms**:
- "Failed to connect to Ollama" errors
- Timeouts during AI analysis
- Empty AI explanations

**Solutions**:
1. Verify Ollama is running:
```bash
ollama serve
```

2. Check model is loaded:
```bash
ollama pull deepseek-coder:6.7b
```

3. Set environment variables:
```bash
export OLLAMA_API_URL="http://localhost:11434"
export OLLAMA_MODEL="deepseek-coder:6.7b"
```

---

### Terminal UI (TUI) Issues

#### Terminal Compatibility
**Symptoms**:
- Garbled display
- Input not working
- Crashes on startup

**Affected Terminals**:
- Some WSL terminals
- Older terminal emulators
- Terminals without UTF-8 support

**Solutions**:
1. Use modern terminal (iTerm2, Windows Terminal)
2. Set UTF-8 locale:
```bash
export LANG=en_US.UTF-8
```

---

## 📋 Error Messages Reference

### Common Error Messages and Solutions

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `UnsupportedLanguage` | Missing tree-sitter | Build with `--features=tree-sitter` |
| `Failed to initialize database` | DB path issues | Check permissions, use absolute path |
| `Plugin not found` | Missing plugin feature | Build with `--features=wasm-plugins` |
| `Port already in use` | Service conflict | Use different port or kill process |
| `Ollama connection failed` | Ollama not running | Start Ollama server |
| `Memory limit exceeded` | Large codebase | Enable memory optimization |
| `Timeout during analysis` | Too many files | Increase timeout or analyze subdirs |

---

## 🔧 Troubleshooting Steps

### General Debugging Process

1. **Check build flags**:
```bash
cargo build --features=production --verbose
```

2. **Enable debug logging**:
```bash
export RUST_LOG=debug
uveddi analyze . 2> debug.log
```

3. **Run doctor command**:
```bash
uveddi doctor
```

4. **Verify dependencies**:
```bash
# Check Rust version
rustc --version  # Should be 1.70+

# Check available features
cargo build --features=production --dry-run
```

5. **Test with minimal example**:
```bash
# Create test file with obvious anti-pattern
echo "class GodObject { 
  // 100+ methods would go here
}" > test.js

# Run analysis
uveddi analyze test.js --output-format json
```

---

## 🐛 Reporting New Issues

When reporting issues, please include:

1. **System Information**:
```bash
uveddi --version
rustc --version
uname -a
```

2. **Build Command**:
```bash
# How you built Uveddi
cargo build --features=...
```

3. **Error Output**:
```bash
RUST_LOG=debug uveddi [command] 2> error.log
```

4. **Minimal Reproduction**:
- Sample code that triggers the issue
- Exact command that fails
- Expected vs actual behavior

Report issues at: https://github.com/org/uveddi/issues

---

## ✅ Fixed Issues

These issues have been resolved in the current version:

- ✅ **Alpha feature build fails** - Fixed compilation errors
- ✅ **WASM plugin features missing** - Added to production builds
- ✅ **Memory optimization conflicts** - Added conditional compilation
- ✅ **Missing logging macros** - Added proper imports

---

## 🔮 Planned Fixes

Issues scheduled for upcoming releases:

### v0.9.5-beta
- Stable plugin system
- Improved TypeScript support
- Web dashboard MVP
- Reduced test failures

### v1.0.0
- All tests passing
- Production-ready web services
- Full TypeScript support
- Complete documentation

---

*This document is actively maintained. Last review: January 2025*