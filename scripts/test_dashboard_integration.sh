#!/usr/bin/env bash

# Dashboard Integration Test
# This script tests the integration between the CLI analysis tool and the dashboard

set -e

echo "⚙️  Starting Dashboard Integration Test"
echo

# Set up test directory
TEST_DIR="/tmp/uveddi_dashboard_test"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create a sample file with code issues
cat > test_file.rs << 'EOF'
// Test file with some issues

// Dead code - unused function
fn unused_function() {
    println!("This function is never used");
}

// Large class - too many methods
struct LargeClass {
    field1: i32,
    field2: String,
    field3: Vec<i32>,
    field4: Option<String>,
    field5: bool,
}

impl LargeClass {
    fn method1(&self) { }
    fn method2(&self) { }
    fn method3(&self) { }
    fn method4(&self) { }
    fn method5(&self) { }
    fn method6(&self) { }
    fn method7(&self) { }
    fn method8(&self) { }
    fn method9(&self) { }
    fn method10(&self) { }
}

fn main() {
    let x = LargeClass {
        field1: 1,
        field2: "test".to_string(),
        field3: vec![1, 2, 3],
        field4: Some("test".to_string()),
        field5: true,
    };
}
EOF

echo "✅ Created test project"

# Run the analysis with dashboard notification
echo "🔍 Running analysis..."
UVEDDI_BIN="uveddi"
if [ ! -f "$(which uveddi)" ]; then
    # If uveddi is not in PATH, try building from source
    echo "Building uveddi from source..."
    cd -  # Go back to previous directory
    cargo build --release
    UVEDDI_BIN="./target/release/uveddi"
fi

$UVEDDI_BIN analyze "$TEST_DIR" --output-format=json --output="$TEST_DIR/report.json"

echo "✅ Analysis completed, results written to report.json"

# Check if report file exists
if [ ! -f "$TEST_DIR/report.json" ]; then
    echo "❌ ERROR: Report file not created"
    exit 1
fi

# Check for notification file
NOTIFICATION_DIR="./.uveddi/notifications"
if [ -d "$NOTIFICATION_DIR" ]; then
    NOTIFICATION_COUNT=$(ls "$NOTIFICATION_DIR" | grep -c "analysis_complete_")
    echo "✅ Found $NOTIFICATION_COUNT notification(s) in $NOTIFICATION_DIR"
else
    echo "⚠️  Warning: No notification directory found"
fi

# Start the dashboard server in background
echo "🚀 Starting dashboard server..."
$UVEDDI_BIN serve &
SERVE_PID=$!

# Wait for server to start
sleep 5

# Check if server is running
if kill -0 $SERVE_PID 2>/dev/null; then
    echo "✅ Dashboard server started successfully"
else
    echo "❌ ERROR: Dashboard server failed to start"
    exit 1
fi

# Make a request to the dashboard API
echo "🔍 Testing dashboard API..."
if command -v curl >/dev/null 2>&1; then
    HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/health)
    if [ "$HTTP_STATUS" == "200" ]; then
        echo "✅ Dashboard API is working"
    else
        echo "❌ ERROR: Dashboard API returned status $HTTP_STATUS"
    fi
else
    echo "⚠️  Warning: curl not found, skipping API test"
fi

# Stop the server
kill $SERVE_PID
echo "✅ Dashboard server stopped"

echo
echo "✅ Dashboard Integration Test Completed Successfully"
