#!/bin/bash
set -e

# Uveddi Plugin Build Script
# Builds and packages WASM plugins for distribution

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
PLUGIN_DIR=""
OUTPUT_DIR=""
OPTIMIZE=true
RUN_TESTS=true
VERBOSE=false

usage() {
    cat << EOF
Usage: $0 [OPTIONS] <plugin-directory>

Build and package an Uveddi WASM plugin.

OPTIONS:
    -o, --output DIR        Output directory for built plugin (default: plugin-dir/dist)
    -n, --no-optimize      Skip WASM optimization with wasm-opt
    -t, --no-tests         Skip running tests
    -v, --verbose          Verbose output
    -h, --help            Show this help message

EXAMPLES:
    $0 my-plugin/                         # Build plugin in my-plugin/
    $0 -o dist/ --no-optimize my-plugin/  # Build without optimization to dist/
    $0 --no-tests my-plugin/              # Build without running tests

REQUIREMENTS:
    - rust (with wasm32-unknown-unknown target)
    - wasm-pack
    - wasm-opt (from binaryen, optional)
EOF
}

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

check_requirements() {
    log "Checking build requirements..."
    
    # Check for Rust
    if ! command -v rustc >/dev/null 2>&1; then
        error "Rust is not installed. Please install Rust from https://rustup.rs/"
    fi
    
    # Check for wasm32 target
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        log "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi
    
    # Check for wasm-pack
    if ! command -v wasm-pack >/dev/null 2>&1; then
        error "wasm-pack is not installed. Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    fi
    
    # Check for wasm-opt (optional)
    if $OPTIMIZE && ! command -v wasm-opt >/dev/null 2>&1; then
        warn "wasm-opt not found. Install binaryen for WASM optimization."
        warn "On macOS: brew install binaryen"
        warn "On Ubuntu: apt-get install binaryen"
        OPTIMIZE=false
    fi
}

validate_plugin() {
    local plugin_dir="$1"
    
    log "Validating plugin structure..."
    
    if [[ ! -f "$plugin_dir/Cargo.toml" ]]; then
        error "No Cargo.toml found in $plugin_dir"
    fi
    
    if [[ ! -f "$plugin_dir/plugin.toml" ]]; then
        error "No plugin.toml manifest found in $plugin_dir"
    fi
    
    if [[ ! -d "$plugin_dir/src" ]]; then
        error "No src/ directory found in $plugin_dir"
    fi
    
    # Check for required crate-type
    if ! grep -q 'crate-type.*=.*\["cdylib"\]' "$plugin_dir/Cargo.toml"; then
        error "Cargo.toml must specify crate-type = [\"cdylib\"]"
    fi
}

run_tests() {
    local plugin_dir="$1"
    
    if $RUN_TESTS; then
        log "Running plugin tests..."
        cd "$plugin_dir"
        
        # Run unit tests
        if $VERBOSE; then
            cargo test --verbose
        else
            cargo test
        fi
        
        # Run clippy lints
        if command -v cargo-clippy >/dev/null 2>&1; then
            log "Running clippy lints..."
            cargo clippy -- -D warnings
        fi
    else
        log "Skipping tests (--no-tests specified)"
    fi
}

build_wasm() {
    local plugin_dir="$1"
    local output_dir="$2"
    
    log "Building WASM plugin..."
    cd "$plugin_dir"
    
    # Create temporary build directory
    local build_dir="$plugin_dir/pkg"
    rm -rf "$build_dir"
    
    # Build with wasm-pack
    local wasm_pack_args=("build" "--target" "web" "--out-dir" "pkg")
    
    if $VERBOSE; then
        wasm_pack_args+=("--verbose")
    fi
    
    if ! wasm-pack "${wasm_pack_args[@]}"; then
        error "Failed to build WASM plugin"
    fi
    
    # Find the generated WASM file
    local wasm_file
    wasm_file=$(find "$build_dir" -name "*.wasm" | head -n 1)
    
    if [[ -z "$wasm_file" ]]; then
        error "No WASM file generated"
    fi
    
    log "WASM file generated: $wasm_file"
    
    # Optimize if requested
    if $OPTIMIZE; then
        log "Optimizing WASM file..."
        local optimized_wasm="${wasm_file%.*}_opt.wasm"
        
        if wasm-opt -Oz "$wasm_file" -o "$optimized_wasm"; then
            local original_size
            local optimized_size
            original_size=$(wc -c < "$wasm_file")
            optimized_size=$(wc -c < "$optimized_wasm")
            
            log "Optimization complete: $original_size bytes -> $optimized_size bytes"
            mv "$optimized_wasm" "$wasm_file"
        else
            warn "WASM optimization failed, using unoptimized version"
        fi
    fi
    
    # Package the plugin
    package_plugin "$plugin_dir" "$output_dir"
}

package_plugin() {
    local plugin_dir="$1"
    local output_dir="$2"
    
    log "Packaging plugin..."
    
    # Create output directory
    mkdir -p "$output_dir"
    
    # Get plugin name from Cargo.toml
    local plugin_name
    plugin_name=$(grep '^name = ' "$plugin_dir/Cargo.toml" | cut -d'"' -f2)
    
    if [[ -z "$plugin_name" ]]; then
        error "Could not determine plugin name from Cargo.toml"
    fi
    
    # Create plugin package directory
    local package_dir="$output_dir/$plugin_name"
    rm -rf "$package_dir"
    mkdir -p "$package_dir"
    
    # Copy WASM file
    local wasm_file
    wasm_file=$(find "$plugin_dir/pkg" -name "*.wasm" | head -n 1)
    cp "$wasm_file" "$package_dir/plugin.wasm"
    
    # Copy manifest
    cp "$plugin_dir/plugin.toml" "$package_dir/"
    
    # Copy README if it exists
    if [[ -f "$plugin_dir/README.md" ]]; then
        cp "$plugin_dir/README.md" "$package_dir/"
    fi
    
    # Create plugin info
    cat > "$package_dir/BUILD_INFO.txt" << EOF
Built: $(date -u)
Build Host: $(hostname)
Rust Version: $(rustc --version)
wasm-pack Version: $(wasm-pack --version | head -n1)
Optimized: $OPTIMIZE
EOF
    
    # Create ZIP package
    local zip_file="$output_dir/${plugin_name}.zip"
    cd "$output_dir"
    zip -r "${plugin_name}.zip" "$plugin_name/"
    
    log "Plugin packaged successfully:"
    log "  Directory: $package_dir"
    log "  Archive: $zip_file"
    log "  WASM Size: $(wc -c < "$package_dir/plugin.wasm") bytes"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -n|--no-optimize)
            OPTIMIZE=false
            shift
            ;;
        -t|--no-tests)
            RUN_TESTS=false
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            error "Unknown option: $1"
            ;;
        *)
            if [[ -z "$PLUGIN_DIR" ]]; then
                PLUGIN_DIR="$1"
            else
                error "Multiple plugin directories specified"
            fi
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "$PLUGIN_DIR" ]]; then
    error "Plugin directory is required"
fi

if [[ ! -d "$PLUGIN_DIR" ]]; then
    error "Plugin directory does not exist: $PLUGIN_DIR"
fi

# Set default output directory
if [[ -z "$OUTPUT_DIR" ]]; then
    OUTPUT_DIR="$PLUGIN_DIR/dist"
fi

# Convert to absolute paths
PLUGIN_DIR=$(realpath "$PLUGIN_DIR")
OUTPUT_DIR=$(realpath "$OUTPUT_DIR")

log "Building plugin in: $PLUGIN_DIR"
log "Output directory: $OUTPUT_DIR"

# Main build process
check_requirements
validate_plugin "$PLUGIN_DIR"
run_tests "$PLUGIN_DIR"
build_wasm "$PLUGIN_DIR" "$OUTPUT_DIR"

log "Build complete! 🎉"