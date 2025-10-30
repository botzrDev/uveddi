#!/bin/bash
# Incremental build approach for WSL - builds features progressively to avoid timeout

set -e

echo "=== WSL Incremental Feature Build ==="
echo "Building Uveddi with all features by progressively adding them"
echo "This avoids the 500-second WSL timeout issue"
echo ""

# Use temp target directory for faster I/O
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/uveddi-target}"
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C codegen-units=256 -C incremental=true"

# Create target directory
mkdir -p "$CARGO_TARGET_DIR"

echo "[1/7] Building minimal core..."
cargo build --release --features dev-minimal
echo "Cooling down for 5 seconds..."
sleep 5

echo "[2/7] Adding Rust language support..."
cargo build --release --features "dev-minimal rust-lang"
echo "Cooling down for 5 seconds..."
sleep 5

echo "[3/7] Adding all tree-sitter languages..."
cargo build --release --features "dev-minimal tree-sitter"
echo "Cooling down for 5 seconds..."
sleep 5

echo "[4/7] Adding web features..."
cargo build --release --features "dev-minimal tree-sitter web-full"
echo "Cooling down for 5 seconds..."
sleep 5

echo "[5/7] Adding security features..."
cargo build --release --features "dev-minimal tree-sitter web-full security"
echo "Cooling down for 5 seconds..."
sleep 5

echo "[6/7] Adding memory optimization..."
cargo build --release --features "dev-minimal tree-sitter web-full security memory-optimization"
echo "Cooling down for 5 seconds..."
sleep 5

echo "[7/7] Building with full production features..."
cargo build --release --features production

echo ""
echo "=== Build Complete ==="
echo "Binary: $CARGO_TARGET_DIR/release/uveddi"
echo ""
echo "Running tests with full features..."
cargo test --release --features production --no-fail-fast

echo ""
echo "Full feature build and test completed successfully!"