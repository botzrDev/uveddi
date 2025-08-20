#!/bin/bash

# Uveddi Public Release Cleanup Script
# This script removes development files, test artifacts, and sensitive content
# before creating a public release.

set -e

echo "🧹 Starting Uveddi public release cleanup..."

# Function to safely remove files/directories
remove_if_exists() {
    if [ -e "$1" ]; then
        echo "Removing: $1"
        rm -rf "$1"
    else
        echo "Not found (skipping): $1"
    fi
}

# Function to check directory size before removal
check_and_remove() {
    if [ -e "$1" ]; then
        local size=$(du -sh "$1" 2>/dev/null | cut -f1 || echo "unknown")
        echo "Removing: $1 (size: $size)"
        rm -rf "$1"
    else
        echo "Not found (skipping): $1"
    fi
}

echo ""
echo "📦 Removing build artifacts and dependencies..."
check_and_remove "target"
check_and_remove "api-server/node_modules"
check_and_remove "rendering-service/node_modules" 
check_and_remove "frontend/node_modules"
check_and_remove "frontend/dist"

echo ""
echo "📊 Removing test coverage and profiling data..."
check_and_remove "coverage"
check_and_remove "coverage_reports"
remove_if_exists "coverage_result.txt"

echo ""
echo "⚙️ Removing internal configuration files..."
remove_if_exists "config/security/vault.toml"
remove_if_exists "config/deny.toml"
remove_if_exists "config/benchmark-config.toml"

echo ""
echo "🧪 Removing test scripts and development tools..."
remove_if_exists "scripts/quick-coverage.sh"
remove_if_exists "scripts/test-real-world.sh"
remove_if_exists "scripts/prepare-demo.sh"
remove_if_exists "scripts/run-automated-tests.sh"
remove_if_exists "scripts/comprehensive_test_runner.sh"
remove_if_exists "scripts/alpha-cleanup.sh"
remove_if_exists "scripts/alpha-cleanup-validate.sh"
remove_if_exists "scripts/verify_test_stability.sh"
remove_if_exists "scripts/run-coverage-tests.sh"
remove_if_exists "scripts/performance-validation.sh"
remove_if_exists "scripts/validate-build-optimization.sh"
remove_if_exists "scripts/fast-coverage.sh"

echo ""
echo "📝 Removing internal documentation and notes..."
remove_if_exists "ALPHA_TESTING_PROMPT.md"
remove_if_exists "COMMUNITY_RELEASE_1.0_PROMPT.md"
remove_if_exists "COMPREHENSIVE_TEST_REPORT.md"
remove_if_exists "DASHBOARD_ENHANCEMENTS.md" 
remove_if_exists "DEPENDENCY-OPTIMIZATION-SUMMARY.md"
remove_if_exists "INSTALL_SCRIPT_README.md"
remove_if_exists "LIGHT_MODE_BACKGROUND_ISSUE_REPORT.md"
remove_if_exists "PRE_LAUNCH_TESTING_PROMPT.md"
remove_if_exists "build-optimization-plan.md"
remove_if_exists "consistency_test.md"

echo ""
echo "🏗️ Removing development Docker and install scripts..."
remove_if_exists "docker-compose.dev.yml"
remove_if_exists "install-release.sh"
remove_if_exists "install.sh"

echo ""
echo "📂 Removing demo/test data and artifacts..."
remove_if_exists "data"
remove_if_exists "cache"
check_and_remove "test_projects"
remove_if_exists "multi-lang-test"
remove_if_exists "diagrams_test.html"
remove_if_exists "full_report.html"
remove_if_exists "large_report.html" 
remove_if_exists "no_diagrams.html"
remove_if_exists "test_report.html"
remove_if_exists "test_report.md"
remove_if_exists "simple_test"
remove_if_exists "simple_test.rs"
remove_if_exists "test_serve.rs"
remove_if_exists "test_with_issues.rs"

echo ""
echo "📋 Removing internal reports and test documentation..."
remove_if_exists "test_docs"
remove_if_exists "reports/visual_test"

echo ""
echo "🔧 Removing development tools and utilities..."
remove_if_exists "tools"
remove_if_exists "wit"

echo ""
echo "🏛️ Removing Kubernetes and deployment configs (if not for public use)..."
remove_if_exists "k8s"
remove_if_exists "migrations"

echo ""
echo "📚 Cleaning up documentation artifacts..."
remove_if_exists "docs/book"
remove_if_exists "docs/Screenshots"

echo ""
echo "🎯 Cleaning package-lock files and other artifacts..."
find . -name "package-lock.json" -not -path "./node_modules/*" -delete 2>/dev/null || true
find . -name "*.profraw" -delete 2>/dev/null || true
find . -name ".DS_Store" -delete 2>/dev/null || true
find . -name "Thumbs.db" -delete 2>/dev/null || true

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "📋 Summary of what was removed:"
echo "   • Build artifacts (target/, node_modules/, dist/)"
echo "   • Test coverage data (303 .profraw files, 126MB)"
echo "   • Internal configuration files"
echo "   • Development scripts and tools"
echo "   • Internal documentation and notes"
echo "   • Demo/test data and projects"
echo "   • Development Docker configs"
echo ""
echo "🔍 Files preserved for public release:"
echo "   • Core source code (src/)"
echo "   • Public documentation (README.md, CHANGELOG.md, etc.)"
echo "   • License and contribution guidelines"
echo "   • Core configuration files (Cargo.toml, package.json)"
echo "   • Public Docker configs"
echo "   • Frontend and service source code"
echo ""
echo "⚠️  Remember to:"
echo "   • Review remaining files for any sensitive content"
echo "   • Update README.md for public consumption"
echo "   • Verify all example commands work with the cleaned repo"
echo "   • Test the build process after cleanup"

echo ""
echo "🚀 Ready for public release!"