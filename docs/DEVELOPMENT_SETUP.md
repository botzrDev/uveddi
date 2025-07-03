# Uveddi Development Environment Setup

## Quick Start

For new team members, use the automated setup script:

```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
./scripts/setup-dev-environment.sh
```

This script will install all required tools and configure your development environment.

## Manual Setup Guide

If you prefer manual setup or need to troubleshoot the automated script:

### Prerequisites

#### Required Tools

1. **Rust Toolchain** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Git** (version 2.20+)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install git
   
   # macOS
   brew install git
   
   # Windows
   # Download from https://git-scm.com/
   ```

3. **Python 3.8+** (for pre-commit hooks and backend)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install python3 python3-pip
   
   # macOS
   brew install python3
   
   # Windows
   # Download from https://python.org/
   ```

#### Optional Tools

4. **Docker** (for backend development)
   ```bash
   # Follow instructions at https://docs.docker.com/get-docker/
   ```

5. **Node.js 18+** (for frontend development)
   ```bash
   # Ubuntu/Debian
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   
   # macOS
   brew install node@18
   
   # Windows
   # Download from https://nodejs.org/
   ```

### Rust Development Setup

#### 1. Install Rust Components

```bash
# Essential components
rustup component add rustfmt clippy llvm-tools-preview

# Optional: Additional targets for cross-compilation
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

#### 2. Install Cargo Tools

```bash
# Essential development tools
cargo install --locked cargo-audit      # Security auditing
cargo install --locked cargo-deny       # Dependency policy enforcement
cargo install --locked cargo-edit       # Cargo.toml editing
cargo install --locked cargo-llvm-cov   # Code coverage
cargo install --locked cargo-outdated   # Dependency updates

# Optional but recommended
cargo install --locked cargo-watch      # File watching for development
cargo install --locked cargo-expand     # Macro expansion debugging
cargo install --locked cargo-tree       # Dependency tree visualization
```

#### 3. Configure Development Environment

Create local environment configuration:

```bash
# Copy template
cp .env.local .env

# Edit with your preferences
nano .env
```

Example `.env` configuration:
```bash
# Development settings
RUST_LOG=debug
RUST_BACKTRACE=1

# AI Provider API Keys (optional)
# OPENAI_API_KEY=your_openai_key_here
# ANTHROPIC_API_KEY=your_anthropic_key_here

# Local AI configuration
OLLAMA_API_URL=http://localhost:11434
OLLAMA_MODEL=deepseek-coder:6.7b-instruct-q4_0

# Database configuration (for backend development)
DATABASE_URL=sqlite:./uveddi.db
```

### Code Quality Tools

#### 1. Pre-commit Hooks

```bash
# Install pre-commit
pip3 install --user pre-commit

# Install hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

#### 2. Editor Configuration

##### VS Code Setup

Install recommended extensions:
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb",
    "ms-vscode.vscode-json"
  ]
}
```

Create `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-targets", "--all-features"],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

##### Vim/Neovim Setup

For Rust development with vim:
```vim
" Add to your .vimrc or init.vim
Plug 'rust-lang/rust.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}

" Rust-specific settings
let g:rustfmt_autosave = 1
let g:rust_clip_command = 'xclip -selection clipboard'
```

### Building and Testing

#### 1. Build the Project

```bash
# Full build with all features
cargo build --all-features

# Release build
cargo build --release --all-features

# Minimal build (for testing)
cargo build --no-default-features
```

#### 2. Run Tests

```bash
# All tests
cargo test --all-features

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_god_object_detection

# With output
cargo test -- --nocapture
```

#### 3. Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run lints
cargo clippy --all-features -- -D warnings

# Security audit
cargo audit

# Dependency check
cargo deny check
```

#### 4. Performance Testing

```bash
# Run benchmarks
cargo bench

# Generate test data
cargo run --bin generate_benchmark_data

# Profile with specific features
cargo bench --features "analysis,local-ai"
```

### Development Workflow

#### 1. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes and test
cargo test --all-features
cargo clippy --all-features

# Commit with conventional commits
git commit -m "feat: add new analysis detector"

# Push and create PR
git push origin feature/your-feature-name
```

#### 2. Code Review Process

Before submitting a PR:

```bash
# Run full quality check
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo audit
./scripts/validate_architecture.sh

# Check different feature combinations
cargo test --no-default-features
cargo test --features "analysis"
cargo test --features "analysis,local-ai"
```

#### 3. Debugging

```bash
# Debug build with symbols
cargo build --all-features

# Run with debugging
RUST_LOG=debug cargo run -- analyze ./test-project

# Use debugger (with VS Code)
# Set breakpoints and use F5 to start debugging

# Memory profiling
cargo install --locked cargo-profdata
cargo profdata -- analyze ./large-project
```

### Backend Development (Optional)

If working on the Python FastAPI backend:

#### 1. Setup Python Environment

```bash
cd backend

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

#### 2. Database Setup

```bash
# Using Docker (recommended)
./docker-compose.sh up

# Or manual setup
./setup_database.sh
./run_migrations.sh
```

#### 3. Run Backend

```bash
# Development server
./run_backend.sh

# Or with Docker
make docker-run
```

### Frontend Development (Optional)

If working on the React frontend:

#### 1. Setup Node Environment

```bash
cd frontend

# Install dependencies
npm ci

# Start development server
npm run dev
```

#### 2. Run Tests

```bash
# E2E tests
npm run cy:run

# Open Cypress UI
npm run cy:open
```

### Local AI Setup (Optional)

For local AI development with Ollama:

#### 1. Install Ollama

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows
# Download from https://ollama.ai/download
```

#### 2. Setup Models

```bash
# Start Ollama service
ollama serve

# Pull recommended model
ollama pull deepseek-coder:6.7b-instruct-q4_0

# Test integration
cargo run -- init-local-ai
```

### Troubleshooting

#### Common Issues

**Build Errors**:
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check for conflicts
cargo tree --duplicates
```

**Test Failures**:
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run tests serially (avoid conflicts)
cargo test -- --test-threads=1

# Skip integration tests
cargo test --lib
```

**Performance Issues**:
```bash
# Check compilation time
cargo build --timings

# Profile test execution
cargo test --release

# Use faster linker (Linux)
sudo apt-get install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

**IDE Issues**:
```bash
# Restart rust-analyzer
# VS Code: Ctrl+Shift+P -> "Rust Analyzer: Restart Server"

# Clear rust-analyzer cache
rm -rf ~/.cache/rust-analyzer

# Regenerate Cargo.lock
rm Cargo.lock
cargo build
```

#### Getting Help

1. **Check Documentation**: Review docs/ folder for specific guides
2. **Architecture Validation**: Run `./scripts/validate_architecture.sh`
3. **CI Logs**: Check GitHub Actions for detailed error messages
4. **Team Communication**: Use established team channels
5. **Issue Tracker**: Create GitHub issues for bugs or questions

### Performance Optimization

#### Development Build Speed

```bash
# Use faster linker
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"  # Linux
export RUSTFLAGS="-C link-arg=-fuse-ld=zld"  # macOS

# Parallel compilation
export CARGO_BUILD_JOBS=8

# Incremental compilation
export CARGO_INCREMENTAL=1
```

#### Test Execution Speed

```bash
# Parallel test execution
cargo test -- --test-threads=8

# Skip slow integration tests during development
cargo test --lib

# Use release mode for benchmarks
cargo test --release performance_tests
```

### Security Best Practices

#### API Key Management

```bash
# Never commit API keys
echo "*.env" >> .gitignore
echo ".env.local" >> .gitignore

# Use environment variables
export OPENAI_API_KEY="your-key-here"

# Or use secure storage
# macOS: Use Keychain Access
# Linux: Use gnome-keyring or similar
# Windows: Use Windows Credential Manager
```

#### Dependency Security

```bash
# Regular security audits
cargo audit

# Check for updates
cargo outdated

# Review new dependencies
cargo deny check
```

---

*This setup guide ensures a consistent development environment across all team members. For questions or improvements, please update this documentation.*