#!/bin/bash

# Uveddi Development Environment Setup Script
# This script sets up a complete development environment for new team members

set -e

echo "🚀 Setting up Uveddi development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on supported OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

print_status "Detected OS: $MACHINE"

# Check for required tools
check_tool() {
    if command -v "$1" &> /dev/null; then
        print_success "$1 is installed"
        return 0
    else
        print_error "$1 is not installed"
        return 1
    fi
}

print_status "Checking required tools..."

# Check Rust installation
if ! check_tool "rustc"; then
    print_error "Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi

if ! check_tool "cargo"; then
    print_error "Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Git
if ! check_tool "git"; then
    print_error "Git is not installed. Please install Git first."
    exit 1
fi

# Install Rust components
print_status "Installing Rust components..."
rustup component add rustfmt clippy llvm-tools-preview

# Install cargo tools
print_status "Installing cargo tools..."
CARGO_TOOLS=(
    "cargo-audit"
    "cargo-deny"
    "cargo-edit"
    "cargo-llvm-cov"
    "cargo-outdated"
)

for tool in "${CARGO_TOOLS[@]}"; do
    if ! cargo install --list | grep -q "^$tool "; then
        print_status "Installing $tool..."
        cargo install --locked "$tool"
    else
        print_success "$tool is already installed"
    fi
done

# Install pre-commit if Python is available
if command -v python3 &> /dev/null || command -v python &> /dev/null; then
    print_status "Installing pre-commit hooks..."
    
    # Try to install pre-commit
    if command -v pip3 &> /dev/null; then
        pip3 install --user pre-commit
    elif command -v pip &> /dev/null; then
        pip install --user pre-commit
    else
        print_warning "pip not found, skipping pre-commit installation"
    fi
    
    # Install pre-commit hooks if pre-commit is available
    if command -v pre-commit &> /dev/null; then
        pre-commit install
        print_success "Pre-commit hooks installed"
    else
        print_warning "Pre-commit not available, skipping hook installation"
    fi
else
    print_warning "Python not found, skipping pre-commit installation"
fi

# Build the project
print_status "Building Uveddi..."
if cargo build --all-features; then
    print_success "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# Run tests
print_status "Running tests..."
if cargo test --all-features; then
    print_success "All tests passed"
else
    print_warning "Some tests failed - this might be expected in development"
fi

# Check code formatting
print_status "Checking code formatting..."
if cargo fmt --check; then
    print_success "Code is properly formatted"
else
    print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
fi

# Run clippy
print_status "Running Clippy lints..."
if cargo clippy --all-features -- -D warnings; then
    print_success "No Clippy warnings found"
else
    print_warning "Clippy warnings found. Please review and fix."
fi

# Security audit
print_status "Running security audit..."
if cargo audit; then
    print_success "No security vulnerabilities found"
else
    print_warning "Security audit found issues. Please review."
fi

# Architecture validation
if [ -f "scripts/validate_architecture.sh" ]; then
    print_status "Running architecture validation..."
    if chmod +x scripts/validate_architecture.sh && ./scripts/validate_architecture.sh; then
        print_success "Architecture validation passed"
    else
        print_warning "Architecture validation found issues"
    fi
fi

# Setup local configuration
print_status "Setting up local configuration..."
if [ ! -f ".env.local" ]; then
    cat > .env.local << EOF
# Local development environment variables
# Copy this to .env and customize as needed

# AI Provider API Keys (optional for development)
# OPENAI_API_KEY=your_openai_key_here
# ANTHROPIC_API_KEY=your_anthropic_key_here

# Ollama configuration for local AI
OLLAMA_API_URL=http://localhost:11434
OLLAMA_MODEL=deepseek-coder:6.7b-instruct-q4_0

# Development settings
RUST_LOG=debug
RUST_BACKTRACE=1
EOF
    print_success "Created .env.local template"
else
    print_success ".env.local already exists"
fi

# Create development database
print_status "Setting up development database..."
if [ -d "backend" ]; then
    cd backend
    if [ -f "setup_database.sh" ]; then
        chmod +x setup_database.sh
        ./setup_database.sh || print_warning "Database setup failed - this might be expected"
    fi
    cd ..
fi

echo ""
print_success "🎉 Development environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Review and customize .env.local with your API keys"
echo "2. Run 'cargo run -- analyze .' to test the CLI"
echo "3. Check out docs/ARCHITECTURE.md for project overview"
echo "4. Join the team communication channels"
echo ""
echo "Happy coding! 🦀"