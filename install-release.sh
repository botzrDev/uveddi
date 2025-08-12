#!/bin/bash
set -e

# Uveddi Release Installer
# Installs the latest release binary from GitHub
# Requires GITHUB_TOKEN environment variable for private repository access

REPO="botzrDev/uveddi"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="uveddi"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if GitHub token is provided
if [ -z "$GITHUB_TOKEN" ]; then
    log_error "GitHub Personal Access Token required for private repository access."
    echo
    echo "To get access to Uveddi alpha releases:"
    echo "1. Contact the Uveddi team for alpha access"
    echo "2. Create a GitHub Personal Access Token:"
    echo "   - Go to: https://github.com/settings/tokens"
    echo "   - Click 'Generate new token (classic)'"
    echo "   - Select scope: 'Contents' (read permission)"
    echo "   - Copy the generated token"
    echo "3. Run the installer with your token:"
    echo "   GITHUB_TOKEN=your_token_here curl -sSL https://uveddi.org/install-release.sh | bash"
    echo
    exit 1
fi

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$OSTYPE" in
        linux*)
            os="unknown-linux-gnu"
            ;;
        darwin*)
            os="apple-darwin"
            ;;
        msys*|cygwin*|mingw*)
            os="pc-windows-msvc"
            ;;
        *)
            log_error "Unsupported operating system: $OSTYPE"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${arch}-${os}"
}

# Get the latest release info
get_latest_release() {
    log_info "Fetching latest release information..."
    
    local response
    response=$(curl -s -H "Authorization: token $GITHUB_TOKEN" \
        "https://api.github.com/repos/$REPO/releases/latest")
    
    if [ $? -ne 0 ]; then
        log_error "Failed to fetch release information"
        exit 1
    fi
    
    # Check if we got an error response
    if echo "$response" | grep -q '"message"'; then
        local message
        message=$(echo "$response" | grep '"message"' | cut -d '"' -f 4)
        log_error "GitHub API error: $message"
        if echo "$message" | grep -q "rate limit"; then
            log_error "Rate limit exceeded. Please try again later."
        elif echo "$message" | grep -q "Not Found"; then
            log_error "Repository not found or access denied. Please check your token permissions."
        fi
        exit 1
    fi
    
    echo "$response"
}

# Download and install binary
install_binary() {
    local platform="$1"
    local release_info="$2"
    
    # Determine file extension
    local file_ext=".tar.gz"
    if [[ "$platform" == *"windows"* ]]; then
        file_ext=".zip"
    fi
    
    local asset_name="uveddi-${platform}${file_ext}"
    
    # Extract download URL from release info
    local download_url
    download_url=$(echo "$release_info" | grep -o "\"browser_download_url\":\s*\"[^\"]*${asset_name}\"" | cut -d '"' -f 4)
    
    if [ -z "$download_url" ]; then
        log_error "Could not find download URL for platform: $platform"
        log_error "Available assets:"
        echo "$release_info" | grep -o '"name":\s*"[^"]*"' | cut -d '"' -f 4
        exit 1
    fi
    
    log_info "Downloading $asset_name..."
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Download the asset
    if ! curl -L -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/octet-stream" \
        -o "$asset_name" \
        "$download_url"; then
        log_error "Failed to download $asset_name"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    log_info "Extracting binary..."
    
    # Extract based on file type
    if [[ "$asset_name" == *.tar.gz ]]; then
        tar -xzf "$asset_name"
    elif [[ "$asset_name" == *.zip ]]; then
        unzip -q "$asset_name"
    fi
    
    # Find the binary
    local binary_path
    if [[ "$platform" == *"windows"* ]]; then
        binary_path="$BINARY_NAME.exe"
    else
        binary_path="$BINARY_NAME"
    fi
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found in archive"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Install the binary
    log_info "Installing to $INSTALL_DIR/$BINARY_NAME..."
    cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Cleanup
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    log_info "Installation completed successfully!"
    
    # Check if install directory is in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        log_warn "Warning: $INSTALL_DIR is not in your PATH"
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
        echo
        echo "Or run the binary directly: $INSTALL_DIR/$BINARY_NAME"
    else
        echo
        echo "You can now run: $BINARY_NAME --help"
    fi
}

# Verify installation
verify_installation() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        log_info "Verifying installation..."
        local version
        version=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        log_info "Installed version: $version"
        return 0
    else
        log_error "Installation verification failed"
        return 1
    fi
}

# Main installation process
main() {
    log_info "Starting Uveddi installation..."
    
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: $platform"
    
    local release_info
    release_info=$(get_latest_release)
    
    local tag_name
    tag_name=$(echo "$release_info" | grep '"tag_name"' | cut -d '"' -f 4)
    log_info "Latest release: $tag_name"
    
    install_binary "$platform" "$release_info"
    verify_installation
    
    echo
    log_info "🎉 Uveddi alpha has been installed successfully!"
    echo "Need help? Check out the documentation or contact the alpha team."
}

# Run main function
main "$@"