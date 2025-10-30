#!/bin/bash
# WSL-optimized build script for full feature compilation
# Addresses WSL timeout issues after ~500 seconds

set -e

echo "=== WSL Full Feature Build Script ==="
echo "This script optimizes the build process to avoid WSL timeouts"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_status() {
    echo -e "${GREEN}[STATUS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running in WSL
if ! grep -q Microsoft /proc/version 2>/dev/null; then
    print_warning "Not running in WSL, but continuing anyway..."
fi

# Step 1: Optimize WSL2 memory settings
print_status "Checking WSL2 memory configuration..."
if [ ! -f ~/.wslconfig ]; then
    print_warning "No .wslconfig found. Creating optimized configuration..."
    cat > ~/.wslconfig << 'EOF'
[wsl2]
memory=8GB
processors=4
swap=4GB
localhostForwarding=true

[experimental]
autoMemoryReclaim=gradual
sparseVhd=true
EOF
    print_warning "WSL config created. You may need to restart WSL with 'wsl --shutdown' for changes to take effect."
fi

# Step 2: Install build optimizations if not present
print_status "Checking for build optimization tools..."

# Check for lld linker
if ! command -v lld &> /dev/null; then
    print_warning "LLD linker not found. Installing for faster linking..."
    sudo apt-get update && sudo apt-get install -y lld clang
fi

# Check for sccache
if ! command -v sccache &> /dev/null; then
    print_warning "sccache not found. Installing for build caching..."
    cargo install sccache || print_warning "Failed to install sccache, continuing without it"
fi

# Step 3: Set up environment variables
print_status "Setting up optimized environment variables..."
export CARGO_BUILD_JOBS=4
export CARGO_INCREMENTAL=1
export RUSTC_WRAPPER=""  # Clear wrapper initially
export CARGO_TARGET_DIR="/tmp/uveddi-target"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C link-arg=-fuse-ld=lld -C codegen-units=256"

# Use sccache if available
if command -v sccache &> /dev/null; then
    export RUSTC_WRAPPER="sccache"
    print_status "Using sccache for compilation caching"
    sccache --start-server 2>/dev/null || true
    sccache --show-stats
fi

# Step 4: Clean and prepare build directory
print_status "Preparing build directory..."
mkdir -p "$CARGO_TARGET_DIR"

# Use WSL-optimized config
if [ -f .cargo/config.toml.wsl-full ]; then
    print_status "Using WSL-optimized cargo configuration"
    cp .cargo/config.toml .cargo/config.toml.backup 2>/dev/null || true
    cp .cargo/config.toml.wsl-full .cargo/config.toml
fi

# Step 5: Pre-build dependency fetching
print_status "Pre-fetching and building dependencies..."
cargo fetch --locked 2>/dev/null || cargo fetch

# Step 6: Build dependencies first (without our code)
print_status "Building dependencies separately to avoid timeout..."
cargo build --features production --lib --bins 2>&1 | while IFS= read -r line; do
    echo "$line"
    # Reset WSL idle timer by doing small filesystem operation
    touch /tmp/.wsl-keepalive-$$ 2>/dev/null || true
done

# Step 7: Build with full features
print_status "Building with full production features..."
print_warning "This may take several minutes. The build is kept alive to prevent WSL timeout."

# Build with periodic keepalive
(
    while true; do
        sleep 30
        # Keep WSL active
        df -h /tmp > /dev/null 2>&1
        echo -n "."
    done
) &
KEEPALIVE_PID=$!

# Trap to ensure we kill the keepalive process
trap "kill $KEEPALIVE_PID 2>/dev/null || true" EXIT

# Main build command
if cargo build --release --features production 2>&1 | tee build.log; then
    print_status "Build completed successfully!"
else
    print_error "Build failed. Check build.log for details."
    kill $KEEPALIVE_PID 2>/dev/null || true
    exit 1
fi

# Kill keepalive
kill $KEEPALIVE_PID 2>/dev/null || true

# Step 8: Run tests with full features
print_status "Running tests with full features..."
print_warning "Tests will run with keepalive to prevent timeout..."

# Start new keepalive for tests
(
    while true; do
        sleep 30
        touch /tmp/.wsl-test-keepalive-$$ 2>/dev/null || true
        echo -n "."
    done
) &
TEST_KEEPALIVE_PID=$!

# Run tests
if cargo test --release --features production --no-fail-fast 2>&1 | tee test.log; then
    print_status "All tests passed!"
else
    print_warning "Some tests failed. Check test.log for details."
fi

# Kill test keepalive
kill $TEST_KEEPALIVE_PID 2>/dev/null || true

# Step 9: Show build statistics
if command -v sccache &> /dev/null; then
    print_status "Build cache statistics:"
    sccache --show-stats
fi

# Step 10: Restore original config if backed up
if [ -f .cargo/config.toml.backup ]; then
    print_status "Restoring original cargo configuration"
    mv .cargo/config.toml.backup .cargo/config.toml
fi

print_status "WSL full build process completed!"
print_status "Binary location: $CARGO_TARGET_DIR/release/uveddi"

# Provide summary
echo ""
echo "=== Build Summary ==="
echo "Features: production (all features enabled)"
echo "Profile: release"
echo "Target directory: $CARGO_TARGET_DIR"
if [ -f "$CARGO_TARGET_DIR/release/uveddi" ]; then
    SIZE=$(du -h "$CARGO_TARGET_DIR/release/uveddi" | cut -f1)
    echo "Binary size: $SIZE"
fi
echo ""
echo "To run Uveddi with full features:"
echo "  $CARGO_TARGET_DIR/release/uveddi analyze ./src"
echo ""
echo "To copy binary to standard location:"
echo "  cp $CARGO_TARGET_DIR/release/uveddi ./target/release/"