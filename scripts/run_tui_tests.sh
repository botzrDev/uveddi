#!/bin/bash
set -e

echo "🔍 Running TUI Integration Tests..."
echo "=================================="

# Ensure we have the tui feature enabled
export RUST_LOG=info

# Run TUI integration tests with timeout
echo "📋 Running TUI Integration Tests..."
timeout 300s cargo test --test tui_integration --features tui

echo ""
echo "📝 Running TUI Form Validation Tests..."
timeout 300s cargo test --test tui_form_validation --features tui

echo ""
echo "🔄 Running TUI End-to-End Tests..."
timeout 300s cargo test --test tui_e2e --features tui

echo ""
echo "⚡ Running TUI Performance Tests..."
timeout 300s cargo test --test tui_performance --features tui

echo ""
echo "✅ All TUI tests completed successfully!"
echo ""
echo "📊 Test Summary:"
echo "=================="
echo "✓ TUI Integration Tests - Core functionality and backend integration"
echo "✓ TUI Form Validation Tests - Form data validation and conversion"
echo "✓ TUI End-to-End Tests - Complete user workflows"
echo "✓ TUI Performance Tests - Performance and load testing"
echo ""
echo "🎯 Coverage Areas:"
echo "- State management (TEA pattern)"
echo "- Message handling and event flow"
echo "- Form validation and data conversion"  
echo "- Backend integration and analysis orchestration"
echo "- Error handling and user feedback"
echo "- Performance under load"
echo "- Concurrent operations"
echo "- Memory usage stability"
echo ""
echo "🚀 TUI is ready for production use!"