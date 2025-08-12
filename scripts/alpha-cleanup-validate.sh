#!/bin/bash
# Uveddi Alpha Cleanup Validation Script
# Jira: UV-243 - Validate files before removal
# Purpose: Dry-run analysis of what would be removed

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_highlight() { echo -e "${CYAN}🔍 $1${NC}"; }

# Function to get directory size in human readable format
get_size() {
    if [ -d "$1" ] || [ -f "$1" ]; then
        du -sh "$1" 2>/dev/null | cut -f1
    else
        echo "0B"
    fi
}

# Function to analyze what would be removed
analyze_removal() {
    local path="$1"
    local description="$2"
    local priority="$3"
    
    if [ -e "$path" ]; then
        local size=$(get_size "$path")
        echo -e "${priority} ${description}: ${CYAN}${size}${NC} - ${path}"
        total_to_remove=$((total_to_remove + $(du -sb "$path" 2>/dev/null | cut -f1 || echo 0)))
        removal_count=$((removal_count + 1))
    else
        echo -e "${BLUE}ℹ️${NC}  ${description}: Not found - ${path}"
    fi
}

# Initialize counters
total_to_remove=0
removal_count=0

echo "🔍 Uveddi Alpha Cleanup Validation Report"
echo "========================================"
echo "This script analyzes what files would be removed WITHOUT actually removing them."
echo ""

# Get initial size
initial_size=$(get_size ".")
initial_bytes=$(du -sb "." 2>/dev/null | cut -f1)
log_info "Current repository size: $initial_size"
echo ""

echo "🔥 CRITICAL REMOVALS (Security & Size)"
echo "====================================="

analyze_removal "target/" "Rust build artifacts" "🔥"
analyze_removal "dev_prompts/" "Development prompts (SENSITIVE)" "🔥"
analyze_removal "reports/" "Test reports and artifacts" "🔥"

echo ""
echo "⚠️  HIGH PRIORITY REMOVALS"
echo "========================="

analyze_removal "test_repos/" "Test repositories" "⚠️ "
analyze_removal "test_small/" "Small test projects" "⚠️ "
analyze_removal "test_with_issues/" "Test projects with issues" "⚠️ "
analyze_removal "alpha_testing/" "Alpha testing artifacts" "⚠️ "
analyze_removal "cache/" "Development cache" "⚠️ "
analyze_removal ".uveddi_cache/" "Hidden cache directory" "⚠️ "
analyze_removal "logs/" "Development logs" "⚠️ "
analyze_removal "uveddi_cache.db" "Development database" "⚠️ "

echo ""
echo "🔧 MEDIUM PRIORITY REMOVALS"
echo "=========================="

analyze_removal "simple_cycle_demo" "Demo artifacts" "🔧"
analyze_removal "true/" "Unknown directory" "🔧"
analyze_removal "sample.ts" "Sample TypeScript file" "🔧"
analyze_removal "dependency_analysis_report.txt" "Development analysis report" "🔧"

# Check for HTML test files
html_files=$(find . -maxdepth 1 -name "test_typescript_*.html" -o -name "*_analysis_report.html" 2>/dev/null | wc -l)
if [ $html_files -gt 0 ]; then
    echo -e "🔧 Development test HTML files: ${CYAN}$(du -sh test_typescript_*.html *_analysis_report.html 2>/dev/null | awk '{sum+=$1} END{print sum"B"}' || echo "~1MB")${NC} - Multiple files"
fi

# Check for temporary files
temp_files=$(find . -name "*.tmp" -o -name "*.bak" -o -name "*.orig" -o -name "service.pid" 2>/dev/null | wc -l)
if [ $temp_files -gt 0 ]; then
    echo -e "🔧 Temporary/backup files: ${CYAN}~100KB${NC} - $temp_files files"
fi

echo ""
echo "📄 DEVELOPMENT DOCUMENTATION"
echo "============================"

dev_docs=(
    "INVESTIGATION_PROMPT.md"
    "EXECUTIVE_ACTION_PLAN.md" 
    "SECURITY_FIXES_PLAN.md"
    "TEST_STABILITY_PLAN.md"
    "ALPHA_READINESS_REPORT.md"
    "ALPHA_RELEASE_SETUP.md"
)

for doc in "${dev_docs[@]}"; do
    analyze_removal "$doc" "Development documentation" "📄"
done

echo ""
echo "🧪 TEST FILES TO REMOVE"
echo "======================="

test_files=(
    "tests/cache_performance_test.rs"
    "tests/memory_optimization_phase4.rs"
    "tests/simple_test_infrastructure.rs"
    "tests/tui_performance.rs"
    "tests/unit/analysis/engine_tests_standalone.rs"
)

for test_file in "${test_files[@]}"; do
    analyze_removal "$test_file" "Development test file" "🧪"
done

# Count test_*.rs files
test_pattern_files=$(find tests/ -name "test_*.rs" 2>/dev/null | wc -l)
if [ $test_pattern_files -gt 0 ]; then
    echo -e "🧪 Test pattern files (test_*.rs): ${CYAN}~50KB${NC} - $test_pattern_files files"
fi

echo ""
echo "✅ FILES TO KEEP (Production Ready)"
echo "=================================="

essential_files=(
    "src/"
    "Cargo.toml"
    "Cargo.lock"
    "build.rs"
    "docs/"
    "README.md"
    "CHANGELOG.md"
    "CONTRIBUTING.md"
    "config/"
    ".github/"
    "k8s/"
    "docker-compose.yml"
    "Dockerfile"
    "migrations/"
    "assets/"
    "templates/"
    "wit/"
    "plugins/"
)

for file in "${essential_files[@]}"; do
    if [ -e "$file" ]; then
        size=$(get_size "$file")
        echo -e "✅ ${file}: ${GREEN}${size}${NC} - KEEP"
    else
        echo -e "❓ ${file}: Not found"
    fi
done

echo ""
echo "📊 CLEANUP IMPACT ANALYSIS"
echo "=========================="

# Calculate space savings
if [ $total_to_remove -gt 0 ]; then
    savings_mb=$((total_to_remove / 1024 / 1024))
    savings_percentage=$((total_to_remove * 100 / initial_bytes))
    
    log_highlight "Files/directories to remove: $removal_count"
    log_highlight "Space to reclaim: ${savings_mb}MB (${savings_percentage}%)"
    log_highlight "Estimated final size: $((initial_bytes - total_to_remove)) bytes"
else
    log_info "No files found to remove"
fi

echo ""
echo "🚨 SECURITY REVIEW"
echo "=================="

# Check for potentially sensitive files
sensitive_patterns=(
    "*.key"
    "*.pem"
    "*.crt"
    "*password*"
    "*secret*"
    "*.env"
    ".env.*"
)

echo "Scanning for sensitive files that should definitely be removed..."
sensitive_found=false

for pattern in "${sensitive_patterns[@]}"; do
    files=$(find . -name "$pattern" -type f 2>/dev/null)
    if [ ! -z "$files" ]; then
        log_error "Sensitive files found: $files"
        sensitive_found=true
    fi
done

if [ "$sensitive_found" = false ]; then
    log_success "No obvious sensitive files detected"
fi

echo ""
echo "🎯 RECOMMENDATIONS"
echo "=================="

log_info "1. Review the dev_prompts/ directory - contains internal development strategy"
log_info "2. The target/ directory is 5.4GB - definitely remove before packaging"
log_info "3. Test HTML files contain development artifacts - safe to remove"
log_info "4. Consider if Cargo.lock should be included in alpha distribution"
log_warning "5. Review any custom files not listed above"

echo ""
echo "🚀 NEXT STEPS"
echo "============="

echo "To perform the actual cleanup:"
echo "  ./scripts/alpha-cleanup.sh"
echo ""
echo "To run this validation again:"
echo "  ./scripts/alpha-cleanup-validate.sh"

echo ""
log_success "Validation complete! Review the analysis above before proceeding."
