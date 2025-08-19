#!/bin/bash
set -euo pipefail

# Automated Testing Script for Uveddi TUI & CLI
# This script runs comprehensive automated tests for both TUI and CLI components

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FEATURES="community"
TIMEOUT=300  # 5 minutes timeout for tests
PARALLEL_JOBS=$(nproc)
COVERAGE_THRESHOLD=80

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    # Check required tools
    local tools=("jq" "timeout")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_warning "$tool not found. Some tests may be skipped."
        fi
    done
    
    log_success "Dependencies checked"
}

build_binaries() {
    log_info "Building binaries with features: $FEATURES"
    
    # Build main CLI binary
    if ! cargo build --release --features="$FEATURES" --bin uveddi; then
        log_error "Failed to build CLI binary"
        exit 1
    fi
    
    # Build TUI test binary if available
    if cargo build --features="$FEATURES" --bin tui_test 2>/dev/null; then
        log_success "TUI test binary built"
    else
        log_warning "TUI test binary not available"
    fi
    
    log_success "Binaries built successfully"
}

run_unit_tests() {
    log_info "Running unit tests..."
    
    local test_patterns=(
        "tui_component_automation"
        "tui_form_validation" 
        "tui_virtual_terminal_automation"
    )
    
    local failed_tests=()
    
    for pattern in "${test_patterns[@]}"; do
        log_info "Running test: $pattern"
        
        if timeout "$TIMEOUT" cargo test --test "$pattern" --features="$FEATURES" -- --nocapture; then
            log_success "Test $pattern passed"
        else
            log_error "Test $pattern failed"
            failed_tests+=("$pattern")
        fi
    done
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All unit tests passed"
        return 0
    else
        log_error "Failed unit tests: ${failed_tests[*]}"
        return 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    
    local test_patterns=(
        "cli_comprehensive_automation"
        "cli_integration"
        "tui_e2e"
    )
    
    local failed_tests=()
    
    for pattern in "${test_patterns[@]}"; do
        log_info "Running integration test: $pattern"
        
        if timeout "$TIMEOUT" cargo test --test "$pattern" --features="$FEATURES" -- --nocapture; then
            log_success "Integration test $pattern passed"
        else
            log_error "Integration test $pattern failed"
            failed_tests+=("$pattern")
        fi
    done
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All integration tests passed"
        return 0
    else
        log_error "Failed integration tests: ${failed_tests[*]}"
        return 1
    fi
}

run_cli_smoke_tests() {
    log_info "Running CLI smoke tests..."
    
    local binary="./target/release/uveddi"
    
    if [[ ! -x "$binary" ]]; then
        log_error "CLI binary not found or not executable"
        return 1
    fi
    
    # Test basic commands
    log_info "Testing --version"
    if ! "$binary" --version; then
        log_error "Version command failed"
        return 1
    fi
    
    log_info "Testing --help"
    if ! "$binary" --help > /dev/null; then
        log_error "Help command failed"
        return 1
    fi
    
    log_info "Testing analyze --help"
    if ! "$binary" analyze --help > /dev/null; then
        log_error "Analyze help command failed"
        return 1
    fi
    
    # Test error handling
    log_info "Testing error handling with invalid path"
    if "$binary" analyze /nonexistent/path/nowhere 2>/dev/null; then
        log_error "Should have failed with invalid path"
        return 1
    fi
    
    log_info "Testing error handling with invalid format"
    if "$binary" analyze . --output-format=invalid 2>/dev/null; then
        log_error "Should have failed with invalid format"
        return 1
    fi
    
    log_success "CLI smoke tests passed"
    return 0
}

run_functional_tests() {
    log_info "Running functional tests with test project..."
    
    # Create test project
    local test_dir="/tmp/uveddi_functional_test_$$"
    mkdir -p "$test_dir"
    
    # Create test files with various patterns
    cat > "$test_dir/main.rs" << 'EOF'
// Test file with detectable patterns
pub struct LargeClass {
    field1: String, field2: i32, field3: f64, field4: bool,
    field5: Vec<String>, field6: std::collections::HashMap<String, i32>,
    field7: Option<String>, field8: Result<i32, String>,
    field9: u64, field10: char, field11: Box<dyn std::fmt::Display>,
}

impl LargeClass {
    pub fn new() -> Self { unimplemented!() }
    pub fn method1(&self) -> String { unimplemented!() }
    pub fn method2(&self) -> i32 { unimplemented!() }
    pub fn method3(&self) -> f64 { unimplemented!() }
    pub fn method4(&self) -> bool { unimplemented!() }
    pub fn method5(&self) -> Vec<String> { unimplemented!() }
    
    // Dead code
    #[allow(dead_code)]
    fn unused_method(&self) {
        println!("This method is never called");
    }
}

// Dead code function
#[allow(dead_code)]
fn unused_function() {
    println!("This function is never used");
}

pub fn main() {
    let instance = LargeClass::new();
    println!("{:?}", instance.method1());
}
EOF

    cat > "$test_dir/lib.rs" << 'EOF'
pub mod utils;

pub use main::LargeClass;

pub fn library_function() -> String {
    "library function".to_string()
}
EOF

    cat > "$test_dir/utils.rs" << 'EOF'
pub struct Utility {
    data: String,
}

impl Utility {
    pub fn new() -> Self {
        Self { data: String::new() }
    }
    
    #[allow(dead_code)]
    fn unused_utility(&self) {
        println!("Unused utility method");
    }
}
EOF
    
    local binary="./target/release/uveddi"
    local failed_tests=()
    
    # Test markdown output
    log_info "Testing markdown output"
    if ! "$binary" analyze "$test_dir" --output-format=markdown > "$test_dir/report.md"; then
        log_error "Markdown output test failed"
        failed_tests+=("markdown")
    elif ! grep -q "# Uveddi Analysis Report" "$test_dir/report.md"; then
        log_error "Markdown report doesn't contain expected header"
        failed_tests+=("markdown_content")
    else
        log_success "Markdown output test passed"
    fi
    
    # Test JSON output
    log_info "Testing JSON output"
    if ! "$binary" analyze "$test_dir" --output-format=json > "$test_dir/report.json"; then
        log_error "JSON output test failed"
        failed_tests+=("json")
    elif ! command -v jq &> /dev/null || ! jq -e '.run_id and .issues and .summary' "$test_dir/report.json" > /dev/null; then
        log_error "JSON report doesn't contain expected structure"
        failed_tests+=("json_structure")
    else
        log_success "JSON output test passed"
    fi
    
    # Test with options
    log_info "Testing with analysis options"
    if ! "$binary" analyze "$test_dir" \
        --output-format=json \
        --dead-code-confidence=0.8 \
        --large-classes-max-loc=50 \
        --large-classes-max-methods=3 \
        --large-classes-max-fields=5 > "$test_dir/detailed.json"; then
        log_error "Detailed analysis test failed"
        failed_tests+=("detailed")
    else
        log_success "Detailed analysis test passed"
    fi
    
    # Test output to file
    log_info "Testing file output"
    if ! "$binary" analyze "$test_dir" --output-format=json --output "$test_dir/file_output.json"; then
        log_error "File output test failed"  
        failed_tests+=("file_output")
    elif [[ ! -f "$test_dir/file_output.json" ]]; then
        log_error "Output file was not created"
        failed_tests+=("file_creation")
    else
        log_success "File output test passed"
    fi
    
    # Cleanup
    rm -rf "$test_dir"
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All functional tests passed"
        return 0
    else
        log_error "Failed functional tests: ${failed_tests[*]}"
        return 1
    fi
}

run_performance_tests() {
    log_info "Running performance tests..."
    
    # Create larger test project
    local test_dir="/tmp/uveddi_perf_test_$$"
    mkdir -p "$test_dir"
    
    # Generate multiple files
    for i in {1..50}; do
        cat > "$test_dir/file$i.rs" << EOF
pub struct TestStruct$i {
    field1: String,
    field2: i32,
    field3: f64,
}

impl TestStruct$i {
    pub fn new() -> Self {
        Self {
            field1: String::new(),
            field2: 0,
            field3: 0.0,
        }
    }
    
    pub fn method1(&self) -> String { self.field1.clone() }
    pub fn method2(&self) -> i32 { self.field2 }
    
    #[allow(dead_code)]
    fn unused_method$i(&self) {
        println!("Unused method in file $i");
    }
}
EOF
    done
    
    local binary="./target/release/uveddi"
    
    # Time the analysis
    log_info "Running performance test on larger project (50 files)"
    
    local start_time=$(date +%s)
    if ! timeout 60 "$binary" analyze "$test_dir" --output-format=json > /dev/null; then
        log_error "Performance test failed or timed out"
        rm -rf "$test_dir"
        return 1
    fi
    local end_time=$(date +%s)
    
    local duration=$((end_time - start_time))
    log_info "Analysis completed in $duration seconds"
    
    if [[ $duration -gt 30 ]]; then
        log_warning "Analysis took longer than expected ($duration seconds)"
    else
        log_success "Performance test passed ($duration seconds)"
    fi
    
    # Cleanup
    rm -rf "$test_dir"
    return 0
}

run_coverage_tests() {
    log_info "Running coverage tests..."
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warning "cargo-tarpaulin not installed, skipping coverage tests"
        return 0
    fi
    
    # Run coverage for TUI components
    log_info "Collecting coverage data..."
    
    if ! cargo tarpaulin --features="$FEATURES" --out Xml --output-dir coverage/ --timeout 300; then
        log_error "Coverage collection failed"
        return 1
    fi
    
    # Check coverage threshold
    if [[ -f coverage/cobertura.xml ]] && command -v xmllint &> /dev/null; then
        local line_rate=$(xmllint --xpath "string(//coverage/@line-rate)" coverage/cobertura.xml 2>/dev/null || echo "0")
        local coverage_percent=$(echo "$line_rate * 100" | bc -l 2>/dev/null | cut -d. -f1)
        
        if [[ -n "$coverage_percent" ]] && [[ "$coverage_percent" -ge "$COVERAGE_THRESHOLD" ]]; then
            log_success "Coverage test passed: ${coverage_percent}%"
        else
            log_warning "Coverage below threshold: ${coverage_percent}% < ${COVERAGE_THRESHOLD}%"
        fi
    else
        log_warning "Could not parse coverage data"
    fi
    
    return 0
}

generate_test_report() {
    log_info "Generating test report..."
    
    local report_file="test_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Uveddi Automated Test Report

Generated: $(date)

## Test Summary

- **Unit Tests**: $unit_test_result
- **Integration Tests**: $integration_test_result  
- **CLI Smoke Tests**: $cli_smoke_result
- **Functional Tests**: $functional_test_result
- **Performance Tests**: $performance_test_result
- **Coverage Tests**: $coverage_test_result

## Build Information

- **Features**: $FEATURES
- **Rust Version**: $(rustc --version)
- **Cargo Version**: $(cargo --version)

## Environment

- **OS**: $(uname -s)
- **Architecture**: $(uname -m)
- **CPU Cores**: $PARALLEL_JOBS

## Test Details

### Failed Tests
$(if [[ ${#failed_test_categories[@]} -eq 0 ]]; then echo "None"; else printf '%s\n' "${failed_test_categories[@]}"; fi)

### Warnings
$(if [[ ${#warnings[@]} -eq 0 ]]; then echo "None"; else printf '%s\n' "${warnings[@]}"; fi)
EOF
    
    log_success "Test report generated: $report_file"
}

main() {
    log_info "Starting Uveddi automated test suite..."
    
    local start_time=$(date +%s)
    local failed_test_categories=()
    local warnings=()
    
    # Initialize result variables
    local unit_test_result="SKIPPED"
    local integration_test_result="SKIPPED"
    local cli_smoke_result="SKIPPED"
    local functional_test_result="SKIPPED"
    local performance_test_result="SKIPPED"
    local coverage_test_result="SKIPPED"
    
    # Run test phases
    check_dependencies
    build_binaries
    
    # Unit tests
    if run_unit_tests; then
        unit_test_result="PASSED"
    else
        unit_test_result="FAILED"
        failed_test_categories+=("Unit Tests")
    fi
    
    # Integration tests
    if run_integration_tests; then
        integration_test_result="PASSED"
    else
        integration_test_result="FAILED"
        failed_test_categories+=("Integration Tests")
    fi
    
    # CLI smoke tests
    if run_cli_smoke_tests; then
        cli_smoke_result="PASSED"
    else
        cli_smoke_result="FAILED"
        failed_test_categories+=("CLI Smoke Tests")
    fi
    
    # Functional tests
    if run_functional_tests; then
        functional_test_result="PASSED"
    else
        functional_test_result="FAILED"
        failed_test_categories+=("Functional Tests")
    fi
    
    # Performance tests
    if run_performance_tests; then
        performance_test_result="PASSED"
    else
        performance_test_result="FAILED"
        failed_test_categories+=("Performance Tests")
    fi
    
    # Coverage tests (optional)
    if run_coverage_tests; then
        coverage_test_result="PASSED"
    else
        coverage_test_result="WARNING"
        warnings+=("Coverage tests had issues")
    fi
    
    # Generate report
    generate_test_report
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    log_info "Test suite completed in $total_duration seconds"
    
    if [[ ${#failed_test_categories[@]} -eq 0 ]]; then
        log_success "All automated tests passed! 🎉"
        exit 0
    else
        log_error "Some tests failed: ${failed_test_categories[*]}"
        exit 1
    fi
}

# Handle script arguments
case "${1:-all}" in
    "unit")
        check_dependencies
        build_binaries
        run_unit_tests
        ;;
    "integration")
        check_dependencies
        build_binaries  
        run_integration_tests
        ;;
    "cli")
        check_dependencies
        build_binaries
        run_cli_smoke_tests
        ;;
    "functional")
        check_dependencies
        build_binaries
        run_functional_tests
        ;;
    "performance")
        check_dependencies
        build_binaries
        run_performance_tests
        ;;
    "coverage")
        check_dependencies
        build_binaries
        run_coverage_tests
        ;;
    "all"|"")
        main
        ;;
    *)
        echo "Usage: $0 [unit|integration|cli|functional|performance|coverage|all]"
        exit 1
        ;;
esac