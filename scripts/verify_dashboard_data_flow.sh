#!/bin/bash

# Verify Detector Data Flow to React Dashboard
# This script ensures all detector results properly flow through the entire pipeline:
# Detectors → JSON Reports → API → Dashboard

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
UVEDDI_DIR="/home/austingreen/Documents/botzr/projects/uveddi"
TEST_DIR="$UVEDDI_DIR/dashboard_flow_test"
REPORTS_DIR="$TEST_DIR/reports"
FRONTEND_DIR="$UVEDDI_DIR/frontend"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Function definitions
print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Create comprehensive test file with known detector triggers
create_test_source() {
    local test_file="$TEST_DIR/comprehensive_test.rs"
    
    cat > "$test_file" << 'EOF'
//! Comprehensive test file designed to trigger all detectors

use std::collections::HashMap;

// God Object - many methods and fields (should trigger GodObjectDetector)
pub struct UserManager {
    pub users: HashMap<String, User>,
    pub sessions: HashMap<String, Session>,
    pub permissions: HashMap<String, Vec<String>>,
    pub roles: HashMap<String, Role>,
    pub groups: HashMap<String, Group>,
    pub audit_log: Vec<AuditEntry>,
    pub config: SystemConfig,
    pub cache: CacheManager,
    pub database_pool: ConnectionPool,
}

impl UserManager {
    pub fn create_user(&self) {}
    pub fn delete_user(&self) {}
    pub fn update_user(&self) {}
    pub fn authenticate_user(&self) {}
    pub fn authorize_action(&self) {}
    pub fn assign_role(&self) {}
    pub fn remove_role(&self) {}
    pub fn create_group(&self) {}
    pub fn add_to_group(&self) {}
    pub fn remove_from_group(&self) {}
    pub fn audit_action(&self) {}
    pub fn cleanup_sessions(&self) {}
    pub fn backup_data(&self) {}
    pub fn restore_data(&self) {}
    pub fn generate_report(&self) {}
    pub fn send_notification(&self) {}
    pub fn log_event(&self) {}
    pub fn validate_permission(&self) {}
    pub fn cache_result(&self) {}
    pub fn invalidate_cache(&self) {}
}

// Code Duplication - identical functions (should trigger CodeDuplicationDetector)
fn process_payment_v1(amount: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err("Invalid amount".to_string());
    }
    let fee = amount * 0.03;
    let total = amount + fee;
    let transaction_id = format!("tx_{}", rand::random::<u64>());
    println!("Processing payment: ${:.2}", total);
    Ok(transaction_id)
}

fn process_payment_v2(amount: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err("Invalid amount".to_string());
    }
    let fee = amount * 0.03;
    let total = amount + fee;
    let transaction_id = format!("tx_{}", rand::random::<u64>());
    println!("Processing payment: ${:.2}", total);
    Ok(transaction_id)
}

// Dead Code - unused functions (should trigger DeadCodeDetector)
fn unused_helper_function() {
    println!("This function is never called");
}

fn another_unused_function(data: &str) -> String {
    format!("Processed: {}", data)
}

// Large Class - many lines and methods (should trigger LargeClassDetector)
pub struct DataProcessor {
    data: Vec<String>,
    config: ProcessingConfig,
    cache: HashMap<String, ProcessedData>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            config: ProcessingConfig::default(),
            cache: HashMap::new(),
        }
    }
    
    pub fn process_batch(&mut self) {
        // Large method with many lines
        for item in &self.data {
            let processed = self.preprocess(item);
            let validated = self.validate(&processed);
            let transformed = self.transform(&validated);
            let enriched = self.enrich(&transformed);
            let formatted = self.format(&enriched);
            let stored = self.store(&formatted);
            self.update_cache(&stored);
            self.log_processing(&stored);
            self.notify_completion(&stored);
            self.cleanup_temp_data(&stored);
        }
    }
    
    fn preprocess(&self, data: &str) -> String { data.to_string() }
    fn validate(&self, data: &str) -> String { data.to_string() }
    fn transform(&self, data: &str) -> String { data.to_string() }
    fn enrich(&self, data: &str) -> String { data.to_string() }
    fn format(&self, data: &str) -> String { data.to_string() }
    fn store(&self, data: &str) -> String { data.to_string() }
    fn update_cache(&mut self, data: &str) {}
    fn log_processing(&self, data: &str) {}
    fn notify_completion(&self, data: &str) {}
    fn cleanup_temp_data(&self, data: &str) {}
}

// Long Method - method with many lines (should trigger LongMethodsDetector)
pub fn very_long_computation(input: Vec<i32>) -> i32 {
    let mut result = 0;
    let mut temp1 = 0;
    let mut temp2 = 0;
    let mut temp3 = 0;
    let mut temp4 = 0;
    let mut temp5 = 0;
    
    for (i, value) in input.iter().enumerate() {
        temp1 = value * 2;
        temp2 = temp1 + i as i32;
        temp3 = temp2 * temp2;
        temp4 = temp3 / (i as i32 + 1);
        temp5 = temp4 % 100;
        result += temp5;
        
        if result > 1000 {
            result -= 500;
        } else if result < -1000 {
            result += 500;
        }
        
        if i % 10 == 0 {
            result *= 2;
        }
        
        if i % 20 == 0 {
            result /= 3;
        }
        
        println!("Step {}: temp1={}, temp2={}, temp3={}, temp4={}, temp5={}, result={}", 
                 i, temp1, temp2, temp3, temp4, temp5, result);
    }
    
    result
}

// Magic Values - hardcoded constants (should trigger MagicValuesDetector)
pub fn configuration_example() {
    let timeout = 5000; // Magic number
    let max_retries = 3; // Magic number
    let buffer_size = 8192; // Magic number
    let cache_ttl = 300; // Magic number
    let rate_limit = 100; // Magic number
    
    println!("Config: timeout={}, retries={}, buffer={}, ttl={}, rate={}", 
             timeout, max_retries, buffer_size, cache_ttl, rate_limit);
}

// Tight Coupling - dependencies between modules (should trigger TightCouplingDetector)
pub struct OrderService {
    payment_processor: PaymentProcessor,
    inventory_manager: InventoryManager,
    notification_service: NotificationService,
    audit_logger: AuditLogger,
}

impl OrderService {
    pub fn process_order(&self, order: Order) {
        self.inventory_manager.reserve_items(&order);
        self.payment_processor.charge_customer(&order);
        self.notification_service.send_confirmation(&order);
        self.audit_logger.log_order(&order);
    }
}

// Supporting structs
#[derive(Default)]
struct ProcessingConfig;
struct ProcessedData;
struct User;
struct Session;
struct Role;
struct Group;
struct AuditEntry;
struct SystemConfig;
struct CacheManager;
struct ConnectionPool;
struct Order;
struct PaymentProcessor;
struct InventoryManager;
struct NotificationService;
struct AuditLogger;

impl PaymentProcessor {
    fn charge_customer(&self, _order: &Order) {}
}

impl InventoryManager {
    fn reserve_items(&self, _order: &Order) {}
}

impl NotificationService {
    fn send_confirmation(&self, _order: &Order) {}
}

impl AuditLogger {
    fn log_order(&self, _order: &Order) {}
}
EOF

    print_success "Created comprehensive test file: $test_file"
}

# Run analysis and capture results
run_analysis() {
    print_status "Running Uveddi analysis on test file..."
    
    cd "$UVEDDI_DIR"
    
    # Run analysis with JSON output
    local json_output="$REPORTS_DIR/test_analysis_${TIMESTAMP}.json"
    local html_output="$REPORTS_DIR/test_analysis_${TIMESTAMP}.html"
    
    if cargo run --bin uveddi --features=community -- analyze "$TEST_DIR/comprehensive_test.rs" \
        --output-format json \
        --output "$json_output" 2>&1 | tee "$REPORTS_DIR/analysis.log"; then
        
        print_success "Analysis completed successfully"
        
        # Also generate HTML for comparison
        cargo run --bin uveddi --features=community -- analyze "$TEST_DIR/comprehensive_test.rs" \
            --output-format html \
            --output "$html_output" 2>&1 > /dev/null
            
        return 0
    else
        print_error "Analysis failed"
        return 1
    fi
}

# Verify all expected detectors fired
verify_detector_output() {
    local json_file="$REPORTS_DIR/test_analysis_${TIMESTAMP}.json"
    
    print_status "Verifying detector output in JSON report..."
    
    if [ ! -f "$json_file" ]; then
        print_error "JSON report not found: $json_file"
        return 1
    fi
    
    # Expected anti-patterns
    local expected_patterns=(
        "God Object"
        "Code Duplication"
        "Dead Code"
        "Large Class"
        "Long Method"
        "Magic Values"
        "Tight Coupling"
    )
    
    local detected_patterns=()
    local missing_patterns=()
    
    # Check for each expected pattern
    for pattern in "${expected_patterns[@]}"; do
        if jq -e --arg pattern "$pattern" '.issues[] | select(.antiPatternType == $pattern)' "$json_file" > /dev/null 2>&1; then
            local count=$(jq --arg pattern "$pattern" '[.issues[] | select(.antiPatternType == $pattern)] | length' "$json_file")
            print_success "✓ $pattern: $count instances detected"
            detected_patterns+=("$pattern")
        else
            print_warning "✗ $pattern: not detected"
            missing_patterns+=("$pattern")
        fi
    done
    
    # Summary
    echo ""
    print_status "Detection Summary:"
    print_status "Detected: ${#detected_patterns[@]}/${#expected_patterns[@]} patterns"
    
    if [ ${#missing_patterns[@]} -gt 0 ]; then
        print_warning "Missing patterns: ${missing_patterns[*]}"
        return 1
    else
        print_success "All expected patterns detected!"
        return 0
    fi
}

# Test web services startup
start_web_services() {
    print_status "Starting Uveddi web services..."
    
    cd "$UVEDDI_DIR"
    
    # Start services in background
    cargo run --bin uveddi --features=community -- serve --port 8888 --rendering-port 3333 \
        > "$REPORTS_DIR/server.log" 2>&1 &
    
    local SERVER_PID=$!
    echo $SERVER_PID > "$REPORTS_DIR/server.pid"
    
    # Wait for services to start
    print_status "Waiting for services to start..."
    local attempts=0
    local max_attempts=30
    
    while [ $attempts -lt $max_attempts ]; do
        if curl -s http://localhost:8888/health > /dev/null 2>&1; then
            print_success "✓ API server is running"
            break
        fi
        
        sleep 2
        attempts=$((attempts + 1))
        
        if [ $attempts -eq $max_attempts ]; then
            print_error "API server failed to start within 60 seconds"
            return 1
        fi
    done
    
    # Check rendering service
    if curl -s http://localhost:3333/health > /dev/null 2>&1; then
        print_success "✓ Rendering service is running"
    else
        print_warning "⚠ Rendering service not responding"
    fi
    
    return 0
}

# Test API endpoints
test_api_endpoints() {
    print_status "Testing API endpoints..."
    
    # Test health endpoint
    if curl -s http://localhost:8888/health | jq -e '.status == "ok"' > /dev/null 2>&1; then
        print_success "✓ Health endpoint working"
    else
        print_error "✗ Health endpoint failed"
        return 1
    fi
    
    # Test API version endpoint
    if curl -s http://localhost:8888/api/v1 > /dev/null 2>&1; then
        print_success "✓ API v1 endpoint accessible"
    else
        print_warning "⚠ API v1 endpoint not accessible"
    fi
    
    # Test analysis endpoint (if it exists)
    if curl -s http://localhost:8888/api/v1/analysis > /dev/null 2>&1; then
        print_success "✓ Analysis API endpoint accessible"
        
        # Save response for analysis
        curl -s http://localhost:8888/api/v1/analysis > "$REPORTS_DIR/api_response.json"
    else
        print_warning "⚠ Analysis API endpoint not implemented yet"
    fi
    
    return 0
}

# Test dashboard accessibility
test_dashboard_access() {
    print_status "Testing dashboard accessibility..."
    
    # Test main dashboard
    if curl -s http://localhost:8888 > /dev/null 2>&1; then
        print_success "✓ Dashboard is accessible"
        
        # Save dashboard HTML for inspection
        curl -s http://localhost:8888 > "$REPORTS_DIR/dashboard.html"
        
        # Check if it contains expected elements
        if grep -q "uveddi\|dashboard\|analysis" "$REPORTS_DIR/dashboard.html"; then
            print_success "✓ Dashboard contains expected content"
        else
            print_warning "⚠ Dashboard content may be incomplete"
        fi
    else
        print_error "✗ Dashboard not accessible"
        return 1
    fi
    
    return 0
}

# Verify data flow from analysis to dashboard
verify_data_flow() {
    print_status "Verifying data flow from analysis to dashboard..."
    
    local json_report="$REPORTS_DIR/test_analysis_${TIMESTAMP}.json"
    
    # 1. Check if JSON report exists
    if [ ! -f "$json_report" ]; then
        print_error "JSON report not found for data flow verification"
        return 1
    fi
    
    # 2. Extract key data from JSON report
    local total_issues=$(jq '.issues | length' "$json_report")
    local issue_types=$(jq -r '[.issues[].antiPatternType] | unique | join(", ")' "$json_report")
    
    print_status "JSON Report Data:"
    print_status "  Total issues: $total_issues"
    print_status "  Issue types: $issue_types"
    
    # 3. Check if API serves similar data
    if [ -f "$REPORTS_DIR/api_response.json" ]; then
        print_status "Comparing with API response..."
        
        # Basic structure comparison
        if jq -e 'has("issues") or has("data") or has("analysis")' "$REPORTS_DIR/api_response.json" > /dev/null 2>&1; then
            print_success "✓ API response has expected structure"
        else
            print_warning "⚠ API response structure unclear"
        fi
    fi
    
    # 4. Check if dashboard can access the data
    if [ -f "$REPORTS_DIR/dashboard.html" ]; then
        # Look for data loading indicators
        if grep -q "fetch\|axios\|api" "$REPORTS_DIR/dashboard.html"; then
            print_success "✓ Dashboard appears to load data via API"
        else
            print_warning "⚠ Dashboard data loading mechanism unclear"
        fi
    fi
    
    return 0
}

# Check if frontend exists and can be tested
test_frontend_integration() {
    print_status "Checking frontend integration..."
    
    # Check if frontend directory exists
    if [ -d "$FRONTEND_DIR" ]; then
        print_status "Frontend directory found: $FRONTEND_DIR"
        
        # Check for package.json
        if [ -f "$FRONTEND_DIR/package.json" ]; then
            print_success "✓ Frontend package.json found"
            
            # Extract frontend info
            local frontend_name=$(jq -r '.name // "unknown"' "$FRONTEND_DIR/package.json")
            local frontend_version=$(jq -r '.version // "unknown"' "$FRONTEND_DIR/package.json")
            
            print_status "Frontend: $frontend_name v$frontend_version"
            
            # Check for React/dashboard components
            if find "$FRONTEND_DIR" -name "*.jsx" -o -name "*.tsx" -o -name "*.js" | head -5 | while read -r file; do
                if grep -l "dashboard\|analysis\|detector" "$file" 2>/dev/null; then
                    print_success "✓ Found dashboard-related frontend components"
                    return 0
                fi
            done; then
                :
            else
                print_warning "⚠ No dashboard-related components found in frontend"
            fi
        else
            print_warning "⚠ Frontend package.json not found"
        fi
    else
        print_warning "⚠ Frontend directory not found"
    fi
}

# Stop web services
stop_web_services() {
    if [ -f "$REPORTS_DIR/server.pid" ]; then
        local SERVER_PID=$(cat "$REPORTS_DIR/server.pid")
        if kill $SERVER_PID 2>/dev/null; then
            print_status "Stopped web services (PID: $SERVER_PID)"
        fi
        rm -f "$REPORTS_DIR/server.pid"
    fi
    
    # Fallback: kill any uveddi serve processes
    pkill -f "uveddi.*serve" 2>/dev/null || true
}

# Generate comprehensive report
generate_report() {
    local report_file="$REPORTS_DIR/dashboard_flow_report_${TIMESTAMP}.md"
    
    print_status "Generating comprehensive report..."
    
    cat > "$report_file" << 'EOF'
# Uveddi Dashboard Data Flow Verification Report

## Test Overview
This report verifies that all detector results properly flow from analysis through to the React dashboard.

### Test Methodology
1. Created comprehensive test file with known anti-pattern triggers
2. Ran Uveddi analysis to generate JSON reports
3. Started web services (API + rendering)
4. Verified API endpoints serve detector data
5. Checked dashboard accessibility and data loading

## Test Results

### Detector Coverage
EOF
    
    # Add detector results
    if [ -f "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" ]; then
        echo "**JSON Report Analysis:**" >> "$report_file"
        local total_issues=$(jq '.issues | length' "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json")
        echo "- Total issues detected: $total_issues" >> "$report_file"
        
        echo "" >> "$report_file"
        echo "**Detector Breakdown:**" >> "$report_file"
        
        # Count issues by type
        jq -r '.issues | group_by(.antiPatternType) | .[] | "\(.[0].antiPatternType): \(length) issues"' \
            "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" | while read -r line; do
            echo "- $line" >> "$report_file"
        done
    fi
    
    echo "" >> "$report_file"
    echo "### Web Services Status" >> "$report_file"
    
    # Check if services were tested
    if [ -f "$REPORTS_DIR/server.log" ]; then
        if grep -q "error\|Error\|ERROR" "$REPORTS_DIR/server.log"; then
            echo "- ⚠ Server logs contain errors" >> "$report_file"
        else
            echo "- ✓ Web services started successfully" >> "$report_file"
        fi
    fi
    
    echo "" >> "$report_file"
    echo "### Data Flow Verification" >> "$report_file"
    
    # Check each component in the flow
    if [ -f "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" ]; then
        echo "- ✓ Detectors → JSON Report: Working" >> "$report_file"
    else
        echo "- ✗ Detectors → JSON Report: Failed" >> "$report_file"
    fi
    
    if [ -f "$REPORTS_DIR/api_response.json" ]; then
        echo "- ✓ JSON Report → API: Working" >> "$report_file"
    else
        echo "- ⚠ JSON Report → API: Needs verification" >> "$report_file"
    fi
    
    if [ -f "$REPORTS_DIR/dashboard.html" ]; then
        echo "- ✓ API → Dashboard: Accessible" >> "$report_file"
    else
        echo "- ✗ API → Dashboard: Failed" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Critical Issues" >> "$report_file"
    echo "" >> "$report_file"
    
    # Check for missing detectors
    local missing_count=$(grep -c "not detected" "$REPORTS_DIR/analysis.log" 2>/dev/null || echo "0")
    if [ "$missing_count" -gt 0 ]; then
        echo "- $missing_count expected detectors did not fire" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. **Detector Coverage**: Ensure all 7 core detectors are active and working" >> "$report_file"
    echo "2. **API Integration**: Implement full REST API for dashboard data access" >> "$report_file"
    echo "3. **Frontend Testing**: Add automated tests for React dashboard components" >> "$report_file"
    echo "4. **Data Validation**: Verify data integrity through entire pipeline" >> "$report_file"
    
    echo "" >> "$report_file"
    echo "## Test Artifacts" >> "$report_file"
    echo "" >> "$report_file"
    echo "- Test source: \`$TEST_DIR/comprehensive_test.rs\`" >> "$report_file"
    echo "- JSON report: \`$REPORTS_DIR/test_analysis_${TIMESTAMP}.json\`" >> "$report_file"
    echo "- HTML report: \`$REPORTS_DIR/test_analysis_${TIMESTAMP}.html\`" >> "$report_file"
    echo "- Server logs: \`$REPORTS_DIR/server.log\`" >> "$report_file"
    echo "- Dashboard HTML: \`$REPORTS_DIR/dashboard.html\`" >> "$report_file"
    
    print_success "Report generated: $report_file"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    stop_web_services
}

# Main execution
main() {
    print_status "=== Uveddi Dashboard Data Flow Verification ==="
    print_status "Timestamp: $TIMESTAMP"
    
    # Setup
    mkdir -p "$TEST_DIR" "$REPORTS_DIR"
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Phase 1: Create test data
    print_status "=== Phase 1: Creating test data ==="
    create_test_source
    
    # Phase 2: Run analysis
    print_status "=== Phase 2: Running analysis ==="
    if ! run_analysis; then
        print_error "Analysis failed, aborting test"
        exit 1
    fi
    
    # Phase 3: Verify detector output
    print_status "=== Phase 3: Verifying detector output ==="
    verify_detector_output
    
    # Phase 4: Test web services
    print_status "=== Phase 4: Testing web services ==="
    if start_web_services; then
        test_api_endpoints
        test_dashboard_access
        verify_data_flow
    fi
    
    # Phase 5: Test frontend integration
    print_status "=== Phase 5: Frontend integration ==="
    test_frontend_integration
    
    # Phase 6: Generate report
    print_status "=== Phase 6: Generating report ==="
    generate_report
    
    print_success "=== Dashboard flow verification complete! ==="
    print_status "Reports available in: $REPORTS_DIR"
    
    # Final summary
    echo ""
    print_status "=== SUMMARY ==="
    if [ -f "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" ]; then
        local total_issues=$(jq '.issues | length' "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json")
        print_status "Total issues detected: $total_issues"
        
        if [ "$total_issues" -gt 0 ]; then
            print_success "✓ Detectors are working and generating output"
        else
            print_warning "⚠ No issues detected - check detector configuration"
        fi
    fi
    
    if curl -s http://localhost:8888/health > /dev/null 2>&1; then
        print_success "✓ Dashboard is accessible at http://localhost:8888"
    else
        print_warning "⚠ Dashboard may not be accessible"
    fi
}

# Parse command line arguments
case "${1:-}" in
    --help)
        echo "Usage: $0 [--quick|--help]"
        echo ""
        echo "Options:"
        echo "  --quick  Skip web services testing"
        echo "  --help   Show this help message"
        echo ""
        echo "This script verifies that all Uveddi detectors properly"
        echo "flow their results through to the React dashboard."
        ;;
    --quick)
        # Quick mode: skip web services
        mkdir -p "$TEST_DIR" "$REPORTS_DIR"
        create_test_source
        run_analysis
        verify_detector_output
        generate_report
        ;;
    *)
        main
        ;;
esac