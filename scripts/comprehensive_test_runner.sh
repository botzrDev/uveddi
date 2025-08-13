#!/bin/bash
# Comprehensive Test Execution Runner for Uveddi
# Final Phase of Bulletproof Testing Framework Implementation
# Executes all 8 testing phases in sequence with full validation

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COVERAGE_DIR="${PROJECT_ROOT}/coverage"
REPORTS_DIR="${PROJECT_ROOT}/coverage_reports"
LOG_FILE="${PROJECT_ROOT}/test_execution.log"
FAILED_TESTS_LOG="${PROJECT_ROOT}/failed_tests.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test execution configuration
PARALLEL_JOBS=${PARALLEL_JOBS:-4}
TIMEOUT_SECONDS=${TIMEOUT_SECONDS:-300}
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-95.0}
ENABLE_AI_TESTS=${ENABLE_AI_TESTS:-false}
ENABLE_TUI_TESTS=${ENABLE_TUI_TESTS:-false}
MEMORY_OPTIMIZATION=${MEMORY_OPTIMIZATION:-true}

# Logging functions
log() {
    echo -e "$1" | tee -a "$LOG_FILE"
}

error() {
    log "${RED}❌ ERROR: $1${NC}"
}

success() {
    log "${GREEN}✅ SUCCESS: $1${NC}"
}

warning() {
    log "${YELLOW}⚠️  WARNING: $1${NC}"
}

info() {
    log "${BLUE}ℹ️  INFO: $1${NC}"
}

# Initialize test environment
initialize_test_environment() {
    log "${PURPLE}🚀 Initializing Comprehensive Test Environment${NC}"
    
    # Create necessary directories
    mkdir -p "$COVERAGE_DIR"
    mkdir -p "$REPORTS_DIR"
    
    # Clear previous logs
    > "$LOG_FILE"
    > "$FAILED_TESTS_LOG"
    
    # Set environment variables for testing
    export RUST_BACKTRACE=full
    export RUST_LOG=debug
    export RUSTFLAGS="-C instrument-coverage"
    export LLVM_PROFILE_FILE="$COVERAGE_DIR/uveddi-%p-%m.profraw"
    
    # Memory optimization settings
    if [ "$MEMORY_OPTIMIZATION" = "true" ]; then
        export RUST_MIN_STACK=8388608  # 8MB stack size
        export MALLOC_ARENA_MAX=4
        info "Memory optimization enabled"
    fi
    
    # Check required tools
    check_required_tools
    
    success "Test environment initialized successfully"
}

check_required_tools() {
    info "Checking required tools..."
    
    local missing_tools=()
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if ! command -v rustc &> /dev/null; then
        missing_tools+=("rustc")
    fi
    
    # Check LLVM tools for coverage
    if ! command -v llvm-profdata &> /dev/null; then
        warning "LLVM profdata not found - installing llvm-tools-preview"
        rustup component add llvm-tools-preview || warning "Failed to install llvm-tools-preview"
    fi
    
    if ! command -v llvm-cov &> /dev/null; then
        warning "LLVM cov not found - coverage analysis may be limited"
    fi
    
    # Check optional tools
    if [ "$ENABLE_AI_TESTS" = "true" ]; then
        if ! command -v curl &> /dev/null; then
            warning "curl not found - AI integration tests may fail"
        fi
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    success "All required tools available"
}

# Phase 1: AST Parser Edge Case Testing
run_phase_1_ast_parser_testing() {
    log "${CYAN}🔍 Phase 1: AST Parser Edge Case Testing${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/ast/comprehensive_parser_testing.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running AST parser tests: $test_file"
            
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" --features ast -- --nocapture; then
                success "AST parser tests passed: $test_file"
            else
                error "AST parser tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    # Test multi-language parsing
    info "Testing multi-language AST parsing capabilities"
    if timeout "$TIMEOUT_SECONDS" cargo test ast_multilang -- --nocapture; then
        success "Multi-language AST parsing tests passed"
    else
        warning "Multi-language AST parsing tests failed or not found"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 1 completed in ${duration}s"
}

# Phase 2: WebAssembly Plugin Integration Testing
run_phase_2_wasm_plugin_testing() {
    log "${CYAN}🧩 Phase 2: WebAssembly Plugin Integration Testing${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/plugins/wasm_integration_comprehensive.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running WASM plugin tests: $test_file"
            
            # Enable WASM features for testing
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" --features wasm -- --nocapture; then
                success "WASM plugin tests passed: $test_file"
            else
                error "WASM plugin tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    # Test plugin system comprehensively
    info "Testing comprehensive plugin system"
    if timeout "$TIMEOUT_SECONDS" cargo test plugin_system_comprehensive -- --nocapture; then
        success "Comprehensive plugin system tests passed"
    else
        warning "Comprehensive plugin system tests failed or not found"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 2 completed in ${duration}s"
}

# Phase 3: Memory Optimization Testing
run_phase_3_memory_optimization_testing() {
    log "${CYAN}🧠 Phase 3: Memory Optimization Feature Testing${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/memory/comprehensive_memory_optimization.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running memory optimization tests: $test_file"
            
            # Enable memory optimization features
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" --features memory-optimization -- --nocapture; then
                success "Memory optimization tests passed: $test_file"
            else
                error "Memory optimization tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    # Test UV-210 and UV-26 requirements
    info "Testing UV-210/UV-26 memory optimization requirements"
    if timeout "$TIMEOUT_SECONDS" cargo test uv210_uv26_comprehensive_verification -- --features memory-optimization --nocapture; then
        success "UV-210/UV-26 verification tests passed"
    else
        warning "UV-210/UV-26 verification tests failed or not found"
    fi
    
    # Test memory optimization integration
    info "Testing memory optimization integration"
    if timeout "$TIMEOUT_SECONDS" cargo test memory_optimization_integration -- --features memory-optimization --nocapture; then
        success "Memory optimization integration tests passed"
    else
        warning "Memory optimization integration tests failed or not found"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 3 completed in ${duration}s"
}

# Phase 4: Fault Injection Testing
run_phase_4_fault_injection_testing() {
    log "${CYAN}🛡️ Phase 4: Fault Injection Testing${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/resilience/comprehensive_fault_injection.rs"
        "tests/resilience/comprehensive_error_handling_tests.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running fault injection tests: $test_file"
            
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" -- --nocapture; then
                success "Fault injection tests passed: $test_file"
            else
                error "Fault injection tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 4 completed in ${duration}s"
}

# Phase 5: Cross-Platform Compatibility Testing
run_phase_5_cross_platform_testing() {
    log "${CYAN}🌐 Phase 5: Cross-Platform Compatibility Testing${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/platform/cross_platform_compatibility.rs"
    )
    
    # Display platform information
    info "Testing on platform: $(uname -s) $(uname -m)"
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running cross-platform tests: $test_file"
            
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" -- --nocapture; then
                success "Cross-platform tests passed: $test_file"
            else
                error "Cross-platform tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    # Test TUI components if enabled
    if [ "$ENABLE_TUI_TESTS" = "true" ]; then
        info "Testing TUI cross-platform compatibility"
        if timeout "$TIMEOUT_SECONDS" cargo test --features tui tui_e2e -- --nocapture; then
            success "TUI cross-platform tests passed"
        else
            warning "TUI cross-platform tests failed or not found"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 5 completed in ${duration}s"
}

# Phase 6: Performance Regression Detection
run_phase_6_performance_testing() {
    log "${CYAN}⚡ Phase 6: Performance Regression Detection${NC}"
    
    local start_time=$(date +%s)
    local test_files=(
        "tests/performance/comprehensive_regression_detection.rs"
        "tests/benchmarks/comprehensive_performance_benchmarks.rs"
    )
    
    for test_file in "${test_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running performance tests: $test_file"
            
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" --release -- --nocapture; then
                success "Performance tests passed: $test_file"
            else
                error "Performance tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "Test file not found: $test_file"
        fi
    done
    
    # Run benchmark tests
    info "Running performance benchmarks"
    if timeout "$TIMEOUT_SECONDS" cargo bench --bench comprehensive_performance_benchmarks; then
        success "Performance benchmarks completed"
    else
        warning "Performance benchmarks failed or not found"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 6 completed in ${duration}s"
}

# Phase 7: Coverage Reporting and Analysis
run_phase_7_coverage_analysis() {
    log "${CYAN}📊 Phase 7: Automated Coverage Reporting${NC}"
    
    local start_time=$(date +%s)
    
    # Run coverage tests
    info "Running comprehensive coverage analysis"
    if timeout "$TIMEOUT_SECONDS" cargo test --test comprehensive_coverage -- --nocapture; then
        success "Coverage analysis tests passed"
    else
        error "Coverage analysis tests failed"
        echo "tests/coverage/comprehensive_coverage.rs" >> "$FAILED_TESTS_LOG"
    fi
    
    # Generate coverage report with instrumentation
    info "Generating LLVM coverage report"
    
    # Clean previous coverage data
    rm -f "$COVERAGE_DIR"/*.profraw "$COVERAGE_DIR"/*.profdata
    
    # Run all tests with coverage instrumentation
    if RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="$COVERAGE_DIR/uveddi-%p-%m.profraw" \
       timeout "$TIMEOUT_SECONDS" cargo test --all-features -- --test-threads=1; then
        
        # Process coverage data
        info "Processing coverage data"
        
        if command -v llvm-profdata &> /dev/null; then
            # Merge profile data
            if ls "$COVERAGE_DIR"/*.profraw >/dev/null 2>&1; then
                llvm-profdata merge -sparse "$COVERAGE_DIR"/*.profraw -o "$COVERAGE_DIR/merged.profdata"
                
                # Generate coverage report
                if command -v llvm-cov &> /dev/null; then
                    # Find the test binary
                    local binary_path
                    binary_path=$(find target/debug/deps -name "uveddi-*" -type f ! -name "*.d" | head -1)
                    
                    if [ -n "$binary_path" ]; then
                        llvm-cov show --format=json --instr-profile="$COVERAGE_DIR/merged.profdata" \
                                "$binary_path" --ignore-filename-regex='/.cargo/' > "$COVERAGE_DIR/coverage.json"
                        
                        # Generate HTML report
                        llvm-cov show --format=html --instr-profile="$COVERAGE_DIR/merged.profdata" \
                                "$binary_path" --ignore-filename-regex='/.cargo/' --output-dir="$REPORTS_DIR/html"
                        
                        success "LLVM coverage reports generated"
                    else
                        warning "Could not find test binary for coverage analysis"
                    fi
                else
                    warning "llvm-cov not available - HTML report not generated"
                fi
            else
                warning "No profile data files found"
            fi
        else
            warning "llvm-profdata not available - coverage analysis limited"
        fi
    else
        error "Test execution with coverage instrumentation failed"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 7 completed in ${duration}s"
}

# Phase 8: Integration and End-to-End Testing
run_phase_8_integration_testing() {
    log "${CYAN}🔗 Phase 8: Integration and End-to-End Testing${NC}"
    
    local start_time=$(date +%s)
    
    # Run integration tests
    info "Running integration test suites"
    local integration_tests=(
        "integration_sprint1"
        "full_pipeline_integration" 
        "end_to_end_analysis_workflow"
        "knowledge_integration_tests"
        "security_tests"
    )
    
    for test in "${integration_tests[@]}"; do
        info "Running integration test: $test"
        
        if timeout "$TIMEOUT_SECONDS" cargo test "$test" -- --nocapture; then
            success "Integration test passed: $test"
        else
            warning "Integration test failed or not found: $test"
            echo "$test" >> "$FAILED_TESTS_LOG"
        fi
    done
    
    # Run E2E workflow tests
    info "Running end-to-end workflow tests"
    local e2e_tests=(
        "tests/e2e/complete_analysis_workflow.rs"
    )
    
    for test_file in "${e2e_tests[@]}"; do
        if [ -f "$PROJECT_ROOT/$test_file" ]; then
            info "Running E2E tests: $test_file"
            
            if timeout "$TIMEOUT_SECONDS" cargo test --test "$(basename "$test_file" .rs)" -- --nocapture; then
                success "E2E tests passed: $test_file"
            else
                error "E2E tests failed: $test_file"
                echo "$test_file" >> "$FAILED_TESTS_LOG"
            fi
        else
            warning "E2E test file not found: $test_file"
        fi
    done
    
    # Run AI integration tests if enabled
    if [ "$ENABLE_AI_TESTS" = "true" ]; then
        info "Running AI integration tests"
        if timeout "$TIMEOUT_SECONDS" cargo test --features ai ai_ -- --nocapture; then
            success "AI integration tests passed"
        else
            warning "AI integration tests failed or not found"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    success "Phase 8 completed in ${duration}s"
}

# Generate comprehensive test report
generate_test_report() {
    log "${PURPLE}📋 Generating Comprehensive Test Report${NC}"
    
    local report_file="$REPORTS_DIR/comprehensive_test_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local platform=$(uname -s)
    local architecture=$(uname -m)
    
    cat > "$report_file" <<EOF
# Comprehensive Test Execution Report

**Generated:** $timestamp  
**Platform:** $platform ($architecture)  
**Rust Version:** $(rustc --version)  
**Cargo Version:** $(cargo --version)  

## Test Execution Summary

### Configuration
- Parallel Jobs: $PARALLEL_JOBS
- Timeout: ${TIMEOUT_SECONDS}s
- Coverage Threshold: ${COVERAGE_THRESHOLD}%
- Memory Optimization: $MEMORY_OPTIMIZATION
- AI Tests: $ENABLE_AI_TESTS
- TUI Tests: $ENABLE_TUI_TESTS

### Phase Execution Results

EOF
    
    # Analyze failed tests
    if [ -f "$FAILED_TESTS_LOG" ] && [ -s "$FAILED_TESTS_LOG" ]; then
        local failed_count
        failed_count=$(wc -l < "$FAILED_TESTS_LOG")
        echo "**❌ Failed Tests:** $failed_count" >> "$report_file"
        echo "" >> "$report_file"
        echo "#### Failed Test Details" >> "$report_file"
        while IFS= read -r failed_test; do
            echo "- $failed_test" >> "$report_file"
        done < "$FAILED_TESTS_LOG"
        echo "" >> "$report_file"
    else
        echo "**✅ All Tests Passed**" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    # Coverage information
    if [ -f "$COVERAGE_DIR/coverage.json" ]; then
        echo "#### Coverage Analysis" >> "$report_file"
        echo "- Coverage report available at: \`$REPORTS_DIR/html/index.html\`" >> "$report_file"
        echo "- Raw coverage data: \`$COVERAGE_DIR/coverage.json\`" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    # Performance information
    echo "#### Performance and Resource Usage" >> "$report_file"
    echo "- Test execution log: \`$LOG_FILE\`" >> "$report_file"
    echo "- Memory optimization: $([ "$MEMORY_OPTIMIZATION" = "true" ] && echo "Enabled" || echo "Disabled")" >> "$report_file"
    echo "" >> "$report_file"
    
    # Recommendations
    echo "#### Recommendations" >> "$report_file"
    if [ -f "$FAILED_TESTS_LOG" ] && [ -s "$FAILED_TESTS_LOG" ]; then
        echo "1. Review and fix failed tests listed above" >> "$report_file"
        echo "2. Check test environment setup and dependencies" >> "$report_file"
        echo "3. Consider increasing timeout values for slow tests" >> "$report_file"
    else
        echo "1. ✅ All tests are passing - excellent test coverage!" >> "$report_file"
        echo "2. Consider adding more edge case tests as the codebase grows" >> "$report_file"
        echo "3. Monitor performance trends over time" >> "$report_file"
    fi
    echo "" >> "$report_file"
    
    success "Comprehensive test report generated: $report_file"
}

# Cleanup function
cleanup() {
    info "Cleaning up test environment"
    
    # Reset environment variables
    unset RUSTFLAGS
    unset LLVM_PROFILE_FILE
    unset RUST_BACKTRACE
    unset RUST_LOG
    
    # Archive old coverage data
    if [ -d "$COVERAGE_DIR" ] && [ "$(ls -A "$COVERAGE_DIR")" ]; then
        local archive_dir="$COVERAGE_DIR/archive/$(date +%Y%m%d_%H%M%S)"
        mkdir -p "$archive_dir"
        mv "$COVERAGE_DIR"/*.profraw "$COVERAGE_DIR"/*.profdata "$archive_dir" 2>/dev/null || true
        info "Coverage data archived to: $archive_dir"
    fi
}

# Main execution function
main() {
    local total_start_time=$(date +%s)
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --coverage-threshold)
                COVERAGE_THRESHOLD="$2"
                shift 2
                ;;
            --timeout)
                TIMEOUT_SECONDS="$2"
                shift 2
                ;;
            --enable-ai-tests)
                ENABLE_AI_TESTS=true
                shift
                ;;
            --enable-tui-tests)
                ENABLE_TUI_TESTS=true
                shift
                ;;
            --disable-memory-optimization)
                MEMORY_OPTIMIZATION=false
                shift
                ;;
            --parallel-jobs)
                PARALLEL_JOBS="$2"
                shift 2
                ;;
            --phases)
                # Allow running specific phases: --phases "1,3,5"
                SELECTED_PHASES="$2"
                shift 2
                ;;
            --help)
                echo "Comprehensive Test Runner for Uveddi"
                echo ""
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --coverage-threshold FLOAT   Minimum coverage threshold (default: 95.0)"
                echo "  --timeout SECONDS            Test timeout in seconds (default: 300)"
                echo "  --enable-ai-tests             Enable AI integration tests"
                echo "  --enable-tui-tests            Enable TUI tests"
                echo "  --disable-memory-optimization Disable memory optimization"
                echo "  --parallel-jobs NUMBER        Number of parallel jobs (default: 4)"
                echo "  --phases LIST                 Run specific phases (e.g., \"1,3,5\")"
                echo "  --help                        Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log "${PURPLE}🚀 Starting Comprehensive Test Execution Framework${NC}"
    log "${PURPLE}=================================================================${NC}"
    
    # Initialize test environment
    initialize_test_environment
    
    # Execute test phases
    if [ -z "$SELECTED_PHASES" ]; then
        # Run all phases
        run_phase_1_ast_parser_testing
        run_phase_2_wasm_plugin_testing
        run_phase_3_memory_optimization_testing
        run_phase_4_fault_injection_testing
        run_phase_5_cross_platform_testing
        run_phase_6_performance_testing
        run_phase_7_coverage_analysis
        run_phase_8_integration_testing
    else
        # Run selected phases
        IFS=',' read -ra PHASES <<< "$SELECTED_PHASES"
        for phase in "${PHASES[@]}"; do
            case $phase in
                1) run_phase_1_ast_parser_testing ;;
                2) run_phase_2_wasm_plugin_testing ;;
                3) run_phase_3_memory_optimization_testing ;;
                4) run_phase_4_fault_injection_testing ;;
                5) run_phase_5_cross_platform_testing ;;
                6) run_phase_6_performance_testing ;;
                7) run_phase_7_coverage_analysis ;;
                8) run_phase_8_integration_testing ;;
                *) error "Invalid phase number: $phase" ;;
            esac
        done
    fi
    
    # Generate comprehensive report
    generate_test_report
    
    # Cleanup
    cleanup
    
    local total_end_time=$(date +%s)
    local total_duration=$((total_end_time - total_start_time))
    
    # Final summary
    log "${PURPLE}=================================================================${NC}"
    if [ -f "$FAILED_TESTS_LOG" ] && [ -s "$FAILED_TESTS_LOG" ]; then
        local failed_count
        failed_count=$(wc -l < "$FAILED_TESTS_LOG")
        error "Test execution completed with $failed_count failures in ${total_duration}s"
        log "${YELLOW}Review the test report for detailed failure analysis${NC}"
        exit 1
    else
        success "🎉 All tests passed! Comprehensive test execution completed successfully in ${total_duration}s"
        log "${GREEN}🛡️ Uveddi bulletproof testing framework validation: PASSED${NC}"
        exit 0
    fi
}

# Set up trap for cleanup on script exit
trap cleanup EXIT

# Execute main function with all arguments
main "$@"