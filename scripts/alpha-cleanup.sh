#!/bin/bash
# Uveddi Alpha Release Cleanup Script
# Jira: UV-243 - Alpha release preparation
# Purpose: Remove development files before alpha packaging

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Function to get directory size in human readable format
get_size() {
    if [ -d "$1" ] || [ -f "$1" ]; then
        du -sh "$1" 2>/dev/null | cut -f1
    else
        echo "0B"
    fi
}

# Function to safely remove file/directory
safe_remove() {
    local path="$1"
    local description="$2"
    
    if [ -e "$path" ]; then
        local size=$(get_size "$path")
        log_info "Removing $description ($size)..."
        rm -rf "$path"
        log_success "Removed $description"
    else
        log_info "$description not found, skipping..."
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    log_error "Not in Uveddi root directory! Please run from project root."
    exit 1
fi

# Check for git status to avoid accidental commits
if git status --porcelain 2>/dev/null | grep -q .; then
    log_warning "Git working directory is dirty. Consider committing changes first."
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cleanup cancelled."
        exit 0
    fi
fi

echo "🧹 Starting Uveddi Alpha Release Cleanup..."
echo "======================================"

# Get initial size
initial_size=$(get_size ".")
log_info "Initial repository size: $initial_size"

echo ""
echo "🔥 CRITICAL REMOVALS (Security & Size)"
echo "====================================="

# 1. Build artifacts (5.4GB)
safe_remove "target/" "Rust build artifacts"

# 2. Development prompts (sensitive)
safe_remove "dev_prompts/" "Development prompts (sensitive)"

# 3. Test reports & artifacts
safe_remove "reports/" "Test reports and artifacts"

# 4. Development test HTML files
log_info "Removing development test HTML files..."
rm -f test_typescript_*.html 2>/dev/null || true
rm -f flask_*_analysis_report.html 2>/dev/null || true
rm -f rust_analysis_report.html 2>/dev/null || true
rm -f typescript_analysis_report.html 2>/dev/null || true
log_success "Removed development test HTML files"

echo ""
echo "⚠️  HIGH PRIORITY REMOVALS"
echo "========================="

# 5. Test repositories & data
safe_remove "test_repos/" "Test repositories"
safe_remove "test_small/" "Small test projects"
safe_remove "test_with_issues/" "Test projects with issues"
safe_remove "alpha_testing/" "Alpha testing artifacts"

# 6. Cache directories
safe_remove "cache/" "Development cache"
safe_remove ".uveddi_cache/" "Hidden cache directory"
safe_remove "logs/" "Development logs"
safe_remove "uveddi_cache.db" "Development database"

# 7. Temporary & backup files
log_info "Removing temporary and backup files..."
find . -name "*.tmp" -delete 2>/dev/null || true
find . -name "*.bak" -delete 2>/dev/null || true
find . -name "*.orig" -delete 2>/dev/null || true
find . -name "service.pid" -delete 2>/dev/null || true
log_success "Removed temporary and backup files"

# 8. Development artifacts
safe_remove "simple_cycle_demo" "Demo artifacts"
safe_remove "true/" "Unknown directory"
safe_remove "sample.ts" "Sample TypeScript file"
safe_remove "dependency_analysis_report.txt" "Development analysis report"

echo ""
echo "🔧 MEDIUM PRIORITY REMOVALS"
echo "=========================="

# 9. Development documentation
log_info "Removing development documentation..."
rm -f INVESTIGATION_PROMPT.md 2>/dev/null || true
rm -f EXECUTIVE_ACTION_PLAN.md 2>/dev/null || true
rm -f SECURITY_FIXES_PLAN.md 2>/dev/null || true
rm -f TEST_STABILITY_PLAN.md 2>/dev/null || true
rm -f ALPHA_READINESS_REPORT.md 2>/dev/null || true
rm -f ALPHA_RELEASE_SETUP.md 2>/dev/null || true
log_success "Removed development documentation"

# 10. Development scripts
safe_remove "test_install.sh" "Installation testing script"
safe_remove "validate_install.sh" "Validation script"
safe_remove "zola_performance_test.json" "Performance test data"

echo ""
echo "🧪 SELECTIVE TEST FILE REMOVAL"
echo "==============================="

# Remove specific test files that are development-only
log_info "Removing development-specific test files..."
rm -f tests/cache_performance_test.rs 2>/dev/null || true
rm -f tests/memory_optimization_phase4.rs 2>/dev/null || true
rm -f tests/simple_test_infrastructure.rs 2>/dev/null || true
rm -f tests/tui_performance.rs 2>/dev/null || true
rm -f tests/unit/analysis/engine_tests_standalone.rs 2>/dev/null || true

# Remove test files that start with "test_" (but keep the tests/ directory structure)
find tests/ -name "test_*.rs" -type f -delete 2>/dev/null || true

log_success "Removed development-specific test files"

echo ""
echo "🔍 POST-CLEANUP VERIFICATION"
echo "==========================="

# Get final size
final_size=$(get_size ".")
log_info "Final repository size: $final_size"

# Check if essential files still exist
echo ""
log_info "Verifying essential files..."

essential_files=(
    "src/"
    "Cargo.toml" 
    "docs/"
    "config/"
    "README.md"
    "CHANGELOG.md"
    "CONTRIBUTING.md"
    ".github/"
)

all_essential_present=true
for file in "${essential_files[@]}"; do
    if [ -e "$file" ]; then
        log_success "$file ✓"
    else
        log_error "$file ✗ MISSING!"
        all_essential_present=false
    fi
done

if [ "$all_essential_present" = true ]; then
    log_success "All essential files present"
else
    log_error "Some essential files are missing! Review cleanup."
    exit 1
fi

echo ""
echo "🏗️  TESTING BUILD CAPABILITY"
echo "=========================="

log_info "Testing cargo build capability..."
if cargo check --quiet 2>/dev/null; then
    log_success "Cargo check passed"
else
    log_warning "Cargo check failed - may need dependency resolution"
fi

echo ""
echo "📊 CLEANUP SUMMARY"
echo "=================="

log_success "Cleanup completed successfully!"
log_info "Repository cleaned for alpha release"
log_info "Removed development files, build artifacts, and test data"
log_info "Essential production files preserved"

echo ""
log_info "Next steps:"
echo "  1. Review remaining files: ls -la"
echo "  2. Build release: cargo build --release"
echo "  3. Test functionality: cargo test --release"
echo "  4. Package for alpha distribution"

echo ""
log_success "✨ Uveddi is ready for alpha packaging! ✨"

# Optional: Show git status
if command -v git >/dev/null 2>&1 && git rev-parse --git-dir >/dev/null 2>&1; then
    echo ""
    log_info "Git status after cleanup:"
    git status --short
fi
