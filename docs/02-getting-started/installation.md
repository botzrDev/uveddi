# Uveddi Alpha Installation Guide

## ⚠️ CRITICAL ALPHA WARNING

**This is v0.9.0-alpha pre-release software** with significant gaps between documentation and implementation:
- **40% of documented features may not work**
- **Web dashboard is completely non-functional**
- **Anti-pattern detection requires specific build flags**
- **18 tests currently failing (97.3% pass rate)**

⚠️ **DO NOT USE IN PRODUCTION** without thorough testing and understanding of limitations.

## Prerequisites

### Required
- **Rust toolchain**: 1.70.0 or later (MANDATORY - no binaries available)
- **Git**: For source code management
- **C/C++ compiler**: Required for native dependencies
- **Memory**: Minimum 4GB RAM, 8GB+ strongly recommended
- **Storage**: 3-5GB free space for build artifacts
- **Patience**: Build times can exceed 20 minutes for full features

> **⚠️ Reality Check**: Memory optimization is experimental and may not work as expected. Large codebases (>10k files) may cause out-of-memory errors even with 16GB RAM.

### Optional (Experimental/Non-functional)
- **PostgreSQL**: Database features exist in code but are NOT functional
- **Ollama**: AI analysis is highly experimental and often fails
- **Node.js/npm**: Required for web services (which don't work properly)
- **Playwright**: Required for diagram rendering (unstable)

## System Dependencies

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

### RHEL/CentOS/Fedora
```bash
sudo dnf install -y gcc g++ openssl-devel git pkg-config
```

### macOS
```bash
# Install Xcode command line tools
xcode-select --install

# Or via Homebrew
brew install git pkg-config openssl
```

### Windows (WSL2 Recommended)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

## Installation (Source Build ONLY)

### ⚠️ No Binary Releases Available

You MUST build from source. There are no pre-built binaries.

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/botzrDev/uveddi.git
   cd uveddi
   ```

2. **CRITICAL: Choose the right build for your needs:**
   
   ```bash
   # For basic testing (FAST, but NO anti-pattern detection):
   cargo build --features=dev-core
   # Build time: ~13 seconds
   
   # For full analysis with anti-patterns (REQUIRED for real use):
   cargo build --release --features=production
   # Build time: 5-20 minutes (may timeout on WSL)
   
   # WSL users: Use special script to avoid timeouts
   ./scripts/wsl-build-incremental.sh
   ```

3. Verify build:
   ```bash
   ls -la target/release/uveddi
   ./target/release/uveddi --help
   ```

### System Installation (Optional)
```bash
# Install to system PATH
cargo install --path . --features="alpha"

# Then use as 'uveddi' instead of './target/release/uveddi'
uveddi --help
```

### What's NOT Available
- **Binary releases**: Must build from source
- **Cargo package**: Not on crates.io
- **Docker images**: Not available
- **Package managers**: No Homebrew, apt, etc.
- **Installer scripts**: Manual build only
- **Auto-updates**: Manual git pull and rebuild

## Verification

### Test What Actually Works
```bash
# Basic CLI (should work)
./target/release/uveddi --help
./target/release/uveddi analyze --help
./target/release/uveddi config show

# Basic analysis (works ONLY with production build):
./target/release/uveddi analyze ./src --output-format json

# If you see "0 issues found" on obvious bad code:
# YOU BUILT WITHOUT tree-sitter SUPPORT!
# Rebuild with: cargo build --release --features=production
```

### Test What DOESN'T Work
```bash
# Web services (will crash or hang):
./target/release/uveddi serve  # ❌ Unstable

# Plugin system (may crash):
./target/release/uveddi plugin list  # ⚠️ Experimental

# TUI (terminal issues):
./target/release/uveddi tui  # ⚠️ May not display correctly
```

## Expected Alpha Behavior

### Actually Working Features ✅
- Basic `analyze` command (with proper build flags)
- `config` command for settings management
- JSON/Markdown/HTML report generation
- Basic anti-pattern detection (production build only)

### Partially Working ⚠️
- Plugin system (crashes frequently)
- Web services (very unstable)
- AI integration (requires Ollama, often fails)
- TUI interface (display issues)

### Not Working ❌
- Web dashboard UI
- Real-time analysis
- Enterprise features (auth, RBAC)
- Many documented API endpoints
- TypeScript complex type analysis

## Known Limitations

### Critical Alpha Limitations
- **Anti-Pattern Detection**: ONLY works with `--features=tree-sitter` or `production`
  - Without these flags, analysis finds 0 issues even on bad code
  - This is the #1 source of user confusion
  
- **Documentation vs Reality**: 40% gap between what's documented and what works
  - Many features in docs are planned but not implemented
  - Configuration options may be ignored
  - API endpoints may not exist

### Build Issues
- **Test Failures**: 18 of 679 tests fail consistently
  - Detector registry count mismatches (5 tests)
  - Template loading failures (4 tests)  
  - Observability initialization (3 tests)
  - Cache and plugin issues (6 tests)
  
- **Build Time**: Can be excessive
  - `dev-core`: ~13 seconds (no anti-patterns)
  - `production`: 5-20 minutes (all features)
  - WSL: May timeout after 500 seconds

### Platform-Specific Issues
- **WSL Build Timeouts**: Windows Subsystem for Linux users may experience build timeouts
  - Use `./scripts/wsl-build-incremental.sh` for WSL-optimized builds
  - See [Troubleshooting WSL](#wsl-build-timeout-issues-windows-subsystem-for-linux) section
  - Build times can exceed 10 minutes on some WSL configurations

### Memory and Performance
- **Large Codebases**: Memory optimization features are available but untested at scale
- **Build Dependencies**: Full feature builds require significant memory (4GB+ RAM recommended)
- **Feature Sets**: Use `--features=dev-minimal` for faster development builds

### Service Architecture
- **Web Services**: API server and rendering service architecture is implemented but analysis endpoints will fail
- **Database Integration**: PostgreSQL integration exists but is not active in alpha
- **Plugin System**: WebAssembly plugin architecture exists but plugin loading may be unstable

### Documentation and Examples
- **Configuration**: Some documented configuration options may not be fully implemented
- **API Endpoints**: REST API documentation describes planned beta functionality
- **Examples**: Code examples demonstrate intended workflow but will fail until analysis engine is complete

### Compatibility
- **Tree-sitter Dependencies**: Heavy native dependencies may cause build issues on some systems
- **Terminal Compatibility**: TUI may not work properly in all terminal environments
- **Operating System**: Windows native builds are not tested; WSL2 is recommended

## Troubleshooting

### Build Issues
```bash
# Clean build if problems occur
cargo clean
cargo build --release --features="alpha"

# Update Rust if build fails
rustup update stable
```

### Memory Issues
```bash
# Reduce build parallelism if out of memory
export CARGO_BUILD_JOBS=1
cargo build --release --features="alpha"
```

### Permission Issues (Linux/macOS)
```bash
# Make binary executable
chmod +x target/release/uveddi
```

## Environment Configuration

### Working Environment Variables
```bash
# Logging (actually works)
export RUST_LOG=debug  # or info, warn, error

# Ollama (experimental, often fails)
export OLLAMA_API_URL='http://localhost:11434'
export OLLAMA_MODEL='deepseek-coder:6.7b'
```

### Non-Functional Variables (Documented but ignored)
```bash
# These are in docs but DON'T WORK:
export OPENAI_API_KEY='your-key'  # ❌ Not implemented
export ANTHROPIC_API_KEY='your-key'  # ❌ Not implemented
export UVEDDI_PLUGINS_DIR='/path'  # ❌ Ignored
```

## Next Steps

1. **Understand limitations** - Read [Known Issues](../known-issues.md)
2. **Check feature status** - See [Feature Status Matrix](../feature-status-matrix.md)
3. **Test carefully** - This is alpha software, expect failures
4. **Report issues** - But check if they're already known first
5. **Lower expectations** - Many documented features don't work yet

## Getting Help

- **Known Issues**: [docs/known-issues.md](../known-issues.md)
- **Feature Status**: [docs/feature-status-matrix.md](../feature-status-matrix.md)
- **GitHub Issues**: https://github.com/org/uveddi/issues
- **Discord**: Community support (see README for link)

⚠️ **Remember**: This is v0.9.0-alpha, not production software!
