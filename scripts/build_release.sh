#!/usr/bin/env bash
# Uveddi v1.0.0 Reproducible Release Build Script
# Produces signed, checksummed artifacts for Linux, macOS, and Windows
#
# Usage: ./scripts/build_release.sh [--skip-sign] [--skip-container]
#
# Environment Variables:
#   UVEDDI_GPG_KEY: GPG key ID for signing (default: auto-detect)
#   UVEDDI_BUILD_TARGET: Specific target to build (default: all)
#   UVEDDI_SKIP_TESTS: Skip test execution (default: false)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

readonly VERSION="1.0.0"
readonly PROJECT_NAME="uveddi"
readonly BUILD_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
readonly GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
readonly BUILD_HOST="$(hostname)"

# Directories
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DIST_DIR="${PROJECT_ROOT}/dist"
readonly DIST_BINARIES="${DIST_DIR}/binaries"
readonly DIST_CHECKSUMS="${DIST_DIR}/checksums"
readonly DIST_SIGNATURES="${DIST_DIR}/signatures"
readonly DIST_ARCHIVES="${DIST_DIR}/archives"

# Build configuration
readonly CARGO_PROFILE="release"
readonly CARGO_FEATURES="cli-standard"  # Production-secure CLI build
readonly RUST_TARGET_DIR="${PROJECT_ROOT}/target"

# Color codes
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly MAGENTA='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Build targets
declare -a TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

# Command-line options
SKIP_SIGNING=false
SKIP_CONTAINER=false
SKIP_TESTS="${UVEDDI_SKIP_TESTS:-false}"

# ============================================================================
# Logging Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*" >&2
}

log_header() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}$*${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

log_step() {
    echo -e "${MAGENTA}▸${NC} $*"
}

# ============================================================================
# Utility Functions
# ============================================================================

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "Required command not found: $1"
        return 1
    fi
    return 0
}

check_prerequisites() {
    log_header "Checking Prerequisites"
    
    local missing_deps=()
    
    # Essential tools
    for cmd in cargo rustc git tar gzip sha256sum; do
        if ! check_command "$cmd"; then
            missing_deps+=("$cmd")
        fi
    done
    
    # Optional but recommended tools
    for cmd in sha512sum gpg; do
        if ! check_command "$cmd"; then
            log_warning "Optional tool not found: $cmd"
        fi
    done
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

setup_build_environment() {
    log_header "Setting Up Build Environment"
    
    # Clean and recreate dist directory
    log_step "Creating distribution directories"
    rm -rf "${DIST_DIR}"
    mkdir -p "${DIST_BINARIES}" "${DIST_CHECKSUMS}" "${DIST_SIGNATURES}" "${DIST_ARCHIVES}"
    
    # Set reproducible build environment variables
    export SOURCE_DATE_EPOCH="$(git log -1 --format=%ct 2>/dev/null || date +%s)"
    export RUSTFLAGS="-C link-arg=-Wl,--build-id=none"
    
    log_success "Build environment configured"
    log_info "  Version: ${VERSION}"
    log_info "  Commit: ${GIT_COMMIT}"
    log_info "  Date: ${BUILD_DATE}"
    log_info "  Host: ${BUILD_HOST}"
    log_info "  Profile: ${CARGO_PROFILE}"
    log_info "  Features: ${CARGO_FEATURES}"
}

# ============================================================================
# Build Functions
# ============================================================================

build_target() {
    local target="$1"
    local target_dir="${RUST_TARGET_DIR}/${target}/${CARGO_PROFILE}"
    local binary_name="${PROJECT_NAME}"
    
    log_step "Building for ${target}"
    
    # Add .exe extension for Windows
    if [[ "${target}" == *"windows"* ]]; then
        binary_name="${binary_name}.exe"
    fi
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "${target}"; then
        log_info "Installing target: ${target}"
        rustup target add "${target}" || {
            log_warning "Failed to install target ${target}, skipping"
            return 0
        }
    fi
    
    # Build the binary
    if ! cargo build \
        --release \
        --features "${CARGO_FEATURES}" \
        --target "${target}" \
        --target-dir "${RUST_TARGET_DIR}" 2>&1 | grep -E "(Compiling|Finished|error)"; then
        log_warning "Build failed for ${target}, skipping"
        return 0
    fi
    
    # Check if binary was created
    if [ ! -f "${target_dir}/${binary_name}" ]; then
        log_warning "Binary not found for ${target}, skipping"
        return 0
    fi
    
    # Copy binary to dist
    local output_binary="${DIST_BINARIES}/${PROJECT_NAME}-${VERSION}-${target}${binary_name##*uveddi}"
    cp "${target_dir}/${binary_name}" "${output_binary}"
    
    # Strip debug symbols (except for musl which may fail)
    if [[ "${target}" != *"musl"* ]] && [[ "${target}" != *"windows"* ]]; then
        strip "${output_binary}" 2>/dev/null || log_warning "Failed to strip ${target}"
    fi
    
    log_success "Built ${target}: $(du -h "${output_binary}" | cut -f1)"
}

build_all_targets() {
    log_header "Building Release Binaries"
    
    if [ -n "${UVEDDI_BUILD_TARGET:-}" ]; then
        build_target "${UVEDDI_BUILD_TARGET}"
    else
        for target in "${TARGETS[@]}"; do
            build_target "${target}" || true
        done
    fi
    
    log_success "Binary builds complete"
}

# ============================================================================
# Packaging Functions
# ============================================================================

create_archives() {
    log_header "Creating Distribution Archives"
    
    cd "${DIST_BINARIES}"
    
    for binary in *; do
        if [ -f "${binary}" ]; then
            local archive_name="${binary%.exe}"
            local target_name="${archive_name##*-}"
            
            # Create staging directory
            local stage_dir="${DIST_DIR}/staging/${archive_name}"
            mkdir -p "${stage_dir}"
            
            # Copy binary
            cp "${binary}" "${stage_dir}/${PROJECT_NAME}${binary##*uveddi}"
            
            # Copy documentation
            cp "${PROJECT_ROOT}/README.md" "${stage_dir}/" 2>/dev/null || true
            cp "${PROJECT_ROOT}/LICENSE" "${stage_dir}/" 2>/dev/null || true
            cp "${PROJECT_ROOT}/CHANGELOG.md" "${stage_dir}/" 2>/dev/null || true
            
            # Create archive
            cd "${DIST_DIR}/staging"
            if [[ "${binary}" == *.exe ]]; then
                # Windows: create zip
                zip -q -r "${DIST_ARCHIVES}/${archive_name}.zip" "${archive_name}"
                log_success "Created: ${archive_name}.zip"
            else
                # Unix: create tar.gz
                tar -czf "${DIST_ARCHIVES}/${archive_name}.tar.gz" "${archive_name}"
                log_success "Created: ${archive_name}.tar.gz"
            fi
            cd "${DIST_BINARIES}"
        fi
    done
    
    # Cleanup staging
    rm -rf "${DIST_DIR}/staging"
    
    log_success "Archives created"
}

# ============================================================================
# Checksums and Signing
# ============================================================================

generate_checksums() {
    log_header "Generating Checksums"
    
    cd "${DIST_ARCHIVES}"
    
    # SHA256
    if check_command sha256sum; then
        sha256sum * > "${DIST_CHECKSUMS}/SHA256SUMS"
        log_success "Generated SHA256 checksums"
    fi
    
    # SHA512 (optional but recommended)
    if check_command sha512sum; then
        sha512sum * > "${DIST_CHECKSUMS}/SHA512SUMS"
        log_success "Generated SHA512 checksums"
    fi
    
    # Copy checksums to archives directory for convenience
    cp "${DIST_CHECKSUMS}"/* "${DIST_ARCHIVES}/"
}

sign_artifacts() {
    if [ "${SKIP_SIGNING}" = true ]; then
        log_warning "Skipping artifact signing (--skip-sign)"
        return 0
    fi
    
    log_header "Signing Artifacts"
    
    if ! check_command gpg; then
        log_warning "GPG not found, skipping signing"
        return 0
    fi
    
    # Determine GPG key
    local gpg_key="${UVEDDI_GPG_KEY:-}"
    if [ -z "${gpg_key}" ]; then
        gpg_key="$(gpg --list-secret-keys --keyid-format LONG | grep -m1 'sec' | awk '{print $2}' | cut -d'/' -f2 || true)"
    fi
    
    if [ -z "${gpg_key}" ]; then
        log_warning "No GPG key found, skipping signing"
        log_info "Set UVEDDI_GPG_KEY environment variable to specify a key"
        return 0
    fi
    
    log_info "Using GPG key: ${gpg_key}"
    
    cd "${DIST_ARCHIVES}"
    
    for file in *.tar.gz *.zip SHA256SUMS SHA512SUMS 2>/dev/null; do
        if [ -f "${file}" ]; then
            gpg --detach-sign --armor --local-user "${gpg_key}" "${file}"
            mv "${file}.asc" "${DIST_SIGNATURES}/${file}.asc"
            log_success "Signed: ${file}"
        fi
    done
    
    # Copy signatures to archives directory
    cp "${DIST_SIGNATURES}"/* "${DIST_ARCHIVES}/" 2>/dev/null || true
    
    log_success "Artifact signing complete"
}

# ============================================================================
# Testing and Verification
# ============================================================================

run_tests() {
    if [ "${SKIP_TESTS}" = true ]; then
        log_warning "Skipping tests (UVEDDI_SKIP_TESTS=true)"
        return 0
    fi
    
    log_header "Running Tests"
    
    log_step "Running unit tests"
    cargo test --features "${CARGO_FEATURES}" --release -- --nocapture
    
    log_success "All tests passed"
}

verify_artifacts() {
    log_header "Verifying Artifacts"
    
    # Verify checksums
    cd "${DIST_ARCHIVES}"
    if [ -f "SHA256SUMS" ]; then
        if sha256sum -c SHA256SUMS; then
            log_success "SHA256 checksum verification passed"
        else
            log_error "SHA256 checksum verification failed"
            return 1
        fi
    fi
    
    # Verify signatures
    if [ -d "${DIST_SIGNATURES}" ] && [ "$(ls -A "${DIST_SIGNATURES}")" ]; then
        local all_verified=true
        for sig in "${DIST_SIGNATURES}"/*.asc; do
            if [ -f "${sig}" ]; then
                local file="${DIST_ARCHIVES}/$(basename "${sig}" .asc)"
                if gpg --verify "${sig}" "${file}" 2>&1 | grep -q "Good signature"; then
                    log_success "Verified signature: $(basename "${sig}")"
                else
                    log_error "Failed to verify signature: $(basename "${sig}")"
                    all_verified=false
                fi
            fi
        done
        
        if [ "${all_verified}" = true ]; then
            log_success "All signatures verified"
        else
            log_error "Some signatures failed verification"
            return 1
        fi
    fi
    
    log_success "Artifact verification complete"
}

# ============================================================================
# Reporting
# ============================================================================

generate_build_manifest() {
    log_header "Generating Build Manifest"
    
    local manifest="${DIST_DIR}/BUILD_MANIFEST.txt"
    
    cat > "${manifest}" << EOF
Uveddi Release Build Manifest
==============================

Version: ${VERSION}
Build Date: ${BUILD_DATE}
Git Commit: ${GIT_COMMIT}
Build Host: ${BUILD_HOST}
Cargo Profile: ${CARGO_PROFILE}
Features: ${CARGO_FEATURES}

Binaries:
EOF
    
    cd "${DIST_BINARIES}"
    for binary in *; do
        if [ -f "${binary}" ]; then
            local size="$(du -h "${binary}" | cut -f1)"
            local sha256="$(sha256sum "${binary}" | cut -d' ' -f1)"
            echo "  ${binary} (${size})" >> "${manifest}"
            echo "    SHA256: ${sha256}" >> "${manifest}"
        fi
    done
    
    cat >> "${manifest}" << EOF

Archives:
EOF
    
    cd "${DIST_ARCHIVES}"
    for archive in *.tar.gz *.zip 2>/dev/null; do
        if [ -f "${archive}" ]; then
            local size="$(du -h "${archive}" | cut -f1)"
            echo "  ${archive} (${size})" >> "${manifest}"
        fi
    done
    
    log_success "Build manifest generated: ${manifest}"
}

print_summary() {
    log_header "Build Summary"
    
    echo -e "${GREEN}✓ Build completed successfully!${NC}\n"
    
    echo "Artifacts:"
    echo "  Binaries:  ${DIST_BINARIES}"
    echo "  Archives:  ${DIST_ARCHIVES}"
    echo "  Checksums: ${DIST_CHECKSUMS}"
    echo "  Signatures: ${DIST_SIGNATURES}"
    echo ""
    
    if [ -d "${DIST_ARCHIVES}" ]; then
        echo "Distribution packages:"
        cd "${DIST_ARCHIVES}"
        ls -lh *.tar.gz *.zip 2>/dev/null | awk '{printf "  %-15s %s\n", $5, $9}'
        echo ""
    fi
    
    echo "Verification commands:"
    echo "  Verify checksums: cd ${DIST_ARCHIVES} && sha256sum -c SHA256SUMS"
    if [ -d "${DIST_SIGNATURES}" ] && [ "$(ls -A "${DIST_SIGNATURES}")" ]; then
        echo "  Verify signatures: gpg --verify <file>.asc <file>"
    fi
    echo ""
    
    echo "Next steps:"
    echo "  1. Test binaries on target platforms"
    echo "  2. Review ${DIST_DIR}/BUILD_MANIFEST.txt"
    echo "  3. Upload to release distribution"
    echo ""
}

# ============================================================================
# Main Execution
# ============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-sign)
                SKIP_SIGNING=true
                shift
                ;;
            --skip-container)
                SKIP_CONTAINER=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Usage: $0 [--skip-sign] [--skip-container] [--skip-tests]"
                exit 1
                ;;
        esac
    done
}

main() {
    cd "${PROJECT_ROOT}"
    
    parse_args "$@"
    
    log_header "Uveddi v${VERSION} Release Builder"
    
    check_prerequisites
    setup_build_environment
    run_tests
    build_all_targets
    create_archives
    generate_checksums
    sign_artifacts
    verify_artifacts
    generate_build_manifest
    print_summary
    
    log_success "Release build complete!"
}

main "$@"
