#!/bin/bash

# Uveddi Architecture Validation Script
# Validates that layer boundaries and architectural constraints are maintained

set -e

echo "🏗️  Uveddi Architecture Validation"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

VIOLATIONS=0

# Function to report violations
report_violation() {
    echo -e "${RED}❌ VIOLATION:${NC} $1"
    ((VIOLATIONS++))
}

# Function to report warnings
report_warning() {
    echo -e "${YELLOW}⚠️  WARNING:${NC} $1"
}

# Function to report success
report_success() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
}

echo ""
echo "🔍 Checking Layer Boundary Violations..."

# Check 1: CLI layer should not directly import infrastructure modules
echo "Checking CLI layer dependencies..."
CLI_VIOLATIONS=$(grep -r "use crate::\(ast\|ai\|database\|plugin\)::" src/cli/ 2>/dev/null || true)
if [ -n "$CLI_VIOLATIONS" ]; then
    report_violation "CLI layer directly imports infrastructure modules:"
    echo "$CLI_VIOLATIONS" | sed 's/^/    /'
else
    report_success "CLI layer respects infrastructure boundaries"
fi

# Check 2: Infrastructure layer should not import analysis or CLI modules
echo "Checking infrastructure layer dependencies..."
INFRA_VIOLATIONS=$(grep -rE "use crate::(cli|analysis)::" src/ast/ src/ai/ src/database/ src/plugin/ src/cache/ 2>/dev/null || true)
if [ -n "$INFRA_VIOLATIONS" ]; then
    report_violation "Infrastructure layer imports higher-level modules:"
    echo "$INFRA_VIOLATIONS" | sed 's/^/    /'
else
    report_success "Infrastructure layer respects upward dependency rules"
fi

# Check 3: Analysis layer should not import CLI modules
echo "Checking analysis layer dependencies..."
ANALYSIS_VIOLATIONS=$(grep -r "use crate::cli::" src/analysis/ 2>/dev/null || true)
if [ -n "$ANALYSIS_VIOLATIONS" ]; then
    report_violation "Analysis layer imports CLI modules:"
    echo "$ANALYSIS_VIOLATIONS" | sed 's/^/    /'
else
    report_success "Analysis layer respects CLI boundary"
fi

echo ""
echo "🔧 Checking Error Handling Patterns..."

# Check 4: All public functions should return Result<T, UveddiError> or appropriate Result type
echo "Checking error handling patterns..."
NON_RESULT_FUNCS=$(grep -rn "pub fn.*-> [^R]" src/ | grep -v "Result" | grep -v "test" | head -10 || true)
if [ -n "$NON_RESULT_FUNCS" ]; then
    report_warning "Public functions not returning Result (may be intentional):"
    echo "$NON_RESULT_FUNCS" | sed 's/^/    /'
else
    report_success "Error handling patterns look consistent"
fi

# Check 5: No panic! in production code (excluding tests)
echo "Checking for panic! usage..."
PANIC_USAGE=$(grep -rn "panic!" src/ | grep -v test | grep -v "todo!" || true)
if [ -n "$PANIC_USAGE" ]; then
    report_violation "Found panic! usage in production code:"
    echo "$PANIC_USAGE" | sed 's/^/    /'
else
    report_success "No panic! found in production code"
fi

echo ""
echo "📦 Checking Module Organization..."

# Check 6: Ensure proper module structure
echo "Checking module structure..."
MISSING_MODS=()

# Check for lib.rs declarations
for module in database models ast analysis ai report cli config error ingestion plugin cache; do
    if ! grep -q "pub mod $module" src/lib.rs; then
        MISSING_MODS+=("$module")
    fi
done

if [ ${#MISSING_MODS[@]} -gt 0 ]; then
    report_violation "Missing module declarations in lib.rs: ${MISSING_MODS[*]}"
else
    report_success "All modules properly declared in lib.rs"
fi

echo ""
echo "🧪 Checking Test Organization..."

# Check 7: Test coverage for main modules
echo "Checking test coverage..."
MISSING_TESTS=()

for module in analysis ai ast database; do
    if [ ! -d "tests/$module" ] && [ ! -f "tests/${module}.rs" ]; then
        MISSING_TESTS+=("$module")
    fi
done

if [ ${#MISSING_TESTS[@]} -gt 0 ]; then
    report_warning "Missing test directories/files for: ${MISSING_TESTS[*]}"
else
    report_success "Test organization looks good"
fi

echo ""
echo "📋 Checking Documentation..."

# Check 8: Key documentation files exist
echo "Checking architectural documentation..."
REQUIRED_DOCS=("docs/ARCHITECTURE.md" "docs/C4_ARCHITECTURE.md" "docs/SAM.md")
MISSING_DOCS=()

for doc in "${REQUIRED_DOCS[@]}"; do
    if [ ! -f "$doc" ]; then
        MISSING_DOCS+=("$doc")
    fi
done

if [ ${#MISSING_DOCS[@]} -gt 0 ]; then
    report_violation "Missing required documentation: ${MISSING_DOCS[*]}"
else
    report_success "All required architectural documentation present"
fi

echo ""
echo "🔍 Checking Interface Definitions..."

# Check 9: Key traits are properly defined
echo "Checking trait definitions..."
REQUIRED_TRAITS=("AnalysisDetector" "LlmProvider" "AnalysisEngine")
MISSING_TRAITS=()

for trait in "${REQUIRED_TRAITS[@]}"; do
    if ! grep -rq "trait $trait" src/; then
        MISSING_TRAITS+=("$trait")
    fi
done

if [ ${#MISSING_TRAITS[@]} -gt 0 ]; then
    report_violation "Missing required trait definitions: ${MISSING_TRAITS[*]}"
else
    report_success "Core traits properly defined"
fi

echo ""
echo "=================================="

# Final summary
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}🎉 Architecture Validation PASSED${NC}"
    echo -e "All layer boundaries and constraints are properly maintained!"
    exit 0
else
    echo -e "${RED}❌ Architecture Validation FAILED${NC}"
    echo -e "Found $VIOLATIONS violation(s) that need to be addressed."
    echo ""
    echo "To fix these issues:"
    echo "1. Review the architectural documentation in docs/ARCHITECTURE.md"
    echo "2. Ensure dependencies flow downward through layers only"
    echo "3. Use proper error handling with UveddiError"
    echo "4. Add missing documentation or tests as needed"
    exit 1
fi
