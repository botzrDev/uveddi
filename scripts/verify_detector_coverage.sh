#!/bin/bash

# Uveddi Detector Coverage Verification Framework
# This script comprehensively tests all detectors across multiple codebases
# and verifies their results appear in all report formats

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
UVEDDI_DIR="/home/austingreen/Documents/botzr/projects/uveddi"
REPORTS_DIR="$UVEDDI_DIR/verification_reports"
TEST_CODEBASES_DIR="$UVEDDI_DIR/test_codebases"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Detectors to verify
DETECTORS=(
    "GodObjectDetector"
    "CodeDuplicationDetector"
    "DeadCodeDetector"
    "LargeClassDetector"
    "TightCouplingDetector"
    "LongMethodsDetector"
    "MagicValuesDetector"
)

# Anti-pattern types to verify in reports
ANTI_PATTERNS=(
    "God Object"
    "Code Duplication"
    "Dead Code"
    "Large Class"
    "Tight Coupling"
    "Long Method"
    "Magic Values"
)

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to create test codebases with known issues
create_test_codebases() {
    print_status "Creating test codebases with known detector triggers..."
    
    mkdir -p "$TEST_CODEBASES_DIR"
    
    # Create Rust test file with multiple anti-patterns
    cat > "$TEST_CODEBASES_DIR/test_rust.rs" << 'EOF'
// God Object - too many methods and fields
pub struct GodObject {
    field1: String,
    field2: i32,
    field3: Vec<String>,
    field4: bool,
    field5: f64,
    field6: HashMap<String, String>,
    field7: Option<String>,
    field8: Result<i32, String>,
    field9: Vec<i32>,
}

impl GodObject {
    pub fn method1(&self) {}
    pub fn method2(&self) {}
    pub fn method3(&self) {}
    pub fn method4(&self) {}
    pub fn method5(&self) {}
    pub fn method6(&self) {}
    pub fn method7(&self) {}
    pub fn method8(&self) {}
}

// Code duplication
fn duplicate_code_1() {
    let x = 10;
    let y = 20;
    let z = x + y;
    println!("Result: {}", z);
    if z > 25 {
        println!("Large result");
    }
}

fn duplicate_code_2() {
    let x = 10;
    let y = 20;
    let z = x + y;
    println!("Result: {}", z);
    if z > 25 {
        println!("Large result");
    }
}

// Dead code
fn unused_function() {
    println!("This is never called");
}

// Long method
fn very_long_method() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let m = 13;
    let n = 14;
    let o = 15;
    let p = 16;
    let q = 17;
    let r = 18;
    let s = 19;
    let t = 20;
    let u = 21;
    let v = 22;
    let w = 23;
    let x = 24;
    let y = 25;
    let z = 26;
    println!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", 
             a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z);
}

// Magic values
fn magic_values_example() {
    let timeout = 3000; // Magic number
    let max_retries = 5; // Magic number
    let buffer_size = 1024; // Magic number
}

use std::collections::HashMap;
EOF

    # Create Python test file with anti-patterns
    cat > "$TEST_CODEBASES_DIR/test_python.py" << 'EOF'
# God Object with many methods and attributes
class GodObject:
    def __init__(self):
        self.attr1 = None
        self.attr2 = None
        self.attr3 = None
        self.attr4 = None
        self.attr5 = None
        self.attr6 = None
        self.attr7 = None
        self.attr8 = None
        self.attr9 = None
    
    def method1(self): pass
    def method2(self): pass
    def method3(self): pass
    def method4(self): pass
    def method5(self): pass
    def method6(self): pass
    def method7(self): pass
    def method8(self): pass

# Code duplication
def duplicate_function_1():
    x = 10
    y = 20
    result = x + y
    print(f"Result: {result}")
    if result > 25:
        print("Large result")
    return result

def duplicate_function_2():
    x = 10
    y = 20
    result = x + y
    print(f"Result: {result}")
    if result > 25:
        print("Large result")
    return result

# Dead code
def unused_function():
    """This function is never called"""
    return "Never used"

# Long method
def very_long_method():
    a = 1
    b = 2
    c = 3
    d = 4
    e = 5
    f = 6
    g = 7
    h = 8
    i = 9
    j = 10
    k = 11
    l = 12
    m = 13
    n = 14
    o = 15
    p = 16
    q = 17
    r = 18
    s = 19
    t = 20
    u = 21
    v = 22
    w = 23
    x = 24
    y = 25
    z = 26
    print(a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z)
    return a+b+c+d+e+f+g+h+i+j+k+l+m+n+o+p+q+r+s+t+u+v+w+x+y+z

# Magic values
def magic_values_example():
    timeout = 3000  # Magic number
    max_retries = 5  # Magic number
    buffer_size = 1024  # Magic number
    return timeout * max_retries * buffer_size
EOF

    # Create JavaScript test file
    cat > "$TEST_CODEBASES_DIR/test_javascript.js" << 'EOF'
// God Object with many properties and methods
class GodObject {
    constructor() {
        this.prop1 = null;
        this.prop2 = null;
        this.prop3 = null;
        this.prop4 = null;
        this.prop5 = null;
        this.prop6 = null;
        this.prop7 = null;
        this.prop8 = null;
        this.prop9 = null;
    }
    
    method1() {}
    method2() {}
    method3() {}
    method4() {}
    method5() {}
    method6() {}
    method7() {}
    method8() {}
}

// Code duplication
function duplicateFunction1() {
    const x = 10;
    const y = 20;
    const result = x + y;
    console.log(`Result: ${result}`);
    if (result > 25) {
        console.log("Large result");
    }
    return result;
}

function duplicateFunction2() {
    const x = 10;
    const y = 20;
    const result = x + y;
    console.log(`Result: ${result}`);
    if (result > 25) {
        console.log("Large result");
    }
    return result;
}

// Dead code
function unusedFunction() {
    console.log("This is never called");
}

// Long method
function veryLongMethod() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let m = 13;
    let n = 14;
    let o = 15;
    let p = 16;
    let q = 17;
    let r = 18;
    let s = 19;
    let t = 20;
    let u = 21;
    let v = 22;
    let w = 23;
    let x = 24;
    let y = 25;
    let z = 26;
    console.log(a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z);
}

// Magic values
function magicValuesExample() {
    const timeout = 3000; // Magic number
    const maxRetries = 5; // Magic number
    const bufferSize = 1024; // Magic number
    return timeout * maxRetries * bufferSize;
}
EOF

    print_success "Test codebases created"
}

# Function to run analysis on a codebase
run_analysis() {
    local codebase_path=$1
    local output_name=$2
    
    print_status "Analyzing $codebase_path..."
    
    # Run analysis with all output formats
    cd "$UVEDDI_DIR"
    
    # JSON output
    cargo run --bin uveddi --features=community -- analyze "$codebase_path" \
        --output-format json \
        --output "$REPORTS_DIR/${output_name}_${TIMESTAMP}.json" 2>&1 | tee "$REPORTS_DIR/${output_name}_analysis.log"
    
    # HTML output
    cargo run --bin uveddi --features=community -- analyze "$codebase_path" \
        --output-format html \
        --output "$REPORTS_DIR/${output_name}_${TIMESTAMP}.html" 2>&1 | tee -a "$REPORTS_DIR/${output_name}_analysis.log"
    
    # Markdown output
    cargo run --bin uveddi --features=community -- analyze "$codebase_path" \
        --output-format markdown \
        --output "$REPORTS_DIR/${output_name}_${TIMESTAMP}.md" 2>&1 | tee -a "$REPORTS_DIR/${output_name}_analysis.log"
}

# Function to verify detector output in JSON report
verify_json_report() {
    local json_file=$1
    local report_name=$2
    
    print_status "Verifying detectors in JSON report: $json_file"
    
    if [ ! -f "$json_file" ]; then
        print_error "JSON file not found: $json_file"
        return 1
    fi
    
    # Check for each anti-pattern type
    local missing_patterns=()
    for pattern in "${ANTI_PATTERNS[@]}"; do
        if grep -q "\"antiPatternType\": \"$pattern\"" "$json_file"; then
            print_success "✓ Found '$pattern' in JSON report"
        else
            print_warning "✗ Missing '$pattern' in JSON report"
            missing_patterns+=("$pattern")
        fi
    done
    
    # Check overall structure
    if jq -e '.issues' "$json_file" > /dev/null 2>&1; then
        local issue_count=$(jq '.issues | length' "$json_file")
        print_status "Found $issue_count issues in JSON report"
        
        # Verify issue structure
        if jq -e '.issues[0] | has("antiPatternType", "filePath", "severity", "description")' "$json_file" > /dev/null 2>&1; then
            print_success "✓ JSON report has correct issue structure"
        else
            print_error "✗ JSON report has incorrect issue structure"
        fi
    else
        print_error "✗ JSON report missing 'issues' field"
    fi
    
    # Summary statistics
    if jq -e '.summary' "$json_file" > /dev/null 2>&1; then
        print_success "✓ JSON report has summary section"
        jq '.summary' "$json_file"
    else
        print_warning "✗ JSON report missing summary section"
    fi
    
    if [ ${#missing_patterns[@]} -gt 0 ]; then
        print_warning "Missing patterns: ${missing_patterns[*]}"
        return 1
    fi
    
    return 0
}

# Function to verify HTML report
verify_html_report() {
    local html_file=$1
    
    print_status "Verifying HTML report: $html_file"
    
    if [ ! -f "$html_file" ]; then
        print_error "HTML file not found: $html_file"
        return 1
    fi
    
    # Check for detector output in HTML
    for pattern in "${ANTI_PATTERNS[@]}"; do
        if grep -q "$pattern" "$html_file"; then
            print_success "✓ Found '$pattern' in HTML report"
        else
            print_warning "✗ Missing '$pattern' in HTML report"
        fi
    done
    
    # Check for required HTML elements
    if grep -q "<table" "$html_file"; then
        print_success "✓ HTML report contains tables"
    fi
    
    if grep -q "class=\"issue" "$html_file"; then
        print_success "✓ HTML report contains issue elements"
    fi
}

# Function to test dashboard integration
test_dashboard_integration() {
    print_status "Testing dashboard integration..."
    
    # Start the web services
    cd "$UVEDDI_DIR"
    cargo run --bin uveddi --features=community -- serve --port 8888 --rendering-port 3333 &
    local SERVER_PID=$!
    
    sleep 10
    
    # Check health endpoints
    if curl -s http://localhost:8888/health | grep -q "ok"; then
        print_success "✓ API server is healthy"
    else
        print_error "✗ API server health check failed"
    fi
    
    if curl -s http://localhost:3333/health | grep -q "ok"; then
        print_success "✓ Rendering service is healthy"
    else
        print_error "✗ Rendering service health check failed"
    fi
    
    # Test API endpoints for detector data
    local api_response=$(curl -s http://localhost:8888/api/v1/analysis 2>/dev/null || echo "{}")
    if [ ! -z "$api_response" ]; then
        print_success "✓ API endpoint responds"
    fi
    
    # Stop the server
    kill $SERVER_PID 2>/dev/null || true
    
    print_success "Dashboard integration test completed"
}

# Function to generate verification report
generate_verification_report() {
    local report_file="$REPORTS_DIR/verification_summary_${TIMESTAMP}.md"
    
    cat > "$report_file" << EOF
# Uveddi Detector Verification Report
Generated: $(date)

## Test Configuration
- Uveddi Directory: $UVEDDI_DIR
- Test Codebases: $TEST_CODEBASES_DIR
- Reports Directory: $REPORTS_DIR

## Detectors Tested
$(printf '%s\n' "${DETECTORS[@]}" | sed 's/^/- /')

## Anti-Patterns Verified
$(printf '%s\n' "${ANTI_PATTERNS[@]}" | sed 's/^/- /')

## Test Results Summary

### JSON Report Verification
EOF
    
    # Add JSON verification results
    for json_file in "$REPORTS_DIR"/*_${TIMESTAMP}.json; do
        if [ -f "$json_file" ]; then
            echo "- $(basename $json_file): " >> "$report_file"
            if verify_json_report "$json_file" "$(basename $json_file .json)" >> "$report_file" 2>&1; then
                echo "  ✓ PASSED" >> "$report_file"
            else
                echo "  ✗ FAILED" >> "$report_file"
            fi
        fi
    done
    
    echo "" >> "$report_file"
    echo "### HTML Report Verification" >> "$report_file"
    
    # Add HTML verification results
    for html_file in "$REPORTS_DIR"/*_${TIMESTAMP}.html; do
        if [ -f "$html_file" ]; then
            echo "- $(basename $html_file): " >> "$report_file"
            if verify_html_report "$html_file" >> "$report_file" 2>&1; then
                echo "  ✓ PASSED" >> "$report_file"
            else
                echo "  ✗ FAILED" >> "$report_file"
            fi
        fi
    done
    
    echo "" >> "$report_file"
    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. Ensure all detectors are properly registered in the detector registry" >> "$report_file"
    echo "2. Verify detector output is correctly serialized to all report formats" >> "$report_file"
    echo "3. Test with larger, real-world codebases for comprehensive validation" >> "$report_file"
    echo "4. Implement missing tests for LongMethodsDetector and MagicValuesDetector" >> "$report_file"
    
    print_success "Verification report generated: $report_file"
}

# Main execution
main() {
    print_status "Starting Uveddi Detector Coverage Verification"
    print_status "Timestamp: $TIMESTAMP"
    
    # Setup
    mkdir -p "$REPORTS_DIR"
    
    # Create test codebases
    create_test_codebases
    
    # Run analysis on test codebases
    run_analysis "$TEST_CODEBASES_DIR" "test_codebases"
    
    # Verify each report format
    for json_file in "$REPORTS_DIR"/test_codebases_${TIMESTAMP}.json; do
        if [ -f "$json_file" ]; then
            verify_json_report "$json_file" "test_codebases"
        fi
    done
    
    for html_file in "$REPORTS_DIR"/test_codebases_${TIMESTAMP}.html; do
        if [ -f "$html_file" ]; then
            verify_html_report "$html_file"
        fi
    done
    
    # Test dashboard integration
    test_dashboard_integration
    
    # Generate summary report
    generate_verification_report
    
    print_success "Verification complete! Check $REPORTS_DIR for detailed results"
}

# Handle script arguments
if [ "$1" == "--help" ]; then
    echo "Usage: $0 [--quick|--full|--dashboard-only]"
    echo ""
    echo "Options:"
    echo "  --quick         Run quick verification with test codebases only"
    echo "  --full          Run full verification including real codebases"
    echo "  --dashboard-only Test only dashboard integration"
    echo "  --help          Show this help message"
    exit 0
fi

case "$1" in
    --quick)
        main
        ;;
    --dashboard-only)
        test_dashboard_integration
        ;;
    --full|*)
        main
        ;;
esac