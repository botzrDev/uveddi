#!/bin/bash
# Uveddi Uninstallation Script
# Removes uveddi binary and optionally cleans up PATH configuration

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
    echo -e "${BLUE}  Uveddi Uninstaller${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Find uveddi installation
find_installation() {
    BINARY_PATH=""

    # Check common locations
    if [ -f "$HOME/.local/bin/uveddi" ]; then
        BINARY_PATH="$HOME/.local/bin/uveddi"
    elif [ -f "/usr/local/bin/uveddi" ]; then
        BINARY_PATH="/usr/local/bin/uveddi"
    elif command -v uveddi &> /dev/null; then
        BINARY_PATH=$(which uveddi)
    fi

    if [ -z "$BINARY_PATH" ]; then
        print_warning "No uveddi installation found"
        echo ""
        print_info "Checked locations:"
        echo "    - $HOME/.local/bin/uveddi"
        echo "    - /usr/local/bin/uveddi"
        echo "    - PATH directories"
        return 1
    fi

    print_info "Found uveddi at: ${BINARY_PATH}"
    return 0
}

# Remove binary
remove_binary() {
    if [ -z "$BINARY_PATH" ]; then
        return 1
    fi

    print_info "Removing binary..."

    if rm -f "$BINARY_PATH"; then
        print_success "Binary removed: ${BINARY_PATH}"
        return 0
    else
        print_error "Failed to remove binary (permission denied?)"
        return 1
    fi
}

# Clean up configuration cache
cleanup_cache() {
    local cache_dir="$HOME/.uveddi_cache"
    local config_dir="$HOME/.uveddi"

    if [ -d "$cache_dir" ] || [ -d "$config_dir" ]; then
        echo ""
        read -p "Remove cache and configuration data? [y/N]: " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            [ -d "$cache_dir" ] && rm -rf "$cache_dir" && print_success "Removed cache: ${cache_dir}"
            [ -d "$config_dir" ] && rm -rf "$config_dir" && print_success "Removed config: ${config_dir}"
        else
            print_info "Keeping cache and configuration data"
        fi
    fi
}

# Offer to remove PATH configuration
cleanup_path() {
    echo ""
    read -p "Remove PATH configuration from shell config files? [y/N]: " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Keeping PATH configuration"
        return
    fi

    local shell_name=$(basename "$SHELL")
    local config_files=()

    case "$shell_name" in
        bash)
            [ -f "$HOME/.bashrc" ] && config_files+=("$HOME/.bashrc")
            [ -f "$HOME/.bash_profile" ] && config_files+=("$HOME/.bash_profile")
            [ -f "$HOME/.profile" ] && config_files+=("$HOME/.profile")
            ;;
        zsh)
            [ -f "$HOME/.zshrc" ] && config_files+=("$HOME/.zshrc")
            ;;
        fish)
            [ -f "$HOME/.config/fish/config.fish" ] && config_files+=("$HOME/.config/fish/config.fish")
            ;;
        *)
            [ -f "$HOME/.profile" ] && config_files+=("$HOME/.profile")
            ;;
    esac

    local found_config=false
    for config_file in "${config_files[@]}"; do
        if grep -q "Added by uveddi installer" "$config_file" 2>/dev/null; then
            # Create backup
            cp "$config_file" "${config_file}.backup.$(date +%Y%m%d_%H%M%S)"

            # Remove uveddi-related lines
            sed -i.tmp '/# Added by uveddi installer/,+1d' "$config_file"
            rm -f "${config_file}.tmp"

            print_success "Removed PATH configuration from ${config_file}"
            print_info "Backup saved: ${config_file}.backup.*"
            found_config=true
        fi
    done

    if [ "$found_config" = false ]; then
        print_info "No uveddi PATH configuration found in shell config files"
    fi
}

# Print completion message
print_completion() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Uninstallation Complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    print_info "Uveddi has been removed from your system"
    echo ""
    print_warning "Please restart your terminal to apply PATH changes"
    echo ""
}

# Main uninstallation flow
main() {
    print_header

    if ! find_installation; then
        echo ""
        print_info "Nothing to uninstall"
        exit 0
    fi

    echo ""
    print_warning "This will remove uveddi from your system"
    read -p "Continue? [y/N]: " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi

    if remove_binary; then
        cleanup_cache
        cleanup_path
        print_completion
    else
        print_error "Uninstallation failed"
        exit 1
    fi
}

# Run main uninstallation
main
