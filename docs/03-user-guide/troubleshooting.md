# Troubleshooting Guide

## Quick Diagnosis

Before diving into specific issues, run the diagnostic command:

```bash
# Generate a comprehensive diagnostic report
uveddi diagnose --verbose > diagnostic_report.txt

# Quick health check
uveddi health-check
```

## Common Issues and Solutions

### 1. Build and Compilation Issues

#### WSL Build Timeouts (Windows Subsystem for Linux)
**Symptom**: Build process times out after ~500 seconds on WSL
**Impact**: Cannot compile with full features
**Solutions**:

```bash
# Solution 1: Use incremental build script (fastest)
./scripts/wsl-build-incremental.sh

# Solution 2: Install faster linker and build cache
sudo apt-get install lld clang
cargo install sccache
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Solution 3: Use temp directory for faster I/O
export CARGO_TARGET_DIR=/tmp/uveddi-target
cargo build --release --features production

# Solution 4: Progressive feature building
cargo build --release --features dev-minimal
cargo build --release --features "dev-minimal tree-sitter"
cargo build --release --features production
```

**WSL Configuration** (create `~/.wslconfig`):
```ini
[wsl2]
memory=8GB
processors=4
swap=4GB
localhostForwarding=true

[experimental]
autoMemoryReclaim=gradual
sparseVhd=true
```

#### Out of Memory During Compilation
**Symptom**: Compiler killed due to OOM
**Solutions**:
```bash
# Use minimal feature set
cargo build --features dev-minimal

# Reduce codegen units
export CARGO_BUILD_JOBS=2
export RUSTFLAGS="-C codegen-units=1"

# Clear build cache
cargo clean
rm -rf target/
```

#### Tree-sitter API Compatibility Issues
**Symptom**: Compilation errors related to tree-sitter
**Status**: Fixed in current version
**Fallback**:
```bash
# Disable tree-sitter if issues persist
cargo build --no-default-features --features "ai tui security"
```

### 2. Analysis Issues

#### Zero Files Analyzed
**Symptom**: Analysis reports 0 files analyzed despite files being present
**Known Issue**: Affects simple test cases with minimal source files
**Workarounds**:
```bash
# Explicitly specify file extensions
uveddi analyze ./src --extensions rs,py,js,ts

# Use glob patterns
uveddi analyze "src/**/*.rs"

# Force file discovery
uveddi analyze ./src --force-discovery

# Check ignore patterns
uveddi analyze ./src --no-ignore
```

#### Analysis Timeout on Large Codebases
**Symptom**: Analysis times out on projects with >1000 files
**Solutions**:
```bash
# Increase timeout
uveddi analyze ./project --timeout 600

# Enable graceful degradation
uveddi analyze ./project --enable-degradation

# Limit concurrent processing
uveddi analyze ./project --max-concurrent-files 5

# Exclude unnecessary directories
uveddi analyze ./project --exclude "node_modules,target,dist"
```

#### Incomplete Analysis Results
**Symptom**: Some files not analyzed or issues missing
**Solutions**:
```bash
# Clear cache and re-analyze
uveddi cache clear
uveddi analyze ./project --no-cache

# Enable verbose logging
RUST_LOG=debug uveddi analyze ./project

# Check file permissions
find ./project -type f ! -readable
```

### 3. Service and API Issues

#### Service Startup Failures
**Symptom**: API server or rendering service fails to start
**Solutions**:
```bash
# Check port availability
lsof -i :8888
lsof -i :3333

# Use alternative ports
uveddi serve --port 8080 --rendering-port 3030

# Check service dependencies
npx playwright install
npx playwright install-deps

# Manual service startup
uveddi serve --no-auto-start
# Then manually start services
```

#### Health Check Timeouts
**Symptom**: Services report unhealthy despite running
**Solutions**:
```bash
# Increase health check timeout
uveddi serve --health-timeout 60

# Check service logs
uveddi serve --log-level debug

# Test services individually
curl http://localhost:8888/health
curl http://localhost:3333/health
```

#### WebSocket Connection Issues
**Symptom**: Real-time updates not working
**Solutions**:
```bash
# Check WebSocket support
uveddi serve --enable-websocket

# Test WebSocket connection
wscat -c ws://localhost:8888/ws

# Check firewall/proxy settings
# Ensure WebSocket traffic is allowed
```

### 4. AI Integration Issues

#### Ollama Connection Failed
**Symptom**: Cannot connect to Ollama API
**Solutions**:
```bash
# Verify Ollama is running
ollama serve
curl http://localhost:11434/api/tags

# Check model availability
ollama list
ollama pull deepseek-coder:6.7b

# Test with different model
uveddi analyze ./src --ollama-model llama2

# Disable AI temporarily
uveddi analyze ./src --no-ai
```

#### AI Response Timeouts
**Symptom**: AI analysis times out
**Solutions**:
```bash
# Increase AI timeout
uveddi analyze ./src --ai-timeout 60

# Reduce context size
uveddi analyze ./src --max-context-size 1000

# Use faster model
uveddi analyze ./src --ollama-model phi
```

### 5. Plugin System Issues

#### Plugin Loading Failures
**Symptom**: WASM plugins fail to load
**Solutions**:
```bash
# Check plugin directory
ls -la ./plugins/

# Validate plugin format
file ./plugins/*.wasm

# Enable plugin debug logging
RUST_LOG=plugins=debug uveddi analyze ./src

# Disable plugins temporarily
uveddi analyze ./src --no-plugins
```

#### Plugin Performance Issues
**Symptom**: Analysis slow with plugins enabled
**Solutions**:
```bash
# Limit plugin memory
uveddi analyze ./src --plugin-memory-limit 50

# Disable specific plugins
uveddi analyze ./src --disable-plugin security_scanner

# Profile plugin performance
uveddi analyze ./src --profile-plugins
```

### 6. Memory and Performance Issues

#### High Memory Usage
**Symptom**: Excessive RAM consumption during analysis
**Solutions**:
```bash
# Enable memory optimization
cargo build --features memory-optimization
uveddi analyze ./src --memory-limit 500

# Use conservative memory settings
uveddi analyze ./src --memory-strategy conservative

# Clear caches periodically
uveddi analyze ./src --clear-cache-interval 100
```

#### Memory Leaks
**Symptom**: Memory usage grows continuously
**Solutions**:
```bash
# Enable garbage collection
uveddi analyze ./src --enable-gc

# Monitor memory usage
uveddi analyze ./src --monitor-memory

# Use memory profiling
RUST_LOG=memory=debug uveddi analyze ./src
```

### 7. Database Issues

#### Database Locking
**Symptom**: "Database is locked" errors
**Solutions**:
```bash
# Check for other processes
fuser uveddi.db

# Enable WAL mode (Write-Ahead Logging)
sqlite3 uveddi.db "PRAGMA journal_mode=WAL;"

# Use PostgreSQL for concurrent access
uveddi config set database.type postgres
```

#### Database Corruption
**Symptom**: Database errors or crashes
**Solutions**:
```bash
# Backup and recreate database
cp uveddi.db uveddi.db.backup
rm uveddi.db
uveddi database init

# Run integrity check
sqlite3 uveddi.db "PRAGMA integrity_check;"

# Vacuum database
sqlite3 uveddi.db "VACUUM;"
```

### 8. Network and Connectivity Issues

#### Proxy Configuration
**Symptom**: Cannot reach external services through proxy
**Solutions**:
```bash
# Set proxy environment variables
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
export NO_PROXY=localhost,127.0.0.1

# Configure in uveddi.toml
[network]
proxy_url = "http://proxy.company.com:8080"
```

#### SSL/TLS Certificate Issues
**Symptom**: Certificate verification failures
**Solutions**:
```bash
# Disable certificate verification (development only)
export UVEDDI_SSL_VERIFY=false

# Add custom CA certificate
export SSL_CERT_FILE=/path/to/ca-bundle.crt

# Update system certificates
update-ca-certificates
```

### 9. Terminal UI (TUI) Issues

#### TUI Display Corruption
**Symptom**: Garbled or broken UI in terminal
**Solutions**:
```bash
# Reset terminal
reset
clear

# Set proper terminal type
export TERM=xterm-256color

# Check terminal size
tput cols && tput lines  # Should be at least 80x24

# Use alternative terminal
# Try: alacritty, kitty, or Windows Terminal
```

#### TUI Keyboard Not Responding
**Symptom**: Keyboard shortcuts not working
**Solutions**:
```bash
# Check terminal key bindings
stty -a

# Reset terminal settings
stty sane

# Try alternative key bindings
# Use arrows instead of hjkl
# Use Ctrl+C to exit if stuck
```

### 10. Configuration Issues

#### Configuration Not Loading
**Symptom**: Settings in config file ignored
**Solutions**:
```bash
# Validate configuration syntax
uveddi config validate

# Check configuration precedence
uveddi config show --sources

# Test with minimal config
echo "[analysis]\nlanguages = [\"rust\"]" > test.toml
uveddi analyze ./src --config test.toml
```

#### Environment Variables Not Working
**Symptom**: UVEDDI_* variables have no effect
**Solutions**:
```bash
# Check variable format (use underscores)
export UVEDDI_ANALYSIS_MAX_DEPTH=10  # Correct
export UVEDDI_ANALYSIS_MAX-DEPTH=10  # Wrong

# Verify variables are set
env | grep UVEDDI

# Test with explicit variable
UVEDDI_OUTPUT_FORMAT=json uveddi analyze ./src
```

### 11. Report Generation Issues

#### Mermaid Diagram Rendering Failures
**Symptom**: Diagrams not appearing in reports
**Solutions**:
```bash
# Install Playwright dependencies
npx playwright install chromium
npx playwright install-deps

# Start rendering service manually
node rendering-service/server.js

# Use alternative diagram format
uveddi analyze ./src --diagram-format graphviz
```

#### HTML Report Not Opening
**Symptom**: Browser fails to open report
**Solutions**:
```bash
# Open manually
firefox reports/analysis.html
chrome reports/analysis.html

# Use markdown format instead
uveddi analyze ./src --output-format markdown

# Serve reports locally
cd reports && python -m http.server 8000
```

### 12. Testing Issues

#### Test Failures
**Symptom**: Unit tests failing (18 known failures)
**Solutions**:
```bash
# Run stable test subset
cargo test --features dev-core --lib

# Skip known failing tests
cargo test -- --skip registry --skip template --skip observability

# Run integration tests (100% passing)
cargo test --features production --test integration
```

#### Test Timeout
**Symptom**: Tests hang or timeout
**Solutions**:
```bash
# Increase test timeout
cargo test -- --test-threads=1 --nocapture

# Run specific test
cargo test test_name -- --exact

# Enable test logging
RUST_LOG=debug cargo test
```

## Platform-Specific Issues

### macOS

#### Gatekeeper Blocks Execution
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine uveddi

# Or allow in System Preferences > Security & Privacy
```

#### Library Loading Issues
```bash
# Install dependencies via Homebrew
brew install tree-sitter
brew install sqlite
```

### Linux

#### Missing System Libraries
```bash
# Ubuntu/Debian
sudo apt-get install build-essential libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install gcc openssl-devel pkg-config

# Arch
sudo pacman -S base-devel openssl pkgconf
```

#### Permission Denied
```bash
# Make executable
chmod +x uveddi

# Check SELinux/AppArmor
setenforce 0  # Temporary disable for testing
aa-complain /usr/bin/uveddi
```

### Windows

#### Path Length Limitations
```powershell
# Enable long path support (Windows 10+)
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
  -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

#### Defender Blocking
```powershell
# Add exclusion for development
Add-MpPreference -ExclusionPath "C:\Dev\uveddi"
```

## Error Reference

| Error Code | Message | Solution |
|------------|---------|----------|
| UV001 | Database connection failed | Check database configuration and connectivity |
| UV002 | Analysis timeout exceeded | Increase timeout or reduce scope |
| UV003 | Insufficient memory | Reduce concurrent files or use minimal features |
| UV004 | Plugin load failed | Verify plugin compatibility and format |
| UV005 | AI provider unreachable | Check network and provider configuration |
| UV006 | Invalid configuration | Run `uveddi config validate` |
| UV007 | File access denied | Check file permissions |
| UV008 | Service startup failed | Check port availability and dependencies |
| UV009 | Cache corruption | Clear cache with `uveddi cache clear` |
| UV010 | Feature not available | Rebuild with required features |

## Debug Information Collection

When reporting issues, collect comprehensive debug information:

```bash
# Generate full diagnostic report
uveddi diagnose --full > diagnostic.txt

# System information
uname -a >> diagnostic.txt
rustc --version >> diagnostic.txt
cargo --version >> diagnostic.txt

# Configuration dump
uveddi config show --all >> diagnostic.txt

# Recent logs
tail -n 1000 ~/.uveddi/logs/uveddi.log >> diagnostic.txt

# Memory and CPU info
free -h >> diagnostic.txt
lscpu >> diagnostic.txt

# Disk space
df -h . >> diagnostic.txt
```

## Performance Profiling

For performance issues, generate a profile:

```bash
# CPU profile
uveddi analyze ./src --profile-cpu profile.json

# Memory profile
uveddi analyze ./src --profile-memory memory.json

# Flamegraph generation
cargo install flamegraph
cargo flamegraph --bin uveddi -- analyze ./src
```

## Getting Help

### Self-Help Resources

1. **Check known issues**: 
   ```bash
   uveddi known-issues --check
   ```

2. **Search documentation**:
   ```bash
   uveddi docs search "error message"
   ```

3. **Run compatibility check**:
   ```bash
   uveddi compat-check
   ```

### Community Support

1. **GitHub Issues**: https://github.com/botzrDev/uveddi/issues
   - Include diagnostic report
   - Provide minimal reproduction
   - Specify version and platform

2. **Discord Community**: https://discord.gg/uveddi
   - Real-time help from community
   - Share experiences and solutions

3. **Stack Overflow**: Tag with `uveddi`
   - For programming and integration questions

### Commercial Support

For enterprise support options:
- Email: support@uveddi.io
- Priority support available with enterprise license

## Preventive Measures

### Regular Maintenance

```bash
# Weekly cache cleanup
uveddi cache clean --older-than 7d

# Database optimization
uveddi database optimize

# Update to latest version
cargo install --git https://github.com/botzrDev/uveddi
```

### Monitoring

```bash
# Enable monitoring
uveddi monitor start

# Check system health
uveddi health-check --continuous

# Set up alerts
uveddi alerts configure --email admin@company.com
```

### Backup

```bash
# Backup configuration and database
uveddi backup create --output backup.tar.gz

# Restore from backup
uveddi backup restore --input backup.tar.gz
```

## Version-Specific Notes

### v0.9.0-alpha Known Issues

1. **File Discovery**: May report 0 files for simple test cases
2. **Test Failures**: 18 unit tests failing (does not affect functionality)
3. **WSL Builds**: Timeout issues with full features
4. **Plugin System**: WASM plugins in experimental state
5. **Cache Serialization**: Some cache features may be limited

These issues are being addressed for the v1.0 release.