#!/bin/bash

# Comprehensive Real-World Codebase Testing for Uveddi
# Tests all detectors against actual open-source projects

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
UVEDDI_DIR="/home/austingreen/Documents/botzr/projects/uveddi"
TEST_DIR="$UVEDDI_DIR/real_world_tests"
REPORTS_DIR="$TEST_DIR/reports"
CODEBASES_DIR="$TEST_DIR/codebases"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SUMMARY_FILE="$REPORTS_DIR/test_summary_${TIMESTAMP}.json"

# Function definitions
print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Define test codebases
declare -A CODEBASES=(
    ["rust-small"]="https://github.com/serde-rs/serde|Rust serialization library|rust"
    ["rust-medium"]="https://github.com/actix/actix-web|Rust web framework|rust"
    ["python-small"]="https://github.com/psf/requests|Python HTTP library|python"
    ["python-medium"]="https://github.com/pallets/flask|Python web framework|python"
    ["js-small"]="https://github.com/lodash/lodash|JavaScript utility library|javascript"
    ["ts-medium"]="https://github.com/microsoft/TypeScript|TypeScript compiler|typescript"
)

# Expected detectors and their patterns
declare -A DETECTOR_PATTERNS=(
    ["god_object"]="God Object"
    ["code_duplication"]="Code Duplication"
    ["dead_code"]="Dead Code"
    ["large_class"]="Large Class"
    ["tight_coupling"]="Tight Coupling"
    ["long_method"]="Long Method"
    ["magic_values"]="Magic Values"
)

# Initialize summary
init_summary() {
    cat > "$SUMMARY_FILE" << EOF
{
    "timestamp": "$TIMESTAMP",
    "codebases_tested": [],
    "detector_coverage": {},
    "issues_found": {},
    "performance_metrics": {},
    "errors": []
}
EOF
}

# Download or update codebase
download_codebase() {
    local name=$1
    local url=$(echo "${CODEBASES[$name]}" | cut -d'|' -f1)
    local description=$(echo "${CODEBASES[$name]}" | cut -d'|' -f2)
    local language=$(echo "${CODEBASES[$name]}" | cut -d'|' -f3)
    local repo_dir="$CODEBASES_DIR/$name"
    
    print_status "Setting up $name ($description)..."
    
    if [ -d "$repo_dir" ]; then
        print_status "Updating existing repository..."
        cd "$repo_dir"
        git pull --quiet
    else
        print_status "Cloning repository..."
        git clone --depth 1 "$url" "$repo_dir" --quiet
    fi
    
    # Count lines of code
    local loc=$(find "$repo_dir" -type f \( -name "*.rs" -o -name "*.py" -o -name "*.js" -o -name "*.ts" \) -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
    print_success "✓ $name ready ($loc lines of $language code)"
    
    # Update summary
    jq --arg name "$name" --arg desc "$description" --arg lang "$language" --argjson loc "$loc" \
        '.codebases_tested += [{name: $name, description: $desc, language: $lang, lines_of_code: $loc}]' \
        "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"
}

# Run analysis on a codebase
analyze_codebase() {
    local name=$1
    local codebase_path="$CODEBASES_DIR/$name"
    local report_prefix="$REPORTS_DIR/${name}_${TIMESTAMP}"
    
    print_status "Analyzing $name..."
    
    # Measure analysis time
    local start_time=$(date +%s)
    
    # Run analysis with JSON output
    cd "$UVEDDI_DIR"
    if cargo run --bin uveddi --features=community -- analyze "$codebase_path" \
        --output-format json \
        --output "${report_prefix}.json" 2>&1 | tee "${report_prefix}_analysis.log"; then
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        print_success "✓ Analysis completed in ${duration}s"
        
        # Update performance metrics
        jq --arg name "$name" --argjson duration "$duration" \
            '.performance_metrics[$name] = {duration_seconds: $duration}' \
            "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"
        
        # Also generate HTML for manual inspection
        cargo run --bin uveddi --features=community -- analyze "$codebase_path" \
            --output-format html \
            --output "${report_prefix}.html" 2>&1 > /dev/null
            
        return 0
    else
        print_error "✗ Analysis failed for $name"
        
        # Log error
        jq --arg name "$name" --arg error "Analysis failed" \
            '.errors += [{codebase: $name, error: $error}]' \
            "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"
            
        return 1
    fi
}

# Verify detector coverage in report
verify_detector_coverage() {
    local name=$1
    local json_file="$REPORTS_DIR/${name}_${TIMESTAMP}.json"
    
    print_status "Verifying detector coverage for $name..."
    
    if [ ! -f "$json_file" ]; then
        print_error "Report not found: $json_file"
        return 1
    fi
    
    # Check for issues
    local issue_count=$(jq '.issues | length' "$json_file" 2>/dev/null || echo "0")
    print_status "Found $issue_count issues"
    
    # Update issue count in summary
    jq --arg name "$name" --argjson count "$issue_count" \
        '.issues_found[$name] = $count' \
        "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"
    
    # Check for each detector pattern
    local detectors_found=()
    local detectors_missing=()
    
    for detector in "${!DETECTOR_PATTERNS[@]}"; do
        local pattern="${DETECTOR_PATTERNS[$detector]}"
        
        if jq -e --arg pattern "$pattern" '.issues[] | select(.antiPatternType == $pattern)' "$json_file" > /dev/null 2>&1; then
            local count=$(jq --arg pattern "$pattern" '[.issues[] | select(.antiPatternType == $pattern)] | length' "$json_file")
            print_success "  ✓ $pattern: $count instances"
            detectors_found+=("$detector")
            
            # Update detector coverage
            jq --arg name "$name" --arg detector "$detector" --argjson count "$count" \
                '.detector_coverage[$name] = (.detector_coverage[$name] // {}) | .detector_coverage[$name][$detector] = $count' \
                "$SUMMARY_FILE" > "$SUMMARY_FILE.tmp" && mv "$SUMMARY_FILE.tmp" "$SUMMARY_FILE"
        else
            print_warning "  ✗ $pattern: not found"
            detectors_missing+=("$detector")
        fi
    done
    
    # Calculate coverage percentage
    local total_detectors=${#DETECTOR_PATTERNS[@]}
    local found_detectors=${#detectors_found[@]}
    local coverage_percent=$((found_detectors * 100 / total_detectors))
    
    print_status "Detector coverage: $found_detectors/$total_detectors ($coverage_percent%)"
    
    # Validate report structure
    if jq -e 'has("issues", "summary", "metadata")' "$json_file" > /dev/null 2>&1; then
        print_success "✓ Report structure is valid"
    else
        print_error "✗ Report structure is invalid"
    fi
    
    return 0
}

# Test dashboard data flow
test_dashboard_flow() {
    print_status "Testing dashboard data flow..."
    
    # Copy a test report to the expected location
    local test_report="$REPORTS_DIR/rust-small_${TIMESTAMP}.json"
    
    if [ -f "$test_report" ]; then
        # Start services
        cd "$UVEDDI_DIR"
        print_status "Starting web services..."
        cargo run --bin uveddi --features=community -- serve --port 8888 --rendering-port 3333 > "$REPORTS_DIR/server.log" 2>&1 &
        local SERVER_PID=$!
        
        # Wait for services to start
        sleep 15
        
        # Check if services are running
        if curl -s http://localhost:8888/health | jq -e '.status == "ok"' > /dev/null 2>&1; then
            print_success "✓ API server is running"
            
            # Test API endpoints
            if curl -s http://localhost:8888/api/v1/analysis > /dev/null 2>&1; then
                print_success "✓ API analysis endpoint responds"
            fi
            
            # Check if report data can be served
            # This would need actual API endpoint implementation
            print_status "Dashboard available at http://localhost:8888"
        else
            print_error "✗ API server failed to start"
        fi
        
        # Check rendering service
        if curl -s http://localhost:3333/health | jq -e '.status == "ok"' > /dev/null 2>&1; then
            print_success "✓ Rendering service is running"
        else
            print_warning "✗ Rendering service not responding"
        fi
        
        # Cleanup
        kill $SERVER_PID 2>/dev/null || true
        
        print_success "Dashboard flow test completed"
    else
        print_warning "No test report available for dashboard testing"
    fi
}

# Generate final report
generate_final_report() {
    print_status "Generating final report..."
    
    local report_file="$REPORTS_DIR/final_report_${TIMESTAMP}.md"
    
    cat > "$report_file" << 'EOF'
# Uveddi Real-World Testing Report

## Executive Summary
EOF
    
    # Add test results from JSON summary
    echo "**Test Date:** $(date)" >> "$report_file"
    echo "" >> "$report_file"
    
    # Codebases tested
    echo "## Codebases Tested" >> "$report_file"
    jq -r '.codebases_tested[] | "- **\(.name)**: \(.description) (\(.lines_of_code) LOC)"' "$SUMMARY_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # Detector Coverage
    echo "## Detector Coverage Analysis" >> "$report_file"
    echo "" >> "$report_file"
    
    for name in "${!CODEBASES[@]}"; do
        echo "### $name" >> "$report_file"
        
        if jq -e --arg name "$name" '.detector_coverage[$name]' "$SUMMARY_FILE" > /dev/null 2>&1; then
            echo "| Detector | Issues Found |" >> "$report_file"
            echo "|----------|-------------|" >> "$report_file"
            
            for detector in "${!DETECTOR_PATTERNS[@]}"; do
                local pattern="${DETECTOR_PATTERNS[$detector]}"
                local count=$(jq -r --arg name "$name" --arg detector "$detector" '.detector_coverage[$name][$detector] // 0' "$SUMMARY_FILE")
                
                if [ "$count" -gt 0 ]; then
                    echo "| $pattern | ✓ $count |" >> "$report_file"
                else
                    echo "| $pattern | ✗ Not detected |" >> "$report_file"
                fi
            done
            
            echo "" >> "$report_file"
        fi
    done
    
    # Performance Metrics
    echo "## Performance Metrics" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Codebase | Analysis Time |" >> "$report_file"
    echo "|----------|--------------|" >> "$report_file"
    
    jq -r '.performance_metrics | to_entries[] | "| \(.key) | \(.value.duration_seconds)s |"' "$SUMMARY_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # Issues Summary
    echo "## Issues Found Summary" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Codebase | Total Issues |" >> "$report_file"
    echo "|----------|-------------|" >> "$report_file"
    
    jq -r '.issues_found | to_entries[] | "| \(.key) | \(.value) |"' "$SUMMARY_FILE" >> "$report_file"
    echo "" >> "$report_file"
    
    # Errors
    if jq -e '.errors | length > 0' "$SUMMARY_FILE" > /dev/null 2>&1; then
        echo "## Errors Encountered" >> "$report_file"
        jq -r '.errors[] | "- **\(.codebase)**: \(.error)"' "$SUMMARY_FILE" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    # Recommendations
    cat >> "$report_file" << 'EOF'
## Recommendations

### Critical Issues
1. **Detector Coverage**: Ensure all detectors are triggered on appropriate code patterns
2. **Report Generation**: Verify all anti-pattern types appear in reports
3. **Dashboard Integration**: Test data flow from analysis to dashboard

### Performance Optimization
1. Consider caching AST parsing for large codebases
2. Implement parallel detector execution for better performance
3. Optimize memory usage for projects > 500k LOC

### Quality Improvements
1. Add confidence scores to all detector outputs
2. Implement severity level calibration based on project size
3. Enhance AI integration for better explanations

## Test Artifacts
EOF
    
    echo "- JSON Summary: $SUMMARY_FILE" >> "$report_file"
    echo "- Reports Directory: $REPORTS_DIR" >> "$report_file"
    echo "- Analysis Logs: $REPORTS_DIR/*_analysis.log" >> "$report_file"
    
    print_success "Final report generated: $report_file"
    
    # Display summary
    echo ""
    print_status "=== TEST SUMMARY ==="
    jq -r '.codebases_tested | length' "$SUMMARY_FILE" | xargs -I {} echo "Codebases tested: {}"
    jq -r '[.issues_found | to_entries[].value] | add' "$SUMMARY_FILE" | xargs -I {} echo "Total issues found: {}"
    jq -r '[.performance_metrics | to_entries[].value.duration_seconds] | add' "$SUMMARY_FILE" | xargs -I {} echo "Total analysis time: {}s"
    
    # Check for critical failures
    local error_count=$(jq '.errors | length' "$SUMMARY_FILE")
    if [ "$error_count" -gt 0 ]; then
        print_warning "⚠ $error_count errors encountered during testing"
    else
        print_success "✓ All tests completed successfully"
    fi
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    pkill -f "uveddi.*serve" 2>/dev/null || true
}

# Main execution
main() {
    print_status "=== Uveddi Real-World Testing Suite ==="
    print_status "Timestamp: $TIMESTAMP"
    
    # Setup
    mkdir -p "$TEST_DIR" "$REPORTS_DIR" "$CODEBASES_DIR"
    init_summary
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Download/update codebases
    print_status "=== Phase 1: Setting up codebases ==="
    for name in "${!CODEBASES[@]}"; do
        download_codebase "$name"
    done
    
    # Run analysis on each codebase
    print_status "=== Phase 2: Running analysis ==="
    for name in "${!CODEBASES[@]}"; do
        if analyze_codebase "$name"; then
            verify_detector_coverage "$name"
        fi
    done
    
    # Test dashboard integration
    print_status "=== Phase 3: Testing dashboard ==="
    test_dashboard_flow
    
    # Generate final report
    print_status "=== Phase 4: Generating reports ==="
    generate_final_report
    
    print_success "=== Testing complete! ==="
    print_status "Reports available in: $REPORTS_DIR"
}

# Parse arguments
case "${1:-}" in
    --quick)
        # Test only small codebases
        declare -A CODEBASES=(
            ["rust-small"]="https://github.com/serde-rs/serde|Rust serialization library|rust"
            ["python-small"]="https://github.com/psf/requests|Python HTTP library|python"
        )
        main
        ;;
    --dashboard)
        test_dashboard_flow
        ;;
    --help)
        echo "Usage: $0 [--quick|--dashboard|--help]"
        echo ""
        echo "Options:"
        echo "  --quick     Test only small codebases"
        echo "  --dashboard Test only dashboard integration"
        echo "  --help      Show this help message"
        echo ""
        echo "Default: Test all configured codebases"
        ;;
    *)
        main
        ;;
esac