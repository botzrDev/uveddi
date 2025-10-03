#!/bin/bash
# Uveddi v1.0 Release Artifact Builder
# Builds production binaries and packages for distribution

set -e

VERSION="1.0.0"
PROJECT_NAME="uveddi"
BUILD_DATE=$(date +%Y-%m-%d)

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🏗️  Uveddi v${VERSION} Release Artifact Builder"
echo "================================================"
echo "Build Date: $BUILD_DATE"
echo ""

# Create release directory
RELEASE_DIR="release/${PROJECT_NAME}-v${VERSION}"
mkdir -p "$RELEASE_DIR"

echo -e "${BLUE}📁 Release directory: $RELEASE_DIR${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Build for Linux (native)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Building for Linux (x86_64)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TARGET="x86_64-unknown-linux-gnu"
ARCHIVE_NAME="${PROJECT_NAME}-v${VERSION}-${TARGET}"

echo "Installing target: $TARGET"
rustup target add $TARGET 2>&1 | grep -v "info: component" || true

echo "Building release binary..."
cargo build --release --features full --target $TARGET 2>&1 | tail -10

if [ -f "target/${TARGET}/release/${PROJECT_NAME}" ]; then
    mkdir -p "$RELEASE_DIR/${ARCHIVE_NAME}"
    cp "target/${TARGET}/release/${PROJECT_NAME}" "$RELEASE_DIR/${ARCHIVE_NAME}/"

    # Copy documentation
    cp README.md LICENSE CHANGELOG.md "$RELEASE_DIR/${ARCHIVE_NAME}/" 2>/dev/null || true

    # Create archive
    cd release
    tar -czf "${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}"
    cd ..

    print_status "Linux build complete: ${ARCHIVE_NAME}.tar.gz"
else
    print_error "Linux build failed"
fi

echo ""

# Build for macOS (cross-compilation may not work without proper setup)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Building for macOS (x86_64)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TARGET="x86_64-apple-darwin"
ARCHIVE_NAME="${PROJECT_NAME}-v${VERSION}-${TARGET}"

# Check if we can build for macOS
if rustup target list | grep -q "${TARGET} (installed)"; then
    echo "Building release binary for macOS..."
    cargo build --release --features full --target $TARGET 2>&1 | tail -10 || print_warning "macOS build skipped (cross-compilation not configured)"

    if [ -f "target/${TARGET}/release/${PROJECT_NAME}" ]; then
        mkdir -p "$RELEASE_DIR/${ARCHIVE_NAME}"
        cp "target/${TARGET}/release/${PROJECT_NAME}" "$RELEASE_DIR/${ARCHIVE_NAME}/"
        cp README.md LICENSE CHANGELOG.md "$RELEASE_DIR/${ARCHIVE_NAME}/" 2>/dev/null || true

        cd release
        tar -czf "${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}"
        cd ..

        print_status "macOS build complete: ${ARCHIVE_NAME}.tar.gz"
    else
        print_warning "macOS cross-compilation skipped (requires macOS or proper toolchain)"
    fi
else
    print_warning "macOS target not installed, skipping"
fi

echo ""

# Build for Windows (cross-compilation may not work without proper setup)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Building for Windows (x86_64)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TARGET="x86_64-pc-windows-gnu"
ARCHIVE_NAME="${PROJECT_NAME}-v${VERSION}-${TARGET}"

# Check if we have mingw installed
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "Installing target: $TARGET"
    rustup target add $TARGET 2>&1 | grep -v "info: component" || true

    echo "Building release binary for Windows..."
    cargo build --release --features full --target $TARGET 2>&1 | tail -10 || print_warning "Windows build skipped (cross-compilation issues)"

    if [ -f "target/${TARGET}/release/${PROJECT_NAME}.exe" ]; then
        mkdir -p "$RELEASE_DIR/${ARCHIVE_NAME}"
        cp "target/${TARGET}/release/${PROJECT_NAME}.exe" "$RELEASE_DIR/${ARCHIVE_NAME}/"
        cp README.md LICENSE CHANGELOG.md "$RELEASE_DIR/${ARCHIVE_NAME}/" 2>/dev/null || true

        cd release
        zip -r "${ARCHIVE_NAME}.zip" "${ARCHIVE_NAME}" > /dev/null
        cd ..

        print_status "Windows build complete: ${ARCHIVE_NAME}.zip"
    else
        print_warning "Windows cross-compilation skipped (requires mingw-w64)"
    fi
else
    print_warning "mingw-w64 not installed, skipping Windows build"
    print_warning "Install with: apt-get install mingw-w64"
fi

echo ""

# Build frontend
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚛️  Building Frontend"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "frontend" ]; then
    cd frontend

    if [ ! -d "node_modules" ]; then
        echo "Installing frontend dependencies..."
        npm install > /dev/null 2>&1
    fi

    echo "Building production frontend..."
    npm run build 2>&1 | tail -10

    if [ -d "dist" ]; then
        cd ..
        cp -r frontend/dist "$RELEASE_DIR/frontend-dist"
        print_status "Frontend build complete"
    else
        cd ..
        print_error "Frontend build failed"
    fi
else
    print_warning "Frontend directory not found, skipping"
fi

echo ""

# Package documentation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📚 Packaging Documentation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -d "docs" ]; then
    cp -r docs "$RELEASE_DIR/docs"
    print_status "Documentation packaged"
else
    print_warning "Docs directory not found"
fi

echo ""

# Create source tarball
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Creating Source Tarball"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if git rev-parse --git-dir > /dev/null 2>&1; then
    git archive --format=tar.gz --prefix="${PROJECT_NAME}-v${VERSION}/" HEAD > "release/${PROJECT_NAME}-v${VERSION}-source.tar.gz"
    print_status "Source tarball created: ${PROJECT_NAME}-v${VERSION}-source.tar.gz"
else
    print_warning "Not a git repository, skipping source tarball"
fi

echo ""

# Generate checksums
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Generating Checksums"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd release
if command -v sha256sum &> /dev/null; then
    sha256sum *.tar.gz *.zip 2>/dev/null > SHA256SUMS.txt || true
    print_status "SHA256 checksums generated"
else
    print_warning "sha256sum not available, skipping checksums"
fi
cd ..

echo ""

# Create installation script
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Creating Installation Script"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > "$RELEASE_DIR/install.sh" << 'INSTALL_EOF'
#!/bin/bash
# Uveddi Installation Script

set -e

INSTALL_DIR="${HOME}/.local/bin"

echo "🚀 Installing Uveddi..."

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          echo "Unsupported OS: ${OS}"; exit 1;;
esac

# Find binary
if [ -f "uveddi" ]; then
    BINARY="uveddi"
elif [ -f "bin/uveddi" ]; then
    BINARY="bin/uveddi"
else
    echo "Error: uveddi binary not found"
    exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"

# Copy binary
cp "$BINARY" "$INSTALL_DIR/uveddi"
chmod +x "$INSTALL_DIR/uveddi"

echo "✅ Uveddi installed to $INSTALL_DIR/uveddi"
echo ""
echo "Add the following to your shell profile if not already present:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Then run: uveddi --version"
INSTALL_EOF

chmod +x "$RELEASE_DIR/install.sh"
print_status "Installation script created"

echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Build Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Release artifacts created in: $RELEASE_DIR"
echo ""
echo "Generated files:"
ls -lh release/ | tail -n +2 | awk '{printf "  %-10s %s\n", $5, $9}'

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Test binaries: ./release/${PROJECT_NAME}-v${VERSION}-*/uveddi --version"
echo "  2. Verify checksums: cd release && sha256sum -c SHA256SUMS.txt"
echo "  3. Upload to GitHub releases"
echo ""
