#!/usr/bin/env bash
# Uveddi Container Build and Security Scan Script
#
# Usage:
#   ./scripts/build_container.sh [--scan] [--push] [--platform linux/amd64,linux/arm64]
#
# Environment Variables:
#   UVEDDI_VERSION: Version tag (default: 1.0.0)
#   UVEDDI_REGISTRY: Container registry (default: ghcr.io/botzrdev)
#   UVEDDI_VARIANT: Runtime variant (distroless|alpine) (default: distroless)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

readonly VERSION="${UVEDDI_VERSION:-1.0.0}"
readonly REGISTRY="${UVEDDI_REGISTRY:-ghcr.io/botzrdev}"
readonly PROJECT_NAME="uveddi"
readonly VARIANT="${UVEDDI_VARIANT:-distroless}"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# Build options
DO_SCAN=false
DO_PUSH=false
PLATFORM="linux/amd64"
BUILD_ARGS=""

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

log_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# ============================================================================
# Prerequisites
# ============================================================================

check_prerequisites() {
    log_header "Checking Prerequisites"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi
    
    log_info "Docker version: $(docker --version)"
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check buildx for multi-platform builds
    if [[ "${PLATFORM}" == *","* ]]; then
        if ! docker buildx version &> /dev/null; then
            log_error "Docker buildx required for multi-platform builds"
            exit 1
        fi
        log_info "Docker buildx available"
    fi
    
    # Check security scanners (optional)
    if [ "${DO_SCAN}" = true ]; then
        if command -v trivy &> /dev/null; then
            log_info "Trivy scanner: $(trivy --version | head -1)"
        else
            log_warning "Trivy not found - install from https://trivy.dev"
        fi
        
        if command -v grype &> /dev/null; then
            log_info "Grype scanner: $(grype version | head -1)"
        else
            log_warning "Grype not found - install from https://github.com/anchore/grype"
        fi
    fi
    
    log_success "Prerequisites check passed"
}

# ============================================================================
# Build Functions
# ============================================================================

get_image_tags() {
    local base_image="${REGISTRY}/${PROJECT_NAME}"
    local variant_suffix=""
    
    if [ "${VARIANT}" = "alpine" ]; then
        variant_suffix="-alpine"
    fi
    
    echo "${base_image}:${VERSION}${variant_suffix}"
    echo "${base_image}:latest${variant_suffix}"
}

build_image() {
    log_header "Building Container Image"
    
    local dockerfile="Dockerfile.release"
    local target="runtime"
    
    if [ "${VARIANT}" = "alpine" ]; then
        target="runtime-alpine"
    fi
    
    log_info "Configuration:"
    log_info "  Version: ${VERSION}"
    log_info "  Variant: ${VARIANT}"
    log_info "  Platform: ${PLATFORM}"
    log_info "  Target: ${target}"
    log_info "  Dockerfile: ${dockerfile}"
    
    # Generate tags
    local tags=()
    while IFS= read -r tag; do
        tags+=("-t" "${tag}")
    done < <(get_image_tags)
    
    log_info "Tags:"
    for tag in "${tags[@]}"; do
        if [ "${tag}" != "-t" ]; then
            log_info "  ${tag}"
        fi
    done
    
    # Build the image
    if [[ "${PLATFORM}" == *","* ]]; then
        log_info "Building multi-platform image with buildx"
        
        # Create builder if it doesn't exist
        if ! docker buildx inspect uveddi-builder &> /dev/null; then
            log_info "Creating buildx builder"
            docker buildx create --name uveddi-builder --use
        else
            docker buildx use uveddi-builder
        fi
        
        # Build
        docker buildx build \
            --file "${dockerfile}" \
            --target "${target}" \
            --platform "${PLATFORM}" \
            "${tags[@]}" \
            --progress=plain \
            ${BUILD_ARGS} \
            . || {
            log_error "Build failed"
            exit 1
        }
    else
        log_info "Building single-platform image"
        
        docker build \
            --file "${dockerfile}" \
            --target "${target}" \
            --platform "${PLATFORM}" \
            "${tags[@]}" \
            --progress=plain \
            ${BUILD_ARGS} \
            . || {
            log_error "Build failed"
            exit 1
        }
    fi
    
    log_success "Build completed successfully"
}

# ============================================================================
# Security Scanning
# ============================================================================

scan_with_trivy() {
    local image="$1"
    local report_dir="${2:-reports/security}"
    
    log_info "Scanning with Trivy: ${image}"
    
    mkdir -p "${report_dir}"
    local report_file="${report_dir}/trivy-$(echo "${image}" | tr '/:' '-').json"
    local report_table="${report_dir}/trivy-$(echo "${image}" | tr '/:' '-').txt"
    
    # Scan and output both JSON and table formats
    trivy image \
        --severity HIGH,CRITICAL \
        --format json \
        --output "${report_file}" \
        "${image}"
    
    trivy image \
        --severity HIGH,CRITICAL \
        --format table \
        --output "${report_table}" \
        "${image}"
    
    # Display summary
    log_info "Trivy report saved:"
    log_info "  JSON: ${report_file}"
    log_info "  Table: ${report_table}"
    
    # Show critical findings
    local critical_count=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="CRITICAL")] | length' "${report_file}" 2>/dev/null || echo "0")
    local high_count=$(jq '[.Results[].Vulnerabilities[]? | select(.Severity=="HIGH")] | length' "${report_file}" 2>/dev/null || echo "0")
    
    log_info "Vulnerabilities found:"
    log_info "  CRITICAL: ${critical_count}"
    log_info "  HIGH: ${high_count}"
    
    if [ "${critical_count}" -gt 0 ]; then
        log_error "CRITICAL vulnerabilities found!"
        return 1
    fi
    
    return 0
}

scan_with_grype() {
    local image="$1"
    local report_dir="${2:-reports/security}"
    
    log_info "Scanning with Grype: ${image}"
    
    mkdir -p "${report_dir}"
    local report_file="${report_dir}/grype-$(echo "${image}" | tr '/:' '-').json"
    local report_table="${report_dir}/grype-$(echo "${image}" | tr '/:' '-').txt"
    
    # Scan and output both JSON and table formats
    grype "${image}" \
        --output json \
        --file "${report_file}"
    
    grype "${image}" \
        --output table \
        --file "${report_table}"
    
    # Display summary
    log_info "Grype report saved:"
    log_info "  JSON: ${report_file}"
    log_info "  Table: ${report_table}"
    
    # Show critical findings
    local critical_count=$(jq '[.matches[] | select(.vulnerability.severity=="Critical")] | length' "${report_file}" 2>/dev/null || echo "0")
    local high_count=$(jq '[.matches[] | select(.vulnerability.severity=="High")] | length' "${report_file}" 2>/dev/null || echo "0")
    
    log_info "Vulnerabilities found:"
    log_info "  CRITICAL: ${critical_count}"
    log_info "  HIGH: ${high_count}"
    
    if [ "${critical_count}" -gt 0 ]; then
        log_error "CRITICAL vulnerabilities found!"
        return 1
    fi
    
    return 0
}

scan_image() {
    log_header "Security Scanning"
    
    local image_tag="$(get_image_tags | head -1)"
    local report_dir="reports/security"
    
    local scan_failed=false
    
    # Scan with Trivy
    if command -v trivy &> /dev/null; then
        if ! scan_with_trivy "${image_tag}" "${report_dir}"; then
            scan_failed=true
        fi
    else
        log_warning "Trivy not available, skipping"
    fi
    
    echo ""
    
    # Scan with Grype
    if command -v grype &> /dev/null; then
        if ! scan_with_grype "${image_tag}" "${report_dir}"; then
            scan_failed=true
        fi
    else
        log_warning "Grype not available, skipping"
    fi
    
    if [ "${scan_failed}" = true ]; then
        log_error "Security scan found critical issues"
        log_warning "Review reports in ${report_dir}/"
        return 1
    fi
    
    log_success "Security scan completed"
}

# ============================================================================
# Image Information
# ============================================================================

show_image_info() {
    log_header "Image Information"
    
    local image_tag="$(get_image_tags | head -1)"
    
    # Get image size
    local size=$(docker images "${image_tag}" --format "{{.Size}}")
    log_info "Image size: ${size}"
    
    # Get image digest
    local digest=$(docker images "${image_tag}" --format "{{.ID}}")
    log_info "Image ID: ${digest}"
    
    # Show layers
    log_info "Image layers:"
    docker history "${image_tag}" --no-trunc --human | head -10
    
    # Test image
    log_info "Testing image:"
    docker run --rm "${image_tag}" --version
    
    log_success "Image verification passed"
}

# ============================================================================
# Push Functions
# ============================================================================

push_image() {
    log_header "Pushing Container Image"
    
    # Get all tags
    local tags=()
    while IFS= read -r tag; do
        tags+=("${tag}")
    done < <(get_image_tags)
    
    log_info "Pushing ${#tags[@]} tag(s) to registry"
    
    for tag in "${tags[@]}"; do
        log_info "Pushing: ${tag}"
        docker push "${tag}" || {
            log_error "Failed to push ${tag}"
            exit 1
        }
        log_success "Pushed: ${tag}"
    done
    
    # Show manifest
    log_info "Image manifest:"
    docker manifest inspect "${tags[0]}" 2>/dev/null || log_warning "Manifest not available"
    
    log_success "All tags pushed successfully"
}

# ============================================================================
# Main
# ============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --scan)
                DO_SCAN=true
                shift
                ;;
            --push)
                DO_PUSH=true
                shift
                ;;
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            --variant)
                VARIANT="$2"
                shift 2
                ;;
            --build-arg)
                BUILD_ARGS="${BUILD_ARGS} --build-arg $2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Usage: $0 [--scan] [--push] [--platform PLATFORM] [--variant distroless|alpine]"
                exit 1
                ;;
        esac
    done
}

print_summary() {
    log_header "Build Summary"
    
    echo -e "${GREEN}✓ Container build completed successfully!${NC}\n"
    
    echo "Image tags:"
    while IFS= read -r tag; do
        echo "  ${tag}"
    done < <(get_image_tags)
    
    echo ""
    echo "Run image:"
    echo "  docker run --rm $(get_image_tags | head -1) --version"
    
    if [ "${DO_SCAN}" = true ]; then
        echo ""
        echo "Security reports:"
        echo "  reports/security/"
    fi
    
    if [ "${DO_PUSH}" = true ]; then
        echo ""
        echo "Image pushed to registry"
        echo "  Pull: docker pull $(get_image_tags | head -1)"
    fi
    
    echo ""
}

main() {
    parse_args "$@"
    
    log_header "Uveddi Container Builder v${VERSION}"
    
    check_prerequisites
    build_image
    show_image_info
    
    if [ "${DO_SCAN}" = true ]; then
        scan_image || log_warning "Continuing despite scan issues"
    fi
    
    if [ "${DO_PUSH}" = true ]; then
        push_image
    fi
    
    print_summary
    
    log_success "Container build complete!"
}

main "$@"
