#!/bin/bash
# Uveddi Installation Script
# Automatically builds and installs uveddi to your local PATH

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}ℹ${NC}  $1"
}

print_success() {
    echo -e "${GREEN}✓${NC}  $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC}  $1"
}

print_error() {
    echo -e "${RED}✗${NC}  $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Uveddi Installer${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Check if running from uveddi directory
check_directory() {
    if [ ! -f "Cargo.toml" ] || ! grep -q "name = \"uveddi\"" Cargo.toml 2>/dev/null; then
    print_error "This script must be run from the uveddi repository root directory"
    print_info "Usage: cd uveddi && ./deploy/install.sh"
        exit 1
    fi
}

# Check for required tools
check_prerequisites() {
    print_info "Checking prerequisites..."

    if ! command -v cargo &> /dev/null; then
        print_error "Cargo (Rust) is not installed"
        print_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi

    local rust_version=$(rustc --version | awk '{print $2}')
    print_success "Rust ${rust_version} detected"
}

# Determine installation directory
determine_install_dir() {
    # Check for ~/.local/bin (preferred for user installs)
    if [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        # Create ~/.local/bin if it doesn't exist
        print_info "Creating ~/.local/bin directory..."
        mkdir -p "$HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
    fi

    print_info "Installation directory: ${INSTALL_DIR}"
}

# Check if install dir is in PATH
check_path() {
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        print_success "${INSTALL_DIR} is already in your PATH"
        PATH_NEEDS_UPDATE=false
    else
        print_warning "${INSTALL_DIR} is not in your PATH"
        PATH_NEEDS_UPDATE=true
    fi
}

# Add to PATH in shell config
update_shell_config() {
    if [ "$PATH_NEEDS_UPDATE" = false ]; then
        return
    fi

    print_info "Adding ${INSTALL_DIR} to your PATH..."

    # Detect shell
    local shell_name=$(basename "$SHELL")
    local config_file=""

    case "$shell_name" in
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                config_file="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                config_file="$HOME/.bash_profile"
            else
                config_file="$HOME/.profile"
            fi
            ;;
        zsh)
            config_file="$HOME/.zshrc"
            ;;
        fish)
            config_file="$HOME/.config/fish/config.fish"
            ;;
        *)
            config_file="$HOME/.profile"
            ;;
    esac

    # Check if PATH line already exists
    local path_line="export PATH=\"\$HOME/.local/bin:\$PATH\""

    if [ "$shell_name" = "fish" ]; then
        path_line="set -gx PATH \$HOME/.local/bin \$PATH"
    fi

    if [ -f "$config_file" ]; then
        if grep -q ".local/bin" "$config_file"; then
            print_info "PATH configuration already exists in ${config_file}"
        else
            echo "" >> "$config_file"
            echo "# Added by uveddi installer" >> "$config_file"
            echo "$path_line" >> "$config_file"
            print_success "Added PATH configuration to ${config_file}"
            SHELL_CONFIG_UPDATED=true
        fi
    else
        # Create config file
        echo "# Added by uveddi installer" > "$config_file"
        echo "$path_line" >> "$config_file"
        print_success "Created ${config_file} with PATH configuration"
        SHELL_CONFIG_UPDATED=true
    fi
}

# Build the binary
build_binary() {
    print_info "Building uveddi (this may take a few minutes)..."
    echo ""

    if cargo build --release --bin uveddi; then
        echo ""
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Install the binary
install_binary() {
    local binary_path="target/release/uveddi"

    if [ ! -f "$binary_path" ]; then
        print_error "Binary not found at ${binary_path}"
        exit 1
    fi

    print_info "Installing uveddi to ${INSTALL_DIR}..."

    # Remove old installation if it's a directory (cleanup)
    if [ -d "${INSTALL_DIR}/uveddi" ]; then
        print_warning "Removing old directory-based installation"
        rm -rf "${INSTALL_DIR}/uveddi"
    fi

    # Copy binary
    cp "$binary_path" "${INSTALL_DIR}/uveddi"
    chmod +x "${INSTALL_DIR}/uveddi"

    print_success "Binary installed to ${INSTALL_DIR}/uveddi"
}

# Verify installation
verify_installation() {
    print_info "Verifying installation..."

    # Try to run the binary directly from install location
    if "${INSTALL_DIR}/uveddi" --version &>/dev/null; then
        local version=$("${INSTALL_DIR}/uveddi" --version | head -n 1)
        print_success "Installation verified: ${version}"
        return 0
    else
        print_error "Installation verification failed"
        return 1
    fi
}

# Print completion message
print_completion() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Installation Complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    if [ "$SHELL_CONFIG_UPDATED" = true ]; then
        print_warning "Please restart your terminal or run:"
        echo ""
        local shell_name=$(basename "$SHELL")
        if [ "$shell_name" = "bash" ]; then
            echo "    source ~/.bashrc"
        elif [ "$shell_name" = "zsh" ]; then
            echo "    source ~/.zshrc"
        elif [ "$shell_name" = "fish" ]; then
            echo "    source ~/.config/fish/config.fish"
        else
            echo "    source ~/.profile"
        fi
        echo ""
        print_info "Then you can use: uveddi --version"
    else
        print_info "You can now use: uveddi --version"
    fi

    echo ""
    print_info "Quick start:"
    echo "    uveddi --help"
    echo "    uveddi analyze ./src"
    echo ""
    print_info "Binary location: ${INSTALL_DIR}/uveddi"
    echo ""
}

# Main installation flow
main() {
    print_header

    check_directory
    check_prerequisites
    determine_install_dir
    check_path
    update_shell_config
    build_binary
    install_binary

    if verify_installation; then
        print_completion
    else
        print_error "Installation completed but verification failed"
        print_info "Try running: ${INSTALL_DIR}/uveddi --version"
        exit 1
    fi
}

# Run main installation
main
