# Uveddi Alpha Installation Guide

## Alpha Release Notice

**Warning: This is an alpha release** - The CLI interface is fully functional, but the analysis engine is in development. See [Alpha Testing Guide](../../ALPHA_TESTING_GUIDE.md) for complete instructions.

## Prerequisites

### Required
- **Rust toolchain**: 1.70.0 or later
- **Git**: For source code management
- **C/C++ compiler**: Required for native dependencies
- **Memory**: At least 4GB RAM (8GB+ recommended)
- **Storage**: 2GB free space for build artifacts

### Optional (for future features)
- **PostgreSQL**: For database features (not active in alpha)
- **Ollama**: For local AI analysis (CLI ready, engine in development)

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

## Alpha Installation

### From Source (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/botzrDev/uveddi.git
   cd uveddi
   ```

2. Build the alpha release:
   ```bash
   cargo build --release --features="alpha"
   ```
   **Build time**: 5-15 minutes depending on system

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

### Not Available in Alpha
- **Cargo package**: Not published to crates.io yet
- **Docker images**: Not available for alpha
- **Package managers**: Use source build only

## Verification

### Test CLI Interface
```bash
# Test main help (should work)
./target/release/uveddi --help

# Test analysis help (should work)
./target/release/uveddi analyze --help

# Test configuration (should work)
./target/release/uveddi config --help

# Test analysis (will show expected "execution failed" error)
./target/release/uveddi analyze /path/to/project --output-format=json
```

## Expected Alpha Behavior

### Working Features
- All CLI commands and help system
- Argument parsing and validation
- Configuration management
- Error handling and messages

### Expected Issues
- **Analysis commands will fail** with "Analysis execution failed" - this is expected
- **TUI requires separate binary**: `cargo run --bin tui_test --features="tui"`

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

## Environment Configuration (Future)

These will be used when the analysis engine is complete:
```bash
# AI provider setup (for future use)
export OPENAI_API_KEY='your-key'
export ANTHROPIC_API_KEY='your-key'
export OLLAMA_API_URL='http://localhost:11434'
```

## Next Steps

1. **Test the CLI** - Verify all help commands work
2. **Report issues** - Any CLI crashes or build problems
3. **Follow development** - Analysis engine is next milestone
4. **See [Alpha Testing Guide](../../ALPHA_TESTING_GUIDE.md)** - Complete testing instructions
