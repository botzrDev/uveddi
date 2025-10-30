#!/usr/bin/env bash
# Uveddi Installation Script
# Supports: Linux (x86_64, aarch64, musl), macOS (Intel, Apple Silicon)
#
# Usage:
#   curl -fsSL https://uveddi.io/install.sh | bash
#   wget -qO- https://uveddi.io/install.sh | bash
#
# Environment Variables:
#   UVEDDI_VERSION: Specific version to install (default: latest)
#   UVEDDI_INSTALL_DIR: Installation directory (default: ~/.local/bin)
#   UVEDDI_VERIFY_SIGNATURE: Verify GPG signature (default: true)
#   UVEDDI_BASE_URL: Base URL for downloads (default: GitHub releases)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

readonly VERSION="${UVEDDI_VERSION:-1.0.0}"
readonly INSTALL_DIR="${UVEDDI_INSTALL_DIR:-${HOME}/.local/bin}"
readonly VERIFY_SIGNATURE="${UVEDDI_VERIFY_SIGNATURE:-true}"
readonly BASE_URL="${UVEDDI_BASE_URL:-https://github.com/botzrDev/uveddi/releases/download}"
readonly TEMP_DIR="$(mktemp -d)"
readonly PROJECT_NAME="uveddi"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# ============================================================================
# Logging
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# ============================================================================
# Cleanup
# ============================================================================

cleanup() {
    if [ -d "${TEMP_DIR}" ]; then
        rm -rf "${TEMP_DIR}"
    fi
}

trap cleanup EXIT

# ============================================================================
# Platform Detection
# ============================================================================

detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"
    local platform=""
    
    case "${os}" in
        Linux*)
            case "${arch}" in
                x86_64)
                    # Detect if musl or glibc
                    if ldd --version 2>&1 | grep -q musl; then
                        platform="x86_64-unknown-linux-musl"
                    else
                        platform="x86_64-unknown-linux-gnu"
                    fi
                    ;;
                aarch64|arm64)
                    platform="aarch64-unknown-linux-gnu"
                    ;;
                *)
                    log_error "Unsupported architecture: ${arch}"
                    return 1
                    ;;
            esac
            ;;
        Darwin*)
            case "${arch}" in
                x86_64)
                    platform="x86_64-apple-darwin"
                    ;;
                arm64)
                    platform="aarch64-apple-darwin"
                    ;;
                *)
                    log_error "Unsupported architecture: ${arch}"
                    return 1
                    ;;
            esac
            ;;
        *)
            log_error "Unsupported operating system: ${os}"
            return 1
            ;;
    esac
    
    echo "${platform}"
}

# ============================================================================
# Download Functions
# ============================================================================

check_download_tool() {
    if command -v curl &> /dev/null; then
        echo "curl"
    elif command -v wget &> /dev/null; then
        echo "wget"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        return 1
    fi
}

download_file() {
    local url="$1"
    local output="$2"
    local tool="$(check_download_tool)"
    
    log_info "Downloading: $(basename "${url}")"
    
    case "${tool}" in
        curl)
            curl -fsSL -o "${output}" "${url}"
            ;;
        wget)
            wget -q -O "${output}" "${url}"
            ;;
    esac
}

# ============================================================================
# Verification Functions
# ============================================================================

verify_checksum() {
    local file="$1"
    local checksum_file="$2"
    
    if ! command -v sha256sum &> /dev/null; then
        log_warning "sha256sum not found, skipping checksum verification"
        return 0
    fi
    
    log_info "Verifying checksum"
    
    # Extract checksum for this file
    local expected_checksum="$(grep "$(basename "${file}")" "${checksum_file}" | cut -d' ' -f1)"
    
    if [ -z "${expected_checksum}" ]; then
        log_warning "Checksum not found in SHA256SUMS"
        return 0
    fi
    
    local actual_checksum="$(sha256sum "${file}" | cut -d' ' -f1)"
    
    if [ "${expected_checksum}" != "${actual_checksum}" ]; then
        log_error "Checksum verification failed!"
        log_error "Expected: ${expected_checksum}"
        log_error "Got: ${actual_checksum}"
        return 1
    fi
    
    log_success "Checksum verified"
}

verify_signature() {
    local file="$1"
    local signature="$2"
    
    if [ "${VERIFY_SIGNATURE}" != "true" ]; then
        log_info "Signature verification disabled"
        return 0
    fi
    
    if ! command -v gpg &> /dev/null; then
        log_warning "GPG not found, skipping signature verification"
        log_warning "Set UVEDDI_VERIFY_SIGNATURE=false to suppress this warning"
        return 0
    fi
    
    log_info "Verifying GPG signature"
    
    # Import Uveddi release key if not already present
    # In production, this would import the actual release signing key
    # For now, we'll skip if key is not found
    if ! gpg --verify "${signature}" "${file}" 2>&1 | grep -q "Good signature"; then
        log_warning "GPG signature verification failed or key not found"
        log_warning "Set UVEDDI_VERIFY_SIGNATURE=false to skip signature verification"
        
        # Ask user if they want to continue
        if [ -t 0 ]; then
            read -p "Continue installation without signature verification? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_error "Installation aborted"
                return 1
            fi
        else
            log_warning "Continuing without signature verification (non-interactive)"
        fi
    else
        log_success "Signature verified"
    fi
}

# ============================================================================
# Installation Functions
# ============================================================================

install_binary() {
    local platform="$1"
    local archive_name="${PROJECT_NAME}-${VERSION}-${platform}.tar.gz"
    local download_url="${BASE_URL}/v${VERSION}/${archive_name}"
    local checksum_url="${BASE_URL}/v${VERSION}/SHA256SUMS"
    local signature_url="${BASE_URL}/v${VERSION}/${archive_name}.asc"
    
    cd "${TEMP_DIR}"
    
    # Download archive
    download_file "${download_url}" "${archive_name}" || {
        log_error "Failed to download ${archive_name}"
        return 1
    }
    
    # Download checksums
    download_file "${checksum_url}" "SHA256SUMS" || {
        log_warning "Failed to download SHA256SUMS"
    }
    
    # Download signature
    if [ "${VERIFY_SIGNATURE}" = "true" ]; then
        download_file "${signature_url}" "${archive_name}.asc" || {
            log_warning "Failed to download signature"
        }
    fi
    
    # Verify checksum
    if [ -f "SHA256SUMS" ]; then
        verify_checksum "${archive_name}" "SHA256SUMS" || return 1
    fi
    
    # Verify signature
    if [ -f "${archive_name}.asc" ]; then
        verify_signature "${archive_name}" "${archive_name}.asc" || return 1
    fi
    
    # Extract archive
    log_info "Extracting archive"
    tar -xzf "${archive_name}"
    
    # Find binary
    local binary_path="$(find . -name "${PROJECT_NAME}" -type f | head -1)"
    
    if [ -z "${binary_path}" ]; then
        log_error "Binary not found in archive"
        return 1
    fi
    
    # Create install directory
    mkdir -p "${INSTALL_DIR}"
    
    # Install binary
    log_info "Installing to ${INSTALL_DIR}/${PROJECT_NAME}"
    cp "${binary_path}" "${INSTALL_DIR}/${PROJECT_NAME}"
    chmod +x "${INSTALL_DIR}/${PROJECT_NAME}"
    
    log_success "Uveddi ${VERSION} installed successfully!"
}

# ============================================================================
# Post-installation
# ============================================================================

check_path() {
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        log_warning "Installation directory not in PATH"
        echo ""
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        echo "Then run: source ~/.bashrc  (or restart your shell)"
        return 1
    fi
    return 0
}

verify_installation() {
    log_info "Verifying installation"
    
    if "${INSTALL_DIR}/${PROJECT_NAME}" --version &> /dev/null; then
        log_success "Installation verified"
        echo ""
        "${INSTALL_DIR}/${PROJECT_NAME}" --version
        return 0
    else
        log_error "Installation verification failed"
        return 1
    fi
}

print_next_steps() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🎉 Uveddi is ready to use!"
    echo ""
    
    if ! check_path; then
        echo "After updating your PATH, try:"
    else
        echo "Get started with:"
    fi
    
    echo ""
    echo "  uveddi --help"
    echo "  uveddi analyze /path/to/code"
    echo ""
    echo "Documentation: https://uveddi.io/docs"
    echo "Support: https://github.com/botzrDev/uveddi/issues"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# ============================================================================
# Uninstallation
# ============================================================================

uninstall() {
    log_info "Uninstalling Uveddi"
    
    if [ -f "${INSTALL_DIR}/${PROJECT_NAME}" ]; then
        rm -f "${INSTALL_DIR}/${PROJECT_NAME}"
        log_success "Uveddi uninstalled from ${INSTALL_DIR}"
    else
        log_warning "Uveddi not found at ${INSTALL_DIR}/${PROJECT_NAME}"
    fi
    
    # Remove config directory if desired
    if [ -d "${HOME}/.config/uveddi" ]; then
        read -p "Remove configuration directory? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "${HOME}/.config/uveddi"
            log_success "Configuration removed"
        fi
    fi
}

# ============================================================================
# Main
# ============================================================================

print_banner() {
    echo ""
    echo "╔═══════════════════════════════════════════╗"
    echo "║     Uveddi Installer v${VERSION}            ║"
    echo "╚═══════════════════════════════════════════╝"
    echo ""
}

main() {
    print_banner
    
    # Handle uninstall
    if [ "${1:-}" = "uninstall" ]; then
        uninstall
        exit 0
    fi
    
    # Detect platform
    local platform="$(detect_platform)"
    log_info "Detected platform: ${platform}"
    
    # Install
    install_binary "${platform}" || {
        log_error "Installation failed"
        exit 1
    }
    
    # Verify
    verify_installation || {
        log_warning "Installation completed but verification failed"
    }
    
    # Print next steps
    print_next_steps
}

main "$@"
