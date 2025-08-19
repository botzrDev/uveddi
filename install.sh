#!/bin/bash
set -euo pipefail

# Uveddi v1.0 Community Core Installation Script
# Usage: curl -sSL https://uveddi.org/install.sh | bash

# Configuration
REPO_URL="https://github.com/botzrDev/uveddi.git"
INSTALL_DIR="$HOME/.uveddi"
BIN_DIR="$HOME/.local/bin"
VERSION="v1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Error handler
handle_error() {
    log_error "Installation failed at line $1"
    log_error "Please check the error above and try again."
    log_error "For help, visit: https://github.com/botzrDev/uveddi/issues"
    exit 1
}

trap 'handle_error $LINENO' ERR

# Header
echo -e "${BLUE}"
cat << 'EOF'
┌─────────────────────────────────────────┐
│   Uveddi v1.0 Community Core Tool      │
│   Code Analysis & Exploration Engine    │
└─────────────────────────────────────────┘
EOF
echo -e "${NC}"

log_info "Installing Uveddi ${VERSION} Community Core"
log_warning "This is the v1.0 Community Core release - production ready!"

# Check for required tools
check_requirements() {
    log_info "Checking system requirements..."
    
    # Check for git
    if ! command -v git >/dev/null 2>&1; then
        log_error "Git is required but not installed. Please install git first."
        echo "Ubuntu/Debian: sudo apt install git"
        echo "RHEL/CentOS/Fedora: sudo dnf install git"
        echo "macOS: brew install git"
        exit 1
    fi
    
    # Check for Rust
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Rust toolchain is required but not installed."
        log_info "Installing Rust using rustup..."
        
        if command -v curl >/dev/null 2>&1; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        elif command -v wget >/dev/null 2>&1; then
            wget -qO- https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        else
            log_error "Either curl or wget is required to install Rust."
            echo "Please install Rust manually: https://rustup.rs/"
            exit 1
        fi
        
        if ! command -v cargo >/dev/null 2>&1; then
            log_error "Rust installation failed. Please install manually: https://rustup.rs/"
            exit 1
        fi
    fi
    
    # Check Rust version
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    log_info "Found Rust version: $RUST_VERSION"
    
    # Check for build tools
    if ! command -v cc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
        log_warning "C compiler not found. Installing build essentials..."
        
        if command -v apt >/dev/null 2>&1; then
            sudo apt update
            sudo apt install -y build-essential pkg-config libssl-dev
        elif command -v dnf >/dev/null 2>&1; then
            sudo dnf install -y gcc g++ openssl-devel pkg-config
        elif command -v yum >/dev/null 2>&1; then
            sudo yum install -y gcc g++ openssl-devel pkg-config
        elif command -v brew >/dev/null 2>&1; then
            xcode-select --install 2>/dev/null || true
        else
            log_warning "Cannot auto-install build tools. Please install C compiler and pkg-config manually."
        fi
    fi
    
    log_success "System requirements verified"
}

# System detection
detect_system() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        SYSTEM="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        SYSTEM="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        SYSTEM="windows"
        log_warning "Windows detected. WSL2 is recommended for best performance."
    else
        SYSTEM="unknown"
        log_warning "Unknown system type: $OSTYPE"
    fi
    
    log_info "Detected system: $SYSTEM"
}

# Create directories
setup_directories() {
    log_info "Setting up installation directories..."
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$BIN_DIR"
    
    log_success "Directories created"
}

# Clone or update repository
setup_repository() {
    log_info "Setting up Uveddi repository..."
    
    if [[ -d "$INSTALL_DIR/.git" ]]; then
        log_info "Updating existing installation..."
        cd "$INSTALL_DIR"
        git fetch origin
        git checkout alpha  # Using alpha branch for community core
        git pull origin alpha
    else
        log_info "Cloning Uveddi repository..."
        rm -rf "$INSTALL_DIR"
        git clone -b alpha "$REPO_URL" "$INSTALL_DIR"  # Using alpha branch for community core
        cd "$INSTALL_DIR"
    fi
    
    log_success "Repository ready"
}

# Build Uveddi
build_uveddi() {
    log_info "Building Uveddi (this may take 5-15 minutes)..."
    cd "$INSTALL_DIR"
    
    # Clean previous build
    cargo clean >/dev/null 2>&1 || true
    
    # Build with community features and optimizations
    RUST_BACKTRACE=1 cargo build --release --features="community" --locked
    
    # Verify build
    if [[ ! -f "target/release/uveddi" ]]; then
        log_error "Build failed - binary not found"
        exit 1
    fi
    
    # Test binary
    if ! ./target/release/uveddi --help >/dev/null 2>&1; then
        log_error "Build failed - binary not functional"
        exit 1
    fi
    
    log_success "Build completed successfully"
}

# Install binary
install_binary() {
    log_info "Installing Uveddi binary..."
    
    # Copy binary
    cp "$INSTALL_DIR/target/release/uveddi" "$BIN_DIR/uveddi"
    chmod +x "$BIN_DIR/uveddi"
    
    # Verify installation
    if ! "$BIN_DIR/uveddi" --help >/dev/null 2>&1; then
        log_error "Installation verification failed"
        exit 1
    fi
    
    log_success "Binary installed to $BIN_DIR/uveddi"
}

# Setup PATH
setup_path() {
    log_info "Configuring PATH..."
    
    # Check if $BIN_DIR is in PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        log_warning "$BIN_DIR is not in your PATH"
        
        # Determine shell config file
        if [[ -n "${BASH_VERSION:-}" ]]; then
            SHELL_CONFIG="$HOME/.bashrc"
        elif [[ -n "${ZSH_VERSION:-}" ]]; then
            SHELL_CONFIG="$HOME/.zshrc"
        else
            SHELL_CONFIG="$HOME/.profile"
        fi
        
        # Add to PATH
        echo '' >> "$SHELL_CONFIG"
        echo '# Uveddi installation' >> "$SHELL_CONFIG"
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_CONFIG"
        
        log_info "Added $BIN_DIR to PATH in $SHELL_CONFIG"
        log_warning "Please run: source $SHELL_CONFIG"
        log_warning "Or start a new terminal session"
        
        export PATH="$BIN_DIR:$PATH"
    else
        log_success "PATH already configured"
    fi
}

# Final verification
final_verification() {
    log_info "Performing final verification..."
    
    # Test help command
    if "$BIN_DIR/uveddi" --help >/dev/null 2>&1; then
        log_success "Help command works"
    else
        log_error "Help command failed"
        exit 1
    fi
    
    # Test analyze command help
    if "$BIN_DIR/uveddi" analyze --help >/dev/null 2>&1; then
        log_success "Analyze command help works"
    else
        log_error "Analyze command help failed"
        exit 1
    fi
    
    # Get version info
    VERSION_OUTPUT=$("$BIN_DIR/uveddi" --help | head -1 || echo "unknown")
    log_info "Installed version: $VERSION_OUTPUT"
    
    log_success "All verifications passed!"
}

# Usage instructions
show_usage() {
    echo
    echo -e "${GREEN}🎉 Uveddi Alpha installation completed successfully!${NC}"
    echo
    echo -e "${BLUE}Getting Started:${NC}"
    echo "  1. Test the installation:"
    echo "     uveddi --help"
    echo
    echo "  2. Analyze your first project:"
    echo "     uveddi analyze /path/to/your/project --output-format=json"
    echo
    echo "  3. Generate HTML report:"
    echo "     uveddi analyze /path/to/your/project --output-format=html --output=report.html"
    echo
    echo -e "${BLUE}Important Notes:${NC}"
    echo "  • This is an ALPHA release - expect some rough edges"
    echo "  • All critical alpha-blocking issues have been resolved"
    echo "  • Analysis works for Rust, Python, JavaScript, and TypeScript"
    echo "  • Memory optimization is enabled by default"
    echo
    echo -e "${BLUE}Resources:${NC}"
    echo "  • Documentation: https://github.com/botzrDev/uveddi/docs"
    echo "  • Report issues: https://github.com/botzrDev/uveddi/issues"
    echo "  • Community: https://github.com/botzrDev/uveddi/discussions"
    echo
    echo -e "${BLUE}Next Steps:${NC}"
    echo "  • Try analyzing a small project first"
    echo "  • Check the documentation for advanced options"
    echo "  • Join our community to share feedback"
    echo
    echo -e "${YELLOW}If PATH wasn't updated automatically, run:${NC}"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    echo
}

# Cleanup on exit
cleanup() {
    if [[ -n "${INSTALL_DIR:-}" ]] && [[ "$INSTALL_DIR" != "/" ]] && [[ "$INSTALL_DIR" != "$HOME" ]]; then
        # Keep the repository for future updates
        log_info "Installation files kept at $INSTALL_DIR for future updates"
    fi
}

trap cleanup EXIT

# Main installation flow
main() {
    detect_system
    check_requirements
    setup_directories
    setup_repository
    build_uveddi
    install_binary
    setup_path
    final_verification
    show_usage
}

# Run installation
main "$@"