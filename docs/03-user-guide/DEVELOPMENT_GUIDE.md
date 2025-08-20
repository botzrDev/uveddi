# Uveddi Development Master Guide

## Overview
Uveddi is a comprehensive architectural analysis tool built in Rust that combines static code analysis with AI-powered insights. This guide provides definitive instructions for building, testing, and validating the system.

## Prerequisites
- Rust 1.70+ with Cargo
- Node.js 18+ (for web features)
- Git
- SQLite3

## Quick Start (Most Common Use Cases)

### 1. Fast Development Build (Recommended for Development)
```bash
# Ultra-fast build for development iteration (60-80% faster)
cargo build --features=dev-minimal --profile=dev-fast

# Test the build works
./target/dev-fast/uveddi --help
```

### 2. Full Production Build
```bash
# Complete production build with all features
cargo build --release --features=production

# Test the build works
./target/release/uveddi --help
```

### 3. Language-Specific Builds (70-85% faster than full parsing)
```bash
# For JavaScript/TypeScript projects only
cargo build --features=tree-sitter,javascript-lang,typescript-lang --profile=dev-fast

# For Python projects only
cargo build --features=tree-sitter,python-lang --profile=dev-fast

# For Rust projects only
cargo build --features=tree-sitter,rust-lang --profile=dev-fast
```

## Feature Flags Reference

### Development Feature Sets (Build-Optimized)
- **`dev-minimal`**: Ultra-minimal build (essential dependencies only, 60-80% faster)
- **`dev-core`**: Core analysis features without heavy parsing
- **`dev-fast`**: Fast iteration build (minimal deps + basic utilities)
- **`dev-rust-only`**: Rust-only analysis (70-85% faster than multi-language)
- **`dev-python-only`**: Python-only analysis
- **`dev-js-only`**: JavaScript-only analysis
- **`dev-ts-only`**: TypeScript-only analysis

### Production Feature Sets
- **`production`**: Full feature set for deployment
- **`community`**: TUI, tree-sitter, local-ai, security, memory-optimization
- **`tree-sitter`**: Multi-language AST parsing (aggregates all language parsers)

### Individual Language Features
- **`rust-lang`**: Rust language parsing
- **`python-lang`**: Python language parsing
- **`javascript-lang`**: JavaScript language parsing
- **`typescript-lang`**: TypeScript language parsing

### Additional Features
- **`ai`**: Base AI functionality
- **`local-ai`**: Ollama integration
- **`tui`**: Terminal user interface
- **`security`**: Security analysis features
- **`web-full`**: Complete web server stack

## Build Commands Reference

### Development Builds
```bash
# Fastest possible build for quick iteration
cargo build --features=dev-minimal --profile=dev-fast

# Balanced development build with analysis features
cargo build --features=dev-core --profile=dev-fast

# Development build with specific language support
cargo build --features=tree-sitter,rust-lang --profile=dev-fast
```

### Testing Builds
```bash
# Build for running tests
cargo build --features=tree-sitter --profile=dev-fast

# Build with community features for full testing
cargo build --features=community --profile=dev-fast
```

### Production Builds
```bash
# Full production build
cargo build --release --features=production

# Community version build
cargo build --release --features=community
```

## Testing Reference

### Running Analysis Tests

#### 1. Test Single File Analysis
```bash
# Build with language support first
cargo build --features=tree-sitter --profile=dev-fast

# Test JavaScript analysis
./target/dev-fast/uveddi analyze path/to/file.js --output-format json

# Test Python analysis
./target/dev-fast/uveddi analyze path/to/file.py --output-format json

# Test TypeScript analysis
./target/dev-fast/uveddi analyze path/to/file.ts --output-format json

# Test Rust analysis
./target/dev-fast/uveddi analyze path/to/file.rs --output-format json
```

#### 2. Test All Detectors
```bash
# Verify all 7 detectors are working
CARGO_TEST=1 RUST_LOG=info ./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/javascript_extreme/everything_god_object.js --output-format json

# Expected output should show:
# - GodObjectDetector: 1 issue
# - DeadCodeDetector: 98 issues
# - MagicValuesDetector: 70 issues
# - Other detectors: 0 issues
# Total: 169 issues
```

#### 3. Test Different Languages
```bash
# JavaScript (requires javascript-lang feature)
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/javascript_extreme/ --output-format html

# Python (requires python-lang feature)  
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/python_extreme/ --output-format html

# TypeScript (requires typescript-lang feature)
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/typescript_extreme/ --output-format html
```

### Running Unit Tests
```bash
# Run all unit tests (requires community features for full test suite)
cargo test --features=community

# Run specific test modules
cargo test --features=community detector_factory
cargo test --features=community analysis_engine
```

### Performance Testing
```bash
# Run performance validation
./scripts/performance-validation.sh

# Run build optimization validation
./scripts/validate-build-optimization.sh
```

## Troubleshooting Common Issues

### Build Issues

#### "UnsupportedLanguage" Error
**Problem**: `Parse error: UnsupportedLanguage("JavaScript")`
**Solution**: Build with language features:
```bash
cargo build --features=tree-sitter --profile=dev-fast
# Or for specific language:
cargo build --features=javascript-lang --profile=dev-fast
```

#### Slow Build Times
**Problem**: Build takes too long
**Solution**: Use optimized feature sets:
```bash
# Use minimal build for fastest iteration
cargo build --features=dev-minimal --profile=dev-fast

# Use single-language builds when possible
cargo build --features=dev-rust-only --profile=dev-fast
```

#### Out of Memory During Compilation
**Problem**: Build fails with OOM
**Solution**: Use minimal feature sets:
```bash
cargo build --features=dev-minimal --profile=dev-fast
```

### Runtime Issues

#### No Issues Found
**Problem**: Analysis returns 0 issues when expecting some
**Solution**: Check feature flags and file types:
```bash
# Ensure you have the right language features
cargo build --features=tree-sitter --profile=dev-fast

# Check if file extension is supported
./target/dev-fast/uveddi analyze file.js --verbose
```

#### Database Errors
**Problem**: SQLite constraint violations
**Solution**: Delete database and retry:
```bash
rm -f ~/.uveddi/analysis.db
./target/dev-fast/uveddi analyze your_project/
```

## Detector System Validation

### Verify All 7 Detectors Work
```bash
# 1. Build with tree-sitter support
cargo build --features=tree-sitter --profile=dev-fast

# 2. Test with JavaScript god object file (should find 169 issues)
CARGO_TEST=1 RUST_LOG=info ./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/javascript_extreme/everything_god_object.js --output-format json

# 3. Expected detector results:
# - GodObjectDetector: 1 issue
# - CodeDuplicationDetector: 0 issues  
# - DeadCodeDetector: 98 issues
# - LargeClassDetector: 0 issues
# - TightCouplingDetector: 0 issues
# - MagicValuesDetector: 70 issues
# - LongMethodsDetector: Should be present but may find 0 in this specific file
```

### Test Individual Detectors
```bash
# Test code duplication detector
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/rust_extreme/subtle_code_clones.rs --output-format json

# Test magic values detector  
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/python_extreme/magic_number_madness.py --output-format json

# Test long methods detector
./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/typescript_extreme/function_behemoth.ts --output-format json
```

## Web Services Testing

### Start Web Services
```bash
# Build with web features
cargo build --features=production --profile=dev-fast

# Start all services
./target/dev-fast/uveddi serve --port 8888 --rendering-port 3333

# Test health endpoints
curl http://localhost:8888/health
curl http://localhost:3333/health
```

### Development Mode with Frontend
```bash
# Start with frontend dev server
./target/dev-fast/uveddi serve --port 8888 --rendering-port 3333 --frontend-port 3000 --development

# Access dashboard at http://localhost:8888
```

## AI Testing (Optional)

### Setup Ollama
```bash
# Install and start Ollama
ollama serve

# Pull recommended model
ollama pull deepseek-coder:6.7b

# Set environment
export OLLAMA_API_URL="http://localhost:11434"
export OLLAMA_MODEL="deepseek-coder:6.7b"
```

### Test AI Analysis
```bash
# Build with AI features
cargo build --features=community

# Run with AI explanations
./target/release/uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b
```

## Complete Validation Workflow

### For New Developers
```bash
# 1. Clone and setup
git clone https://github.com/botzrDev/uveddi.git
cd uveddi

# 2. Quick validation build
cargo build --features=tree-sitter --profile=dev-fast

# 3. Test basic functionality
./target/dev-fast/uveddi --help

# 4. Validate all detectors work
CARGO_TEST=1 RUST_LOG=info ./target/dev-fast/uveddi analyze detector-test-codebases/benchmark-codebases/extreme_suite/javascript_extreme/everything_god_object.js --output-format json

# 5. Expected success: Should show 169 total issues found
```

### For Production Deployment
```bash
# 1. Full production build
cargo build --release --features=production

# 2. Run comprehensive tests
cargo test --features=production

# 3. Performance validation
./scripts/performance-validation.sh

# 4. Service validation
./target/release/uveddi serve --port 8888 --rendering-port 3333
```

## Quick Debug Commands

### Check What Features Are Enabled
```bash
# See what detectors are available
./target/dev-fast/uveddi --help | grep -A 20 "DETECTORS"

# Check language support
./target/dev-fast/uveddi analyze --help | grep -i language
```

### Verbose Logging
```bash
# Enable debug logging
RUST_LOG=debug ./target/dev-fast/uveddi analyze your_file.js --verbose

# Enable trace logging for specific modules
RUST_LOG=uveddi::analysis::detectors=trace ./target/dev-fast/uveddi analyze your_file.js
```

This guide should provide clear, unambiguous instructions for anyone working with Uveddi. The key insight is matching build features to use cases and providing specific validation commands.