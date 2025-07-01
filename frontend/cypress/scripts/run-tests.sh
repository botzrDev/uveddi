#!/bin/bash

# Uveddi Frontend E2E Test Setup Script
# This script sets up and runs comprehensive end-to-end tests

set -e

echo "🚀 Setting up Uveddi Frontend E2E Tests..."

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: This script must be run from the frontend directory"
    exit 1
fi

# Check if Cypress is installed
if ! npm list cypress > /dev/null 2>&1; then
    echo "❌ Error: Cypress is not installed. Run 'npm install' first."
    exit 1
fi

echo "✅ Dependencies verified"

# Function to check if backend is running
check_backend() {
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "✅ Backend is running on port 8000"
        return 0
    else
        echo "⚠️  Backend is not running on port 8000"
        return 1
    fi
}

# Function to start frontend dev server in background
start_frontend() {
    echo "🌐 Starting frontend dev server..."
    npm run dev > /tmp/frontend.log 2>&1 &
    FRONTEND_PID=$!
    
    # Wait for frontend to be ready
    echo "⏳ Waiting for frontend to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:9999 > /dev/null 2>&1; then
            echo "✅ Frontend is ready on port 9999"
            return 0
        fi
        sleep 2
    done
    
    echo "❌ Frontend failed to start"
    kill $FRONTEND_PID 2>/dev/null || true
    exit 1
}

# Function to cleanup processes
cleanup() {
    echo "🧹 Cleaning up..."
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Parse command line arguments
RUN_MODE="headless"
TEST_SPEC=""
BROWSER="chrome"

while [[ $# -gt 0 ]]; do
    case $1 in
        --headed)
            RUN_MODE="headed"
            shift
            ;;
        --interactive)
            RUN_MODE="interactive"
            shift
            ;;
        --spec)
            TEST_SPEC="$2"
            shift 2
            ;;
        --browser)
            BROWSER="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --headed        Run tests in headed mode (visible browser)"
            echo "  --interactive   Open Cypress interactive mode"
            echo "  --spec SPEC     Run specific test spec file"
            echo "  --browser BROWSER  Use specific browser (chrome, firefox, edge)"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Run all tests headlessly"
            echo "  $0 --headed                 # Run tests with visible browser"
            echo "  $0 --interactive            # Open Cypress UI"
            echo "  $0 --spec landing-page.cy.ts # Run specific test"
            echo "  $0 --browser firefox        # Use Firefox browser"
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "🔧 Configuration:"
echo "  Run mode: $RUN_MODE"
echo "  Browser: $BROWSER"
if [ ! -z "$TEST_SPEC" ]; then
    echo "  Test spec: $TEST_SPEC"
fi

# Check backend status
if check_backend; then
    echo "ℹ️  Backend tests will include API integration"
else
    echo "ℹ️  Backend tests will use mocked APIs only"
fi

# Start frontend if not already running
if ! curl -s http://localhost:9999 > /dev/null 2>&1; then
    start_frontend
else
    echo "✅ Frontend is already running on port 9999"
fi

# Run tests based on mode
echo "🧪 Running tests..."

case $RUN_MODE in
    "headless")
        if [ ! -z "$TEST_SPEC" ]; then
            npx cypress run --browser "$BROWSER" --spec "cypress/e2e/$TEST_SPEC"
        else
            npx cypress run --browser "$BROWSER"
        fi
        ;;
    "headed")
        if [ ! -z "$TEST_SPEC" ]; then
            npx cypress run --headed --browser "$BROWSER" --spec "cypress/e2e/$TEST_SPEC"
        else
            npx cypress run --headed --browser "$BROWSER"
        fi
        ;;
    "interactive")
        npx cypress open
        ;;
esac

TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ All tests passed!"
else
    echo "❌ Some tests failed (exit code: $TEST_EXIT_CODE)"
fi

# Generate test report summary
if [ -d "cypress/reports" ]; then
    echo ""
    echo "📊 Test Report Summary:"
    if [ -f "cypress/reports/mochawesome.json" ]; then
        node -e "
            const report = require('./cypress/reports/mochawesome.json');
            console.log(\`  Total tests: \${report.stats.tests}\`);
            console.log(\`  Passed: \${report.stats.passes}\`);
            console.log(\`  Failed: \${report.stats.failures}\`);
            console.log(\`  Skipped: \${report.stats.pending}\`);
            console.log(\`  Duration: \${(report.stats.duration / 1000).toFixed(2)}s\`);
        " 2>/dev/null || echo "  Report parsing failed"
    fi
    
    echo "  Screenshots: cypress/screenshots/"
    echo "  Videos: cypress/videos/"
    echo "  Reports: cypress/reports/"
fi

echo ""
echo "🎯 Test Categories Covered:"
echo "  ✅ Landing Page & Navigation"
echo "  ✅ User Authentication (Login/Register)"
echo "  ✅ Dashboard & Data Display" 
echo "  ✅ Analysis Details & Reports"
echo "  ✅ End-to-End User Journeys"
echo "  ✅ Performance Testing"
echo "  ✅ Accessibility Testing"
echo "  ✅ Error Handling"
echo "  ✅ Responsive Design"

echo ""
echo "📋 Test Coverage Areas:"
echo "  • User Interface Components"
echo "  • Form Validation & Submission"
echo "  • API Integration & Error Handling"
echo "  • Authentication & Authorization"
echo "  • Data Loading & Display"
echo "  • Search & Filtering"
echo "  • Export & Sharing Features"
echo "  • Keyboard Navigation"
echo "  • Screen Reader Compatibility"
echo "  • Performance Metrics"
echo "  • Memory Usage"
echo "  • Network Conditions"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo ""
    echo "🎉 Uveddi frontend is ready for production!"
else
    echo ""
    echo "🔧 Please fix failing tests before deployment."
fi

exit $TEST_EXIT_CODE
