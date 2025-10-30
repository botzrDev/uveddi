#!/bin/bash

# Comprehensive Uveddi Verification Suite
# Master script that runs all verification tests to ensure detector coverage and dashboard integration

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UVEDDI_DIR="$(dirname "$SCRIPT_DIR")"
VERIFICATION_DIR="$UVEDDI_DIR/comprehensive_verification"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

print_header() { echo -e "${BOLD}${BLUE}=== $1 ===${NC}"; }
print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Initialize verification environment
init_verification() {
    print_header "Initializing Comprehensive Verification"
    
    mkdir -p "$VERIFICATION_DIR"
    cd "$UVEDDI_DIR"
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if Uveddi builds
    print_status "Testing Uveddi build..."
    if cargo build --features=dev-core &> "$VERIFICATION_DIR/build_test.log"; then
        print_success "✓ Uveddi builds successfully"
    else
        print_error "✗ Uveddi build failed"
        print_status "Check build log: $VERIFICATION_DIR/build_test.log"
        exit 1
    fi
    
    # Check optional dependencies
    if command -v jq &> /dev/null; then
        print_success "✓ jq available for JSON processing"
    else
        print_warning "⚠ jq not found - install for better JSON processing"
    fi
    
    if command -v curl &> /dev/null; then
        print_success "✓ curl available for API testing"
    else
        print_warning "⚠ curl not found - install for API testing"
    fi
    
    print_success "Initialization complete"
}

# Run detector unit tests
run_unit_tests() {
    print_header "Running Detector Unit Tests"
    
    cd "$UVEDDI_DIR"
    
    # Run tests with coverage if available
    if cargo test --features=community 2>&1 | tee "$VERIFICATION_DIR/unit_tests.log"; then
        print_success "✓ Unit tests completed"
        
        # Extract test results
        local passed=$(grep "test result:" "$VERIFICATION_DIR/unit_tests.log" | tail -1 | grep -o "[0-9]* passed" | cut -d' ' -f1)
        local failed=$(grep "test result:" "$VERIFICATION_DIR/unit_tests.log" | tail -1 | grep -o "[0-9]* failed" | cut -d' ' -f1)
        
        print_status "Test Results: $passed passed, $failed failed"
        
        if [ "${failed:-0}" -gt 0 ]; then
            print_warning "⚠ Some unit tests failed - check log for details"
        fi
    else
        print_error "✗ Unit tests failed"
        return 1
    fi
}

# Test detector coverage with synthetic data
test_detector_coverage() {
    print_header "Testing Detector Coverage"
    
    if [ -x "$SCRIPT_DIR/verify_detector_coverage.sh" ]; then
        print_status "Running detector coverage verification..."
        
        if "$SCRIPT_DIR/verify_detector_coverage.sh" --quick 2>&1 | tee "$VERIFICATION_DIR/detector_coverage.log"; then
            print_success "✓ Detector coverage test completed"
        else
            print_warning "⚠ Detector coverage test had issues"
        fi
    else
        print_error "✗ Detector coverage script not found or not executable"
        return 1
    fi
}

# Test real-world codebases
test_real_codebases() {
    print_header "Testing Real-World Codebases"
    
    if [ -x "$SCRIPT_DIR/test_real_codebases.sh" ]; then
        print_status "Running real codebase analysis..."
        
        # Use quick mode for faster testing
        if "$SCRIPT_DIR/test_real_codebases.sh" --quick 2>&1 | tee "$VERIFICATION_DIR/real_codebases.log"; then
            print_success "✓ Real codebase testing completed"
        else
            print_warning "⚠ Real codebase testing had issues"
        fi
    else
        print_error "✗ Real codebase test script not found or not executable"
        return 1
    fi
}

# Test dashboard data flow
test_dashboard_flow() {
    print_header "Testing Dashboard Data Flow"
    
    if [ -x "$SCRIPT_DIR/verify_dashboard_data_flow.sh" ]; then
        print_status "Running dashboard data flow verification..."
        
        if "$SCRIPT_DIR/verify_dashboard_data_flow.sh" --quick 2>&1 | tee "$VERIFICATION_DIR/dashboard_flow.log"; then
            print_success "✓ Dashboard data flow test completed"
        else
            print_warning "⚠ Dashboard data flow test had issues"
        fi
    else
        print_error "✗ Dashboard flow script not found or not executable"
        return 1
    fi
}

# Test API and web services
test_web_services() {
    print_header "Testing Web Services Integration"
    
    print_status "Starting web services for testing..."
    
    cd "$UVEDDI_DIR"
    
    # Start services in background
    cargo run --bin uveddi --features=community -- serve --port 8888 --rendering-port 3333 > "$VERIFICATION_DIR/server_test.log" 2>&1 &
    local SERVER_PID=$!
    
    # Wait for startup
    sleep 15
    
    # Test basic endpoints
    local api_ok=false
    local render_ok=false
    
    if curl -s http://localhost:8888/health > /dev/null 2>&1; then
        print_success "✓ API server responding"
        api_ok=true
    else
        print_warning "⚠ API server not responding"
    fi
    
    if curl -s http://localhost:3333/health > /dev/null 2>&1; then
        print_success "✓ Rendering service responding"
        render_ok=true
    else
        print_warning "⚠ Rendering service not responding"
    fi
    
    # Clean up
    kill $SERVER_PID 2>/dev/null || true
    
    if [ "$api_ok" = true ] && [ "$render_ok" = true ]; then
        print_success "✓ Web services test passed"
        return 0
    else
        print_warning "⚠ Web services test had issues"
        return 1
    fi
}

# Generate comprehensive report
generate_final_report() {
    print_header "Generating Comprehensive Report"
    
    local report_file="$VERIFICATION_DIR/comprehensive_verification_report_${TIMESTAMP}.md"
    
    cat > "$report_file" << 'EOF'
# Uveddi Comprehensive Verification Report

## Executive Summary
This report contains the results of comprehensive verification testing for all Uveddi detectors and the dashboard integration.

### Test Categories
1. **Unit Tests**: Core detector functionality
2. **Detector Coverage**: Synthetic data testing 
3. **Real-World Testing**: Analysis of actual codebases
4. **Dashboard Integration**: Data flow verification
5. **Web Services**: API and rendering service testing

## Detailed Results

### Unit Test Results
EOF
    
    # Add unit test results
    if [ -f "$VERIFICATION_DIR/unit_tests.log" ]; then
        echo "```" >> "$report_file"
        grep "test result:" "$VERIFICATION_DIR/unit_tests.log" | tail -1 >> "$report_file"
        echo "```" >> "$report_file"
        echo "" >> "$report_file"
        
        if grep -q "FAILED" "$VERIFICATION_DIR/unit_tests.log"; then
            echo "**⚠ Warning**: Some unit tests failed. See detailed log for investigation." >> "$report_file"
        else
            echo "**✓ Success**: All unit tests passed." >> "$report_file"
        fi
    fi
    
    echo "" >> "$report_file"
    echo "### Detector Coverage Analysis" >> "$report_file"
    
    # Add detector coverage results
    if [ -f "$VERIFICATION_DIR/detector_coverage.log" ]; then
        local detectors_found=$(grep -c "✓.*found" "$VERIFICATION_DIR/detector_coverage.log" || echo "0")
        local detectors_missing=$(grep -c "✗.*missing" "$VERIFICATION_DIR/detector_coverage.log" || echo "0")
        
        echo "- Detectors working: $detectors_found" >> "$report_file"
        echo "- Detectors with issues: $detectors_missing" >> "$report_file"
        echo "" >> "$report_file"
        
        if [ "$detectors_missing" -gt 0 ]; then
            echo "**Issues Found:**" >> "$report_file"
            grep "✗.*missing" "$VERIFICATION_DIR/detector_coverage.log" | sed 's/^/- /' >> "$report_file"
        fi
    fi
    
    echo "" >> "$report_file"
    echo "### Real-World Codebase Testing" >> "$report_file"
    
    # Add real codebase results
    if [ -f "$VERIFICATION_DIR/real_codebases.log" ]; then
        local successful_analyses=$(grep -c "Analysis completed" "$VERIFICATION_DIR/real_codebases.log" || echo "0")
        local failed_analyses=$(grep -c "Analysis failed" "$VERIFICATION_DIR/real_codebases.log" || echo "0")
        
        echo "- Successful analyses: $successful_analyses" >> "$report_file"
        echo "- Failed analyses: $failed_analyses" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    echo "### Dashboard and Web Services" >> "$report_file"
    
    # Add web service results
    local web_status="Unknown"
    if [ -f "$VERIFICATION_DIR/dashboard_flow.log" ]; then
        if grep -q "Dashboard is accessible" "$VERIFICATION_DIR/dashboard_flow.log"; then
            web_status="✓ Working"
        elif grep -q "Dashboard.*failed" "$VERIFICATION_DIR/dashboard_flow.log"; then
            web_status="✗ Failed"
        else
            web_status="⚠ Partial"
        fi
    fi
    
    echo "- Dashboard Status: $web_status" >> "$report_file"
    echo "" >> "$report_file"
    
    echo "## Critical Issues" >> "$report_file"
    echo "" >> "$report_file"
    
    # Compile critical issues
    local critical_issues=()
    
    if [ -f "$VERIFICATION_DIR/unit_tests.log" ] && grep -q "FAILED" "$VERIFICATION_DIR/unit_tests.log"; then
        critical_issues+=("Unit test failures detected")
    fi
    
    if [ -f "$VERIFICATION_DIR/detector_coverage.log" ] && grep -q "✗" "$VERIFICATION_DIR/detector_coverage.log"; then
        critical_issues+=("Some detectors not working properly")
    fi
    
    if [ "$web_status" = "✗ Failed" ]; then
        critical_issues+=("Web services not functioning")
    fi
    
    if [ ${#critical_issues[@]} -gt 0 ]; then
        for issue in "${critical_issues[@]}"; do
            echo "- **$issue**" >> "$report_file"
        done
    else
        echo "No critical issues found." >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. **Fix failing unit tests** - Address any detector implementation issues" >> "$report_file"
    echo "2. **Verify detector registration** - Ensure all detectors are properly registered" >> "$report_file"
    echo "3. **Test dashboard integration** - Verify end-to-end data flow" >> "$report_file"
    echo "4. **Performance testing** - Test with larger codebases" >> "$report_file"
    echo "5. **User acceptance testing** - Validate with real user workflows" >> "$report_file"
    
    echo "" >> "$report_file"
    echo "## Test Artifacts" >> "$report_file"
    echo "" >> "$report_file"
    echo "All test logs and reports are available in:" >> "$report_file"
    echo "\`$VERIFICATION_DIR\`" >> "$report_file"
    
    print_success "Comprehensive report generated: $report_file"
    
    # Display summary
    echo ""
    print_header "VERIFICATION SUMMARY"
    
    if [ ${#critical_issues[@]} -eq 0 ]; then
        print_success "✅ All verification tests passed!"
        print_status "Uveddi appears ready for production use."
    else
        print_warning "⚠ ${#critical_issues[@]} critical issues found:"
        for issue in "${critical_issues[@]}"; do
            print_warning "  - $issue"
        done
        print_status "Address these issues before release."
    fi
    
    print_status "Full report: $report_file"
}

# Cleanup function
cleanup() {
    # Kill any remaining processes
    pkill -f "uveddi.*serve" 2>/dev/null || true
}

# Main execution
main() {
    print_header "Uveddi Comprehensive Verification Suite"
    print_status "Timestamp: $TIMESTAMP"
    print_status "Verification directory: $VERIFICATION_DIR"
    
    # Set cleanup trap
    trap cleanup EXIT
    
    # Run all verification phases
    init_verification
    
    print_status "Starting verification phases..."
    echo ""
    
    # Phase 1: Unit tests
    if ! run_unit_tests; then
        print_warning "Unit tests failed, continuing with other tests..."
    fi
    echo ""
    
    # Phase 2: Detector coverage
    if ! test_detector_coverage; then
        print_warning "Detector coverage test failed, continuing..."
    fi
    echo ""
    
    # Phase 3: Real codebases (optional, may take time)
    if [ "$1" != "--quick" ]; then
        if ! test_real_codebases; then
            print_warning "Real codebase testing failed, continuing..."
        fi
        echo ""
    fi
    
    # Phase 4: Dashboard flow
    if ! test_dashboard_flow; then
        print_warning "Dashboard flow test failed, continuing..."
    fi
    echo ""
    
    # Phase 5: Web services
    if ! test_web_services; then
        print_warning "Web services test failed, continuing..."
    fi
    echo ""
    
    # Generate final report
    generate_final_report
    
    print_success "Comprehensive verification complete!"
}

# Command line argument handling
case "${1:-}" in
    --help)
        echo "Usage: $0 [--quick|--help]"
        echo ""
        echo "Comprehensive verification suite for Uveddi detector coverage and dashboard integration."
        echo ""
        echo "Options:"
        echo "  --quick  Skip time-consuming real codebase testing"
        echo "  --help   Show this help message"
        echo ""
        echo "This script runs:"
        echo "  1. Unit tests for all detectors"
        echo "  2. Detector coverage verification with synthetic data"
        echo "  3. Real-world codebase analysis (unless --quick)"
        echo "  4. Dashboard data flow verification"
        echo "  5. Web services integration testing"
        echo ""
        echo "Results are compiled into a comprehensive report."
        ;;
    *)
        main "$@"
        ;;
esac