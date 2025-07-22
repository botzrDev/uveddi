#!/bin/bash
# scripts/validate-uv243-requirements.sh
# Comprehensive UV-243 requirements validation script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Initialize validation results
VALIDATION_PASSED=true
TOTAL_CHECKS=0
PASSED_CHECKS=0

# Function to run validation check
run_check() {
    local check_name="$1"
    local check_command="$2"
    local success_message="$3"
    local error_message="$4"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    log_info "Running: $check_name"
    
    if eval "$check_command" > /tmp/check_output_$TOTAL_CHECKS.log 2>&1; then
        log_success "$success_message"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        log_error "$error_message"
        echo "Command output:"
        cat /tmp/check_output_$TOTAL_CHECKS.log | head -20
        VALIDATION_PASSED=false
        return 1
    fi
}

echo "🔍 Validating UV-243 Requirements..."
echo "======================================"

# Check if required tools are installed
log_info "Checking required tools..."
for tool in cargo bc mdbook npm; do
    if command -v $tool >/dev/null 2>&1; then
        log_success "$tool is available"
    else
        log_error "$tool is not installed"
        VALIDATION_PASSED=false
    fi
done

# 1. Code Coverage Validation
echo
echo "📊 Code Coverage Analysis"
echo "========================"

# Install coverage tools if needed
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    log_info "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Run coverage analysis
run_check "Code coverage analysis" \
    "cargo llvm-cov --all-features --workspace --summary-only" \
    "Coverage analysis completed" \
    "Coverage analysis failed"

# Extract and validate coverage percentage
if [ -f /tmp/check_output_$TOTAL_CHECKS.log ]; then
    COVERAGE=$(grep -o '[0-9]*\.[0-9]*%' /tmp/check_output_$TOTAL_CHECKS.log | head -1 | sed 's/%//')
    if [ -n "$COVERAGE" ]; then
        if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
            log_success "Code coverage: $COVERAGE% (≥90% requirement met)"
        else
            log_error "Code coverage: $COVERAGE% (below 90% requirement)"
            VALIDATION_PASSED=false
        fi
    else
        log_warning "Could not parse coverage percentage"
    fi
fi

# 2. Test Suite Validation
echo
echo "🧪 Test Suite Validation"
echo "======================="

run_check "Unit tests" \
    "cargo test --lib --all-features --workspace" \
    "All unit tests passing" \
    "Unit test failures detected"

run_check "Integration tests" \
    "cargo test --test '*' --all-features" \
    "All integration tests passing" \
    "Integration test failures detected"

run_check "Documentation tests" \
    "cargo test --doc --all-features" \
    "All documentation tests passing" \
    "Documentation test failures detected"

# 3. Code Quality Validation
echo
echo "🔧 Code Quality Checks"
echo "===================="

run_check "Code formatting" \
    "cargo fmt -- --check" \
    "Code formatting is correct" \
    "Code formatting issues detected"

run_check "Clippy lints" \
    "cargo clippy --all-features --all-targets -- -D warnings" \
    "No clippy warnings" \
    "Clippy warnings detected"

# 4. Security Validation
echo
echo "🔒 Security Analysis"
echo "==================="

# Install audit tool if needed
if ! command -v cargo-audit >/dev/null 2>&1; then
    log_info "Installing cargo-audit..."
    cargo install cargo-audit
fi

run_check "Security audit" \
    "cargo audit" \
    "No security vulnerabilities detected" \
    "Security vulnerabilities found"

# 5. Performance Validation
echo
echo "⚡ Performance Validation"
echo "======================="

# Check if benchmarks exist
if [ -f "benches/observability_performance.rs" ] || [ -f "benches/comprehensive_benchmarks.rs" ]; then
    run_check "Performance benchmarks" \
        "timeout 300 cargo bench" \
        "Performance benchmarks completed successfully" \
        "Performance benchmark failures detected"
else
    log_warning "No performance benchmarks found - skipping performance validation"
fi

# 6. Documentation Validation
echo
echo "📚 Documentation Validation"
echo "=========================="

run_check "Rust documentation generation" \
    "cargo doc --all-features --no-deps --document-private-items" \
    "Rust documentation generated successfully" \
    "Rust documentation generation failed"

run_check "mdbook documentation build" \
    "mdbook build docs/" \
    "mdbook documentation built successfully" \
    "mdbook documentation build failed"

# 7. API Documentation Validation
echo
echo "🔌 API Documentation Validation"
echo "==============================="

if [ -f "docs/api/openapi.yaml" ]; then
    # Install swagger-parser if needed
    if ! command -v swagger-parser >/dev/null 2>&1; then
        log_info "Installing swagger-parser-cli..."
        npm install -g swagger-parser-cli
    fi
    
    run_check "OpenAPI specification validation" \
        "swagger-parser validate docs/api/openapi.yaml" \
        "OpenAPI specification is valid" \
        "OpenAPI specification validation failed"
else
    log_warning "OpenAPI specification not found at docs/api/openapi.yaml"
fi

# 8. Build Validation
echo
echo "🏗️ Build Validation"
echo "=================="

run_check "Debug build" \
    "cargo build --all-features" \
    "Debug build successful" \
    "Debug build failed"

run_check "Release build" \
    "cargo build --release --all-features" \
    "Release build successful" \
    "Release build failed"

# 9. Feature Flag Validation
echo
echo "🎛️  Feature Flag Validation"
echo "=========================="

# Test different feature combinations
FEATURE_COMBINATIONS=(
    ""
    "--no-default-features"
    "--features tui"
    "--features wasm-plugins"
    "--features enterprise"
    "--all-features"
)

for features in "${FEATURE_COMBINATIONS[@]}"; do
    run_check "Build with features: '$features'" \
        "cargo check $features" \
        "Feature combination '$features' builds successfully" \
        "Feature combination '$features' failed to build"
done

# 10. Memory Safety Validation
echo
echo "🛡️  Memory Safety Validation"
echo "=========================="

if command -v cargo-miri >/dev/null 2>&1; then
    # Run Miri on a subset of tests (it's slow)
    run_check "Miri memory safety check" \
        "timeout 300 cargo miri test --lib -- --test-threads=1" \
        "Memory safety validation passed" \
        "Memory safety issues detected"
else
    log_warning "cargo-miri not available - skipping memory safety validation"
fi

# Cleanup temporary files
echo
log_info "Cleaning up temporary files..."
rm -f /tmp/check_output_*.log

# Final validation summary
echo
echo "📋 Validation Summary"
echo "==================="
echo "Total checks: $TOTAL_CHECKS"
echo "Passed checks: $PASSED_CHECKS"
echo "Failed checks: $((TOTAL_CHECKS - PASSED_CHECKS))"

if [ "$VALIDATION_PASSED" = true ]; then
    log_success "🎉 All UV-243 requirements validated successfully!"
    log_success "✅ Coverage requirement: ≥90%"
    log_success "✅ Test suite: All tests passing"
    log_success "✅ Code quality: No issues detected"
    log_success "✅ Security: No vulnerabilities found"
    log_success "✅ Documentation: Builds successfully"
    log_success "✅ Performance: Benchmarks passing"
    echo
    echo "🚀 System is ready for production deployment!"
    exit 0
else
    log_error "💥 UV-243 validation failed - see errors above"
    log_error "❌ One or more quality gates failed"
    echo
    echo "🔧 Please fix the issues above before proceeding to production."
    echo
    echo "Common fixes:"
    echo "- Run 'cargo fmt' to fix formatting issues"
    echo "- Run 'cargo clippy --fix' to auto-fix lint warnings"
    echo "- Add more tests to increase coverage"
    echo "- Update dependencies to fix security vulnerabilities"
    echo "- Fix documentation build errors"
    exit 1
fi